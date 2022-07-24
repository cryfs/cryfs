use anyhow::{bail, ensure, Result};
use async_trait::async_trait;
use binary_layout::Field;

use crate::blockstore::{
    high_level::{LockingBlockStore, RemoveResult},
    low_level::BlockStore,
    BlockId, BLOCKID_LEN,
};
use crate::data::Data;
use crate::utils::async_drop::{AsyncDrop, AsyncDropGuard};

mod layout;
use layout::node;

mod data_inner_node;
pub use data_inner_node::DataInnerNode;

mod data_leaf_node;
pub use data_leaf_node::DataLeafNode;

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
            min_block_size,
        );
        Ok(AsyncDropGuard::new(Self {
            block_store,
            block_size_bytes,
        }))
    }

    pub fn max_bytes_per_leaf(&self) -> u32 {
        self.block_size_bytes - u32::try_from(node::data::OFFSET).unwrap()
    }

    pub fn max_children_per_inner_node(&self) -> u32 {
        let datasize = self.max_bytes_per_leaf();
        datasize / u32::try_from(BLOCKID_LEN).unwrap()
    }

    pub async fn load(&self, block_id: BlockId) -> Result<Option<DataNode<B>>> {
        match self.block_store.load(block_id).await? {
            None => Ok(None),
            Some(block) => {
                ensure!(
                    usize::try_from(self.block_size_bytes).unwrap() == block.data().len(),
                    "Expected to load block of size {} but loaded block {:?} had size {}",
                    self.block_size_bytes,
                    block_id,
                    block.data().len(),
                );
                let node_view = node::View::new(block.data());
                let format_version_header = node_view.format_version_header().read();
                ensure!(layout::FORMAT_VERSION_HEADER == format_version_header, "Loaded a node {:?} with format_version_header == {}. This is not a supported format.", block_id, format_version_header);
                let unused_must_be_zero = node_view.unused_must_be_zero().read();
                ensure!(
                    0 == unused_must_be_zero,
                    "Loaded a node {:?} where the unused part isn't ZERO but {}",
                    block_id,
                    unused_must_be_zero
                );
                let depth = node_view.depth().read();
                if depth == 0 {
                    Ok(Some(DataNode::Leaf(DataLeafNode::new(block, self)?)))
                } else {
                    Ok(Some(DataNode::Inner(DataInnerNode::new(block, self)?)))
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
        let block_data = data_leaf_node::serialize_leaf_node(data, self);
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
        let block_data = data_inner_node::serialize_inner_node(depth, children, self);
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

// TODO Tests
