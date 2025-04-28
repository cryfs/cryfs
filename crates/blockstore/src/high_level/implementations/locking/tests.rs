#![allow(non_snake_case)]

use anyhow::anyhow;
use byte_unit::Byte;
use mockall::predicate::{always, function};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

use super::*;
use crate::tests::data;
use crate::{BlockId, high_level::interface::BlockStore as _, low_level::MockBlockStore};
use cryfs_utils::async_drop::AsyncDropGuard;

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
    // TODO This is potentially flaky. Let's make sure cache doesn't get pruned, maybe set flush time to infinity?
    let mut underlying_store = make_mock_block_store();
    underlying_store
        .expect_exists()
        .returning(|_| Box::pin(async { Ok(false) }));
    underlying_store.expect_store().never();
    underlying_store.expect_remove().never();
    let mut store = LockingBlockStore::new(underlying_store);

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

    // TODO This is potentially flaky. Let's make sure cache doesn't get pruned, maybe set flush time to infinity?
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
async fn test_block_size_from_physical_block_size() {
    let expected_overhead = Byte::from_u64(234354);

    let mut underlying_store = make_mock_block_store();
    underlying_store
        .expect_block_size_from_physical_block_size()
        .returning(move |x| Ok(x.subtract(expected_overhead).unwrap()));
    let mut store = LockingBlockStore::new(underlying_store);

    assert_eq!(
        Byte::from_u64(0),
        store
            .block_size_from_physical_block_size(expected_overhead)
            .unwrap()
    );
    assert_eq!(
        Byte::from_u64(500),
        store
            .block_size_from_physical_block_size(
                Byte::from_u64(500).add(expected_overhead).unwrap()
            )
            .unwrap()
    );

    store.async_drop().await.unwrap();
}

// TODO Test flush_block
