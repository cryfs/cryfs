use anyhow::Result;
use async_trait::async_trait;

use crate::blobstore::{on_blocks::data_node_store::DataNodeStore, RemoveResult};
use crate::blockstore::high_level::LockingBlockStore;
use crate::blockstore::low_level::BlockStore;
use crate::blockstore::BlockId;
use crate::data::Data;
use crate::utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};

use super::tree::DataTree;

#[derive(Debug)]
pub struct DataTreeStore<B: BlockStore + Send + Sync> {
    node_store: AsyncDropGuard<DataNodeStore<B>>,
}

impl<B: BlockStore + Send + Sync> DataTreeStore<B> {
    pub fn new(
        block_store: AsyncDropGuard<LockingBlockStore<B>>,
        block_size_bytes: u32,
    ) -> Result<AsyncDropGuard<Self>> {
        Ok(AsyncDropGuard::new(Self {
            node_store: DataNodeStore::new(block_store, block_size_bytes)?,
        }))
    }
}

impl<B: BlockStore + Send + Sync> DataTreeStore<B> {
    pub async fn load_tree(&self, root_node_id: BlockId) -> Result<Option<DataTree<'_, B>>> {
        Ok(self
            .node_store
            .load(root_node_id)
            .await?
            .map(|root_node| DataTree::new(root_node, &self.node_store)))
    }

    pub async fn create_tree(&self) -> Result<DataTree<'_, B>> {
        let new_leaf = self
            .node_store
            .create_new_leaf_node(&Data::from(vec![]))
            .await?;
        Ok(DataTree::new(new_leaf.upcast(), &self.node_store))
    }

    pub async fn remove_tree_by_id(&self, root_node_id: BlockId) -> Result<RemoveResult> {
        match self.load_tree(root_node_id).await? {
            Some(tree) => {
                DataTree::remove(tree).await?;
                Ok(RemoveResult::SuccessfullyRemoved)
            }
            None => Ok(RemoveResult::NotRemovedBecauseItDoesntExist),
        }
    }

    pub async fn num_nodes(&self) -> Result<u64> {
        self.node_store.num_nodes().await
    }

    pub fn estimate_space_for_num_blocks_left(&self) -> Result<u64> {
        self.node_store.estimate_space_for_num_blocks_left()
    }

    pub fn virtual_block_size_bytes(&self) -> u32 {
        self.node_store.virtual_block_size_bytes()
    }
}

#[async_trait]
impl<B: BlockStore + Send + Sync> AsyncDrop for DataTreeStore<B> {
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.node_store.async_drop().await
    }
}
