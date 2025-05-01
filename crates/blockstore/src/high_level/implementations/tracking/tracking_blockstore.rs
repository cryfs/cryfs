use anyhow::Result;
use async_trait::async_trait;
use byte_unit::Byte;
use futures::stream::BoxStream;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use super::{ActionCounts, tracking_block::TrackingBlock};
use crate::BlockStore;
use crate::{BlockId, InvalidBlockSizeError, RemoveResult, TryCreateResult};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};
use cryfs_utils::data::Data;

#[derive(Debug)]
pub struct TrackingBlockStore<B>
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
    B::Block: Send + Sync,
{
    underlying_store: AsyncDropGuard<B>,

    counts: Arc<Mutex<ActionCounts>>,
}

impl<B> TrackingBlockStore<B>
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
    B::Block: Send + Sync,
{
    pub fn new(underlying_store: AsyncDropGuard<B>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            underlying_store,
            counts: Arc::new(Mutex::new(ActionCounts::ZERO)),
        })
    }

    pub fn counts(&self) -> ActionCounts {
        *self.counts.lock().unwrap()
    }

    pub fn get_and_reset_counts(&self) -> ActionCounts {
        std::mem::replace(&mut self.counts.lock().unwrap(), ActionCounts::ZERO)
    }
}

#[async_trait]
impl<B> BlockStore for TrackingBlockStore<B>
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
    B::Block: Send + Sync,
{
    type Block = TrackingBlock<B::Block>;

    async fn load(&self, block_id: BlockId) -> Result<Option<Self::Block>> {
        self.counts.lock().unwrap().store_load += 1;
        let result = self.underlying_store.load(block_id).await?;
        Ok(result.map(|block| TrackingBlock::new(block, Arc::clone(&self.counts))))
    }

    async fn try_create(&self, block_id: &BlockId, data: &Data) -> Result<TryCreateResult> {
        self.counts.lock().unwrap().store_try_create += 1;
        self.underlying_store.try_create(block_id, data).await
    }

    async fn overwrite(&self, block_id: &BlockId, data: &Data) -> Result<()> {
        self.counts.lock().unwrap().store_overwrite += 1;
        self.underlying_store.overwrite(block_id, data).await
    }

    async fn remove_by_id(&self, block_id: &BlockId) -> Result<RemoveResult> {
        self.counts.lock().unwrap().store_remove_by_id += 1;
        self.underlying_store.remove_by_id(block_id).await
    }

    async fn remove(&self, block: Self::Block) -> Result<()> {
        self.counts.lock().unwrap().store_remove += 1;
        self.underlying_store.remove(block.into_inner()).await
    }

    async fn num_blocks(&self) -> Result<u64> {
        self.counts.lock().unwrap().store_num_blocks += 1;
        self.underlying_store.num_blocks().await
    }

    fn estimate_num_free_bytes(&self) -> Result<Byte> {
        self.counts.lock().unwrap().store_estimate_num_free_bytes += 1;
        self.underlying_store.estimate_num_free_bytes()
    }

    fn block_size_from_physical_block_size(
        &self,
        block_size: Byte,
    ) -> Result<Byte, InvalidBlockSizeError> {
        self.counts
            .lock()
            .unwrap()
            .store_block_size_from_physical_block_size += 1;
        self.underlying_store
            .block_size_from_physical_block_size(block_size)
    }

    async fn all_blocks(&self) -> Result<BoxStream<'static, Result<BlockId>>> {
        self.counts.lock().unwrap().store_all_blocks += 1;
        self.underlying_store.all_blocks().await
    }

    async fn create(&self, data: &Data) -> Result<BlockId> {
        let block_id = self.underlying_store.create(data).await?;
        self.counts.lock().unwrap().store_create += 1;
        Ok(block_id)
    }

    async fn flush_block(&self, block: &mut Self::Block) -> Result<()> {
        self.counts.lock().unwrap().store_flush_block += 1;
        self.underlying_store.flush_block(block.inner_mut()).await
    }

    #[cfg(any(test, feature = "testutils"))]
    async fn clear_cache_slow(&self) -> Result<()> {
        self.underlying_store.clear_cache_slow().await
    }

    #[cfg(any(test, feature = "testutils"))]
    async fn clear_unloaded_blocks_from_cache(&self) -> Result<()> {
        self.underlying_store
            .clear_unloaded_blocks_from_cache()
            .await
    }
}

#[async_trait]
impl<B> AsyncDrop for TrackingBlockStore<B>
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
    B::Block: Send + Sync,
{
    type Error = <B as AsyncDrop>::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.underlying_store.async_drop().await?;

        Ok(())
    }
}
