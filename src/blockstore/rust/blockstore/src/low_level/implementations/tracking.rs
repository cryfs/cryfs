use anyhow::Result;
use async_trait::async_trait;
use futures::stream::BoxStream;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Mutex;

use crate::{
    low_level::{BlockStore, BlockStoreDeleter, BlockStoreReader, OptimizedBlockStoreWriter},
    BlockId, RemoveResult, TryCreateResult,
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    data::Data,
};

#[derive(Debug, Default)]
pub struct ActionCount {
    count: Mutex<HashMap<BlockId, u32>>,
}

impl ActionCount {
    pub fn get(&self, id: &BlockId) -> u32 {
        *self.count.lock().unwrap().get(id).unwrap_or(&0)
    }

    pub fn total(&self) -> u32 {
        self.count.lock().unwrap().values().sum()
    }

    fn increment(&self, id: &BlockId) {
        let mut entry = self.count.lock().unwrap();
        let counter = entry.entry(id.clone()).or_insert(0);
        *counter += 1;
    }
}

/// Wraps a BlockStore into a block store that counts the number of loaded, resized, written, ... blocks.
/// It is used for testing that operations only access few blocks (performance tests).
#[derive(Debug)]
pub struct TrackingBlockStore<B: Debug + Sync + Send + AsyncDrop<Error = anyhow::Error>> {
    underlying_store: AsyncDropGuard<B>,

    loaded: ActionCount,
    written: ActionCount,
    removed: ActionCount,
    created: ActionCount,
}

impl<B: Debug + Sync + Send + AsyncDrop<Error = anyhow::Error>> TrackingBlockStore<B> {
    pub fn new(underlying: AsyncDropGuard<B>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            underlying_store: underlying,
            loaded: ActionCount::default(),
            written: ActionCount::default(),
            removed: ActionCount::default(),
            created: ActionCount::default(),
        })
    }

    pub fn count_loaded(&self) -> &ActionCount {
        &self.loaded
    }

    pub fn count_written(&self) -> &ActionCount {
        &self.written
    }

    pub fn count_removed(&self) -> &ActionCount {
        &self.removed
    }

    pub fn count_created(&self) -> &ActionCount {
        &self.created
    }
}

#[async_trait]
impl<B: BlockStoreReader + Debug + Sync + Send + AsyncDrop<Error = anyhow::Error>> BlockStoreReader
    for TrackingBlockStore<B>
{
    async fn exists(&self, id: &BlockId) -> Result<bool> {
        self.underlying_store.exists(id).await
    }

    async fn load(&self, id: &BlockId) -> Result<Option<Data>> {
        self.loaded.increment(id);
        self.underlying_store.load(id).await
    }

    async fn num_blocks(&self) -> Result<u64> {
        self.underlying_store.num_blocks().await
    }

    fn estimate_num_free_bytes(&self) -> Result<u64> {
        self.underlying_store.estimate_num_free_bytes()
    }

    fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64> {
        self.underlying_store
            .block_size_from_physical_block_size(block_size)
    }

    async fn all_blocks(&self) -> Result<BoxStream<'static, Result<BlockId>>> {
        self.underlying_store.all_blocks().await
    }
}

#[async_trait]
impl<B: BlockStoreDeleter + Debug + Sync + Send + AsyncDrop<Error = anyhow::Error>>
    BlockStoreDeleter for TrackingBlockStore<B>
{
    async fn remove(&self, id: &BlockId) -> Result<RemoveResult> {
        self.removed.increment(id);
        self.underlying_store.remove(id).await
    }
}

#[async_trait]
impl<B: OptimizedBlockStoreWriter + Debug + Sync + Send + AsyncDrop<Error = anyhow::Error>>
    OptimizedBlockStoreWriter for TrackingBlockStore<B>
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
        self.created.increment(id);
        self.underlying_store.try_create_optimized(id, data).await
    }

    async fn store_optimized(&self, id: &BlockId, data: Self::BlockData) -> Result<()> {
        self.written.increment(id);
        self.underlying_store.store_optimized(id, data).await
    }
}

#[async_trait]
impl<B: Sync + Send + Debug + AsyncDrop<Error = anyhow::Error>> AsyncDrop
    for TrackingBlockStore<B>
{
    type Error = anyhow::Error;
    async fn async_drop_impl(&mut self) -> Result<()> {
        self.underlying_store.async_drop().await?;
        Ok(())
    }
}

impl<B: BlockStore + OptimizedBlockStoreWriter + Sync + Send + Debug> BlockStore
    for TrackingBlockStore<B>
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instantiate_blockstore_tests;
    use crate::low_level::{BlockStoreWriter, InMemoryBlockStore};
    use crate::tests::Fixture;

    struct TestFixture {}
    #[async_trait]
    impl Fixture for TestFixture {
        type ConcreteBlockStore = TrackingBlockStore<InMemoryBlockStore>;
        fn new() -> Self {
            Self {}
        }
        async fn store(&mut self) -> AsyncDropGuard<Self::ConcreteBlockStore> {
            TrackingBlockStore::new(InMemoryBlockStore::new())
        }
        async fn yield_fixture(&self, _store: &Self::ConcreteBlockStore) {}
    }

    instantiate_blockstore_tests!(TestFixture, (flavor = "multi_thread"));

    #[tokio::test]
    async fn counters_start_at_zero() {
        let mut fixture = TestFixture::new();
        let mut store = fixture.store().await;

        assert_eq!(0, store.count_loaded().total());
        assert_eq!(0, store.count_written().total());
        assert_eq!(0, store.count_removed().total());
        assert_eq!(0, store.count_created().total());

        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn load_increases_counter() {
        let mut fixture = TestFixture::new();
        let mut store = fixture.store().await;

        let id1 = BlockId::from_hex("715db62b0b4e333f8b16c76ee886c95b").unwrap();
        let id2 = BlockId::from_hex("62b0b4e333f8b16c76ee886c95b715db").unwrap();

        assert_eq!(
            TryCreateResult::SuccessfullyCreated,
            store.try_create(&id1, &[1, 2, 3]).await.unwrap()
        );
        assert_eq!(
            TryCreateResult::SuccessfullyCreated,
            store.try_create(&id2, &[1, 2, 3]).await.unwrap()
        );

        store.load(&id1).await.unwrap();
        store.load(&id1).await.unwrap();
        store.load(&id1).await.unwrap();
        store.load(&id2).await.unwrap();

        assert_eq!(3, store.count_loaded().get(&id1));
        assert_eq!(1, store.count_loaded().get(&id2));
        assert_eq!(4, store.count_loaded().total());

        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn store_increases_counter() {
        let mut fixture = TestFixture::new();
        let mut store = fixture.store().await;

        let id1 = BlockId::from_hex("715db62b0b4e333f8b16c76ee886c95b").unwrap();
        let id2 = BlockId::from_hex("62b0b4e333f8b16c76ee886c95b715db").unwrap();

        store.store(&id1, &[1, 2, 3]).await.unwrap();
        store.store(&id1, &[1, 2, 3]).await.unwrap();
        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&id1).await.unwrap()
        );
        store.store(&id1, &[1, 2, 3]).await.unwrap();
        store.store(&id2, &[1, 2, 3]).await.unwrap();

        assert_eq!(3, store.count_written().get(&id1));
        assert_eq!(1, store.count_written().get(&id2));
        assert_eq!(4, store.count_written().total());

        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn store_optimized_increases_counter() {
        let mut fixture = TestFixture::new();
        let mut store = fixture.store().await;

        let id1 = BlockId::from_hex("715db62b0b4e333f8b16c76ee886c95b").unwrap();
        let id2 = BlockId::from_hex("62b0b4e333f8b16c76ee886c95b715db").unwrap();

        store
            .store_optimized(&id1, TrackingBlockStore::<InMemoryBlockStore>::allocate(3))
            .await
            .unwrap();
        store
            .store_optimized(&id1, TrackingBlockStore::<InMemoryBlockStore>::allocate(3))
            .await
            .unwrap();
        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&id1).await.unwrap()
        );
        store
            .store_optimized(&id1, TrackingBlockStore::<InMemoryBlockStore>::allocate(3))
            .await
            .unwrap();
        store
            .store_optimized(&id2, TrackingBlockStore::<InMemoryBlockStore>::allocate(3))
            .await
            .unwrap();

        assert_eq!(3, store.count_written().get(&id1));
        assert_eq!(1, store.count_written().get(&id2));
        assert_eq!(4, store.count_written().total());

        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn remove_increases_counter() {
        let mut fixture = TestFixture::new();
        let mut store = fixture.store().await;

        let id1 = BlockId::from_hex("715db62b0b4e333f8b16c76ee886c95b").unwrap();
        let id2 = BlockId::from_hex("62b0b4e333f8b16c76ee886c95b715db").unwrap();

        // Check it also increases the counter when the block doesn't exist
        assert_eq!(
            RemoveResult::NotRemovedBecauseItDoesntExist,
            store.remove(&id1).await.unwrap()
        );

        assert_eq!(
            TryCreateResult::SuccessfullyCreated,
            store.try_create(&id1, &[1, 2, 3]).await.unwrap()
        );
        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&id1).await.unwrap()
        );
        assert_eq!(
            TryCreateResult::SuccessfullyCreated,
            store.try_create(&id1, &[1, 2, 3]).await.unwrap()
        );
        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&id1).await.unwrap()
        );
        assert_eq!(
            RemoveResult::NotRemovedBecauseItDoesntExist,
            store.remove(&id1).await.unwrap()
        );
        assert_eq!(
            TryCreateResult::SuccessfullyCreated,
            store.try_create(&id2, &[1, 2, 3]).await.unwrap()
        );
        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&id2).await.unwrap()
        );

        assert_eq!(4, store.count_removed().get(&id1));
        assert_eq!(1, store.count_removed().get(&id2));
        assert_eq!(5, store.count_removed().total());

        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn try_create_increases_counter() {
        let mut fixture = TestFixture::new();
        let mut store = fixture.store().await;

        let id1 = BlockId::from_hex("715db62b0b4e333f8b16c76ee886c95b").unwrap();
        let id2 = BlockId::from_hex("62b0b4e333f8b16c76ee886c95b715db").unwrap();

        assert_eq!(
            TryCreateResult::SuccessfullyCreated,
            store.try_create(&id1, &[1, 2, 3]).await.unwrap()
        );
        // check it increases the counter even when try_create fails
        assert_eq!(
            TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists,
            store.try_create(&id1, &[1, 2, 3]).await.unwrap()
        );
        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&id1).await.unwrap()
        );
        assert_eq!(
            TryCreateResult::SuccessfullyCreated,
            store.try_create(&id1, &[1, 2, 3]).await.unwrap()
        );

        assert_eq!(
            TryCreateResult::SuccessfullyCreated,
            store.try_create(&id2, &[1, 2, 3]).await.unwrap()
        );

        assert_eq!(3, store.count_created().get(&id1));
        assert_eq!(1, store.count_created().get(&id2));
        assert_eq!(4, store.count_created().total());

        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn try_create_optimized_increases_counter() {
        let mut fixture = TestFixture::new();
        let mut store = fixture.store().await;

        let id1 = BlockId::from_hex("715db62b0b4e333f8b16c76ee886c95b").unwrap();
        let id2 = BlockId::from_hex("62b0b4e333f8b16c76ee886c95b715db").unwrap();

        assert_eq!(
            TryCreateResult::SuccessfullyCreated,
            store
                .try_create_optimized(&id1, TrackingBlockStore::<InMemoryBlockStore>::allocate(3))
                .await
                .unwrap()
        );
        // check it increases the counter even when try_create_optimized fails
        assert_eq!(
            TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists,
            store
                .try_create_optimized(&id1, TrackingBlockStore::<InMemoryBlockStore>::allocate(3))
                .await
                .unwrap()
        );
        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&id1).await.unwrap()
        );
        assert_eq!(
            TryCreateResult::SuccessfullyCreated,
            store
                .try_create_optimized(&id1, TrackingBlockStore::<InMemoryBlockStore>::allocate(3))
                .await
                .unwrap()
        );

        assert_eq!(
            TryCreateResult::SuccessfullyCreated,
            store
                .try_create_optimized(&id2, TrackingBlockStore::<InMemoryBlockStore>::allocate(3))
                .await
                .unwrap()
        );

        assert_eq!(3, store.count_created().get(&id1));
        assert_eq!(1, store.count_created().get(&id2));
        assert_eq!(4, store.count_created().total());

        store.async_drop().await.unwrap();
    }

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
