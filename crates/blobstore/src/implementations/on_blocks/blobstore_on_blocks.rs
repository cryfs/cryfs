use anyhow::Result;
use async_trait::async_trait;
use byte_unit::Byte;
use std::fmt;
use std::fmt::Debug;

use super::blob_on_blocks::BlobOnBlocks;
use super::data_tree_store::DataTreeStore;
use crate::{BlobId, BlobStore, RemoveResult};
use cryfs_blockstore::{BlockId, BlockStore, InvalidBlockSizeError};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

pub struct BlobStoreOnBlocks<B: BlockStore<Block: Send + Sync> + AsyncDrop + Debug + Send + Sync> {
    tree_store: AsyncDropGuard<DataTreeStore<B>>,
}

impl<B: BlockStore<Block: Send + Sync> + AsyncDrop + Debug + Send + Sync> BlobStoreOnBlocks<B> {
    pub async fn new(
        blockstore: AsyncDropGuard<B>,
        physical_block_size: Byte,
    ) -> Result<AsyncDropGuard<Self>, InvalidBlockSizeError> {
        Ok(AsyncDropGuard::new(Self {
            tree_store: DataTreeStore::new(blockstore, physical_block_size).await?,
        }))
    }

    pub async fn load_block_depth(&self, id: &BlockId) -> Result<Option<u8>> {
        self.tree_store.load_block_depth(id).await
    }

    pub fn into_inner_tree_store(this: AsyncDropGuard<Self>) -> AsyncDropGuard<DataTreeStore<B>> {
        this.unsafe_into_inner_dont_drop().tree_store
    }
}

#[async_trait]
impl<B: BlockStore<Block: Send + Sync> + AsyncDrop + Debug + Send + Sync> BlobStore
    for BlobStoreOnBlocks<B>
{
    type ConcreteBlob<'a>
        = BlobOnBlocks<'a, B>
    where
        B: 'a;

    async fn create(&self) -> Result<Self::ConcreteBlob<'_>> {
        Ok(BlobOnBlocks::new(self.tree_store.create_tree().await?))
    }

    async fn try_create(&self, id: &BlobId) -> Result<Option<Self::ConcreteBlob<'_>>> {
        Ok(self
            .tree_store
            .try_create_tree(id.root)
            .await?
            .map(|tree| BlobOnBlocks::new(tree)))
    }

    async fn load(&self, id: &BlobId) -> Result<Option<Self::ConcreteBlob<'_>>> {
        Ok(self
            .tree_store
            .load_tree(id.root)
            .await?
            .map(|tree| BlobOnBlocks::new(tree)))
    }

    async fn remove_by_id(&self, id: &BlobId) -> Result<RemoveResult> {
        Ok(self.tree_store.remove_tree_by_id(id.root).await?)
    }

    async fn num_nodes(&self) -> Result<u64> {
        self.tree_store.num_nodes().await
    }

    fn estimate_space_for_num_blocks_left(&self) -> Result<u64> {
        self.tree_store.estimate_space_for_num_blocks_left()
    }

    fn logical_block_size_bytes(&self) -> Byte {
        self.tree_store.logical_block_size_bytes()
    }

    #[cfg(any(test, feature = "testutils"))]
    async fn clear_cache_slow(&self) -> Result<()> {
        self.tree_store.clear_cache_slow().await
    }

    #[cfg(test)]
    async fn all_blobs(&self) -> Result<Vec<BlobId>> {
        Ok(self
            .tree_store
            .all_tree_roots()
            .await?
            .into_iter()
            .map(|root| BlobId { root })
            .collect())
    }
}

impl<B: BlockStore<Block: Send + Sync> + AsyncDrop + Debug + Send + Sync> fmt::Debug
    for BlobStoreOnBlocks<B>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockStoreOnBlocks")
    }
}

#[async_trait]
impl<B: BlockStore<Block: Send + Sync> + AsyncDrop + Debug + Send + Sync> AsyncDrop
    for BlobStoreOnBlocks<B>
{
    type Error = <B as AsyncDrop>::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.tree_store.async_drop().await
    }
}
