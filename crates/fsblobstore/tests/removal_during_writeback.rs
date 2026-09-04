//! Desired behavior: requesting removal of a blob while its normal writeback
//! (the `Dropping` state in ConcurrentStore) is in flight must either remove it
//! or report that it no longer exists. It must never panic.
//!
//! On both origin/main and feature/cachingfsblobstore, `LoadedBlobs::request_removal`
//! builds a drop closure that captures an `AsyncDropGuard<AsyncDropArc<FsBlobStore>>`.
//! When `ConcurrentStore::request_immediate_drop` answers `AlreadyDropping` (entry is
//! `Dropping`, or already has a removal requested), that closure is dropped unused and
//! the guard's Drop impl panics with "Forgot to call async_drop". In the `Loaded`
//! arm this happens while the `entries` mutex is held, poisoning it for good.
//!
//! This test races `async_drop` (writeback) against `request_removal_by_id` many
//! times to hit the `Dropping` window. It is timing dependent; a pass does not
//! prove absence of the bug, a failure proves presence.

use std::time::Duration;

use byte_unit::Byte;
use cryfs_blobstore::{BlobId, BlobStoreOnBlocks};
use cryfs_blockstore::{InMemoryBlockStore, LockingBlockStore};
use cryfs_fsblobstore::concurrentfsblobstore::ConcurrentFsBlobStore;
use cryfs_fsblobstore::fsblobstore::{FlushBehavior, FsBlobStore};
use cryfs_fsblobstore::{Gid, Mode, Uid};
use cryfs_utils::async_drop::{AsyncDropArc, AsyncDropGuard};

type Store = ConcurrentFsBlobStore<BlobStoreOnBlocks<LockingBlockStore<InMemoryBlockStore>>>;

async fn make_store() -> AsyncDropGuard<AsyncDropArc<Store>> {
    let blockstore = LockingBlockStore::new(InMemoryBlockStore::new());
    let blobstore = BlobStoreOnBlocks::new(blockstore, Byte::from_u64(32 * 1024))
        .await
        .unwrap();
    AsyncDropArc::new(ConcurrentFsBlobStore::new(FsBlobStore::new(blobstore)))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn removal_requested_while_writeback_in_flight_never_panics() {
    let store = make_store().await;
    let root_id = BlobId::new_random();
    store.create_root_dir_blob(&root_id).await.unwrap();

    for i in 0..300 {
        // Unflushed dir blob: dropping it has real writeback work to do.
        let mut dir = store
            .create_dir_blob(&root_id, FlushBehavior::DontFlush)
            .await
            .unwrap();
        let id = dir.blob_id();
        // Dirty it so async_drop serializes entries.
        dir.with_lock(async |b| {
            b.as_dir_mut()
                .unwrap()
                .add_entry_file(
                    cryfs_utils::path::PathComponentBuf::try_from_string(format!("f{i}")).unwrap(),
                    BlobId::new_random(),
                    Mode::from(0o100644),
                    Uid::from(0),
                    Gid::from(0),
                    std::time::SystemTime::now(),
                    std::time::SystemTime::now(),
                )
                .unwrap();
        })
        .await;

        let dropper = tokio::spawn(async move { dir.async_drop().await.unwrap() });
        // Give the drop a moment to reach the Dropping state (or not; that's the race).
        tokio::task::yield_now().await;

        let s = AsyncDropArc::clone(&store);
        let remover = tokio::spawn(async move {
            let s = s;
            let fut = s.request_removal_by_id(&id).await;
            let r = fut.await;
            let mut s = s;
            s.async_drop().await.unwrap();
            r
        });

        let r = tokio::time::timeout(Duration::from_secs(15), remover)
            .await
            .expect("remover hung")
            .unwrap_or_else(|join_err| panic!("iteration {i}: remover task panicked: {join_err}"));
        let _ = r.unwrap_or_else(|e| panic!("iteration {i}: removal failed: {e:?}"));
        tokio::time::timeout(Duration::from_secs(15), dropper)
            .await
            .expect("dropper hung")
            .unwrap_or_else(|join_err| panic!("iteration {i}: dropper task panicked: {join_err}"));
    }

    let mut store = store;
    store.async_drop().await.unwrap();
}
