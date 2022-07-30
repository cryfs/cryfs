use anyhow::{anyhow, bail, ensure, Result};
use async_trait::async_trait;
use binary_layout::Field;

pub use crate::blockstore::high_level::RemoveResult;
use crate::blockstore::{
    high_level::LockingBlockStore, low_level::BlockStore, BlockId, BLOCKID_LEN,
};
use crate::data::Data;
use crate::utils::async_drop::{AsyncDrop, AsyncDropGuard};

mod layout;
use layout::node;
pub use layout::NodeLayout;

mod data_node;
pub use data_node::{DataInnerNode, DataLeafNode, DataNode};

#[derive(Debug)]
pub struct DataNodeStore<B: BlockStore + Send + Sync> {
    block_store: AsyncDropGuard<LockingBlockStore<B>>,
    layout: NodeLayout,
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
            layout: NodeLayout { block_size_bytes },
        }))
    }

    pub fn layout(&self) -> &NodeLayout {
        &self.layout
    }

    pub async fn load(&self, block_id: BlockId) -> Result<Option<DataNode<B>>> {
        match self.block_store.load(block_id).await? {
            None => Ok(None),
            Some(block) => DataNode::parse(block, &self.layout).map(Some),
        }
    }

    fn _allocate_data_for_leaf_node(&self) -> Data {
        let mut data = Data::from(vec![
            0;
            usize::try_from(self.layout.max_bytes_per_leaf())
                .unwrap()
        ]);
        data.shrink_to_subregion(node::data::OFFSET..);
        data
    }

    pub async fn create_new_leaf_node(&self) -> Result<DataLeafNode<B>> {
        let data = self._allocate_data_for_leaf_node();
        let block_data = data_node::serialize_leaf_node_optimized(data, &self.layout);
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
        let block_data = data_node::serialize_inner_node(depth, children, &self.layout);
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

    pub async fn create_new_node_as_copy_from(&self, source: &DataNode<B>) -> Result<DataNode<B>> {
        let source_data = source.raw_blockdata();
        assert_eq!(usize::try_from(self.layout.block_size_bytes).unwrap(), source_data.len(), "Source node has wrong layout and has {} bytes. We expected {} bytes. Is it from the same DataNodeStore?", source_data.len(), self.layout.block_size_bytes);
        // TODO Use create_optimized instead of create?
        let blockid = self.block_store.create(source_data).await?;
        // TODO Avoid extra load here. Do our callers actually need this object? If no, just return the block id. If yes, maybe change block store API to return the block?
        self.load(blockid)
            .await?
            .ok_or_else(|| anyhow!("We just created {:?} but now couldn't find it", blockid))
    }

    pub async fn overwrite_leaf_node(&self, block_id: &BlockId, data: &[u8]) -> Result<()> {
        let mut data_obj = self._allocate_data_for_leaf_node();
        // TODO Make an overwrite_leaf_node_optimized version that requires that enough prefix bytes are already available in the data input and that doesn't require us to copy_from_slice here?
        data_obj.as_mut().copy_from_slice(data);
        let block_data = data_node::serialize_leaf_node_optimized(data_obj, &self.layout);
        // TODO Use store_optimized instead of store?
        self.block_store.overwrite(block_id, &block_data).await
    }

    pub async fn remove_by_id(&self, block_id: &BlockId) -> Result<RemoveResult> {
        self.block_store.remove(block_id).await
    }

    pub async fn num_nodes(&self) -> Result<u64> {
        self.block_store.num_blocks().await
    }

    pub fn estimate_space_for_num_blocks_left(&self) -> Result<u64> {
        Ok(self.block_store.estimate_num_free_bytes()? / u64::from(self.layout.block_size_bytes))
    }

    pub fn virtual_block_size_bytes(&self) -> u32 {
        self.layout.block_size_bytes
    }

    // cpputils::unique_ref<DataNode> overwriteNodeWith(cpputils::unique_ref<DataNode> target, const DataNode &source);

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
