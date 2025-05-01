use anyhow::{Result, anyhow};
use async_trait::async_trait;
use byte_unit::Byte;
use futures::stream::BoxStream;
use std::fmt::{self, Debug};

use crate::{Blob, BlobId, BlobStore};
use cryfs_blockstore::{
    BlockId, BlockStoreDeleter, BlockStoreReader, BlockStoreWriter, InvalidBlockSizeError,
    LLBlockStore, RemoveResult, TryCreateResult,
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    data::Data,
};

/// Wrap a [BlobStore] into a [BlockStore] so that we can run the regular block store tests on it.
/// Each block is stored as a blob.
pub struct BlockStoreAdapter<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
{
    underlying_store: AsyncDropGuard<B>,
}

impl<B> BlockStoreAdapter<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
{
    pub async fn new(underlying_store: AsyncDropGuard<B>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self { underlying_store })
    }

    pub fn inner(&self) -> &B {
        &self.underlying_store
    }

    pub async fn clear_cache_slow(&self) -> Result<()> {
        self.underlying_store.clear_cache_slow().await
    }
}

// TODO Should we implement [BlockStore] instead of [LLBlockStore] for this adapter and run the high level tests? Seems to be a closer fit? Same for data_node_store::test_as_blockstore

#[async_trait]
impl<B> BlockStoreReader for BlockStoreAdapter<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
{
    async fn exists(&self, id: &BlockId) -> Result<bool> {
        Ok(self
            .underlying_store
            .load(&BlobId { root: *id })
            .await?
            .is_some())
    }

    async fn load(&self, id: &BlockId) -> Result<Option<Data>> {
        let blob_id = BlobId { root: *id };
        let loaded = self.underlying_store.load(&blob_id);
        if let Some(mut blob) = loaded.await? {
            Ok(Some(blob.read_all().await?))
        } else {
            Ok(None)
        }
    }

    async fn num_blocks(&self) -> Result<u64> {
        let blob_ids = self.underlying_store.all_blobs().await?;
        Ok(blob_ids.len() as u64)
    }

    fn estimate_num_free_bytes(&self) -> Result<Byte> {
        self.underlying_store
            .virtual_block_size_bytes()
            .multiply(
                usize::try_from(self.underlying_store.estimate_space_for_num_blocks_left()?)
                    .unwrap(),
            )
            .ok_or_else(|| anyhow!("overflow"))
    }

    fn block_size_from_physical_block_size(
        &self,
        block_size: Byte,
    ) -> Result<Byte, InvalidBlockSizeError> {
        Ok(block_size)
    }

    async fn all_blocks(&self) -> Result<BoxStream<'static, Result<BlockId>>> {
        let blob_ids = self
            .underlying_store
            .all_blobs()
            .await?
            .into_iter()
            .map(|blob_id| Ok(blob_id.root));
        Ok(Box::pin(futures::stream::iter(blob_ids)))
    }
}

#[async_trait]
impl<B> BlockStoreDeleter for BlockStoreAdapter<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
{
    async fn remove(&self, id: &BlockId) -> Result<RemoveResult> {
        self.underlying_store
            .remove_by_id(&BlobId { root: *id })
            .await
    }
}

#[async_trait]
impl<B> BlockStoreWriter for BlockStoreAdapter<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
{
    async fn try_create(&self, id: &BlockId, data: &[u8]) -> Result<TryCreateResult> {
        let Some(mut blob) = self
            .underlying_store
            .try_create(&BlobId { root: *id })
            .await?
        else {
            return Ok(TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists);
        };
        blob.resize(data.len() as u64).await?;
        blob.write(data, 0).await?;
        Ok(TryCreateResult::SuccessfullyCreated)
    }

    async fn store(&self, id: &BlockId, data: &[u8]) -> Result<()> {
        let mut blob = if let Some(blob) = self.underlying_store.load(&BlobId { root: *id }).await?
        {
            blob
        } else {
            self.underlying_store
                .try_create(&BlobId { root: *id })
                .await?
                .expect("We just checked that it doesn't exist, so it must be creatable.")
        };
        if blob.num_bytes().await? != data.len() as u64 {
            blob.resize(data.len() as u64).await?;
        }
        blob.write(data, 0).await?;
        Ok(())
    }
}

impl<B> Debug for BlockStoreAdapter<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockStoreAdapter")
    }
}

#[async_trait]
impl<B> AsyncDrop for BlockStoreAdapter<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
{
    type Error = anyhow::Error;
    async fn async_drop_impl(&mut self) -> Result<()> {
        self.underlying_store.async_drop().await
    }
}

impl<B> LLBlockStore for BlockStoreAdapter<B> where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static
{
}
