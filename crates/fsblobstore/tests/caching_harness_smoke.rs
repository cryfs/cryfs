//! Smoke test for an in-memory `CachingFsBlobStore` harness.
//! Reproduction tests for the soundness audit can copy `make_store()` from here.

use std::time::Duration;

use byte_unit::Byte;
use cryfs_blobstore::BlobStoreOnBlocks;
use cryfs_blockstore::{InMemoryBlockStore, LockingBlockStore};
use cryfs_fsblobstore::cachingfsblobstore::CachingFsBlobStore;
use cryfs_fsblobstore::concurrentfsblobstore::ConcurrentFsBlobStore;
use cryfs_fsblobstore::fsblobstore::{FlushBehavior, FsBlobStore};
use cryfs_utils::async_drop::AsyncDropGuard;

type Store = CachingFsBlobStore<BlobStoreOnBlocks<LockingBlockStore<InMemoryBlockStore>>>;

async fn make_store() -> AsyncDropGuard<Store> {
    let blockstore = LockingBlockStore::new(InMemoryBlockStore::new());
    let blobstore = BlobStoreOnBlocks::new(blockstore, Byte::from_u64(32 * 1024))
        .await
        .unwrap();
    CachingFsBlobStore::new(ConcurrentFsBlobStore::new(FsBlobStore::new(blobstore)))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn harness_compiles_and_round_trips() {
    let mut store = make_store().await;
    let root_id = cryfs_blobstore::BlobId::new_random();
    store.create_root_dir_blob(&root_id).await.unwrap();

    let mut file = store
        .create_file_blob(&root_id, FlushBehavior::DontFlush)
        .await
        .unwrap();
    let file_id = file.blob_id();
    file.async_drop().await.unwrap();

    // Reload should hit the cache and return the same blob.
    let mut loaded = tokio::time::timeout(Duration::from_secs(5), store.load(&file_id))
        .await
        .expect("load timed out")
        .unwrap()
        .expect("blob should exist");
    assert_eq!(loaded.blob_id(), file_id);
    loaded.async_drop().await.unwrap();

    store.async_drop().await.unwrap();
}
