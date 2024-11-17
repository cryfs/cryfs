use anyhow::{anyhow, Result};
use async_trait::async_trait;
use byte_unit::Byte;
use futures::stream::BoxStream;
use std::fmt::{self, Debug};

use super::super::BlobStoreOnBlocks;
use crate::{Blob, BlobId, BlobStore};
use cryfs_blockstore::{
    tests::Fixture, BlockId, BlockStore, BlockStoreDeleter, BlockStoreReader, BlockStoreWriter,
    InMemoryBlockStore, InvalidBlockSizeError, LockingBlockStore, RemoveResult, TryCreateResult,
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    data::Data,
};

/// Wrap a [BlobStore] into a [BlockStore] so that we can run the regular block store tests on it.
/// Each block is stored as a blob.
pub struct BlockStoreAdapter {
    underlying_store: AsyncDropGuard<BlobStoreOnBlocks<InMemoryBlockStore>>,
    block_size: Byte,
}

impl BlockStoreAdapter {
    pub async fn new(block_size: Byte) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            underlying_store: BlobStoreOnBlocks::new(
                LockingBlockStore::new(InMemoryBlockStore::new()),
                block_size,
            )
            .await
            .unwrap(),
            block_size,
        })
    }
}

#[async_trait]
impl BlockStoreReader for BlockStoreAdapter {
    async fn exists(&self, id: &BlockId) -> Result<bool> {
        Ok(self
            .underlying_store
            .load(&BlobId { root: *id })
            .await?
            .is_some())
    }

    async fn load(&self, id: &BlockId) -> Result<Option<Data>> {
        if let Some(mut blob) = self.underlying_store.load(&BlobId { root: *id }).await? {
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
        let overhead = Byte::from_u64(u64::from(self.underlying_store.virtual_block_size_bytes()))
            .subtract(self.block_size)
            .unwrap();
        block_size
            .subtract(overhead)
            .ok_or_else(|| InvalidBlockSizeError::new(format!("block size out of range")))
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
impl BlockStoreDeleter for BlockStoreAdapter {
    async fn remove(&self, id: &BlockId) -> Result<RemoveResult> {
        self.underlying_store
            .remove_by_id(&BlobId { root: *id })
            .await
    }
}

#[async_trait]
impl BlockStoreWriter for BlockStoreAdapter {
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

impl Debug for BlockStoreAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockStoreAdapter")
    }
}

#[async_trait]
impl AsyncDrop for BlockStoreAdapter {
    type Error = anyhow::Error;
    async fn async_drop_impl(&mut self) -> Result<()> {
        self.underlying_store.async_drop().await
    }
}

impl BlockStore for BlockStoreAdapter {}

/// TestFixtureAdapter takes a [Fixture] for a [BlockStore] and makes it into
/// a [Fixture] that creates a [DataNodeStore] based on that [BlockStore].
/// This allows using our block store test suite on [DataNodeStore].
pub struct TestFixtureAdapter<const FLUSH_CACHE_ON_YIELD: bool, const BLOCK_SIZE_BYTES: u64> {}
#[async_trait]
impl<const FLUSH_CACHE_ON_YIELD: bool, const BLOCK_SIZE_BYTES: u64> Fixture
    for TestFixtureAdapter<FLUSH_CACHE_ON_YIELD, BLOCK_SIZE_BYTES>
{
    type ConcreteBlockStore = BlockStoreAdapter;
    fn new() -> Self {
        Self {}
    }
    async fn store(&mut self) -> AsyncDropGuard<Self::ConcreteBlockStore> {
        BlockStoreAdapter::new(Byte::from_u64(BLOCK_SIZE_BYTES)).await
    }
    async fn yield_fixture(&self, store: &Self::ConcreteBlockStore) {
        if FLUSH_CACHE_ON_YIELD {
            store.underlying_store.clear_cache_slow().await.unwrap();
        }
    }
}
