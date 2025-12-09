use crate::filesystem_driver::FilesystemDriver;
use crate::filesystem_fixture::ActionCounts;
use crate::filesystem_fixture::NUM_BYTES_FOR_THREE_LEVEL_TREE;
use crate::perf_test_macro::FixtureType;
use crate::test_driver::TestDriver;
use crate::test_driver::TestReady;
use crate::utils::maybe_close;
use cryfs_blobstore::BlobStoreActionCounts;
use cryfs_blockstore::HLActionCounts;
use cryfs_blockstore::LLActionCounts;
use cryfs_rustfs::NumBytes;
use cryfs_utils::path::{AbsolutePath, PathComponent};

crate::perf_test_macro::perf_test!(
    ftruncate,
    [
        grow_empty_file_small::<false>,
        grow_empty_file_small::<true>,
        grow_empty_file_large::<false>,
        grow_empty_file_large::<true>,
        shrink_file_small::<false>,
        shrink_file_small::<true>,
        shrink_file_large::<false>,
        shrink_file_large::<true>,
        grow_nonempty_file_small::<false>,
        grow_nonempty_file_small::<true>,
        grow_nonempty_file_large::<false>,
        grow_nonempty_file_large::<true>,
        file_in_rootdir::<false>,
        file_in_rootdir::<true>,
        file_in_nesteddir::<false>,
        file_in_nesteddir::<true>,
        file_in_deeplynesteddir::<false>,
        file_in_deeplynesteddir::<true>,
    ]
);

fn grow_empty_file_small<const CLOSE_AFTER: bool>(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file so we have an empty file to grow
            fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("testfile.txt").unwrap())
                .await
                .unwrap()
        })
        .test(async |fixture, (file, file_handle)| {
            // Grow by a small amount (1 byte)
            fixture
                .filesystem
                .ftruncate(file.clone(), &file_handle, NumBytes::from(1))
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
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + close_after,
                        FixtureType::FuserWithoutInodeCache => 3 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + close_after,
                        FixtureType::FuserWithoutInodeCache => 3 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_write: close_after,
                    blob_resize: 1 + close_after,
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
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + close_after,
                        FixtureType::FuserWithoutInodeCache => 3 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            16 + 9 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 23 + 16 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data_mut: 1 + close_after,
                    store_flush_block: 2 * close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: 1,
                    store: 1 + close_after,
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn grow_empty_file_large<const CLOSE_AFTER: bool>(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file so we have an empty file to grow
            fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("testfile.txt").unwrap())
                .await
                .unwrap()
        })
        .test(async |fixture, (file, file_handle)| {
            fixture
                .filesystem
                .ftruncate(
                    file.clone(),
                    &file_handle,
                    NumBytes::from(NUM_BYTES_FOR_THREE_LEVEL_TREE),
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
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + close_after,
                        FixtureType::FuserWithoutInodeCache => 3 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + close_after,
                        FixtureType::FuserWithoutInodeCache => 3 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_write: close_after,
                    blob_resize: 1 + close_after,
                    blob_flush: 2 * close_after,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                        FixtureType::FuserWithoutInodeCache => 2 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            31 + 5 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 32 + 10 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_create: 25,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            186 + 37 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 193 + 72 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_flush_block: 2 * close_after,
                    blob_data_mut: 35 + close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    exists: 25,
                    load: 1,
                    store: 26 + close_after,
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn shrink_file_small<const CLOSE_AFTER: bool>(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file, and then make it large
            let (file, handle) = fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("testfile.txt").unwrap())
                .await
                .unwrap();
            fixture
                .filesystem
                .ftruncate(
                    file.clone(),
                    &handle,
                    NumBytes::from(NUM_BYTES_FOR_THREE_LEVEL_TREE),
                )
                .await
                .unwrap();

            (file, handle)
        })
        .test(async |fixture, (file, file_handle)| {
            // Now shrink it down by a small amount (1 byte less)
            fixture
                .filesystem
                .ftruncate(
                    file.clone(),
                    &file_handle,
                    NumBytes::from(NUM_BYTES_FOR_THREE_LEVEL_TREE - 1),
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
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + close_after,
                        FixtureType::FuserWithoutInodeCache => 3 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + close_after,
                        FixtureType::FuserWithoutInodeCache => 3 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_write: close_after,
                    blob_resize: 1 + close_after,
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
                            12 + 5 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 17 + 10 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            95 + 37 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 130 + 72 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data_mut: 3 + close_after,
                    store_flush_block: 2 * close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: 5,
                    store: 3 + close_after,
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn shrink_file_large<const CLOSE_AFTER: bool>(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file, and then make it large
            let (file, handle) = fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("testfile.txt").unwrap())
                .await
                .unwrap();
            fixture
                .filesystem
                .ftruncate(
                    file.clone(),
                    &handle,
                    NumBytes::from(NUM_BYTES_FOR_THREE_LEVEL_TREE),
                )
                .await
                .unwrap();

            (file, handle)
        })
        .test(async |fixture, (file, file_handle)| {
            fixture
                .filesystem
                .ftruncate(file.clone(), &file_handle, NumBytes::from(1))
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
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + close_after,
                        FixtureType::FuserWithoutInodeCache => 3 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + close_after,
                        FixtureType::FuserWithoutInodeCache => 3 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_write: close_after,
                    blob_resize: 1 + close_after,
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
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 12 + close_after,
                        FixtureType::FuserWithoutInodeCache => 17 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_remove: 4,
                    store_remove_by_id: 21,
                    store_flush_block: 2 * close_after,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            97 + 9 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 132 + 16 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data_mut: 4 + close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: 6,
                    store: 1 + close_after,
                    remove: 25,
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn grow_nonempty_file_small<const CLOSE_AFTER: bool>(
    test_driver: impl TestDriver,
) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create a file with some data
            let (file, handle) = fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("testfile.txt").unwrap())
                .await
                .unwrap();
            fixture
                .filesystem
                .ftruncate(file.clone(), &handle, NumBytes::from(100))
                .await
                .unwrap();

            (file, handle)
        })
        .test(async |fixture, (file, file_handle)| {
            // Now grow it by a small amount (add 1 byte)
            fixture
                .filesystem
                .ftruncate(file.clone(), &file_handle, NumBytes::from(101))
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
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + close_after,
                        FixtureType::FuserWithoutInodeCache => 3 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + close_after,
                        FixtureType::FuserWithoutInodeCache => 3 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_write: close_after,
                    blob_resize: 1 + close_after,
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
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + close_after,
                        FixtureType::FuserWithoutInodeCache => 3 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            16 + 9 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 23 + 16 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data_mut: 1 + close_after,
                    store_flush_block: 2 * close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: 1,
                    store: 1 + close_after,
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn grow_nonempty_file_large<const CLOSE_AFTER: bool>(
    test_driver: impl TestDriver,
) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create a file with some data
            let (file, handle) = fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("testfile.txt").unwrap())
                .await
                .unwrap();
            fixture
                .filesystem
                .ftruncate(file.clone(), &handle, NumBytes::from(100))
                .await
                .unwrap();

            (file, handle)
        })
        .test(async |fixture, (file, file_handle)| {
            fixture
                .filesystem
                .ftruncate(
                    file.clone(),
                    &file_handle,
                    NumBytes::from(NUM_BYTES_FOR_THREE_LEVEL_TREE),
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
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + close_after,
                        FixtureType::FuserWithoutInodeCache => 3 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + close_after,
                        FixtureType::FuserWithoutInodeCache => 3 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_write: close_after,
                    blob_resize: 1 + close_after,
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
                            31 + 5 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 32 + 10 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_create: 25,
                    store_flush_block: 2 * close_after,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            186 + 37 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 193 + 72 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data_mut: 35 + close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    exists: 25,
                    load: 1,
                    store: 26 + close_after,
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn file_in_rootdir<const CLOSE_AFTER: bool>(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file so we have something to truncate
            fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("testfile.txt").unwrap())
                .await
                .unwrap()
        })
        .test(async |fixture, (file, file_handle)| {
            fixture
                .filesystem
                .ftruncate(file.clone(), &file_handle, NumBytes::from(1024))
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
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + close_after,
                        FixtureType::FuserWithoutInodeCache => 3 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + close_after,
                        FixtureType::FuserWithoutInodeCache => 3 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_write: close_after,
                    blob_resize: 1 + close_after,
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
                            11 + 3 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 12 + 6 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_create: 7,
                    store_flush_block: 2 * close_after,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            80 + 23 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 87 + 44 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data_mut: 21 + close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    exists: 7,
                    load: 1,
                    store: 8 + close_after,
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn file_in_nesteddir<const CLOSE_AFTER: bool>(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // Create a nested directory and a file in it
            let parent = fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap();

            let (file, file_handle) = fixture
                .filesystem
                .create_and_open_file(
                    Some(parent.clone()),
                    PathComponent::try_from_str("testfile.txt").unwrap(),
                )
                .await
                .unwrap();

            (file, file_handle)
        })
        .test(async |fixture, (file, file_handle)| {
            fixture
                .filesystem
                .ftruncate(file.clone(), &file_handle, NumBytes::from(1024))
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
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + close_after,
                        FixtureType::FuserWithoutInodeCache => 4 + 3 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + close_after,
                        FixtureType::FuserWithoutInodeCache => 4 + 3 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_write: close_after,
                    blob_resize: 1 + close_after,
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
                            11 + 3 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 13 + 7 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_create: 7,
                    store_flush_block: 2 * close_after,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            80 + 23 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 96 + 53 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data_mut: 21 + close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    exists: 7,
                    load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                        FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store: 8 + close_after,
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
            let (file, file_handle) = fixture
                .filesystem
                .create_and_open_file(
                    Some(nested_dir.clone()),
                    PathComponent::try_from_str("testfile.txt").unwrap(),
                )
                .await
                .unwrap();

            (file, file_handle)
        })
        .test(async |fixture, (file, file_handle)| {
            fixture
                .filesystem
                .ftruncate(file.clone(), &file_handle, NumBytes::from(1024))
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
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + close_after,
                        FixtureType::FuserWithoutInodeCache => 8 + 7 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 5 + 5 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + close_after,
                        FixtureType::FuserWithoutInodeCache => 8 + 7 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_resize: 1 + close_after,
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
                            11 + 3 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 17 + 11 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_create: 7,
                    store_flush_block: 2 * close_after,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            80 + 23 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 132 + 89 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data_mut: 21 + close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    exists: 7,
                    load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                        FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store: 8 + close_after,
                    ..LLActionCounts::ZERO
                },
            }
        })
}
