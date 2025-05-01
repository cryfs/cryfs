use async_trait::async_trait;
use byte_unit::Byte;
use futures::stream::StreamExt;
use pretty_assertions::assert_eq;

use cryfs_utils::async_drop::AsyncDropGuard;
use cryfs_utils::data::Data;

use super::{ActionCounts, TrackingBlockStore};
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
            store_load: 0,
            store_try_create: 0,
            store_overwrite: 0,
            store_remove_by_id: 0,
            store_remove: 0,
            store_num_blocks: 0,
            store_estimate_num_free_bytes: 0,
            store_block_size_from_physical_block_size: 0,
            store_all_blocks: 0,
            store_create: 0,
            store_flush_block: 0,
            blob_data: 0,
            blob_data_mut: 0,
            blob_resize: 0,
        },
        store.counts(),
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
            store_load: 4,
            store_create: 2,
            ..ActionCounts::ZERO
        },
        store.counts()
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
            store_overwrite: 3,
            store_create: 2,
            ..ActionCounts::ZERO
        },
        store.counts()
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
            store_remove_by_id: 4,
            store_create: 2,
            ..ActionCounts::ZERO
        },
        store.counts()
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
            store_remove: 2,
            store_load: 2,
            store_create: 2,
            ..ActionCounts::ZERO
        },
        store.counts()
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
            store_try_create: 4,
            store_remove_by_id: 1,
            ..ActionCounts::ZERO
        },
        store.counts()
    );

    store.async_drop().await.unwrap();
}

#[tokio::test]
async fn create_increases_counter() {
    let mut fixture = TestFixture::<false>::new();
    let mut store = fixture.store().await;

    store.create(&Data::from(vec![1, 2, 3])).await.unwrap();
    store.create(&Data::from(vec![4, 5, 6])).await.unwrap();

    assert_eq!(
        ActionCounts {
            store_create: 2,
            ..ActionCounts::ZERO
        },
        store.counts()
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
            store_create: 2,
            store_load: 2,
            blob_resize: 3,
            ..ActionCounts::ZERO
        },
        store.counts()
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
            store_create: 2,
            store_load: 2,
            store_flush_block: 3,
            ..ActionCounts::ZERO
        },
        store.counts()
    );

    std::mem::drop(block1);
    std::mem::drop(block2);

    store.async_drop().await.unwrap();
}

#[tokio::test]
async fn data_increases_counter() {
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
            store_load: 2,
            blob_data: 4,
            store_create: 2,
            ..ActionCounts::ZERO
        },
        store.counts()
    );

    std::mem::drop(block1);
    std::mem::drop(block2);

    store.async_drop().await.unwrap();
}

#[tokio::test]
async fn data_mut_increases_counter() {
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
            store_load: 2,
            blob_data_mut: 4,
            store_create: 2,
            ..ActionCounts::ZERO
        },
        store.counts()
    );

    std::mem::drop(block1);
    std::mem::drop(block2);

    store.async_drop().await.unwrap();
}

#[tokio::test]
async fn num_blocks_increases_counter() {
    let mut fixture = TestFixture::<false>::new();
    let mut store = fixture.store().await;

    // Create some blocks to make the test more meaningful
    store.create(&Data::from(vec![1, 2, 3])).await.unwrap();
    store.create(&Data::from(vec![4, 5, 6])).await.unwrap();

    // Call num_blocks multiple times
    store.num_blocks().await.unwrap();
    store.num_blocks().await.unwrap();
    store.num_blocks().await.unwrap();

    assert_eq!(
        ActionCounts {
            store_num_blocks: 3,
            store_create: 2,
            ..ActionCounts::ZERO
        },
        store.counts()
    );

    store.async_drop().await.unwrap();
}

#[tokio::test]
async fn estimate_num_free_bytes_increases_counter() {
    let mut fixture = TestFixture::<false>::new();
    let mut store = fixture.store().await;

    store.estimate_num_free_bytes().unwrap();
    store.estimate_num_free_bytes().unwrap();
    store.estimate_num_free_bytes().unwrap();
    store.estimate_num_free_bytes().unwrap();

    assert_eq!(
        ActionCounts {
            store_estimate_num_free_bytes: 4,
            ..ActionCounts::ZERO
        },
        store.counts()
    );

    store.async_drop().await.unwrap();
}

#[tokio::test]
async fn block_size_from_physical_block_size_increases_counter() {
    let mut fixture = TestFixture::<false>::new();
    let mut store = fixture.store().await;

    store
        .block_size_from_physical_block_size(Byte::from_u64(1024))
        .unwrap();
    store
        .block_size_from_physical_block_size(Byte::from_u64(2048))
        .unwrap();
    store
        .block_size_from_physical_block_size(Byte::from_u64(4096))
        .unwrap();

    assert_eq!(
        ActionCounts {
            store_block_size_from_physical_block_size: 3,
            ..ActionCounts::ZERO
        },
        store.counts()
    );

    store.async_drop().await.unwrap();
}

#[tokio::test]
async fn all_blocks_increases_counter() {
    let mut fixture = TestFixture::<false>::new();
    let mut store = fixture.store().await;

    // Create some blocks to make the test more meaningful
    store.create(&Data::from(vec![1, 2, 3])).await.unwrap();
    store.create(&Data::from(vec![4, 5, 6])).await.unwrap();

    // Call all_blocks multiple times and execute streams to completion
    let s1 = store.all_blocks().await.unwrap();
    let _ = s1.collect::<Vec<_>>().await;
    let s2 = store.all_blocks().await.unwrap();
    let _ = s2.collect::<Vec<_>>().await;

    assert_eq!(
        ActionCounts {
            store_all_blocks: 2,
            store_create: 2,
            ..ActionCounts::ZERO
        },
        store.counts()
    );

    store.async_drop().await.unwrap();
}

#[tokio::test]
async fn get_and_reset_totals_works() {
    let mut fixture = TestFixture::<false>::new();
    let mut store = fixture.store().await;

    let id1 = BlockId::from_hex("715db62b0b4e333f8b16c76ee886c95b").unwrap();
    let id2 = BlockId::from_hex("62b0b4e333f8b16c76ee886c95b715db").unwrap();

    // Create some blocks using try_create
    for _ in 0..2 {
        let _ = store
            .try_create(&id1, &Data::from(vec![1, 2, 3]))
            .await
            .unwrap();
    }

    // Create some blocks using create
    for _ in 0..3 {
        let _ = store.create(&Data::from(vec![7, 8, 9])).await.unwrap();
    }

    // Remove some blocks by ID
    for _ in 0..3 {
        let _ = store.remove_by_id(&id1).await.unwrap();
    }

    // Remove some blocks by object
    for _ in 0..2 {
        let id = store.create(&Data::from(vec![10, 11, 12])).await.unwrap();
        let block = store.load(id).await.unwrap().unwrap();
        store.remove(block).await.unwrap();
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
        block.data_mut()[0] = block.data()[0].overflowing_add(1).0;
    }

    // Call num_blocks multiple times
    for _ in 0..3 {
        store.num_blocks().await.unwrap();
    }

    // Call estimate_num_free_bytes multiple times
    for _ in 0..2 {
        store.estimate_num_free_bytes().unwrap();
    }

    // Call block_size_from_physical_block_size multiple times
    for _ in 0..4 {
        store
            .block_size_from_physical_block_size(Byte::from_u64(1024))
            .unwrap();
    }

    // Call all_blocks multiple times
    for _ in 0..3 {
        let _ = store.all_blocks().await.unwrap();
    }

    // Call flush_block
    for _ in 0..2 {
        if let Some(mut block) = store.load(id2).await.unwrap() {
            store.flush_block(&mut block).await.unwrap();
        }
    }

    // Call resize
    if let Some(mut block) = store.load(id2).await.unwrap() {
        block.resize(10).await;
        block.resize(20).await;
    }

    assert_eq!(
        ActionCounts {
            store_load: 5 + 2 + 1 + 2, // Original 5 + 2 for flush_block + 1 for resize + 2 for remove
            blob_data: 15,
            blob_data_mut: 5,
            store_overwrite: 4,
            store_try_create: 2,
            store_create: 3 + 2, // 3 direct creates + 2 for remove test
            store_remove_by_id: 3,
            store_remove: 2,
            store_num_blocks: 3,
            store_estimate_num_free_bytes: 2,
            store_block_size_from_physical_block_size: 4,
            store_all_blocks: 3,
            store_flush_block: 2,
            blob_resize: 2,
        },
        store.get_and_reset_counts(),
    );

    assert_eq!(ActionCounts::ZERO, store.get_and_reset_counts(),);

    store.async_drop().await.unwrap();
}
