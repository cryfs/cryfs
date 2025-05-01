use anyhow::Result;
use async_trait::async_trait;
use byte_unit::Byte;
use derive_more::{Add, AddAssign, Sum};
use futures::stream::BoxStream;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use super::tracking_block::TrackingBlock;
use crate::BlockStore;
use crate::{BlockId, InvalidBlockSizeError, RemoveResult, TryCreateResult};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};
use cryfs_utils::data::Data;

#[derive(Debug, Default, Add, AddAssign, Sum, PartialEq, Eq, Clone, Copy)]
pub struct ActionCounts {
    pub loaded: u32,
    pub read: u32,
    pub written: u32,
    pub overwritten: u32,
    pub created: u32,
    pub removed: u32,
    pub resized: u32,
    pub flushed: u32,
}

impl ActionCounts {
    pub const ZERO: Self = Self {
        loaded: 0,
        read: 0,
        written: 0,
        overwritten: 0,
        created: 0,
        removed: 0,
        resized: 0,
        flushed: 0,
    };
}

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
        std::mem::take(&mut self.counts.lock().unwrap())
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
        self.counts.lock().unwrap().loaded += 1;
        let result = self.underlying_store.load(block_id).await?;
        Ok(result.map(|block| TrackingBlock::new(block, Arc::clone(&self.counts))))
    }

    async fn try_create(&self, block_id: &BlockId, data: &Data) -> Result<TryCreateResult> {
        self.counts.lock().unwrap().created += 1;
        self.underlying_store.try_create(block_id, data).await
    }

    async fn overwrite(&self, block_id: &BlockId, data: &Data) -> Result<()> {
        self.counts.lock().unwrap().overwritten += 1;
        self.underlying_store.overwrite(block_id, data).await
    }

    async fn remove_by_id(&self, block_id: &BlockId) -> Result<RemoveResult> {
        self.counts.lock().unwrap().removed += 1;
        self.underlying_store.remove_by_id(block_id).await
    }

    async fn remove(&self, block: Self::Block) -> Result<()> {
        self.counts.lock().unwrap().removed += 1;
        self.underlying_store.remove(block.into_inner()).await
    }

    async fn num_blocks(&self) -> Result<u64> {
        self.underlying_store.num_blocks().await
    }

    fn estimate_num_free_bytes(&self) -> Result<Byte> {
        self.underlying_store.estimate_num_free_bytes()
    }

    fn block_size_from_physical_block_size(
        &self,
        block_size: Byte,
    ) -> Result<Byte, InvalidBlockSizeError> {
        self.underlying_store
            .block_size_from_physical_block_size(block_size)
    }

    async fn all_blocks(&self) -> Result<BoxStream<'static, Result<BlockId>>> {
        self.underlying_store.all_blocks().await
    }

    async fn create(&self, data: &Data) -> Result<BlockId> {
        let block_id = self.underlying_store.create(data).await?;
        self.counts.lock().unwrap().created += 1;
        Ok(block_id)
    }

    async fn flush_block(&self, block: &mut Self::Block) -> Result<()> {
        self.counts.lock().unwrap().flushed += 1;
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
