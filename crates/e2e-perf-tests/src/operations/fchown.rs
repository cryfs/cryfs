use crate::filesystem_driver::FilesystemDriver;
use crate::filesystem_fixture::ActionCounts;
use crate::perf_test_macro::FixtureType;
use crate::test_driver::TestDriver;
use crate::test_driver::TestReady;
use crate::utils::maybe_close;
use cryfs_blobstore::BlobStoreActionCounts;
use cryfs_blockstore::HLActionCounts;
use cryfs_blockstore::LLActionCounts;
use cryfs_rustfs::Gid;
use cryfs_rustfs::Uid;
use cryfs_utils::path::{AbsolutePath, PathComponent};

crate::perf_test_macro::perf_test!(
    fchown,
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
        .test(async |fixture, (file, file_handle)| {
            fixture
                .filesystem
                .fchown(
                    file.clone(),
                    &file_handle,
                    Some(Uid::from(1000)),
                    Some(Gid::from(1000)),
                )
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, file_handle).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_write: close_after,
                    blob_resize: close_after,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                        FixtureType::FuserWithoutInodeCache => 2 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: 2 * close_after,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            7 + 9 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 14 + 16 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data_mut: close_after,
                    store_flush_block: 2 * close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: 1,
                    store: close_after,
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
        .test(async |fixture, (file, file_handle)| {
            fixture
                .filesystem
                .fchown(
                    file.clone(),
                    &file_handle,
                    Some(Uid::from(1000)),
                    Some(Gid::from(1000)),
                )
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, file_handle).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            1 + 1 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 3 + 3 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + 1 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            1 + 1 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 3 + 3 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_write: close_after,
                    blob_resize: close_after,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                        FixtureType::FuserWithoutInodeCache => 2 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: 2 * close_after,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            1 + 1 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 3 + 3 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            7 + 9 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 23 + 25 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data_mut: close_after,
                    store_flush_block: 2 * close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                        FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store: close_after,
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
            // Then create and open a file in that directory
            fixture
                .filesystem
                .create_and_open_file(
                    Some(nested_dir),
                    PathComponent::try_from_str("testfile.txt").unwrap(),
                )
                .await
                .unwrap()
        })
        .test(async |fixture, (file, file_handle)| {
            fixture
                .filesystem
                .fchown(
                    file.clone(),
                    &file_handle,
                    Some(Uid::from(1000)),
                    Some(Gid::from(1000)),
                )
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, file_handle).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            1 + 1 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 7 + 7 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 5 + 5 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            1 + 1 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 7 + 7 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_resize: close_after,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                        FixtureType::FuserWithoutInodeCache => 2 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_write: close_after,
                    blob_flush: 2 * close_after,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            1 + 1 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 7 + 7 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            7 + 9 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 59 + 61 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data_mut: close_after,
                    store_flush_block: 2 * close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                        FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store: close_after,
                    ..LLActionCounts::ZERO
                },
            }
        })
}
