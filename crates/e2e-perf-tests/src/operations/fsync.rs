use pretty_assertions::assert_eq;
use rstest::rstest;
use rstest_reuse::apply;

use crate::filesystem_driver::FilesystemDriver as _;
use crate::fixture::ActionCounts;
use crate::fixture::BLOCKSIZE_BYTES;
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

// TODO Same TODOs as in flush.rs
//    - Some flush operations in here seem to load blocks in low_level, i.e. below the cache??? Why is that? If it's not loaded, shouldn't we just ignore it since it's already flushed? Also, generally, for a simple flush, there's a lot of operations going on in the high level stores.
//    - Some flush-after-write operations in here don't have a store in low level, that's weird. Shouldn't they need to store to flush the write?

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn unchanged_empty_file_in_rootdir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
    #[values(true, false)] datasync: bool,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create and open a file to flush
    let (file, fh) = fixture
        .ops(async |fs| {
            fs.create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap()
        })
        .await;

    let counts = fixture
        .count_ops_noflush(async |fs| {
            fs.fsync(file.clone(), &fh, datasync).await.unwrap();
        })
        .await;

    let datasync = if datasync { 1 } else { 0 };

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 4 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 - datasync,
                    FixtureType::FuserWithoutInodeCache => 2 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 4 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_num_bytes: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                    FixtureType::FuserWithoutInodeCache => 1, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_flush: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 4 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 16 - 9 * datasync,
                    FixtureType::FuserWithoutInodeCache => 32 - 9 * datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_flush_block: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn unchanged_file_with_data_in_rootdir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
    #[values(true, false)] datasync: bool,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create and open a file, write some data, then flush
    let (file, fh) = fixture
        .ops(async |fs| {
            let (file, fh) = fs
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();
            let data = vec![b'X'; 100];
            fs.write(file.clone(), &fh, NumBytes::from(0), data)
                .await
                .unwrap();
            (file, fh)
        })
        .await;

    let counts = fixture
        .count_ops_noflush(async |fs| {
            fs.fsync(file.clone(), &fh, datasync).await.unwrap();
        })
        .await;

    let datasync = if datasync { 1 } else { 0 };

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 4 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 - datasync,
                    FixtureType::FuserWithoutInodeCache => 2 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 4 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_num_bytes: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                    FixtureType::FuserWithoutInodeCache => 1, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_flush: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 4 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_flush_block: 1,
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 16 - 9 * datasync,
                    FixtureType::FuserWithoutInodeCache => 32 - 9 * datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn unchanged_large_file_in_rootdir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
    #[values(true, false)] datasync: bool,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create and open a file, write a large amount of data, then flush
    let (file, fh) = fixture
        .ops(async |fs| {
            let (file, fh) = fs
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();
            let data = vec![b'X'; NUM_BYTES_FOR_THREE_LEVEL_TREE as usize];
            fs.write(file.clone(), &fh, NumBytes::from(0), data)
                .await
                .unwrap();
            (file, fh)
        })
        .await;

    let counts = fixture
        .count_ops_noflush(async |fs| {
            fs.fsync(file.clone(), &fh, datasync).await.unwrap();
        })
        .await;

    let datasync = if datasync { 1 } else { 0 };

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 4 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 - datasync,
                    FixtureType::FuserWithoutInodeCache => 2 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 4 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_flush: 1,
                blob_num_bytes: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                    FixtureType::FuserWithoutInodeCache => 1, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 6 - datasync,
                    FixtureType::FuserWithoutInodeCache => 12 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_flush_block: 1,
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 44 - 9 * datasync,
                    FixtureType::FuserWithoutInodeCache => 88 - 9 * datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 6 - datasync,
                    FixtureType::FuserWithoutInodeCache => 6, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn unchanged_file_in_nested_dir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
    #[values(true, false)] datasync: bool,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create a nested directory with a file
    let (file, fh) = fixture
        .ops(async |fs| {
            let dir = fs
                .mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap();
            fs.create_and_open_file(Some(dir), PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap()
        })
        .await;

    let counts = fixture
        .count_ops_noflush(async |fs| {
            fs.fsync(file.clone(), &fh, datasync).await.unwrap();
        })
        .await;

    let datasync = if datasync { 1 } else { 0 };

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 6 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 - datasync,
                    FixtureType::FuserWithoutInodeCache => 4 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 6 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_num_bytes: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                    FixtureType::FuserWithoutInodeCache => 1, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_flush: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 6 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_flush_block: 1,
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 16 - 9 * datasync,
                    FixtureType::FuserWithoutInodeCache => 50 - 9 * datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 3, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn unchanged_file_in_deeply_nested_dir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
    #[values(true, false)] datasync: bool,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create a deeply nested directory with a file
    let (file, fh) = fixture
        .ops(async |fs| {
            let deeply_nested = fs
                .mkdir_recursive(AbsolutePath::try_from_str("/deep/nested/dir").unwrap())
                .await
                .unwrap();
            fs.create_and_open_file(
                Some(deeply_nested),
                PathComponent::try_from_str("file.txt").unwrap(),
            )
            .await
            .unwrap()
        })
        .await;

    let counts = fixture
        .count_ops_noflush(async |fs| {
            fs.fsync(file.clone(), &fh, datasync).await.unwrap();
        })
        .await;

    let datasync = if datasync { 1 } else { 0 };

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 10 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 - datasync,
                    FixtureType::FuserWithoutInodeCache => 8 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 10 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_flush: 1,
                blob_num_bytes: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                    FixtureType::FuserWithoutInodeCache => 1, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 10 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_flush_block: 1,
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 16 - 9 * datasync,
                    FixtureType::FuserWithoutInodeCache => 86 - 9 * datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn after_small_write_to_empty_file(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
    #[values(true, false)] datasync: bool,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create and open a file, perform a small write operation
    let (file, fh) = fixture
        .ops(async |fs| {
            let (file, fh) = fs
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();

            (file, fh)
        })
        .await;

    fixture
        .ops_noflush(async |fs| {
            // Perform small write without flushing
            let data = vec![b'A'; 1];
            fs.write(file.clone(), &fh, NumBytes::from(0), data)
                .await
                .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops_noflush(async |fs| {
            fs.fsync(file.clone(), &fh, datasync).await.unwrap();
        })
        .await;

    let datasync = if datasync { 1 } else { 0 };

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 4 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 - datasync,
                    FixtureType::FuserWithoutInodeCache => 2 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 4 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_num_bytes: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                    FixtureType::FuserWithoutInodeCache => 1, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_flush: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 4 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_flush_block: 1,
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 16 - 9 * datasync,
                    FixtureType::FuserWithoutInodeCache => 32 - 9 * datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                store: 1,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn after_small_write_to_middle_of_small_file(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
    #[values(true, false)] datasync: bool,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create and open a file, write some initial data, then write a small amount in the middle
    let (file, fh) = fixture
        .ops(async |fs| {
            let (file, fh) = fs
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();

            // Write initial data
            let initial_data = vec![b'X'; 2 * BLOCKSIZE_BYTES as usize];
            fs.write(file.clone(), &fh, NumBytes::from(0), initial_data)
                .await
                .unwrap();
            (file, fh)
        })
        .await;

    fixture
        .ops_noflush(async |fs| {
            // Write a small amount in the middle
            let data = vec![b'A'; 1];
            fs.write(file.clone(), &fh, NumBytes::from(BLOCKSIZE_BYTES), data)
                .await
                .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops_noflush(async |fs| {
            fs.fsync(file.clone(), &fh, datasync).await.unwrap();
        })
        .await;

    let datasync = if datasync { 1 } else { 0 };

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 4 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 - datasync,
                    FixtureType::FuserWithoutInodeCache => 2 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 4 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_flush: 1,
                blob_num_bytes: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                    FixtureType::FuserWithoutInodeCache => 1, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 4 - datasync,
                    FixtureType::FuserWithoutInodeCache => 8 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_flush_block: 1,
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 30 - 9 * datasync,
                    FixtureType::FuserWithoutInodeCache => 60 - 9 * datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                // TODO Why no store? Shouldn't this store to flush the write?
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn after_small_write_beyond_end_of_small_file(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
    #[values(true, false)] datasync: bool,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create and open a file, write some initial data, then write a small amount beyond its end
    let (file, fh) = fixture
        .ops(async |fs| {
            let (file, fh) = fs
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();

            // Write initial data
            let initial_data = vec![b'X'; 2 * BLOCKSIZE_BYTES as usize];
            fs.write(file.clone(), &fh, NumBytes::from(0), initial_data)
                .await
                .unwrap();
            (file, fh)
        })
        .await;

    fixture
        .ops_noflush(async |fs| {
            // Write a small amount beyond the end
            let data = vec![b'A'; 1];
            fs.write(file.clone(), &fh, NumBytes::from(3 * BLOCKSIZE_BYTES), data)
                .await
                .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops_noflush(async |fs| {
            fs.fsync(file.clone(), &fh, datasync).await.unwrap();
        })
        .await;

    let datasync = if datasync { 1 } else { 0 };

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 4 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 - datasync,
                    FixtureType::FuserWithoutInodeCache => 2 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 4 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_flush: 1,
                blob_num_bytes: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                    FixtureType::FuserWithoutInodeCache => 1, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 4 - datasync,
                    FixtureType::FuserWithoutInodeCache => 8 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_flush_block: 1,
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 30 - 9 * datasync,
                    FixtureType::FuserWithoutInodeCache => 60 - 9 * datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                store: 1,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn after_small_write_to_middle_of_large_file(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
    #[values(true, false)] datasync: bool,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create and open a file, write some initial large data, then write a small amount in the middle
    let (file, fh) = fixture
        .ops(async |fs| {
            let (file, fh) = fs
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();

            // Write initial large data
            let initial_data = vec![b'X'; 2 * NUM_BYTES_FOR_THREE_LEVEL_TREE as usize];
            fs.write(file.clone(), &fh, NumBytes::from(0), initial_data)
                .await
                .unwrap();
            (file, fh)
        })
        .await;

    fixture
        .ops_noflush(async |fs| {
            // Write a small amount in the middle
            let data = vec![b'A'; 1];
            fs.write(
                file.clone(),
                &fh,
                NumBytes::from(NUM_BYTES_FOR_THREE_LEVEL_TREE),
                data,
            )
            .await
            .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops_noflush(async |fs| {
            fs.fsync(file.clone(), &fh, datasync).await.unwrap();
        })
        .await;

    let datasync = if datasync { 1 } else { 0 };

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 4 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 - datasync,
                    FixtureType::FuserWithoutInodeCache => 2 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 4 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_flush: 1,
                blob_num_bytes: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                    FixtureType::FuserWithoutInodeCache => 1, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 6 - datasync,
                    FixtureType::FuserWithoutInodeCache => 12 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_flush_block: 1,
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 44 - 9 * datasync,
                    FixtureType::FuserWithoutInodeCache => 88 - 9 * datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                // TODO Why no store? Shouldn't this store to flush the write?
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn after_small_write_beyond_end_of_large_file(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
    #[values(true, false)] datasync: bool,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create and open a file, write some initial large data, then write a small amount beyond its end
    let (file, fh) = fixture
        .ops(async |fs| {
            let (file, fh) = fs
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();

            // Write initial large data
            let initial_data = vec![b'X'; 2 * NUM_BYTES_FOR_THREE_LEVEL_TREE as usize];
            fs.write(file.clone(), &fh, NumBytes::from(0), initial_data)
                .await
                .unwrap();
            (file, fh)
        })
        .await;

    fixture
        .ops_noflush(async |fs| {
            // Write a small amount beyond the end
            let data = vec![b'A'; 1];
            fs.write(
                file.clone(),
                &fh,
                NumBytes::from(3 * NUM_BYTES_FOR_THREE_LEVEL_TREE),
                data,
            )
            .await
            .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops_noflush(async |fs| {
            fs.fsync(file.clone(), &fh, datasync).await.unwrap();
        })
        .await;

    let datasync = if datasync { 1 } else { 0 };

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 4 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 - datasync,
                    FixtureType::FuserWithoutInodeCache => 2 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 4 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_flush: 1,
                blob_num_bytes: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                    FixtureType::FuserWithoutInodeCache => 1, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 6 - datasync,
                    FixtureType::FuserWithoutInodeCache => 12 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_flush_block: 1,
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 44 - 9 * datasync,
                    FixtureType::FuserWithoutInodeCache => 88 - 9 * datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                store: 1,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn after_large_write_to_empty_file(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
    #[values(true, false)] datasync: bool,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create and open a file, perform a small write operation
    let (file, fh) = fixture
        .ops(async |fs| {
            let (file, fh) = fs
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();

            (file, fh)
        })
        .await;

    fixture
        .ops_noflush(async |fs| {
            // Perform small write without flushing
            let data = vec![b'A'; 2 * NUM_BYTES_FOR_THREE_LEVEL_TREE as usize];
            fs.write(file.clone(), &fh, NumBytes::from(0), data)
                .await
                .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops_noflush(async |fs| {
            fs.fsync(file.clone(), &fh, datasync).await.unwrap();
        })
        .await;

    let datasync = if datasync { 1 } else { 0 };

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 4 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 - datasync,
                    FixtureType::FuserWithoutInodeCache => 2 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 4 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_flush: 1,
                blob_num_bytes: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                    FixtureType::FuserWithoutInodeCache => 1, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 6 - datasync,
                    FixtureType::FuserWithoutInodeCache => 12 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_flush_block: 1,
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 44 - 9 * datasync,
                    FixtureType::FuserWithoutInodeCache => 88 - 9 * datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                store: 1,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn after_large_write_to_middle_of_large_file(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
    #[values(true, false)] datasync: bool,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create and open a file, write some initial large data, then write large data in the middle
    let (file, fh) = fixture
        .ops(async |fs| {
            let (file, fh) = fs
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();

            // Write initial large data
            let initial_data = vec![b'X'; 3 * NUM_BYTES_FOR_THREE_LEVEL_TREE as usize];
            fs.write(file.clone(), &fh, NumBytes::from(0), initial_data)
                .await
                .unwrap();

            (file, fh)
        })
        .await;

    fixture
        .ops_noflush(async |fs| {
            // Write large data in the middle
            let data = vec![b'A'; NUM_BYTES_FOR_THREE_LEVEL_TREE as usize];
            fs.write(
                file.clone(),
                &fh,
                NumBytes::from(NUM_BYTES_FOR_THREE_LEVEL_TREE),
                data,
            )
            .await
            .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops_noflush(async |fs| {
            fs.fsync(file.clone(), &fh, datasync).await.unwrap();
        })
        .await;

    let datasync = if datasync { 1 } else { 0 };

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 4 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 - datasync,
                    FixtureType::FuserWithoutInodeCache => 2 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 4 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_flush: 1,
                blob_num_bytes: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                    FixtureType::FuserWithoutInodeCache => 1, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 6 - datasync,
                    FixtureType::FuserWithoutInodeCache => 12 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_flush_block: 1,
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 44 - 9 * datasync,
                    FixtureType::FuserWithoutInodeCache => 88 - 9 * datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                // TODO Why no store? Shouldn't this store to flush the write?
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn after_large_write_beyond_end_of_large_file(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
    #[values(true, false)] datasync: bool,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create and open a file, write some initial large data, then write large data beyond its end
    let (file, fh) = fixture
        .ops(async |fs| {
            let (file, fh) = fs
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();

            // Write initial large data
            let initial_data = vec![b'X'; NUM_BYTES_FOR_THREE_LEVEL_TREE as usize];
            fs.write(file.clone(), &fh, NumBytes::from(0), initial_data)
                .await
                .unwrap();

            (file, fh)
        })
        .await;

    fixture
        .ops_noflush(async |fs| {
            // Write large data beyond the end
            let data = vec![b'A'; NUM_BYTES_FOR_THREE_LEVEL_TREE as usize];
            fs.write(
                file.clone(),
                &fh,
                NumBytes::from(2 * NUM_BYTES_FOR_THREE_LEVEL_TREE),
                data,
            )
            .await
            .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops_noflush(async |fs| {
            fs.fsync(file.clone(), &fh, datasync).await.unwrap();
        })
        .await;

    let datasync = if datasync { 1 } else { 0 };

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 4 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 - datasync,
                    FixtureType::FuserWithoutInodeCache => 2 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 4 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_flush: 1,
                blob_num_bytes: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                    FixtureType::FuserWithoutInodeCache => 1, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 6 - datasync,
                    FixtureType::FuserWithoutInodeCache => 12 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_flush_block: 1,
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 44 - 9 * datasync,
                    FixtureType::FuserWithoutInodeCache => 88 - 9 * datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                store: 1,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn after_write_to_file_in_nested_dir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
    #[values(true, false)] datasync: bool,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create a nested directory and file, then write to the file
    let (file, fh) = fixture
        .ops(async |fs| {
            let parent = fs
                .mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap();

            let (file, fh) = fs
                .create_and_open_file(
                    Some(parent),
                    PathComponent::try_from_str("nestedfile.txt").unwrap(),
                )
                .await
                .unwrap();

            (file, fh)
        })
        .await;

    fixture
        .ops_noflush(async |fs| {
            // Write to the file
            let data = vec![b'A'; 1];
            fs.write(file.clone(), &fh, NumBytes::from(0), data)
                .await
                .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops_noflush(async |fs| {
            fs.fsync(file.clone(), &fh, datasync).await.unwrap();
        })
        .await;

    let datasync = if datasync { 1 } else { 0 };

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 6 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 - datasync,
                    FixtureType::FuserWithoutInodeCache => 4 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 6 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_flush: 1,
                blob_num_bytes: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                    FixtureType::FuserWithoutInodeCache => 1, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 6 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_flush_block: 1,
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 16 - 9 * datasync,
                    FixtureType::FuserWithoutInodeCache => 50 - 9 * datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 0,
                store: 1,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn after_small_write_to_file_in_deeply_nested_dir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
    #[values(true, false)] datasync: bool,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create a deeply nested directory and file, then write to the file
    let (file, fh) = fixture
        .ops(async |fs| {
            let deeply_nested = fs
                .mkdir_recursive(AbsolutePath::try_from_str("/deep/nested/dir").unwrap())
                .await
                .unwrap();

            let (file, fh) = fs
                .create_and_open_file(
                    Some(deeply_nested),
                    PathComponent::try_from_str("deepfile.txt").unwrap(),
                )
                .await
                .unwrap();

            (file, fh)
        })
        .await;

    fixture
        .ops_noflush(async |fs| {
            // Write to the file
            let data = vec![b'A'; 1];
            fs.write(file.clone(), &fh, NumBytes::from(0), data)
                .await
                .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops_noflush(async |fs| {
            fs.fsync(file.clone(), &fh, datasync).await.unwrap();
        })
        .await;

    let datasync = if datasync { 1 } else { 0 };

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 10 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 - datasync,
                    FixtureType::FuserWithoutInodeCache => 8 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 10 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_flush: 1,
                blob_num_bytes: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                    FixtureType::FuserWithoutInodeCache => 1, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 - datasync,
                    FixtureType::FuserWithoutInodeCache => 10 - datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_flush_block: 1,
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 16 - 9 * datasync,
                    FixtureType::FuserWithoutInodeCache => 86 - 9 * datasync, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                store: 1,
                ..LLActionCounts::ZERO
            },
        }
    );
}
