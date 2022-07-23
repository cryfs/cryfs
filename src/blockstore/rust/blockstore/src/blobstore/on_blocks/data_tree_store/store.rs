use anyhow::Result;

use crate::blockstore::high_level::LockingBlockStore;
use crate::blockstore::low_level::BlockStore;
use crate::utils::async_drop::AsyncDropGuard;
use crate::blockstore::BlockId;
use crate::blobstore::on_blocks::data_node_store::{DataNode, DataNodeStore};
use crate::utils::async_drop::AsyncDropArc;

use super::tree::DataTree;

#[derive(Debug)]
pub struct DataTreeStore<B: BlockStore + Send + Sync> {
    node_store: AsyncDropGuard<AsyncDropArc<DataNodeStore<B>>>,
}

impl<B: BlockStore + Send + Sync> DataTreeStore<B> {
    pub fn new(block_store: AsyncDropGuard<LockingBlockStore<B>>, block_size_bytes: u32) -> Result<Self> {
        Ok(Self {
            node_store: AsyncDropArc::new(DataNodeStore::new(block_store, block_size_bytes)?),
        })
    }
}

impl <B: BlockStore + Send + Sync> DataTreeStore<B> {
    pub async fn load_tree(&self, root_node_id: BlockId) -> Result<Option<DataTree<B>>> {
        Ok(self.node_store.load(root_node_id).await?.map(|root_node| {
            DataTree::new(root_node, AsyncDropArc::clone(&self.node_store))
        }))
    }

    pub async fn create_tree(&self) -> Result<DataTree<B>> {
        let new_leaf = self.node_store.create_new_leaf_node().await?;
        Ok(DataTree::new(DataNode::Leaf(new_leaf), AsyncDropArc::clone(&self.node_store)))
    }
}
