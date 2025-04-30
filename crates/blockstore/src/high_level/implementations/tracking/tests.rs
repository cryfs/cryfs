use async_trait::async_trait;
use cryfs_utils::async_drop::AsyncDropGuard;
use cryfs_utils::data::Data;

use super::TrackingBlockStore;
use crate::high_level::implementations::tracking::tracking_blockstore::ActionCounts;
use crate::{
    Block, BlockId, BlockStore as _, instantiate_blockstore_tests_for_highlevel_blockstore,
};
use crate::{InMemoryBlockStore, LockingBlockStore, tests::high_level::HLFixture};

struct TestFixture<const FLUSH_CACHE_ON_YIELD: bool> {}
#[async_trait]
impl<const FLUSH_CACHE_ON_YIELD: bool> HLFixture for TestFixture<FLUSH_CACHE_ON_YIELD> {
    type ConcreteBlockStore = TrackingBlockStore<LockingBlockStore<InMemoryBlockStore>>;
    fn new() -> Self {
        Self {}
    }
    async fn store(&mut self) -> AsyncDropGuard<Self::ConcreteBlockStore> {
        TrackingBlockStore::new(LockingBlockStore::new(InMemoryBlockStore::new()))
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

#[tokio::test]
async fn counters_start_at_zero() {
    let mut fixture = TestFixture::<false>::new();
    let mut store = fixture.store().await;

    assert_eq!(
        ActionCounts {
            loaded: 0,
            read: 0,
            written: 0,
            overwritten: 0,
            created: 0,
            removed: 0,
            resized: 0,
            flushed: 0,
        },
        store.totals(),
    );

    store.async_drop().await.unwrap();
}

#[tokio::test]
async fn load_increases_counter() {
    let mut fixture = TestFixture::<false>::new();
    let mut store = fixture.store().await;

    let id1 = store.create(&Data::from(vec![1, 2, 3])).await.unwrap();
    let id2 = store.create(&Data::from(vec![4, 5, 6])).await.unwrap();

    store.load(id1).await.unwrap();
    store.load(id1).await.unwrap();
    store.load(id1).await.unwrap();
    store.load(id2).await.unwrap();

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
async fn overwrite_increases_counter() {
    let mut fixture = TestFixture::<false>::new();
    let mut store = fixture.store().await;

    let id1 = store.create(&Data::from(vec![1, 2, 3])).await.unwrap();
    let id2 = store.create(&Data::from(vec![4, 5, 6])).await.unwrap();

    store
        .overwrite(&id1, &Data::from(vec![7, 8, 9]))
        .await
        .unwrap();
    store
        .overwrite(&id1, &Data::from(vec![10, 11, 12]))
        .await
        .unwrap();
    store
        .overwrite(&id2, &Data::from(vec![13, 14, 15]))
        .await
        .unwrap();

    assert_eq!(
        ActionCounts {
            overwritten: 2,
            created: 1,
            ..ActionCounts::default()
        },
        store.counts_for_block(id1)
    );
    assert_eq!(
        ActionCounts {
            overwritten: 1,
            created: 1,
            ..ActionCounts::default()
        },
        store.counts_for_block(id2)
    );
    assert_eq!(
        ActionCounts {
            overwritten: 3,
            created: 2,
            ..ActionCounts::default()
        },
        store.totals()
    );

    store.async_drop().await.unwrap();
}

#[tokio::test]
async fn remove_by_id_increases_counter() {
    let mut fixture = TestFixture::<false>::new();
    let mut store = fixture.store().await;

    let id1 = store.create(&Data::from(vec![1, 2, 3])).await.unwrap();
    let id2 = store.create(&Data::from(vec![4, 5, 6])).await.unwrap();
    let unknown_id = BlockId::from_hex("715db62b0b4e333f8b16c76ee886c95b").unwrap();

    // Check it increases counter even when the block doesn't exist
    let _ = store.remove_by_id(&unknown_id).await.unwrap();

    let _ = store.remove_by_id(&id1).await.unwrap();
    let _ = store.remove_by_id(&id1).await.unwrap(); // Already removed
    let _ = store.remove_by_id(&id2).await.unwrap();

    assert_eq!(
        ActionCounts {
            removed: 2,
            created: 1,
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
            removed: 1,
            ..ActionCounts::default()
        },
        store.counts_for_block(unknown_id)
    );
    assert_eq!(
        ActionCounts {
            removed: 4,
            created: 2,
            ..ActionCounts::default()
        },
        store.totals()
    );

    store.async_drop().await.unwrap();
}

#[tokio::test]
async fn remove_increases_counter() {
    let mut fixture = TestFixture::<false>::new();
    let mut store = fixture.store().await;

    let id1 = store.create(&Data::from(vec![1, 2, 3])).await.unwrap();
    let id2 = store.create(&Data::from(vec![4, 5, 6])).await.unwrap();

    let block1 = store.load(id1).await.unwrap().unwrap();
    let block2 = store.load(id2).await.unwrap().unwrap();

    store.remove(block1).await.unwrap();
    store.remove(block2).await.unwrap();

    assert_eq!(
        ActionCounts {
            removed: 1,
            loaded: 1,
            created: 1,
            ..ActionCounts::default()
        },
        store.counts_for_block(id1)
    );
    assert_eq!(
        ActionCounts {
            removed: 1,
            loaded: 1,
            created: 1,
            ..ActionCounts::default()
        },
        store.counts_for_block(id2)
    );
    assert_eq!(
        ActionCounts {
            removed: 2,
            loaded: 2,
            created: 2,
            ..ActionCounts::default()
        },
        store.totals()
    );

    store.async_drop().await.unwrap();
}

#[tokio::test]
async fn try_create_increases_counter() {
    let mut fixture = TestFixture::<false>::new();
    let mut store = fixture.store().await;

    let id1 = BlockId::from_hex("715db62b0b4e333f8b16c76ee886c95b").unwrap();
    let id2 = BlockId::from_hex("62b0b4e333f8b16c76ee886c95b715db").unwrap();

    assert_eq!(
        crate::TryCreateResult::SuccessfullyCreated,
        store
            .try_create(&id1, &Data::from(vec![1, 2, 3]))
            .await
            .unwrap()
    );
    // Check it increases counter even when try_create fails
    assert_eq!(
        crate::TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists,
        store
            .try_create(&id1, &Data::from(vec![1, 2, 3]))
            .await
            .unwrap()
    );
    assert_eq!(
        crate::RemoveResult::SuccessfullyRemoved,
        store.remove_by_id(&id1).await.unwrap()
    );
    assert_eq!(
        crate::TryCreateResult::SuccessfullyCreated,
        store
            .try_create(&id1, &Data::from(vec![1, 2, 3]))
            .await
            .unwrap()
    );

    assert_eq!(
        crate::TryCreateResult::SuccessfullyCreated,
        store
            .try_create(&id2, &Data::from(vec![1, 2, 3]))
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
async fn create_increases_counter() {
    let mut fixture = TestFixture::<false>::new();
    let mut store = fixture.store().await;

    let id1 = store.create(&Data::from(vec![1, 2, 3])).await.unwrap();
    let id2 = store.create(&Data::from(vec![4, 5, 6])).await.unwrap();

    assert_eq!(
        ActionCounts {
            created: 1,
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
            created: 2,
            ..ActionCounts::default()
        },
        store.totals()
    );

    store.async_drop().await.unwrap();
}

#[tokio::test]
async fn resize_increases_counter() {
    let mut fixture = TestFixture::<false>::new();
    let mut store = fixture.store().await;

    let id1 = store.create(&Data::from(vec![1, 2, 3])).await.unwrap();
    let id2 = store.create(&Data::from(vec![4, 5, 6])).await.unwrap();

    let mut block1 = store.load(id1).await.unwrap().unwrap();
    let mut block2 = store.load(id2).await.unwrap().unwrap();

    block1.resize(5).await;
    block1.resize(10).await;
    block2.resize(8).await;

    assert_eq!(
        ActionCounts {
            created: 1,
            loaded: 1,
            resized: 2,
            ..ActionCounts::default()
        },
        store.counts_for_block(id1)
    );
    assert_eq!(
        ActionCounts {
            created: 1,
            loaded: 1,
            resized: 1,
            ..ActionCounts::default()
        },
        store.counts_for_block(id2)
    );
    assert_eq!(
        ActionCounts {
            created: 2,
            loaded: 2,
            resized: 3,
            ..ActionCounts::default()
        },
        store.totals()
    );

    std::mem::drop(block1);
    std::mem::drop(block2);

    store.async_drop().await.unwrap();
}

#[tokio::test]
async fn flush_block_increases_counter() {
    let mut fixture = TestFixture::<false>::new();
    let mut store = fixture.store().await;

    let id1 = store.create(&Data::from(vec![1, 2, 3])).await.unwrap();
    let id2 = store.create(&Data::from(vec![4, 5, 6])).await.unwrap();

    let mut block1 = store.load(id1).await.unwrap().unwrap();
    let mut block2 = store.load(id2).await.unwrap().unwrap();

    store.flush_block(&mut block1).await.unwrap();
    store.flush_block(&mut block1).await.unwrap();
    store.flush_block(&mut block2).await.unwrap();

    assert_eq!(
        ActionCounts {
            created: 1,
            loaded: 1,
            flushed: 2,
            ..ActionCounts::default()
        },
        store.counts_for_block(id1)
    );
    assert_eq!(
        ActionCounts {
            created: 1,
            loaded: 1,
            flushed: 1,
            ..ActionCounts::default()
        },
        store.counts_for_block(id2)
    );
    assert_eq!(
        ActionCounts {
            created: 2,
            loaded: 2,
            flushed: 3,
            ..ActionCounts::default()
        },
        store.totals()
    );

    std::mem::drop(block1);
    std::mem::drop(block2);

    store.async_drop().await.unwrap();
}

#[tokio::test]
async fn read_increases_counter() {
    let mut fixture = TestFixture::<false>::new();
    let mut store = fixture.store().await;

    let id1 = store.create(&Data::from(vec![1, 2, 3])).await.unwrap();
    let id2 = store.create(&Data::from(vec![4, 5, 6])).await.unwrap();

    let block1 = store.load(id1).await.unwrap().unwrap();
    let block2 = store.load(id2).await.unwrap().unwrap();

    // Reading the data multiple times should increase the read counter
    let _ = block1.data();
    let _ = block1.data();
    let _ = block1.data();
    let _ = block2.data();

    assert_eq!(
        ActionCounts {
            loaded: 1,
            read: 3,
            created: 1,
            ..ActionCounts::default()
        },
        store.counts_for_block(id1)
    );
    assert_eq!(
        ActionCounts {
            loaded: 1,
            read: 1,
            created: 1,
            ..ActionCounts::default()
        },
        store.counts_for_block(id2)
    );
    assert_eq!(
        ActionCounts {
            loaded: 2,
            read: 4,
            created: 2,
            ..ActionCounts::default()
        },
        store.totals()
    );

    std::mem::drop(block1);
    std::mem::drop(block2);

    store.async_drop().await.unwrap();
}

#[tokio::test]
async fn write_increases_counter() {
    let mut fixture = TestFixture::<false>::new();
    let mut store = fixture.store().await;

    let id1 = store.create(&Data::from(vec![1, 2, 3])).await.unwrap();
    let id2 = store.create(&Data::from(vec![4, 5, 6])).await.unwrap();

    let mut block1 = store.load(id1).await.unwrap().unwrap();
    let mut block2 = store.load(id2).await.unwrap().unwrap();

    // Mutating the data multiple times should increase the write counter
    block1.data_mut()[0] = 10;
    block1.data_mut()[1] = 20;
    block1.data_mut()[2] = 30;
    block2.data_mut()[0] = 40;

    assert_eq!(
        ActionCounts {
            loaded: 1,
            written: 3,
            created: 1,
            ..ActionCounts::default()
        },
        store.counts_for_block(id1)
    );
    assert_eq!(
        ActionCounts {
            loaded: 1,
            written: 1,
            created: 1,
            ..ActionCounts::default()
        },
        store.counts_for_block(id2)
    );
    assert_eq!(
        ActionCounts {
            loaded: 2,
            written: 4,
            created: 2,
            ..ActionCounts::default()
        },
        store.totals()
    );

    std::mem::drop(block1);
    std::mem::drop(block2);

    store.async_drop().await.unwrap();
}

#[tokio::test]
async fn get_and_reset_totals_works() {
    let mut fixture = TestFixture::<false>::new();
    let mut store = fixture.store().await;

    let id1 = BlockId::from_hex("715db62b0b4e333f8b16c76ee886c95b").unwrap();
    let id2 = BlockId::from_hex("62b0b4e333f8b16c76ee886c95b715db").unwrap();

    // Create some blocks
    for _ in 0..2 {
        let _ = store
            .try_create(&id1, &Data::from(vec![1, 2, 3]))
            .await
            .unwrap();
    }

    // Remove some blocks
    for _ in 0..3 {
        let _ = store.remove_by_id(&id1).await.unwrap();
    }

    // Overwrite some blocks
    for _ in 0..4 {
        store
            .overwrite(&id2, &Data::from(vec![4, 5, 6]))
            .await
            .unwrap();
    }

    // Load some blocks and read/write their data
    for _ in 0..5 {
        let mut block = store.load(id2).await.unwrap().unwrap();

        // Perform reads
        let _ = block.data();
        let _ = block.data();

        // Perform writes
        block.data_mut()[0] += 1;
    }

    assert_eq!(
        ActionCounts {
            loaded: 5,
            read: 10,
            written: 5,
            overwritten: 4,
            created: 2,
            removed: 3,
            resized: 0,
            flushed: 0,
        },
        store.get_and_reset_totals(),
    );

    assert_eq!(
        ActionCounts {
            loaded: 0,
            read: 0,
            written: 0,
            overwritten: 0,
            created: 0,
            removed: 0,
            resized: 0,
            flushed: 0,
        },
        store.get_and_reset_totals(),
    );

    store.async_drop().await.unwrap();
}
