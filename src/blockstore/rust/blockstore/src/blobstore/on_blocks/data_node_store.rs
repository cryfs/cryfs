use anyhow::{bail, ensure, Result};
use async_trait::async_trait;
use binary_layout::{define_layout, Field};

use crate::blockstore::{
    high_level::{Block, LockingBlockStore, RemoveResult},
    low_level::BlockStore,
    BlockId, BLOCKID_LEN,
};
use crate::data::Data;
use crate::utils::async_drop::{AsyncDrop, AsyncDropGuard};

const MAX_DEPTH: u8 = 10;
const FORMAT_VERSION_HEADER: u16 = 0;

define_layout!(node, LittleEndian, {
    format_version_header: u16,

    // Not currently used, only used for alignment.
    _unused: u8,

    // Leaf nodes have a depth of 0. Each layer above has a depth of one higher than the level directly below.
    depth: u8,

    // Leaf nodes store number of data byes here. Inner nodes store number of children.
    size: u32,

    // Data. Leaf nodes just store bytes here. Inner nodes store a list of child block ids.
    data: [u8],
});

pub struct DataInnerNode<B: BlockStore + Send + Sync> {
    block: Block<B>,
}

impl<B: BlockStore + Send + Sync> DataInnerNode<B> {
    pub fn children<'a>(
        &'a self,
    ) -> Result<impl Iterator<Item = BlockId> + ExactSizeIterator + 'a> {
        let view = node::View::new(self.block.data().as_ref());
        let num_children = usize::try_from(view.size().read()).unwrap();
        let children_data = view.into_data().into_slice();
        let children_ids = children_data.chunks_exact(BLOCKID_LEN);
        ensure!(
            num_children <= children_ids.len(),
            "Tried to load an inner node with {} children but support at most {} per inner node",
            num_children,
            children_ids.len(),
        );
        Ok(children_ids
            .take(num_children)
            .map(|id_bytes| BlockId::from_slice(id_bytes).unwrap()))
    }
}

pub struct DataLeafNode<B: BlockStore + Send + Sync> {
    block: Block<B>,
}

pub enum DataNode<B: BlockStore + Send + Sync> {
    Inner(DataInnerNode<B>),
    Leaf(DataLeafNode<B>),
}

#[derive(Debug)]
pub struct DataNodeStore<B: BlockStore + Send + Sync> {
    block_store: AsyncDropGuard<LockingBlockStore<B>>,
    block_size_bytes: u32,
}

impl<B: BlockStore + Send + Sync> DataNodeStore<B> {
    pub fn new(
        block_store: AsyncDropGuard<LockingBlockStore<B>>,
        block_size_bytes: u32,
    ) -> Result<AsyncDropGuard<Self>> {
        // Min block size: enough for header and for inner nodes to have at least two children and form a tree.
        let min_block_size = u32::try_from(node::data::OFFSET + 2 * BLOCKID_LEN).unwrap();
        ensure!(
            block_size_bytes >= min_block_size,
            "Tried to create a DataNodeStore with block size {} but must be at least {}",
            block_size_bytes,
            min_block_size
        );
        Ok(AsyncDropGuard::new(Self {
            block_store,
            block_size_bytes,
        }))
    }

    fn max_bytes_per_leaf(&self) -> u32 {
        self.block_size_bytes - u32::try_from(node::data::OFFSET).unwrap()
    }

    fn max_children_per_inner_node(&self) -> u32 {
        let datasize = self.max_bytes_per_leaf();
        datasize / u32::try_from(BLOCKID_LEN).unwrap()
    }

    pub async fn load(&self, block_id: BlockId) -> Result<Option<DataNode<B>>> {
        match self.block_store.load(block_id).await? {
            None => Ok(None),
            Some(block) => {
                ensure!(
                    usize::try_from(self.block_size_bytes).unwrap() == block.data().len(),
                    "Expected to load block of size {} but loaded block of size {}",
                    self.block_size_bytes,
                    block.data().len(),
                );
                let node_view = node::View::new(block.data());
                let depth = node_view.depth().read();
                if depth == 0 {
                    Ok(Some(DataNode::Leaf(DataLeafNode { block })))
                } else if depth <= MAX_DEPTH {
                    Ok(Some(DataNode::Inner(DataInnerNode { block })))
                } else {
                    bail!("Tree is too deep. Data corruption?");
                }
            }
        }
    }

    fn _allocate_data_for_leaf_node(&self) -> Data {
        let mut data = Data::from(vec![0; usize::try_from(self.max_bytes_per_leaf()).unwrap()]);
        data.shrink_to_subregion(node::data::OFFSET..);
        data
    }

    pub async fn create_new_leaf_node(&self) -> Result<DataLeafNode<B>> {
        let data = self._allocate_data_for_leaf_node();
        let block_data = self._serialize_leaf_node(data);
        // TODO Use create_optimized instead of create?
        let blockid = self.block_store.create(&block_data).await?;
        // TODO Avoid extra load here. Do our callers actually need this object? If no, just return the block id. If yes, maybe change block store API to return the block?
        match self.load(blockid).await? {
            None => bail!("We just created this block, it must exist"),
            Some(DataNode::Inner(_)) => {
                bail!("We just created a leaf node but then it got loaded as an inner node")
            }
            Some(DataNode::Leaf(node)) => Ok(node),
        }
    }

    pub async fn create_new_inner_node(
        &self,
        depth: u8,
        children: &[BlockId],
    ) -> Result<DataInnerNode<B>> {
        let block_data = self._serialize_inner_node(depth, children);
        // TODO Use create_optimized instead of create?
        let blockid = self.block_store.create(&block_data).await?;
        // TODO Avoid extra load here. Do our callers actually need this object? If no, just return the block id. If yes, maybe change block store API to return the block?
        match self.load(blockid).await? {
            None => bail!("We just created this block, it must exist"),
            Some(DataNode::Leaf(_)) => {
                bail!("We just created a inner node but then it got loaded as a leaf node")
            }
            Some(DataNode::Inner(node)) => Ok(node),
        }
    }

    fn _serialize_leaf_node(&self, mut data: Data) -> Data {
        let size: u32 = u32::try_from(data.len()).unwrap();
        assert!(
            size < self.max_bytes_per_leaf(),
            "Tried to create leaf with {} bytes but each leaf can only hold {}",
            size,
            self.max_bytes_per_leaf()
        );
        assert!(data.available_prefix_bytes() >= node::data::OFFSET, "Data objects passed to create_new_leaf_node_optimized must have at least {} prefix bytes available, but only had {}", node::data::OFFSET, data.available_prefix_bytes());
        data.grow_region_fail_if_reallocation_necessary(node::data::OFFSET, 0).expect("Not enough prefix bytes available for data object passed to DataNodeStore::create_new_leaf_node_optimized");
        let mut view = node::View::new(&mut data);
        view.format_version_header_mut()
            .write(FORMAT_VERSION_HEADER);
        view._unused_mut().write(0);
        view.depth_mut().write(0);
        view.size_mut().write(size);
        // view.data is already set correctly because we grew this view from the data input
        data
    }

    fn _serialize_inner_node(&self, depth: u8, children: &[BlockId]) -> Data {
        assert!(
            depth != 0,
            "Inner node cannot have a depth of 0. Is this perhaps a leaf instead?"
        );
        assert!(
            children.len() >= 1,
            "Inner node must have at least one child"
        );
        assert!(
            children.len() <= usize::try_from(self.max_children_per_inner_node()).unwrap(),
            "Inner nodes can only store {} children but tried to store {}",
            self.max_children_per_inner_node(),
            children.len(),
        );

        let mut view = node::View::new(Data::from(vec![
            0;
            self.block_size_bytes.try_into().unwrap()
        ]));
        view.format_version_header_mut()
            .write(FORMAT_VERSION_HEADER);
        view._unused_mut().write(0);
        view.depth_mut().write(depth);
        view.size_mut()
            .write(u32::try_from(children.len()).unwrap());
        _serialize_children(view.data_mut(), children);
        view.into_storage()
    }

    async fn remove(&self, block_id: &BlockId) -> Result<RemoveResult> {
        self.block_store.remove(block_id).await
    }

    // cpputils::unique_ref<DataNode> createNewNodeAsCopyFrom(const DataNode &source);

    // cpputils::unique_ref<DataNode> overwriteNodeWith(cpputils::unique_ref<DataNode> target, const DataNode &source);

    // cpputils::unique_ref<DataLeafNode> overwriteLeaf(const blockstore::BlockId &blockId, cpputils::Data data);

    // void remove(cpputils::unique_ref<DataNode> node);
    // void removeSubtree(uint8_t depth, const blockstore::BlockId &blockId);
    // void removeSubtree(cpputils::unique_ref<DataNode> node);

    // uint64_t virtualBlocksizeBytes() const;
    // uint64_t numNodes() const;
    // uint64_t estimateSpaceForNumNodesLeft() const;

    // void forEachNode(std::function<void (const blockstore::BlockId& nodeId)> callback) const;
}

#[async_trait]
impl<B: BlockStore + Send + Sync> AsyncDrop for DataNodeStore<B> {
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.block_store.async_drop().await
    }
}

fn _serialize_children(dest: &mut [u8], children: &[BlockId]) {
    assert_eq!(dest.len(), children.len() * BLOCKID_LEN, "Serializing {} children requires {} bytes but tried to serialize into a buffer with {} bytes.", children.len(), children.len() * BLOCKID_LEN, dest.len());
    for (index, child) in children.iter().enumerate() {
        // TODO Some way to avoid this copy by not using &[BlockId] or Vec<BlockId> but our own collection type that already has it aligned correctly?
        dest[(BLOCKID_LEN * index)..(BLOCKID_LEN * (index + 1))].copy_from_slice(child.data());
    }
}

// TODO Tests
