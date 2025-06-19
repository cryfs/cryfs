use async_trait::async_trait;
use byte_unit::Byte;
use cryfs_blockstore::{InMemoryBlockStore, LockingBlockStore};
use cryfs_utils::async_drop::AsyncDropGuard;

use super::TrackingBlobStore;
use crate::{BlobId, BlobStoreOnBlocks, tests::fixture::Fixture};

struct TestFixture;
#[async_trait]
impl Fixture for TestFixture {
    type ConcreteBlobStore =
        TrackingBlobStore<BlobStoreOnBlocks<LockingBlockStore<InMemoryBlockStore>>>;
    fn new() -> Self {
        Self {}
    }
    async fn store(&mut self) -> AsyncDropGuard<Self::ConcreteBlobStore> {
        TrackingBlobStore::new(
            BlobStoreOnBlocks::new(
                LockingBlockStore::new(InMemoryBlockStore::new()),
                Byte::from_u64(1024),
            )
            .await
            .unwrap(),
        )
    }
    async fn yield_fixture(&self, _store: &Self::ConcreteBlobStore) {}
}
crate::instantiate_tests_for_blobstore!(TestFixture, (flavor = "multi_thread"));

fn change_blob_id(id: BlobId) -> BlobId {
    let mut id = id.data().clone();
    id[0] = id[0].overflowing_add(1).0;
    BlobId::from_slice(&id).unwrap()
}

mod counter_tests {
    use super::*;

    use crate::{
        Blob as _, BlobId, BlobStore as _,
        implementations::tracking::{BlobStoreActionCounts, tracking_blob::TrackingBlob},
    };
    use cryfs_blockstore::RemoveResult;
    use futures::StreamExt as _;
    use pretty_assertions::assert_eq;

    #[tokio::test(flavor = "multi_thread")]
    async fn counters_start_at_zero() {
        let mut fixture = super::TestFixture::new();
        let mut store = fixture.store().await;

        assert_eq!(
            BlobStoreActionCounts {
                blob_num_bytes: 0,
                blob_resize: 0,
                blob_read_all: 0,
                blob_read: 0,
                blob_try_read: 0,
                blob_write: 0,
                blob_flush: 0,
                blob_num_nodes: 0,
                blob_remove: 0,
                blob_all_blocks: 0,
                store_create: 0,
                store_try_create: 0,
                store_load: 0,
                store_remove_by_id: 0,
                store_num_nodes: 0,
                store_estimate_space_for_num_blocks_left: 0,
                store_logical_block_size_bytes: 0,
            },
            store.counts()
        );

        store.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn store_create_increases_counter() {
        let mut fixture = super::TestFixture::new();
        let mut store = fixture.store().await;

        store.create().await.unwrap().async_drop().await.unwrap();
        store.create().await.unwrap().async_drop().await.unwrap();

        let counts = store.counts();
        assert_eq!(
            BlobStoreActionCounts {
                store_create: 2,
                ..BlobStoreActionCounts::ZERO
            },
            counts
        );

        store.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn store_try_create_increases_counter() {
        let mut fixture = super::TestFixture::new();
        let mut store = fixture.store().await;

        let id = BlobId::from_hex("1bdad9f4879fd1417870e42b8a4e547b").unwrap();

        // Try to create with that ID (should succeed)
        let blob = store.try_create(&id).await.unwrap();
        assert!(blob.is_some());
        blob.unwrap().async_drop().await.unwrap();

        // Try again with same ID (should fail)
        let blob = store.try_create(&id).await.unwrap();
        assert!(blob.is_none());

        let counts = store.counts();
        assert_eq!(
            BlobStoreActionCounts {
                store_try_create: 2,
                ..BlobStoreActionCounts::ZERO
            },
            counts
        );

        store.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn store_load_increases_counter() {
        let mut fixture = super::TestFixture::new();
        let mut store = fixture.store().await;

        let mut blob = store.create().await.unwrap();
        let id = blob.id();
        blob.async_drop().await.unwrap();
        drop(blob);

        store
            .load(&id)
            .await
            .unwrap()
            .unwrap()
            .async_drop()
            .await
            .unwrap();
        store
            .load(&id)
            .await
            .unwrap()
            .unwrap()
            .async_drop()
            .await
            .unwrap();

        // Try loading a non-existent blob
        let nonexistent_id = change_blob_id(id);
        store.load(&nonexistent_id).await.unwrap();

        let counts = store.counts();
        assert_eq!(
            BlobStoreActionCounts {
                store_load: 3,
                store_create: 1,
                ..BlobStoreActionCounts::ZERO
            },
            counts
        );

        store.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn store_remove_by_id_increases_counter() {
        let mut fixture = super::TestFixture::new();
        let mut store = fixture.store().await;

        let mut blob = store.create().await.unwrap();
        let id = blob.id();
        blob.async_drop().await.unwrap();
        drop(blob);

        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove_by_id(&id).await.unwrap()
        );

        // Try removing again (should fail)
        assert_eq!(
            RemoveResult::NotRemovedBecauseItDoesntExist,
            store.remove_by_id(&id).await.unwrap()
        );

        // Try removing non-existent blob
        let nonexistent_id = change_blob_id(id);
        assert_eq!(
            RemoveResult::NotRemovedBecauseItDoesntExist,
            store.remove_by_id(&nonexistent_id).await.unwrap()
        );

        let counts = store.counts();
        assert_eq!(
            BlobStoreActionCounts {
                store_create: 1,
                store_remove_by_id: 3,
                ..BlobStoreActionCounts::ZERO
            },
            counts
        );

        store.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn store_num_nodes_increases_counter() {
        let mut fixture = super::TestFixture::new();
        let mut store = fixture.store().await;

        store.num_nodes().await.unwrap();
        store.num_nodes().await.unwrap();

        let counts = store.counts();
        assert_eq!(
            BlobStoreActionCounts {
                store_num_nodes: 2,
                ..BlobStoreActionCounts::ZERO
            },
            counts
        );

        store.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn store_estimate_space_for_num_blocks_left_increases_counter() {
        let mut fixture = super::TestFixture::new();
        let mut store = fixture.store().await;

        store.estimate_space_for_num_blocks_left().unwrap();
        store.estimate_space_for_num_blocks_left().unwrap();

        let counts = store.counts();
        assert_eq!(
            BlobStoreActionCounts {
                store_estimate_space_for_num_blocks_left: 2,
                ..BlobStoreActionCounts::ZERO
            },
            counts
        );

        store.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn store_logical_block_size_increases_counter() {
        let mut fixture = super::TestFixture::new();
        let mut store = fixture.store().await;

        store.logical_block_size_bytes();
        store.logical_block_size_bytes();

        let counts = store.counts();
        assert_eq!(
            BlobStoreActionCounts {
                store_logical_block_size_bytes: 2,
                ..BlobStoreActionCounts::ZERO
            },
            counts
        );

        store.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn blob_num_bytes_increases_counter() {
        let mut fixture = super::TestFixture::new();
        let mut store = fixture.store().await;

        let mut blob = store.create().await.unwrap();
        blob.num_bytes().await.unwrap();
        blob.num_bytes().await.unwrap();

        let counts = store.counts();
        assert_eq!(
            BlobStoreActionCounts {
                store_create: 1,
                blob_num_bytes: 2,
                ..BlobStoreActionCounts::ZERO
            },
            counts
        );

        blob.async_drop().await.unwrap();
        drop(blob);
        store.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn blob_resize_increases_counter() {
        let mut fixture = super::TestFixture::new();
        let mut store = fixture.store().await;

        let mut blob = store.create().await.unwrap();
        blob.resize(100).await.unwrap();
        blob.resize(200).await.unwrap();

        let counts = store.counts();
        assert_eq!(
            BlobStoreActionCounts {
                store_create: 1,
                blob_resize: 2,
                ..BlobStoreActionCounts::ZERO
            },
            counts
        );

        blob.async_drop().await.unwrap();
        drop(blob);
        store.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn blob_read_all_increases_counter() {
        let mut fixture = super::TestFixture::new();
        let mut store = fixture.store().await;

        let mut blob = store.create().await.unwrap();
        blob.read_all().await.unwrap();
        blob.read_all().await.unwrap();

        let counts = store.counts();
        assert_eq!(
            BlobStoreActionCounts {
                store_create: 1,
                blob_read_all: 2,
                ..BlobStoreActionCounts::ZERO
            },
            counts
        );

        blob.async_drop().await.unwrap();
        drop(blob);
        store.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn blob_read_increases_counter() {
        let mut fixture = super::TestFixture::new();
        let mut store = fixture.store().await;

        let mut blob = store.create().await.unwrap();
        blob.write(&[1, 2, 3], 0).await.unwrap();

        let mut buffer = [0u8; 3];
        blob.read(&mut buffer, 0).await.unwrap();
        blob.read(&mut buffer, 0).await.unwrap();

        let counts = store.counts();
        assert_eq!(
            BlobStoreActionCounts {
                store_create: 1,
                blob_write: 1,
                blob_read: 2,
                ..BlobStoreActionCounts::ZERO
            },
            counts
        );

        blob.async_drop().await.unwrap();
        drop(blob);
        store.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn blob_try_read_increases_counter() {
        let mut fixture = super::TestFixture::new();
        let mut store = fixture.store().await;

        let mut blob = store.create().await.unwrap();
        blob.write(&[1, 2, 3], 0).await.unwrap();

        let mut buffer = [0u8; 3];
        blob.try_read(&mut buffer, 0).await.unwrap();
        blob.try_read(&mut buffer, 1).await.unwrap();

        let counts = store.counts();
        assert_eq!(
            BlobStoreActionCounts {
                store_create: 1,
                blob_write: 1,
                blob_try_read: 2,
                ..BlobStoreActionCounts::ZERO
            },
            counts
        );

        blob.async_drop().await.unwrap();
        drop(blob);
        store.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn blob_write_increases_counter() {
        let mut fixture = super::TestFixture::new();
        let mut store = fixture.store().await;

        // Create a blob and write data
        let mut blob = store.create().await.unwrap();
        blob.write(&[1, 2, 3], 0).await.unwrap();
        blob.write(&[4, 5, 6], 3).await.unwrap();

        let counts = store.counts();
        assert_eq!(
            BlobStoreActionCounts {
                store_create: 1,
                blob_write: 2,
                ..BlobStoreActionCounts::ZERO
            },
            counts
        );

        blob.async_drop().await.unwrap();
        drop(blob);
        store.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn blob_flush_increases_counter() {
        let mut fixture = super::TestFixture::new();
        let mut store = fixture.store().await;

        // Create a blob and flush
        let mut blob = store.create().await.unwrap();
        blob.flush().await.unwrap();
        blob.flush().await.unwrap();

        let counts = store.counts();
        assert_eq!(
            BlobStoreActionCounts {
                store_create: 1,
                blob_flush: 2,
                ..BlobStoreActionCounts::ZERO
            },
            counts
        );

        blob.async_drop().await.unwrap();
        drop(blob);
        store.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn blob_num_nodes_increases_counter() {
        let mut fixture = super::TestFixture::new();
        let mut store = fixture.store().await;

        let mut blob = store.create().await.unwrap();
        blob.num_nodes().await.unwrap();
        blob.num_nodes().await.unwrap();

        let counts = store.counts();
        assert_eq!(
            BlobStoreActionCounts {
                store_create: 1,
                blob_num_nodes: 2,
                ..BlobStoreActionCounts::ZERO
            },
            counts
        );

        blob.async_drop().await.unwrap();
        drop(blob);
        store.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn blob_remove_increases_counter() {
        let mut fixture = super::TestFixture::new();
        let mut store = fixture.store().await;

        let blob = store.create().await.unwrap();
        TrackingBlob::remove(blob).await.unwrap();

        let counts = store.counts();
        assert_eq!(
            BlobStoreActionCounts {
                store_create: 1,
                blob_remove: 1,
                ..BlobStoreActionCounts::ZERO
            },
            counts
        );

        store.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn blob_all_blocks_increases_counter() {
        let mut fixture = super::TestFixture::new();
        let mut store = fixture.store().await;

        let mut blob = store.create().await.unwrap();
        let stream = blob.all_blocks().unwrap();
        // Use the stream to make sure it's properly executed
        let _ = stream.collect::<Vec<_>>().await;

        let stream = blob.all_blocks().unwrap();
        let _ = stream.collect::<Vec<_>>().await;

        let counts = store.counts();
        assert_eq!(
            BlobStoreActionCounts {
                store_create: 1,
                blob_all_blocks: 2,
                ..BlobStoreActionCounts::ZERO
            },
            counts
        );

        blob.async_drop().await.unwrap();
        drop(blob);
        store.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_and_reset_counts_resets_counters() {
        let mut fixture = super::TestFixture::new();
        let mut store = fixture.store().await;

        // Perform operations to increase counters
        let mut blob = store.create().await.unwrap();
        let _ = blob.all_blocks().unwrap();
        blob.async_drop().await.unwrap();
        drop(blob);

        // Check counts were increased
        let counts = store.counts();
        assert_eq!(
            BlobStoreActionCounts {
                store_create: 1,
                blob_all_blocks: 1,
                ..BlobStoreActionCounts::ZERO
            },
            counts
        );

        // Reset and check they're back to zero
        let reset_counts = store.get_and_reset_counts();
        assert_eq!(
            BlobStoreActionCounts {
                store_create: 1,
                blob_all_blocks: 1,
                ..BlobStoreActionCounts::ZERO
            },
            reset_counts
        );

        let new_counts = store.counts();
        assert_eq!(BlobStoreActionCounts::ZERO, new_counts);

        store.async_drop().await.unwrap();
    }
}
