use anyhow::Result;
use async_trait::async_trait;
#[cfg(test)]
use futures::TryStreamExt;
#[cfg(test)]
use std::collections::HashSet;

#[cfg(test)]
use crate::on_blocks::data_node_store::DataNode;
use crate::{on_blocks::data_node_store::DataNodeStore, RemoveResult};
use cryfs_blockstore::{BlockId, BlockStore, LockingBlockStore};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    data::Data,
};

use super::tree::DataTree;

#[derive(Debug)]
pub struct DataTreeStore<B: BlockStore + Send + Sync> {
    node_store: AsyncDropGuard<DataNodeStore<B>>,
}

impl<B: BlockStore + Send + Sync> DataTreeStore<B> {
    pub async fn new(
        block_store: AsyncDropGuard<LockingBlockStore<B>>,
        block_size_bytes: u32,
    ) -> Result<AsyncDropGuard<Self>> {
        Ok(AsyncDropGuard::new(Self {
            node_store: DataNodeStore::new(block_store, block_size_bytes).await?,
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

    #[cfg(test)]
    pub async fn try_create_tree(&self, id: BlockId) -> Result<DataTree<'_, B>> {
        let new_leaf = self
            .node_store
            .try_create_new_leaf_node(id, &Data::from(vec![]))
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

    pub async fn load_block_depth(&self, id: &BlockId) -> Result<Option<u8>> {
        Ok(self.node_store.load(*id).await?.map(|node| node.depth()))
    }

    #[cfg(test)]
    // This needs to load all blocks, so it's not very efficient. Only use it for tests.
    pub async fn all_tree_roots(&self) -> Result<Vec<BlockId>> {
        let all_nodes: Vec<BlockId> = self.node_store.all_nodes().await?.try_collect().await?;
        let mut potential_roots: HashSet<BlockId> = all_nodes.iter().copied().collect();

        for node_id in all_nodes {
            match self.node_store.load(node_id).await? {
                Some(DataNode::Leaf(_)) | None => { /* do nothing */ }
                Some(DataNode::Inner(inner)) => {
                    for child_id in inner.children() {
                        potential_roots.remove(&child_id);
                    }
                }
            }
        }

        Ok(potential_roots.into_iter().collect())
    }

    #[cfg(test)]
    pub async fn clear_cache_slow(&self) -> Result<()> {
        self.node_store.clear_cache_slow().await
    }
}

#[async_trait]
impl<B: BlockStore + Send + Sync> AsyncDrop for DataTreeStore<B> {
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.node_store.async_drop().await
    }
}
