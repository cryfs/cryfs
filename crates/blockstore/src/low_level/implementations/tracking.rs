use anyhow::Result;
use async_trait::async_trait;
use byte_unit::Byte;
use derive_more::{Add, AddAssign, Sum};
use futures::stream::BoxStream;
use std::fmt::Debug;
use std::sync::Mutex;

use crate::{
    BlockId, RemoveResult, TryCreateResult,
    low_level::{
        BlockStoreDeleter, BlockStoreReader, InvalidBlockSizeError, LLBlockStore,
        OptimizedBlockStoreWriter,
    },
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    data::Data,
};

#[derive(Debug, Add, AddAssign, Sum, PartialEq, Eq, Clone, Copy)]
pub struct ActionCounts {
    pub exists: u32,
    pub load: u32,
    pub num_blocks: u32,
    pub estimate_num_free_bytes: u32,
    pub usable_block_size_from_physical_block_size: u32,
    pub all_blocks: u32,
    pub remove: u32,
    pub try_create: u32,
    pub store: u32,
}

impl ActionCounts {
    pub const ZERO: Self = Self {
        exists: 0,
        load: 0,
        num_blocks: 0,
        estimate_num_free_bytes: 0,
        usable_block_size_from_physical_block_size: 0,
        all_blocks: 0,
        remove: 0,
        try_create: 0,
        store: 0,
    };
}

/// Wraps a BlockStore into a block store that counts the number of loaded, resized, written, ... blocks.
/// It is used for testing that operations only access few blocks (performance tests).
#[derive(Debug)]
pub struct TrackingBlockStore<B: Debug + Sync + Send + AsyncDrop<Error = anyhow::Error>> {
    underlying_store: AsyncDropGuard<B>,

    counts: Mutex<ActionCounts>,
}

impl<B: Debug + Sync + Send + AsyncDrop<Error = anyhow::Error>> TrackingBlockStore<B> {
    pub fn new(underlying: AsyncDropGuard<B>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            underlying_store: underlying,
            counts: Mutex::new(ActionCounts::ZERO),
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
impl<B: BlockStoreReader + Debug + Sync + Send + AsyncDrop<Error = anyhow::Error>> BlockStoreReader
    for TrackingBlockStore<B>
{
    async fn exists(&self, id: &BlockId) -> Result<bool> {
        self.counts.lock().unwrap().exists += 1;
        self.underlying_store.exists(id).await
    }

    async fn load(&self, id: &BlockId) -> Result<Option<Data>> {
        self.counts.lock().unwrap().load += 1;
        self.underlying_store.load(id).await
    }

    async fn num_blocks(&self) -> Result<u64> {
        self.counts.lock().unwrap().num_blocks += 1;
        self.underlying_store.num_blocks().await
    }

    fn estimate_num_free_bytes(&self) -> Result<Byte> {
        self.counts.lock().unwrap().estimate_num_free_bytes += 1;
        self.underlying_store.estimate_num_free_bytes()
    }

    fn usable_block_size_from_physical_block_size(
        &self,
        block_size: Byte,
    ) -> Result<Byte, InvalidBlockSizeError> {
        self.counts
            .lock()
            .unwrap()
            .usable_block_size_from_physical_block_size += 1;
        self.underlying_store
            .usable_block_size_from_physical_block_size(block_size)
    }

    async fn all_blocks(&self) -> Result<BoxStream<'static, Result<BlockId>>> {
        self.counts.lock().unwrap().all_blocks += 1;
        self.underlying_store.all_blocks().await
    }
}

#[async_trait]
impl<B: BlockStoreDeleter + Debug + Sync + Send + AsyncDrop<Error = anyhow::Error>>
    BlockStoreDeleter for TrackingBlockStore<B>
{
    async fn remove(&self, id: &BlockId) -> Result<RemoveResult> {
        self.counts.lock().unwrap().remove += 1;
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
        self.counts.lock().unwrap().try_create += 1;
        self.underlying_store.try_create_optimized(id, data).await
    }

    async fn store_optimized(&self, id: &BlockId, data: Self::BlockData) -> Result<()> {
        self.counts.lock().unwrap().store += 1;
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

impl<B: LLBlockStore + OptimizedBlockStoreWriter + Sync + Send + Debug> LLBlockStore
    for TrackingBlockStore<B>
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instantiate_blockstore_tests_for_lowlevel_blockstore;
    use crate::low_level::{BlockStoreWriter, InMemoryBlockStore};
    use crate::tests::low_level::LLFixture;

    struct TestFixture {}
    #[async_trait]
    impl LLFixture for TestFixture {
        type ConcreteBlockStore = TrackingBlockStore<InMemoryBlockStore>;
        fn new() -> Self {
            Self {}
        }
        async fn store(&mut self) -> AsyncDropGuard<Self::ConcreteBlockStore> {
            TrackingBlockStore::new(InMemoryBlockStore::new())
        }
        async fn yield_fixture(&self, _store: &Self::ConcreteBlockStore) {}
    }

    instantiate_blockstore_tests_for_lowlevel_blockstore!(TestFixture, (flavor = "multi_thread"));

    #[tokio::test]
    async fn counters_start_at_zero() {
        let mut fixture = TestFixture::new();
        let mut store = fixture.store().await;

        assert_eq!(
            ActionCounts {
                exists: 0,
                load: 0,
                num_blocks: 0,
                estimate_num_free_bytes: 0,
                usable_block_size_from_physical_block_size: 0,
                all_blocks: 0,
                remove: 0,
                try_create: 0,
                store: 0,
            },
            store.counts(),
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
                exists: 2,
                try_create: 1,
                ..ActionCounts::ZERO
            },
            store.counts()
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
                load: 4,
                try_create: 2,
                ..ActionCounts::ZERO
            },
            store.counts()
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
                store: 4,
                remove: 1,
                ..ActionCounts::ZERO
            },
            store.counts()
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
                store: 4,
                remove: 1,
                ..ActionCounts::ZERO
            },
            store.counts()
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
                remove: 5,
                try_create: 3,
                ..ActionCounts::ZERO
            },
            store.counts()
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
                try_create: 4,
                remove: 1,
                ..ActionCounts::ZERO
            },
            store.counts()
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
                try_create: 4,
                remove: 1,
                ..ActionCounts::ZERO
            },
            store.counts()
        );

        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_usable_block_size_from_physical_block_size() {
        let mut fixture = TestFixture::new();
        let mut store = fixture.store().await;
        let expected_overhead = Byte::from_u64(0);

        assert_eq!(
            Byte::from_u64(0),
            store
                .usable_block_size_from_physical_block_size(expected_overhead)
                .unwrap()
        );
        assert_eq!(
            Byte::from_u64(20),
            store
                .usable_block_size_from_physical_block_size(
                    expected_overhead.add(Byte::from_u64(20)).unwrap()
                )
                .unwrap()
        );

        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn num_blocks_increases_counter() {
        let mut fixture = TestFixture::new();
        let mut store = fixture.store().await;

        // Create some blocks to make the test more meaningful
        let id1 = BlockId::from_hex("715db62b0b4e333f8b16c76ee886c95b").unwrap();
        let id2 = BlockId::from_hex("62b0b4e333f8b16c76ee886c95b715db").unwrap();
        let _ = store.try_create(&id1, &[1, 2, 3]).await.unwrap();
        let _ = store.try_create(&id2, &[4, 5, 6]).await.unwrap();

        // Call num_blocks multiple times
        store.num_blocks().await.unwrap();
        store.num_blocks().await.unwrap();
        store.num_blocks().await.unwrap();

        assert_eq!(
            ActionCounts {
                num_blocks: 3,
                try_create: 2,
                ..ActionCounts::ZERO
            },
            store.counts()
        );

        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn estimate_num_free_bytes_increases_counter() {
        let mut fixture = TestFixture::new();
        let mut store = fixture.store().await;

        // Call estimate_num_free_bytes multiple times
        store.estimate_num_free_bytes().unwrap();
        store.estimate_num_free_bytes().unwrap();
        store.estimate_num_free_bytes().unwrap();
        store.estimate_num_free_bytes().unwrap();

        assert_eq!(
            ActionCounts {
                estimate_num_free_bytes: 4,
                ..ActionCounts::ZERO
            },
            store.counts()
        );

        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn usable_block_size_from_physical_block_size_increases_counter() {
        let mut fixture = TestFixture::new();
        let mut store = fixture.store().await;

        // Call usable_block_size_from_physical_block_size multiple times
        store
            .usable_block_size_from_physical_block_size(Byte::from_u64(1024))
            .unwrap();
        store
            .usable_block_size_from_physical_block_size(Byte::from_u64(2048))
            .unwrap();
        store
            .usable_block_size_from_physical_block_size(Byte::from_u64(4096))
            .unwrap();

        assert_eq!(
            ActionCounts {
                usable_block_size_from_physical_block_size: 3,
                ..ActionCounts::ZERO
            },
            store.counts()
        );

        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn all_blocks_increases_counter() {
        let mut fixture = TestFixture::new();
        let mut store = fixture.store().await;

        // Create some blocks to make the test more meaningful
        let id1 = BlockId::from_hex("715db62b0b4e333f8b16c76ee886c95b").unwrap();
        let id2 = BlockId::from_hex("62b0b4e333f8b16c76ee886c95b715db").unwrap();
        let _ = store.try_create(&id1, &[1, 2, 3]).await.unwrap();
        let _ = store.try_create(&id2, &[4, 5, 6]).await.unwrap();

        // Call all_blocks multiple times and execute the streams to completion
        let s1 = store.all_blocks().await.unwrap();
        let _ = futures::StreamExt::collect::<Vec<_>>(s1).await;
        let s2 = store.all_blocks().await.unwrap();
        let _ = futures::StreamExt::collect::<Vec<_>>(s2).await;

        assert_eq!(
            ActionCounts {
                all_blocks: 2,
                try_create: 2,
                ..ActionCounts::ZERO
            },
            store.counts()
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

        for _ in 0..3 {
            let _ = store.exists(&id1).await.unwrap();
        }

        for _ in 0..2 {
            let _ = store.num_blocks().await.unwrap();
        }

        for _ in 0..4 {
            let _ = store.estimate_num_free_bytes().unwrap();
        }

        for _ in 0..3 {
            let _ = store
                .usable_block_size_from_physical_block_size(Byte::from_u64(1024))
                .unwrap();
        }

        for _ in 0..2 {
            let s = store.all_blocks().await.unwrap();
            let _ = futures::StreamExt::collect::<Vec<_>>(s).await;
        }

        assert_eq!(
            ActionCounts {
                exists: 3,
                try_create: 8,
                remove: 10,
                store: 12,
                load: 14,
                num_blocks: 2,
                estimate_num_free_bytes: 4,
                usable_block_size_from_physical_block_size: 3,
                all_blocks: 2,
            },
            store.get_and_reset_counts(),
        );

        assert_eq!(ActionCounts::ZERO, store.get_and_reset_counts(),);

        store.async_drop().await.unwrap();
    }
}
