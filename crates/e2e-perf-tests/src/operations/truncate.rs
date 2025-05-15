use pretty_assertions::assert_eq;
use rstest::rstest;
use rstest_reuse::apply;

use crate::filesystem_driver::FilesystemDriver as _;
use crate::fixture::ActionCounts;
use crate::fixture::NUM_BYTES_FOR_THREE_LEVEL_TREE;
use crate::rstest::FixtureFactory;
use crate::rstest::FixtureType;
use crate::rstest::{all_atime_behaviors, all_fixtures};
use cryfs_blobstore::BlobStoreActionCounts;
use cryfs_blockstore::HLActionCounts;
use cryfs_blockstore::LLActionCounts;
use cryfs_rustfs::AbsolutePath;
use cryfs_rustfs::AtimeUpdateBehavior;
use cryfs_rustfs::NumBytes;
use cryfs_rustfs::PathComponent;

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn truncate_grow_empty_file_small(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create a file so we have something to truncate
    let file = fixture
        .ops(async |fs| {
            fs.create_file(None, PathComponent::try_from_str("testfile.txt").unwrap())
                .await
                .unwrap()
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            // Grow by a small amount (1 byte)
            fs.truncate(Some(file), NumBytes::from(1)).await.unwrap();
        })
        .await;

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: 1,
                blob_resize: 2,
                blob_num_bytes: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 27,
                    FixtureType::Fusemt => 36,
                    FixtureType::FuserWithoutInodeCache => 43, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 2,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 2,
                store: 2,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn truncate_grow_empty_file_large(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create a file so we have something to truncate
    let file = fixture
        .ops(async |fs| {
            fs.create_file(None, PathComponent::try_from_str("testfile.txt").unwrap())
                .await
                .unwrap()
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.truncate(Some(file), NumBytes::from(NUM_BYTES_FOR_THREE_LEVEL_TREE))
                .await
                .unwrap();
        })
        .await;

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: 1,
                blob_resize: 2,
                blob_num_bytes: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 63,
                    FixtureType::Fusemt => 64,
                    FixtureType::FuserWithoutInodeCache => 65, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_create: 56,
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 336,
                    FixtureType::Fusemt => 345,
                    FixtureType::FuserWithoutInodeCache => 352, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 57,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                exists: 56,
                load: 2,
                store: 58,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn truncate_shrink_file_small(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create a file with data
    let file = fixture
        .ops(async |fs| {
            let file = fs
                .create_file(None, PathComponent::try_from_str("testfile.txt").unwrap())
                .await
                .unwrap();

            // Make file larger first
            fs.truncate(
                Some(file.clone()),
                NumBytes::from(NUM_BYTES_FOR_THREE_LEVEL_TREE),
            )
            .await
            .unwrap();

            file
        })
        .await;

    // Now shrink it down by a small amount (1 byte less)
    let counts = fixture
        .count_ops(async |fs| {
            fs.truncate(
                Some(file),
                NumBytes::from(NUM_BYTES_FOR_THREE_LEVEL_TREE - 1),
            )
            .await
            .unwrap();
        })
        .await;

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: 1,
                blob_resize: 2,
                blob_num_bytes: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 13,
                    FixtureType::Fusemt => 14,
                    FixtureType::FuserWithoutInodeCache => 19, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 106,
                    FixtureType::Fusemt => 115,
                    FixtureType::FuserWithoutInodeCache => 150, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 4,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 6,
                store: 4,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn truncate_shrink_file_large(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create a file with data
    let file = fixture
        .ops(async |fs| {
            let file = fs
                .create_file(None, PathComponent::try_from_str("testfile.txt").unwrap())
                .await
                .unwrap();
            fs.truncate(
                Some(file.clone()),
                NumBytes::from(NUM_BYTES_FOR_THREE_LEVEL_TREE),
            )
            .await
            .unwrap();

            file
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.truncate(Some(file), NumBytes::from(1)).await.unwrap();
        })
        .await;

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: 1,
                blob_resize: 2,
                blob_num_bytes: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 14,
                    FixtureType::Fusemt => 15,
                    FixtureType::FuserWithoutInodeCache => 20, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_remove: 5,
                store_remove_by_id: 51,
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 115,
                    FixtureType::Fusemt => 124,
                    FixtureType::FuserWithoutInodeCache => 159, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 5,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 8,
                store: 2,
                remove: 56,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn truncate_grow_nonempty_file_small(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create a file with some data
    let file = fixture
        .ops(async |fs| {
            let file = fs
                .create_file(None, PathComponent::try_from_str("testfile.txt").unwrap())
                .await
                .unwrap();
            fs.truncate(Some(file.clone()), NumBytes::from(100))
                .await
                .unwrap();

            file
        })
        .await;

    // Now grow it by a small amount (add 1 byte)
    let counts = fixture
        .count_ops(async |fs| {
            fs.truncate(Some(file), NumBytes::from(101)).await.unwrap();
        })
        .await;

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: 1,
                blob_resize: 2,
                blob_num_bytes: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 27,
                    FixtureType::Fusemt => 36,
                    FixtureType::FuserWithoutInodeCache => 43, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 2,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 2,
                store: 2,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn truncate_grow_nonempty_file_large(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create a file with some data
    let file = fixture
        .ops(async |fs| {
            let file = fs
                .create_file(None, PathComponent::try_from_str("testfile.txt").unwrap())
                .await
                .unwrap();
            fs.truncate(Some(file.clone()), NumBytes::from(100))
                .await
                .unwrap();

            file
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.truncate(Some(file), NumBytes::from(NUM_BYTES_FOR_THREE_LEVEL_TREE))
                .await
                .unwrap();
        })
        .await;

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: 1,
                blob_resize: 2,
                blob_num_bytes: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 63,
                    FixtureType::Fusemt => 64,
                    FixtureType::FuserWithoutInodeCache => 65, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_create: 56,
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 336,
                    FixtureType::Fusemt => 345,
                    FixtureType::FuserWithoutInodeCache => 352, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 57,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                exists: 56,
                load: 2,
                store: 58,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn truncate_file_in_rootdir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create a file so we have something to truncate
    let file = fixture
        .ops(async |fs| {
            fs.create_file(None, PathComponent::try_from_str("testfile.txt").unwrap())
                .await
                .unwrap()
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.truncate(Some(file), NumBytes::from(1024)).await.unwrap();
        })
        .await;

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: 1,
                blob_resize: 2,
                blob_num_bytes: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 10,
                    FixtureType::Fusemt => 11,
                    FixtureType::FuserWithoutInodeCache => 12, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_create: 5,
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 79,
                    FixtureType::Fusemt => 88,
                    FixtureType::FuserWithoutInodeCache => 95, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 16,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                exists: 5,
                load: 2,
                store: 7,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn truncate_file_in_nesteddir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create a nested directory and a file in it
    let file = fixture
        .ops(async |fs| {
            let parent = fs
                .mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap();
            fs.create_file(
                Some(parent),
                PathComponent::try_from_str("testfile.txt").unwrap(),
            )
            .await
            .unwrap()
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.truncate(Some(file), NumBytes::from(1024)).await.unwrap();
        })
        .await;

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 7, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 3,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 7, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: 1,
                blob_resize: 2,
                blob_num_bytes: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 10,
                    FixtureType::Fusemt => 12,
                    FixtureType::FuserWithoutInodeCache => 14, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_create: 5,
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 79,
                    FixtureType::Fusemt => 97,
                    FixtureType::FuserWithoutInodeCache => 113, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 16,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                exists: 5,
                load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 3,
                },
                store: 7,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn truncate_file_in_deeplynesteddir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create a deeply nested directory
    let nested_dir = fixture
        .ops(async |fs| {
            fs.mkdir_recursive(AbsolutePath::try_from_str("/nested1/nested2/nested3").unwrap())
                .await
                .unwrap()
        })
        .await;

    // Then create a file in that directory
    let file = fixture
        .ops(async |fs| {
            fs.create_file(
                Some(nested_dir.clone()),
                PathComponent::try_from_str("testfile.txt").unwrap(),
            )
            .await
            .unwrap()
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.truncate(Some(file), NumBytes::from(1024)).await.unwrap();
        })
        .await;

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 7,
                    FixtureType::FuserWithoutInodeCache => 11, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 8, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 7,
                    FixtureType::FuserWithoutInodeCache => 11, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_resize: 2,
                blob_num_bytes: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 10,
                    FixtureType::Fusemt => 14,
                    FixtureType::FuserWithoutInodeCache => 18, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_create: 5,
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 79,
                    FixtureType::Fusemt => 115,
                    FixtureType::FuserWithoutInodeCache => 149, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 16,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                exists: 5,
                load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store: 7,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn truncate_dir_fails(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    let dir = fixture
        .ops(async |fs| {
            fs.mkdir(None, PathComponent::try_from_str("testdir").unwrap())
                .await
                .unwrap()
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.truncate(Some(dir), NumBytes::from(1024))
                .await
                .expect_err("Truncating a directory should fail");
        })
        .await;

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 3, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 3, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 3, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 3, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 18,
                    FixtureType::FuserWithoutInodeCache => 27, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 2,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn truncate_symlink_fails(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    let symlink = fixture
        .ops(async |fs| {
            fs.create_symlink(
                None,
                PathComponent::try_from_str("symlink").unwrap(),
                AbsolutePath::try_from_str("/target.txt").unwrap(),
            )
            .await
            .unwrap()
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.truncate(Some(symlink), NumBytes::from(1024))
                .await
                .expect_err("Truncating a symlink should fail");
        })
        .await;

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 3, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 3, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 3, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 16,
                    FixtureType::FuserWithoutInodeCache => 25, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 2,
                ..LLActionCounts::ZERO
            },
        }
    );
}
