//! Regression tests for concurrent removal requests on `CachingFsBlobStore`.
//!
//! These tests describe the behavior we WANT: two removal requests for the same
//! blob id, racing with a third task that still holds the blob, must complete
//! (one succeeds, the other observes that the blob is gone) instead of hanging.
//!
//! On feature/cachingfsblobstore they currently FAIL by timing out, because
//! `CachingFsBlobStore::remove_by_id` awaits `request_removal_by_id` (which loops
//! on `AlreadyDropping { future } => future.await`) while holding the cache
//! mutex, and `LoadedBlobGuard::request_removal` awaits the same kind of future
//! while its own guard is still alive. The awaited event only fires once every
//! holder has released the blob, and releasing a `CachingFsBlob` needs the cache
//! mutex, so the tasks wait on each other forever.

use std::sync::Arc;
use std::time::Duration;

use byte_unit::Byte;
use cryfs_blobstore::{BlobId, BlobStoreOnBlocks, RemoveResult};
use cryfs_blockstore::{InMemoryBlockStore, LockingBlockStore};
use cryfs_fsblobstore::cachingfsblobstore::{CachingFsBlob, CachingFsBlobStore};
use cryfs_fsblobstore::concurrentfsblobstore::ConcurrentFsBlobStore;
use cryfs_fsblobstore::fsblobstore::{FlushBehavior, FsBlobStore};
use cryfs_utils::async_drop::{AsyncDropArc, AsyncDropGuard};

type Store = CachingFsBlobStore<BlobStoreOnBlocks<LockingBlockStore<InMemoryBlockStore>>>;

async fn make_store() -> AsyncDropGuard<AsyncDropArc<Store>> {
    let blockstore = LockingBlockStore::new(InMemoryBlockStore::new());
    let blobstore = BlobStoreOnBlocks::new(blockstore, Byte::from_u64(32 * 1024))
        .await
        .unwrap();
    AsyncDropArc::new(CachingFsBlobStore::new(ConcurrentFsBlobStore::new(
        FsBlobStore::new(blobstore),
    )))
}

/// Create a file blob under a fresh root dir and return its id. The blob is
/// released (i.e. sits in the cache) when this returns.
async fn create_file(store: &Store) -> BlobId {
    let root_id = BlobId::new_random();
    store.create_root_dir_blob(&root_id).await.unwrap();
    let mut file = store
        .create_file_blob(&root_id, FlushBehavior::DontFlush)
        .await
        .unwrap();
    let id = file.blob_id();
    file.async_drop().await.unwrap();
    id
}

const STEP: Duration = Duration::from_millis(300);
const DEADLINE: Duration = Duration::from_secs(10);

/// Interleaving:
///  T3 loads the blob (cache hit pops it out of the cache) and keeps holding it.
///  T1 remove_by_id: cache miss -> request_removal_by_id sets the removal flag,
///     releases the cache lock, awaits the removal (needs T3 to release).
///  T2 remove_by_id: cache miss -> request_removal_by_id sees the flag
///     (AlreadyDropping) and awaits it WHILE HOLDING THE CACHE LOCK.
///  T3 releases its blob -> CachingFsBlob::async_drop_impl needs the cache lock
///     -> blocked by T2 -> nobody makes progress.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_concurrent_remove_by_id_with_live_holder_complete() {
    let store = make_store().await;
    let id = create_file(&store).await;

    let mut holder = store.load(&id).await.unwrap().expect("blob exists");
    assert_eq!(holder.blob_id(), id);

    let s1 = AsyncDropArc::clone(&store);
    let t1 = tokio::spawn(async move {
        let s1 = s1;
        let r = s1.remove_by_id(&id).await;
        let mut s1 = s1;
        s1.async_drop().await.unwrap();
        r
    });
    tokio::time::sleep(STEP).await;

    let s2 = AsyncDropArc::clone(&store);
    let t2 = tokio::spawn(async move {
        let s2 = s2;
        let r = s2.remove_by_id(&id).await;
        let mut s2 = s2;
        s2.async_drop().await.unwrap();
        r
    });
    tokio::time::sleep(STEP).await;

    // The holder releases the blob; this must not hang.
    tokio::time::timeout(DEADLINE, holder.async_drop())
        .await
        .expect("holder.async_drop() hung: cache mutex is held by a task waiting for this release")
        .unwrap();

    let r1 = tokio::time::timeout(DEADLINE, t1)
        .await
        .expect("first remove_by_id hung")
        .unwrap();
    let r2 = tokio::time::timeout(DEADLINE, t2)
        .await
        .expect("second remove_by_id hung")
        .unwrap();

    let outcomes = [r1.map_err(|e| e.to_string()), r2.map_err(|e| e.to_string())];
    let successes = outcomes
        .iter()
        .filter(|r| matches!(r, Ok(RemoveResult::SuccessfullyRemoved)))
        .count();
    assert_eq!(
        1, successes,
        "exactly one removal should succeed, got {outcomes:?}"
    );
    assert!(
        store.load(&id).await.unwrap().is_none(),
        "blob should be gone"
    );

    let mut store = store;
    store.async_drop().await.unwrap();
}

/// Same shape, but the second remover holds the blob itself and uses
/// `CachingFsBlob::remove` (the rmdir path). It awaits the earlier request's
/// completion while its own guard is still alive, so it waits for itself.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn remove_by_id_racing_with_cachingfsblob_remove_completes() {
    let store = make_store().await;
    let id = create_file(&store).await;

    let blob_for_t2 = store.load(&id).await.unwrap().expect("blob exists");

    let s1 = AsyncDropArc::clone(&store);
    let t1 = tokio::spawn(async move {
        let s1 = s1;
        let r = s1.remove_by_id(&id).await;
        let mut s1 = s1;
        s1.async_drop().await.unwrap();
        r
    });
    tokio::time::sleep(STEP).await;

    let t2 = tokio::spawn(async move { CachingFsBlob::remove(blob_for_t2).await });

    let r1 = tokio::time::timeout(DEADLINE, t1)
        .await
        .expect("remove_by_id hung")
        .unwrap();
    let r2 = tokio::time::timeout(DEADLINE, t2)
        .await
        .expect("CachingFsBlob::remove hung")
        .unwrap();

    let outcomes = [r1.map_err(|e| e.to_string()), r2.map_err(|e| e.to_string())];
    let successes = outcomes
        .iter()
        .filter(|r| matches!(r, Ok(RemoveResult::SuccessfullyRemoved)))
        .count();
    assert_eq!(
        1, successes,
        "exactly one removal should succeed, got {outcomes:?}"
    );
    assert!(
        store.load(&id).await.unwrap().is_none(),
        "blob should be gone"
    );

    let mut store = store;
    store.async_drop().await.unwrap();
}

/// Sanity check that the harness itself is fine: a single removal of a
/// blob that another task holds completes once the holder releases it.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn single_remove_by_id_with_live_holder_completes() {
    let store = make_store().await;
    let id = create_file(&store).await;

    let mut holder = store.load(&id).await.unwrap().expect("blob exists");

    let s1 = AsyncDropArc::clone(&store);
    let t1 = tokio::spawn(async move {
        let s1 = s1;
        let r = s1.remove_by_id(&id).await;
        let mut s1 = s1;
        s1.async_drop().await.unwrap();
        r
    });
    tokio::time::sleep(STEP).await;

    tokio::time::timeout(DEADLINE, holder.async_drop())
        .await
        .expect("holder.async_drop() hung")
        .unwrap();
    let r1 = tokio::time::timeout(DEADLINE, t1)
        .await
        .expect("remove_by_id hung")
        .unwrap()
        .map_err(|e| {
            Arc::try_unwrap(e)
                .map(|e| e.to_string())
                .unwrap_or_else(|e| e.to_string())
        });
    assert!(
        matches!(r1, Ok(RemoveResult::SuccessfullyRemoved)),
        "{r1:?}"
    );
    assert!(store.load(&id).await.unwrap().is_none());

    let mut store = store;
    store.async_drop().await.unwrap();
}
