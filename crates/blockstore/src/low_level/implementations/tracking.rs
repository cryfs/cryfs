use anyhow::Result;
use async_trait::async_trait;
use byte_unit::Byte;
use derive_more::{Add, AddAssign, Sum};
use futures::stream::BoxStream;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Mutex;

use crate::{
    low_level::{
        BlockStore, BlockStoreDeleter, BlockStoreReader, InvalidBlockSizeError,
        OptimizedBlockStoreWriter,
    },
    BlockId, RemoveResult, TryCreateResult,
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    data::Data,
};

#[derive(Debug, Default, Add, AddAssign, Sum, PartialEq, Eq, Clone, Copy)]
pub struct ActionCounts {
    pub exists: u32,
    pub loaded: u32,
    pub stored: u32,
    pub removed: u32,
    pub created: u32,
}

/// Wraps a BlockStore into a block store that counts the number of loaded, resized, written, ... blocks.
/// It is used for testing that operations only access few blocks (performance tests).
#[derive(Debug)]
pub struct TrackingBlockStore<B: Debug + Sync + Send + AsyncDrop<Error = anyhow::Error>> {
    underlying_store: AsyncDropGuard<B>,

    // TODO Do we even need counts_for_block or do we only need totals and can simplify the whole class?
    counts: Mutex<HashMap<BlockId, ActionCounts>>,
}

impl<B: Debug + Sync + Send + AsyncDrop<Error = anyhow::Error>> TrackingBlockStore<B> {
    pub fn new(underlying: AsyncDropGuard<B>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            underlying_store: underlying,
            counts: Mutex::new(HashMap::new()),
        })
    }

    pub fn counts_for_block(&self, block_id: BlockId) -> ActionCounts {
        *self.counts.lock().unwrap().entry(block_id).or_default()
    }

    pub fn totals(&self) -> ActionCounts {
        self.counts.lock().unwrap().values().copied().sum()
    }

    pub fn get_and_reset_totals(&self) -> ActionCounts {
        self.counts.lock().unwrap().drain().map(|(_, v)| v).sum()
    }
}

#[async_trait]
impl<B: BlockStoreReader + Debug + Sync + Send + AsyncDrop<Error = anyhow::Error>> BlockStoreReader
    for TrackingBlockStore<B>
{
    async fn exists(&self, id: &BlockId) -> Result<bool> {
        self.counts.lock().unwrap().entry(*id).or_default().exists += 1;
        self.underlying_store.exists(id).await
    }

    async fn load(&self, id: &BlockId) -> Result<Option<Data>> {
        self.counts.lock().unwrap().entry(*id).or_default().loaded += 1;
        self.underlying_store.load(id).await
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
}

#[async_trait]
impl<B: BlockStoreDeleter + Debug + Sync + Send + AsyncDrop<Error = anyhow::Error>>
    BlockStoreDeleter for TrackingBlockStore<B>
{
    async fn remove(&self, id: &BlockId) -> Result<RemoveResult> {
        self.counts.lock().unwrap().entry(*id).or_default().removed += 1;
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
        self.counts.lock().unwrap().entry(*id).or_default().created += 1;
        self.underlying_store.try_create_optimized(id, data).await
    }

    async fn store_optimized(&self, id: &BlockId, data: Self::BlockData) -> Result<()> {
        self.counts.lock().unwrap().entry(*id).or_default().stored += 1;
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

        assert_eq!(
            ActionCounts {
                exists: 0,
                created: 0,
                stored: 0,
                loaded: 0,
                removed: 0,
            },
            store.totals(),
        );

        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn exists_increases_counter() {
        let mut fixture = TestFixture::new();
        let mut store = fixture.store().await;

        let id1 = BlockId::from_hex("715db62b0b4e333f8b16c76ee886c95b").unwrap();
        let id2 = BlockId::from_hex("62b0b4e333f8b16c76ee886c95b715db").unwrap();

        assert_eq!(
            TryCreateResult::SuccessfullyCreated,
            store.try_create(&id1, &[1, 2, 3]).await.unwrap()
        );

        assert_eq!(true, store.exists(&id1).await.unwrap());
        assert_eq!(false, store.exists(&id2).await.unwrap());

        assert_eq!(
            ActionCounts {
                exists: 1,
                created: 1,
                ..ActionCounts::default()
            },
            store.counts_for_block(id1)
        );
        assert_eq!(
            ActionCounts {
                exists: 1,
                created: 0,
                ..ActionCounts::default()
            },
            store.counts_for_block(id2)
        );
        assert_eq!(
            ActionCounts {
                exists: 2,
                created: 1,
                ..ActionCounts::default()
            },
            store.totals()
        );

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

        assert_eq!(
            ActionCounts {
                loaded: 3,
                created: 1,
                ..ActionCounts::default()
            },
            store.counts_for_block(id1)
        );
        assert_eq!(
            ActionCounts {
                loaded: 1,
                created: 1,
                ..ActionCounts::default()
            },
            store.counts_for_block(id2)
        );
        assert_eq!(
            ActionCounts {
                loaded: 4,
                created: 2,
                ..ActionCounts::default()
            },
            store.totals()
        );

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

        assert_eq!(
            ActionCounts {
                stored: 3,
                removed: 1,
                ..ActionCounts::default()
            },
            store.counts_for_block(id1)
        );
        assert_eq!(
            ActionCounts {
                stored: 1,
                ..ActionCounts::default()
            },
            store.counts_for_block(id2)
        );
        assert_eq!(
            ActionCounts {
                stored: 4,
                removed: 1,
                ..ActionCounts::default()
            },
            store.totals()
        );

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

        assert_eq!(
            ActionCounts {
                stored: 3,
                removed: 1,
                ..ActionCounts::default()
            },
            store.counts_for_block(id1)
        );
        assert_eq!(
            ActionCounts {
                stored: 1,
                ..ActionCounts::default()
            },
            store.counts_for_block(id2)
        );
        assert_eq!(
            ActionCounts {
                stored: 4,
                removed: 1,
                ..ActionCounts::default()
            },
            store.totals()
        );

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

        assert_eq!(
            ActionCounts {
                removed: 4,
                created: 2,
                ..ActionCounts::default()
            },
            store.counts_for_block(id1)
        );
        assert_eq!(
            ActionCounts {
                removed: 1,
                created: 1,
                ..ActionCounts::default()
            },
            store.counts_for_block(id2)
        );
        assert_eq!(
            ActionCounts {
                removed: 5,
                created: 3,
                ..ActionCounts::default()
            },
            store.totals()
        );

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

        assert_eq!(
            ActionCounts {
                created: 3,
                removed: 1,
                ..ActionCounts::default()
            },
            store.counts_for_block(id1)
        );
        assert_eq!(
            ActionCounts {
                created: 1,
                ..ActionCounts::default()
            },
            store.counts_for_block(id2)
        );
        assert_eq!(
            ActionCounts {
                created: 4,
                removed: 1,
                ..ActionCounts::default()
            },
            store.totals()
        );

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

        assert_eq!(
            ActionCounts {
                created: 3,
                removed: 1,
                ..ActionCounts::default()
            },
            store.counts_for_block(id1)
        );
        assert_eq!(
            ActionCounts {
                created: 1,
                ..ActionCounts::default()
            },
            store.counts_for_block(id2)
        );
        assert_eq!(
            ActionCounts {
                created: 4,
                removed: 1,
                ..ActionCounts::default()
            },
            store.totals()
        );

        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_get_and_reset_totals() {
        let mut fixture = TestFixture::new();
        let mut store = fixture.store().await;

        let id1 = BlockId::from_hex("715db62b0b4e333f8b16c76ee886c95b").unwrap();
        let id2 = BlockId::from_hex("62b0b4e333f8b16c76ee886c95b715db").unwrap();

        for _ in 0..2 {
            let _ = store.try_create(&id1, &[1, 2, 3]).await.unwrap();
        }
        for _ in 0..3 {
            let _ = store.remove(&id1).await.unwrap();
        }
        for _ in 0..4 {
            store.store(&id1, &[1, 2, 3]).await.unwrap();
        }
        for _ in 0..5 {
            let _ = store.load(&id1).await.unwrap();
        }
        for _ in 0..6 {
            let _ = store.try_create(&id2, &[1, 2, 3]).await.unwrap();
        }
        for _ in 0..7 {
            let _ = store.remove(&id2).await.unwrap();
        }
        for _ in 0..8 {
            store.store(&id2, &[1, 2, 3]).await.unwrap();
        }
        for _ in 0..9 {
            let _ = store.load(&id2).await.unwrap();
        }

        assert_eq!(
            ActionCounts {
                exists: 0,
                created: 8,
                removed: 10,
                stored: 12,
                loaded: 14,
            },
            store.get_and_reset_totals(),
        );
        assert_eq!(
            ActionCounts {
                exists: 0,
                created: 0,
                removed: 0,
                stored: 0,
                loaded: 0,
            },
            store.get_and_reset_totals(),
        );

        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_block_size_from_physical_block_size() {
        let mut fixture = TestFixture::new();
        let mut store = fixture.store().await;
        let expected_overhead = Byte::from_u64(0);

        assert_eq!(
            Byte::from_u64(0),
            store
                .block_size_from_physical_block_size(expected_overhead)
                .unwrap()
        );
        assert_eq!(
            Byte::from_u64(20),
            store
                .block_size_from_physical_block_size(
                    expected_overhead.add(Byte::from_u64(20)).unwrap()
                )
                .unwrap()
        );

        store.async_drop().await.unwrap();
    }
}
