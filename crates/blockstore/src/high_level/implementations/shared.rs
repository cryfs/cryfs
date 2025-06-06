use anyhow::Result;
use async_trait::async_trait;
use byte_unit::Byte;
use futures::stream::BoxStream;
use std::fmt::Debug;
use std::ops::Deref;

use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};
use cryfs_utils::data::Data;

use crate::{BlockId, BlockStore, Overhead, RemoveResult, TryCreateResult};

#[derive(Debug)]
pub struct SharedBlockStore<B>
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
    B::Block: Send,
{
    underlying_store: AsyncDropGuard<AsyncDropArc<B>>,
}

impl<B> SharedBlockStore<B>
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
    B::Block: Send,
{
    pub fn new(underlying_store: AsyncDropGuard<B>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            underlying_store: AsyncDropArc::new(underlying_store),
        })
    }

    pub fn clone(this: &AsyncDropGuard<Self>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            underlying_store: AsyncDropArc::clone(&this.underlying_store),
        })
    }
}

#[async_trait]
impl<B> BlockStore for SharedBlockStore<B>
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
    B::Block: Send,
{
    type Block = B::Block;

    async fn load(&self, block_id: BlockId) -> Result<Option<Self::Block>> {
        self.underlying_store.load(block_id).await
    }

    async fn try_create(&self, block_id: &BlockId, data: &Data) -> Result<TryCreateResult> {
        self.underlying_store.try_create(block_id, data).await
    }

    async fn overwrite(&self, block_id: &BlockId, data: &Data) -> Result<()> {
        self.underlying_store.overwrite(block_id, data).await
    }

    async fn remove_by_id(&self, block_id: &BlockId) -> Result<RemoveResult> {
        self.underlying_store.remove_by_id(block_id).await
    }

    async fn remove(&self, block: Self::Block) -> Result<()> {
        self.underlying_store.remove(block).await
    }

    async fn num_blocks(&self) -> Result<u64> {
        self.underlying_store.num_blocks().await
    }

    fn estimate_num_free_bytes(&self) -> Result<Byte> {
        self.underlying_store.estimate_num_free_bytes()
    }

    fn overhead(&self) -> Overhead {
        self.underlying_store.overhead()
    }

    async fn all_blocks(&self) -> Result<BoxStream<'static, Result<BlockId>>> {
        self.underlying_store.all_blocks().await
    }

    async fn create(&self, data: &Data) -> Result<BlockId> {
        self.underlying_store.create(data).await
    }

    async fn flush_block(&self, block: &mut Self::Block) -> Result<()> {
        self.underlying_store.flush_block(block).await
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

impl<B> Deref for SharedBlockStore<B>
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
    B::Block: Send,
{
    type Target = B;

    fn deref(&self) -> &Self::Target {
        &self.underlying_store
    }
}

#[async_trait]
impl<B> AsyncDrop for SharedBlockStore<B>
where
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
    B::Block: Send,
{
    type Error = <B as AsyncDrop>::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.underlying_store.async_drop().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::instantiate_blockstore_tests_for_highlevel_blockstore;
    use crate::{InMemoryBlockStore, LockingBlockStore, tests::high_level::HLFixture};

    struct TestFixture<const FLUSH_CACHE_ON_YIELD: bool> {}
    #[async_trait]
    impl<const FLUSH_CACHE_ON_YIELD: bool> HLFixture for TestFixture<FLUSH_CACHE_ON_YIELD> {
        type ConcreteBlockStore = SharedBlockStore<LockingBlockStore<InMemoryBlockStore>>;
        fn new() -> Self {
            Self {}
        }
        async fn store(&mut self) -> AsyncDropGuard<Self::ConcreteBlockStore> {
            SharedBlockStore::new(LockingBlockStore::new(InMemoryBlockStore::new()))
        }
        async fn yield_fixture(&self, store: &Self::ConcreteBlockStore) {
            if FLUSH_CACHE_ON_YIELD {
                store.clear_cache_slow().await.unwrap();
            }
        }
    }

    mod with_flushing {
        use super::*;
        instantiate_blockstore_tests_for_highlevel_blockstore!(
            TestFixture<true>,
            (flavor = "multi_thread")
        );
    }

    mod without_flushing {
        use super::*;
        instantiate_blockstore_tests_for_highlevel_blockstore!(
            TestFixture<false>,
            (flavor = "multi_thread")
        );
    }
}
