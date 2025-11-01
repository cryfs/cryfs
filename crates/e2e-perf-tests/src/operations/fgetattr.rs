use crate::filesystem_driver::FilesystemDriver;
use crate::filesystem_fixture::ActionCounts;
use crate::perf_test_macro::FixtureType;
use crate::test_driver::TestDriver;
use crate::test_driver::TestReady;
use crate::utils::maybe_close;
use cryfs_blobstore::BlobStoreActionCounts;
use cryfs_blockstore::HLActionCounts;
use cryfs_blockstore::LLActionCounts;
use cryfs_rustfs::AbsolutePath;
use cryfs_rustfs::PathComponent;

crate::perf_test_macro::perf_test!(
    fgetattr,
    [
        file_in_rootdir::<false>,
        file_in_rootdir::<true>,
        file_in_nesteddir::<false>,
        file_in_nesteddir::<true>,
        file_in_deeplynesteddir::<false>,
        file_in_deeplynesteddir::<true>,
    ]
);

fn file_in_rootdir<const CLOSE_AFTER: bool>(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("testfile.txt").unwrap())
                .await
                .unwrap()
        })
        .test(async |fixture, (file_ino, file_fh)| {
            fixture
                .filesystem
                .fgetattr(file_ino.clone(), &file_fh)
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file_ino, file_fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO: Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + 2 * close_after,
                    FixtureType::FuserWithoutInodeCache => 4 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                    FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + 2 * close_after,
                    FixtureType::FuserWithoutInodeCache => 4 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_num_bytes: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                    FixtureType::FuserWithoutInodeCache => 2 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_flush: close_after,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO: Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + 2 * close_after,
                    FixtureType::FuserWithoutInodeCache => 4 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 16 + 16 * close_after,
                    FixtureType::FuserWithoutInodeCache => 32 + 32 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_flush_block: close_after,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO: Check if these counts are what we'd expect
                load: 2,
                ..LLActionCounts::ZERO
            },
            }
        })
}

fn file_in_nesteddir<const CLOSE_AFTER: bool>(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            let parent = fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap();
            fixture
                .filesystem
                .create_and_open_file(
                    Some(parent),
                    PathComponent::try_from_str("testfile.txt").unwrap(),
                )
                .await
                .unwrap()
        })
        .test(async |fixture, (file_ino, file_fh)| {
            fixture
                .filesystem
                .fgetattr(file_ino.clone(), &file_fh)
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file_ino, file_fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO: Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + 2 * close_after,
                    FixtureType::FuserWithoutInodeCache => 6 + 6 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                    FixtureType::FuserWithoutInodeCache => 4 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + 2 * close_after,
                    FixtureType::FuserWithoutInodeCache => 6 + 6 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_num_bytes: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                    FixtureType::FuserWithoutInodeCache => 2 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_flush: close_after,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO: Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + 2 * close_after,
                    FixtureType::FuserWithoutInodeCache => 6 + 6 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 16 + 16 * close_after,
                    FixtureType::FuserWithoutInodeCache => 50 + 50 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_flush_block: close_after,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO: Check if these counts are what we'd expect
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 3, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..LLActionCounts::ZERO
            },
            }
        })
}

fn file_in_deeplynesteddir<const CLOSE_AFTER: bool>(
    test_driver: impl TestDriver,
) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create a deeply nested directory
            let nested_dir = fixture
                .filesystem
                .mkdir_recursive(AbsolutePath::try_from_str("/nested1/nested2/nested3").unwrap())
                .await
                .unwrap();
            // Then create a file in that directory
            fixture
                .filesystem
                .create_and_open_file(
                    Some(nested_dir),
                    PathComponent::try_from_str("testfile.txt").unwrap(),
                )
                .await
                .unwrap()
        })
        .test(async |fixture, (file_ino, file_fh)| {
            fixture
                .filesystem
                .fgetattr(file_ino.clone(), &file_fh)
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file_ino, file_fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO: Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + 2 * close_after,
                    FixtureType::FuserWithoutInodeCache => 10 + 10 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                    FixtureType::FuserWithoutInodeCache => 8 + 8 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + 2 * close_after,
                    FixtureType::FuserWithoutInodeCache => 10 + 10 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_num_bytes: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                    FixtureType::FuserWithoutInodeCache => 2 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_flush: close_after,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO: Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + 2 * close_after,
                    FixtureType::FuserWithoutInodeCache => 10 + 10 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 16 + 16 * close_after,
                    FixtureType::FuserWithoutInodeCache => 86 + 86 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_flush_block: close_after,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO: Check if these counts are what we'd expect
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..LLActionCounts::ZERO
            },
            }
        })
}
