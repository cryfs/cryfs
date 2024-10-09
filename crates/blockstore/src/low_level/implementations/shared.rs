use anyhow::Result;
use async_trait::async_trait;
use futures::stream::BoxStream;
use std::fmt::Debug;
use std::ops::Deref;

use crate::{
    low_level::{
        BlockStore, BlockStoreDeleter, BlockStoreReader, InvalidBlockSizeError,
        OptimizedBlockStoreWriter,
    },
    BlockId, RemoveResult, TryCreateResult,
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard},
    data::Data,
};

/// Wraps a BlockStore into an Arc and allows cloning it to different owners.
/// Mostly useful for test cases that need to manipulate the underlying data of
/// a block to test a block store working on top of it. We're restricting this
/// to test code since it's questionable to use it in production code where
/// it could break abstraction layers.
#[cfg(any(test, feature = "testutils"))]
#[derive(Debug)]
pub struct SharedBlockStore<B: Debug + Sync + Send + AsyncDrop<Error = anyhow::Error>> {
    underlying_store: AsyncDropGuard<AsyncDropArc<B>>,
}

impl<B: Debug + Sync + Send + AsyncDrop<Error = anyhow::Error>> SharedBlockStore<B> {
    pub fn new(underlying: AsyncDropGuard<B>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            underlying_store: AsyncDropArc::new(underlying),
        })
    }

    pub fn clone(this: &AsyncDropGuard<Self>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            underlying_store: AsyncDropArc::clone(&this.underlying_store),
        })
    }
}

#[async_trait]
impl<B: BlockStoreReader + Debug + Sync + Send + AsyncDrop<Error = anyhow::Error>> BlockStoreReader
    for SharedBlockStore<B>
{
    async fn exists(&self, id: &BlockId) -> Result<bool> {
        self.underlying_store.exists(id).await
    }

    async fn load(&self, id: &BlockId) -> Result<Option<Data>> {
        self.underlying_store.load(id).await
    }

    async fn num_blocks(&self) -> Result<u64> {
        self.underlying_store.num_blocks().await
    }

    fn estimate_num_free_bytes(&self) -> Result<u64> {
        self.underlying_store.estimate_num_free_bytes()
    }

    fn block_size_from_physical_block_size(
        &self,
        block_size: u64,
    ) -> Result<u64, InvalidBlockSizeError> {
        self.underlying_store
            .block_size_from_physical_block_size(block_size)
    }

    async fn all_blocks(&self) -> Result<BoxStream<'static, Result<BlockId>>> {
        self.underlying_store.all_blocks().await
    }
}

#[async_trait]
impl<B: BlockStoreDeleter + Debug + Sync + Send + AsyncDrop<Error = anyhow::Error>>
    BlockStoreDeleter for SharedBlockStore<B>
{
    async fn remove(&self, id: &BlockId) -> Result<RemoveResult> {
        self.underlying_store.remove(id).await
    }
}

#[async_trait]
impl<B: OptimizedBlockStoreWriter + Debug + Sync + Send + AsyncDrop<Error = anyhow::Error>>
    OptimizedBlockStoreWriter for SharedBlockStore<B>
{
    type BlockData = B::BlockData;

    fn allocate(size: usize) -> Self::BlockData {
        B::allocate(size)
    }

    async fn try_create_optimized(
        &self,
        id: &BlockId,
        data: Self::BlockData,
    ) -> Result<TryCreateResult> {
        self.underlying_store.try_create_optimized(id, data).await
    }

    async fn store_optimized(&self, id: &BlockId, data: Self::BlockData) -> Result<()> {
        self.underlying_store.store_optimized(id, data).await
    }
}

#[async_trait]
impl<B: Sync + Send + Debug + AsyncDrop<Error = anyhow::Error>> AsyncDrop for SharedBlockStore<B> {
    type Error = anyhow::Error;
    async fn async_drop_impl(&mut self) -> Result<()> {
        self.underlying_store.async_drop().await?;
        Ok(())
    }
}

impl<B: BlockStore + OptimizedBlockStoreWriter + Sync + Send + Debug> BlockStore
    for SharedBlockStore<B>
{
}

impl<B: Debug + Sync + Send + AsyncDrop<Error = anyhow::Error>> Deref for SharedBlockStore<B> {
    type Target = B;

    fn deref(&self) -> &Self::Target {
        &self.underlying_store
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instantiate_blockstore_tests;
    use crate::low_level::InMemoryBlockStore;
    use crate::tests::Fixture;

    struct TestFixture {}
    #[async_trait]
    impl Fixture for TestFixture {
        type ConcreteBlockStore = SharedBlockStore<InMemoryBlockStore>;
        fn new() -> Self {
            Self {}
        }
        async fn store(&mut self) -> AsyncDropGuard<Self::ConcreteBlockStore> {
            SharedBlockStore::new(InMemoryBlockStore::new())
        }
        async fn yield_fixture(&self, _store: &Self::ConcreteBlockStore) {}
    }

    instantiate_blockstore_tests!(TestFixture, (flavor = "multi_thread"));

    #[tokio::test]
    async fn test_block_size_from_physical_block_size() {
        let mut fixture = TestFixture::new();
        let mut store = fixture.store().await;
        let expected_overhead: u64 = 0u64;

        assert_eq!(
            0u64,
            store
                .block_size_from_physical_block_size(expected_overhead)
                .unwrap()
        );
        assert_eq!(
            20u64,
            store
                .block_size_from_physical_block_size(expected_overhead + 20u64)
                .unwrap()
        );

        store.async_drop().await.unwrap();
    }
}
