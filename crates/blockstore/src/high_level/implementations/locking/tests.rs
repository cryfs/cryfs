#![allow(non_snake_case)]

use anyhow::anyhow;
use async_trait::async_trait;
use byte_unit::Byte;
use mockall::predicate::{always, function};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

use super::cache::{PRUNE_BLOCKS_INTERVAL, PRUNE_BLOCKS_OLDER_THAN};
use super::*;
use crate::{
    BlockId, InMemoryBlockStore, Overhead, high_level::interface::BlockStore as _,
    low_level::MockBlockStore, tests::high_level::HLFixture,
};
use crate::{instantiate_blockstore_tests_for_highlevel_blockstore, tests::utils::data};
use cryfs_utils::async_drop::AsyncDropGuard;

struct TestFixture<const FLUSH_CACHE_ON_YIELD: bool> {}
#[async_trait]
impl<const FLUSH_CACHE_ON_YIELD: bool> HLFixture for TestFixture<FLUSH_CACHE_ON_YIELD> {
    type ConcreteBlockStore = LockingBlockStore<InMemoryBlockStore>;
    fn new() -> Self {
        Self {}
    }
    async fn store(&mut self) -> AsyncDropGuard<Self::ConcreteBlockStore> {
        LockingBlockStore::new(InMemoryBlockStore::new())
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

fn make_mock_block_store() -> AsyncDropGuard<MockBlockStore> {
    let mut store = AsyncDropGuard::new(MockBlockStore::new());
    store
        .expect_async_drop_impl()
        .times(1)
        .returning(|| Box::pin(async { Ok(()) }));
    store
}

#[tokio::test]
async fn test_whenCallingCreate_thenPassesThroughDataToBaseStore() {
    let mut underlying_store = make_mock_block_store();
    underlying_store
        .expect_exists()
        .returning(|_| Box::pin(async { Ok(false) }));
    underlying_store
        .expect_store()
        .with(always(), function(|v| v == data(1024, 0).as_ref()))
        .returning(|_, _| Box::pin(async { Ok(()) }));
    let mut store = LockingBlockStore::new(underlying_store);

    store.create(&data(1024, 0)).await.unwrap();

    store.async_drop().await.unwrap();
}

#[tokio::test]
async fn test_whenCallingCreate_thenReturnsCorrectBlockId() {
    let mut underlying_store = make_mock_block_store();
    let id_watcher: Arc<Mutex<Option<BlockId>>> = Arc::new(Mutex::new(None));
    let _id_watcher = Arc::clone(&id_watcher);
    underlying_store
        .expect_exists()
        .once()
        .returning(move |id| {
            let mut id_watcher = _id_watcher.lock().unwrap();
            assert_eq!(None, *id_watcher);
            *id_watcher = Some(*id);
            Box::pin(async { Ok(false) })
        });
    let _id_watcher = Arc::clone(&id_watcher);
    underlying_store
        .expect_store()
        .with(always(), function(|v| v == data(1024, 0).as_ref()))
        .once()
        .returning(move |id, _| {
            let id_watcher = _id_watcher.lock().unwrap();
            assert_eq!(*id, id_watcher.expect("id_watcher not set yet"));
            Box::pin(async { Ok(()) })
        });
    let mut store = LockingBlockStore::new(underlying_store);

    let block_id = store.create(&data(1024, 0)).await.unwrap();
    assert_eq!(*id_watcher.lock().unwrap(), Some(block_id));

    store.async_drop().await.unwrap();
}

#[tokio::test]
async fn test_whenRemovingABlockThatWasJustCreatedButNotFlushed_thenWasNeverCreatedAndDoesntRemove()
{
    let mut underlying_store = make_mock_block_store();
    underlying_store
        .expect_exists()
        .returning(|_| Box::pin(async { Ok(false) }));
    underlying_store.expect_store().never();
    underlying_store.expect_remove().never();
    let mut store = LockingBlockStore::new(underlying_store);
    // The block below stays in the cache for the whole test. The periodic pruning would
    // write it back to the base store and evict it, which is exactly what this test
    // asserts doesn't happen, so it must not run on the clock's schedule.
    store.stop_periodic_cache_pruning().await.unwrap();

    let block_id = store.create(&data(1024, 0)).await.unwrap();
    let block = store.load(block_id).await.unwrap().unwrap();
    store.remove(block).await.unwrap();

    store.async_drop().await.unwrap();
}

#[tokio::test]
async fn test_whenRemovingABlockThatWasJustCreatedButThenFlushed_thenActuallyRemoves() {
    // This is a regression test since we had a bug here where flushing wrote the block to the base store,
    // but forgot to set the cache entry to "this block exists in the base store", so a later remove
    // didn't actually remove it from the base store.

    let mut underlying_store = make_mock_block_store();
    underlying_store
        .expect_exists()
        .returning(|_| Box::pin(async { Ok(false) }));
    underlying_store
        .expect_store()
        .once()
        .return_once(|_, _| Box::pin(async { Ok(()) }));
    underlying_store
        .expect_remove()
        .once()
        .return_once(|_| Box::pin(async { Ok(crate::utils::RemoveResult::SuccessfullyRemoved) }));
    let mut store = LockingBlockStore::new(underlying_store);
    // The periodic pruning would evict the block between creating and loading it, so the
    // load would go to the base store and the write-back would be a second store call.
    // This test counts both, so pruning must not run on the clock's schedule.
    store.stop_periodic_cache_pruning().await.unwrap();

    let block_id = store.create(&data(1024, 0)).await.unwrap();
    let mut block = store.load(block_id).await.unwrap().unwrap();
    store.flush_block(&mut block).await.unwrap();
    store.remove(block).await.unwrap();

    store.async_drop().await.unwrap();
}

#[tokio::test]
async fn test_whenCallingCreate_butIdAlreadyExists_thenTriesAgain() {
    let countdown_attempts = AtomicUsize::new(10); // The first 10 attempted ids say the id already exists
    let attempted_ids = Arc::new(Mutex::new(Vec::new()));

    let mut underlying_store = make_mock_block_store();
    let _attempted_ids = Arc::clone(&attempted_ids);
    underlying_store
        .expect_exists()
        .times(10..)
        .returning(move |id| {
            let mut attempted_ids = _attempted_ids.lock().unwrap();
            if attempted_ids.contains(id) {
                // This id was already previously attempted, just return "it exists" again
                // This branch should only be executed if the calling code somehow tries an id multiple times which
                // either means a bug or a very very unlucky random generator, unlikely enough to actually never happen.
                // We're still handling it here because test flakiness is bad style.
                Box::pin(async { Ok(true) })
            } else {
                attempted_ids.push(*id);
                let say_it_exists = countdown_attempts.fetch_sub(1, Ordering::SeqCst) > 0;
                Box::pin(async move { Ok(say_it_exists) })
            }
        });
    let _attempted_ids = Arc::clone(&attempted_ids);
    underlying_store
        .expect_store()
        .with(always(), function(|v| v == data(1024, 0).as_ref()))
        .once()
        .returning(move |id, _| {
            let attempted_ids = _attempted_ids.lock().unwrap();
            assert_eq!(
                *id,
                *attempted_ids.last().expect("attempted_ids not set yet")
            );
            Box::pin(async { Ok(()) })
        });
    let mut store = LockingBlockStore::new(underlying_store);

    let block_id = store.create(&data(1024, 0)).await.unwrap();
    assert_eq!(attempted_ids.lock().unwrap().last(), Some(&block_id));

    store.async_drop().await.unwrap();
}

#[tokio::test]
async fn test_whenCallingCreate_butExistsReturnsError_thenReturnsError() {
    let mut underlying_store = make_mock_block_store();
    underlying_store
        .expect_exists()
        .once()
        .returning(move |_| Box::pin(async { Err(anyhow!("Some error")) }));
    underlying_store.expect_store().never();
    let mut store = LockingBlockStore::new(underlying_store);

    let err = store.create(&data(1024, 0)).await.unwrap_err();
    assert_eq!("Some error", err.to_string());

    store.async_drop().await.unwrap();
}

#[tokio::test]
async fn test_overhead() {
    let expected_overhead = Overhead::new(Byte::from_u64(234354));

    let mut underlying_store = make_mock_block_store();
    underlying_store
        .expect_overhead()
        .returning(move || expected_overhead);
    let mut store = LockingBlockStore::new(underlying_store);

    assert_eq!(expected_overhead, store.overhead());

    store.async_drop().await.unwrap();
}

/// A mock base store for a store that creates one block: the block doesn't exist yet, and
/// it is written back exactly once. `stored` records when that write-back happens.
fn make_mock_block_store_expecting_one_write_back()
-> (AsyncDropGuard<MockBlockStore>, Arc<AtomicBool>) {
    let stored = Arc::new(AtomicBool::new(false));
    let mut underlying_store = make_mock_block_store();
    underlying_store
        .expect_exists()
        .returning(|_| Box::pin(async { Ok(false) }));
    let _stored = Arc::clone(&stored);
    underlying_store
        .expect_store()
        .once()
        .returning(move |_, _| {
            _stored.store(true, Ordering::SeqCst);
            Box::pin(async { Ok(()) })
        });
    (underlying_store, stored)
}

#[tokio::test]
async fn test_whenStoppingPeriodicCachePruningTwice_thenSecondCallIsANoop() {
    let underlying_store = make_mock_block_store();
    let mut store = LockingBlockStore::new(underlying_store);

    store.stop_periodic_cache_pruning().await.unwrap();
    store.stop_periodic_cache_pruning().await.unwrap();

    store.async_drop().await.unwrap();
}

#[tokio::test]
async fn test_whenPeriodicCachePruningIsStopped_thenDirtyBlockIsNotWrittenBackByTheClock() {
    let (underlying_store, stored) = make_mock_block_store_expecting_one_write_back();
    let mut store = LockingBlockStore::new(underlying_store);
    store.stop_periodic_cache_pruning().await.unwrap();

    store.create(&data(1024, 0)).await.unwrap();
    // Long enough for the periodic task, were it still running, to find the block
    // untouched for longer than the threshold and write it back.
    tokio::time::sleep(2 * (PRUNE_BLOCKS_INTERVAL + PRUNE_BLOCKS_OLDER_THAN)).await;
    assert!(
        !stored.load(Ordering::SeqCst),
        "block was written back although periodic pruning is stopped"
    );

    // Destructing the store still writes the dirty block back.
    store.async_drop().await.unwrap();
    assert!(stored.load(Ordering::SeqCst));
}

#[tokio::test]
async fn test_whenPeriodicCachePruningIsStopped_thenExplicitPruningStillWritesBackDirtyBlocks() {
    let (underlying_store, stored) = make_mock_block_store_expecting_one_write_back();
    let mut store = LockingBlockStore::new(underlying_store);
    store.stop_periodic_cache_pruning().await.unwrap();

    store.create(&data(1024, 0)).await.unwrap();
    assert!(!stored.load(Ordering::SeqCst));
    store.clear_unloaded_blocks_from_cache().await.unwrap();
    assert!(
        stored.load(Ordering::SeqCst),
        "explicit pruning must still write the dirty block back"
    );

    store.async_drop().await.unwrap();
}

// TODO Test flush_block
