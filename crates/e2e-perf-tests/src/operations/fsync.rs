use crate::filesystem_driver::FilesystemDriver as _;
use crate::filesystem_fixture::ActionCounts;
use crate::filesystem_fixture::BLOCKSIZE_BYTES;
use crate::filesystem_fixture::NUM_BYTES_FOR_THREE_LEVEL_TREE;
use crate::perf_test_macro::FixtureType;
use crate::test_driver::TestDriver;
use crate::test_driver::TestReady;
use crate::utils::maybe_close;
use cryfs_blobstore::BlobStoreActionCounts;
use cryfs_blockstore::HLActionCounts;
use cryfs_blockstore::LLActionCounts;
use cryfs_rustfs::AbsolutePath;
use cryfs_rustfs::NumBytes;
use cryfs_rustfs::PathComponent;

// TODO Same TODOs as in release.rs
//    - Some flush operations in here seem to load blocks in low_level, i.e. below the cache??? Why is that? If it's not loaded, shouldn't we just ignore it since it's already flushed? Also, generally, for a simple flush, there's a lot of operations going on in the high level stores.
//    - Some flush-after-write operations in here don't have a store in low level, that's weird. Shouldn't they need to store to flush the write?

crate::perf_test_macro::perf_test!(
    fsync_datasync,
    [
        unchanged_empty_file_in_rootdir::<true, false>,
        unchanged_empty_file_in_rootdir::<true, true>,
        unchanged_file_with_data_in_rootdir::<true, false>,
        unchanged_file_with_data_in_rootdir::<true, true>,
        unchanged_large_file_in_rootdir::<true, false>,
        unchanged_large_file_in_rootdir::<true, true>,
        unchanged_file_in_nested_dir::<true, false>,
        unchanged_file_in_nested_dir::<true, true>,
        unchanged_file_in_deeply_nested_dir::<true, false>,
        unchanged_file_in_deeply_nested_dir::<true, true>,
        after_small_write_to_empty_file::<true, false>,
        after_small_write_to_empty_file::<true, true>,
        after_small_write_to_middle_of_small_file::<true, false>,
        after_small_write_to_middle_of_small_file::<true, true>,
        after_small_write_beyond_end_of_small_file::<true, false>,
        after_small_write_beyond_end_of_small_file::<true, true>,
        after_small_write_to_middle_of_large_file::<true, false>,
        after_small_write_to_middle_of_large_file::<true, true>,
        after_small_write_beyond_end_of_large_file::<true, false>,
        after_small_write_beyond_end_of_large_file::<true, true>,
        after_large_write_to_empty_file::<true, false>,
        after_large_write_to_empty_file::<true, true>,
        after_large_write_to_middle_of_large_file::<true, false>,
        after_large_write_to_middle_of_large_file::<true, true>,
        after_large_write_beyond_end_of_large_file::<true, false>,
        after_large_write_beyond_end_of_large_file::<true, true>,
        after_write_to_file_in_nested_dir::<true, false>,
        after_write_to_file_in_nested_dir::<true, true>,
        after_write_to_file_in_deeply_nested_dir::<true, false>,
        after_write_to_file_in_deeply_nested_dir::<true, true>,
    ]
);

crate::perf_test_macro::perf_test!(
    fsync_fullsync,
    [
        unchanged_empty_file_in_rootdir::<false, false>,
        unchanged_empty_file_in_rootdir::<false, true>,
        unchanged_file_with_data_in_rootdir::<false, false>,
        unchanged_file_with_data_in_rootdir::<false, true>,
        unchanged_large_file_in_rootdir::<false, false>,
        unchanged_large_file_in_rootdir::<false, true>,
        unchanged_file_in_nested_dir::<false, false>,
        unchanged_file_in_nested_dir::<false, true>,
        unchanged_file_in_deeply_nested_dir::<false, false>,
        unchanged_file_in_deeply_nested_dir::<false, true>,
        after_small_write_to_empty_file::<false, false>,
        after_small_write_to_empty_file::<false, true>,
        after_small_write_to_middle_of_small_file::<false, false>,
        after_small_write_to_middle_of_small_file::<false, true>,
        after_small_write_beyond_end_of_small_file::<false, false>,
        after_small_write_beyond_end_of_small_file::<false, true>,
        after_small_write_to_middle_of_large_file::<false, false>,
        after_small_write_to_middle_of_large_file::<false, true>,
        after_small_write_beyond_end_of_large_file::<false, false>,
        after_small_write_beyond_end_of_large_file::<false, true>,
        after_large_write_to_empty_file::<false, false>,
        after_large_write_to_empty_file::<false, true>,
        after_large_write_to_middle_of_large_file::<false, false>,
        after_large_write_to_middle_of_large_file::<false, true>,
        after_large_write_beyond_end_of_large_file::<false, false>,
        after_large_write_beyond_end_of_large_file::<false, true>,
        after_write_to_file_in_nested_dir::<false, false>,
        after_write_to_file_in_nested_dir::<false, true>,
        after_write_to_file_in_deeply_nested_dir::<false, false>,
        after_write_to_file_in_deeply_nested_dir::<false, true>,
    ]
);

fn unchanged_empty_file_in_rootdir<const DATASYNC: bool, const CLOSE_AFTER: bool>(
    test_driver: impl TestDriver,
) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file to flush
            fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap()
        })
        .test_noflush(async |fixture, (file, mut fh)| {
            fixture
                .filesystem
                .fsync(file.clone(), &mut fh, DATASYNC)
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let datasync = if DATASYNC { 1 } else { 0 };
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 - datasync + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            1 - datasync + close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 2 - datasync + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 - datasync + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: 1 + close_after,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 - datasync + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            16 - 9 * datasync + 16 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 32 - 9 * datasync + 32 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_flush_block: 1 + close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync * (1 - close_after)
                        }
                        FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn unchanged_file_with_data_in_rootdir<const DATASYNC: bool, const CLOSE_AFTER: bool>(
    test_driver: impl TestDriver,
) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file, write some data, then flush
            let (file, mut fh) = fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();
            let data = vec![b'X'; 100];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(0), data)
                .await
                .unwrap();
            (file, fh)
        })
        .test_noflush(async |fixture, (file, mut fh)| {
            fixture
                .filesystem
                .fsync(file.clone(), &mut fh, DATASYNC)
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let datasync = if DATASYNC { 1 } else { 0 };
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 - datasync + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            1 - datasync + close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 2 - datasync + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 - datasync + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: 1 + close_after,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 - datasync + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_flush_block: 1 + close_after,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            16 - 9 * datasync + 16 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 32 - 9 * datasync + 32 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync * (1 - close_after)
                        }
                        FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn unchanged_large_file_in_rootdir<const DATASYNC: bool, const CLOSE_AFTER: bool>(
    test_driver: impl TestDriver,
) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file, write a large amount of data, then flush
            let (file, mut fh) = fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();
            let data = vec![b'X'; NUM_BYTES_FOR_THREE_LEVEL_TREE as usize];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(0), data)
                .await
                .unwrap();
            (file, fh)
        })
        .test_noflush(async |fixture, (file, mut fh)| {
            fixture
                .filesystem
                .fsync(file.clone(), &mut fh, DATASYNC)
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let datasync = if DATASYNC { 1 } else { 0 };
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 - datasync + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            1 - datasync + close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 2 - datasync + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 - datasync + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: 1 + close_after,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            6 - datasync + 6 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 12 - datasync + 12 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_flush_block: 1 + close_after,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            44 - 9 * datasync + 44 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 88 - 9 * datasync + 88 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            6 - datasync * (1 - close_after)
                        }
                        FixtureType::FuserWithoutInodeCache => 6, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn unchanged_file_in_nested_dir<const DATASYNC: bool, const CLOSE_AFTER: bool>(
    test_driver: impl TestDriver,
) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create a nested directory with a file
            let dir = fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap();
            fixture
                .filesystem
                .create_and_open_file(Some(dir), PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap()
        })
        .test_noflush(async |fixture, (file, mut fh)| {
            fixture
                .filesystem
                .fsync(file.clone(), &mut fh, DATASYNC)
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let datasync = if DATASYNC { 1 } else { 0 };
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 6 - datasync + 6 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            1 - datasync + close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 - datasync + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 6 - datasync + 6 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: 1 + close_after,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 6 - datasync + 6 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_flush_block: 1 + close_after,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            16 - 9 * datasync + 16 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 50 - 9 * datasync + 50 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync * (1 - close_after)
                        }
                        FixtureType::FuserWithoutInodeCache => 3, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn unchanged_file_in_deeply_nested_dir<const DATASYNC: bool, const CLOSE_AFTER: bool>(
    test_driver: impl TestDriver,
) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create a deeply nested directory with a file
            let deeply_nested = fixture
                .filesystem
                .mkdir_recursive(AbsolutePath::try_from_str("/deep/nested/dir").unwrap())
                .await
                .unwrap();
            fixture
                .filesystem
                .create_and_open_file(
                    Some(deeply_nested),
                    PathComponent::try_from_str("file.txt").unwrap(),
                )
                .await
                .unwrap()
        })
        .test_noflush(async |fixture, (file, mut fh)| {
            fixture
                .filesystem
                .fsync(file.clone(), &mut fh, DATASYNC)
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let datasync = if DATASYNC { 1 } else { 0 };
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 10 - datasync + 10 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            1 - datasync + close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 8 - datasync + 8 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 10 - datasync + 10 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: 1 + close_after,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 10 - datasync + 10 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_flush_block: 1 + close_after,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            16 - 9 * datasync + 16 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 86 - 9 * datasync + 86 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync * (1 - close_after)
                        }
                        FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn after_small_write_to_empty_file<const DATASYNC: bool, const CLOSE_AFTER: bool>(
    test_driver: impl TestDriver,
) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file, perform a small write operation
            fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap()
        })
        .setup_noflush(async |fixture, (file, mut fh)| {
            // Perform small write without flushing
            let data = vec![b'A'; 1];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(0), data)
                .await
                .unwrap();

            (file, fh)
        })
        .test_noflush(async |fixture, (file, mut fh)| {
            fixture
                .filesystem
                .fsync(file.clone(), &mut fh, DATASYNC)
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let datasync = if DATASYNC { 1 } else { 0 };
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 - datasync + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            1 - datasync + close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 2 - datasync + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 - datasync + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: 1 + close_after,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 - datasync + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_flush_block: 1 + close_after,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            16 - 9 * datasync + 16 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 32 - 9 * datasync + 32 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store: 1,
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn after_small_write_to_middle_of_small_file<const DATASYNC: bool, const CLOSE_AFTER: bool>(
    test_driver: impl TestDriver,
) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file, write some initial data
            let (file, mut fh) = fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();

            // Write initial data
            let initial_data = vec![b'X'; 2 * BLOCKSIZE_BYTES as usize];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(0), initial_data)
                .await
                .unwrap();
            (file, fh)
        })
        .setup_noflush(async |fixture, (file, mut fh)| {
            // Write a small amount in the middle
            let data = vec![b'A'; 1];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(BLOCKSIZE_BYTES), data)
                .await
                .unwrap();
            (file, fh)
        })
        .test_noflush(async |fixture, (file, mut fh)| {
            fixture
                .filesystem
                .fsync(file.clone(), &mut fh, DATASYNC)
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let datasync = if DATASYNC { 1 } else { 0 };
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 - datasync + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            1 - datasync + close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 2 - datasync + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 - datasync + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: 1 + close_after,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            4 - datasync + 4 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 8 - datasync + 8 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_flush_block: 1 + close_after,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            30 - 9 * datasync + 30 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 60 - 9 * datasync + 60 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    // TODO Why no store? Shouldn't this store to flush the write?
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn after_small_write_beyond_end_of_small_file<const DATASYNC: bool, const CLOSE_AFTER: bool>(
    test_driver: impl TestDriver,
) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file, write some initial data
            let (file, mut fh) = fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();

            // Write initial data
            let initial_data = vec![b'X'; 2 * BLOCKSIZE_BYTES as usize];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(0), initial_data)
                .await
                .unwrap();
            (file, fh)
        })
        .setup_noflush(async |fixture, (file, mut fh)| {
            // Write a small amount beyond the end
            let data = vec![b'A'; 1];
            fixture
                .filesystem
                .write(
                    file.clone(),
                    &mut fh,
                    NumBytes::from(3 * BLOCKSIZE_BYTES),
                    data,
                )
                .await
                .unwrap();
            (file, fh)
        })
        .test_noflush(async |fixture, (file, mut fh)| {
            fixture
                .filesystem
                .fsync(file.clone(), &mut fh, DATASYNC)
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let datasync = if DATASYNC { 1 } else { 0 };
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 - datasync + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            1 - datasync + 1 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 2 - datasync + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 - datasync + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: 1 + close_after,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            4 - datasync + 4 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 8 - datasync + 8 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_flush_block: 1 + close_after,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            30 - 9 * datasync + 30 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 60 - 9 * datasync + 60 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store: 1,
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn after_small_write_to_middle_of_large_file<const DATASYNC: bool, const CLOSE_AFTER: bool>(
    test_driver: impl TestDriver,
) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file, write some initial large data
            let (file, mut fh) = fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();

            // Write initial large data
            let initial_data = vec![b'X'; 2 * NUM_BYTES_FOR_THREE_LEVEL_TREE as usize];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(0), initial_data)
                .await
                .unwrap();
            (file, fh)
        })
        .setup_noflush(async |fixture, (file, mut fh)| {
            // Write a small amount in the middle
            let data = vec![b'A'; 1];
            fixture
                .filesystem
                .write(
                    file.clone(),
                    &mut fh,
                    NumBytes::from(NUM_BYTES_FOR_THREE_LEVEL_TREE),
                    data,
                )
                .await
                .unwrap();
            (file, fh)
        })
        .test_noflush(async |fixture, (file, mut fh)| {
            fixture
                .filesystem
                .fsync(file.clone(), &mut fh, DATASYNC)
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let datasync = if DATASYNC { 1 } else { 0 };
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 - datasync + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            1 - datasync + close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 2 - datasync + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 - datasync + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: 1 + close_after,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            6 - datasync + 6 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 12 - datasync + 12 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_flush_block: 1 + close_after,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            44 - 9 * datasync + 44 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 88 - 9 * datasync + 88 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    // TODO Why no store? Shouldn't this store to flush the write?
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn after_small_write_beyond_end_of_large_file<const DATASYNC: bool, const CLOSE_AFTER: bool>(
    test_driver: impl TestDriver,
) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file, write some initial large data
            let (file, mut fh) = fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();

            // Write initial large data
            let initial_data = vec![b'X'; 2 * NUM_BYTES_FOR_THREE_LEVEL_TREE as usize];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(0), initial_data)
                .await
                .unwrap();
            (file, fh)
        })
        .setup_noflush(async |fixture, (file, mut fh)| {
            // Write a small amount beyond the end
            let data = vec![b'A'; 1];
            fixture
                .filesystem
                .write(
                    file.clone(),
                    &mut fh,
                    NumBytes::from(3 * NUM_BYTES_FOR_THREE_LEVEL_TREE),
                    data,
                )
                .await
                .unwrap();
            (file, fh)
        })
        .test_noflush(async |fixture, (file, mut fh)| {
            fixture
                .filesystem
                .fsync(file.clone(), &mut fh, DATASYNC)
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let datasync = if DATASYNC { 1 } else { 0 };
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 - datasync + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            1 - datasync + close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 2 - datasync + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 - datasync + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: 1 + close_after,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            6 - datasync + 6 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 12 - datasync + 12 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_flush_block: 1 + close_after,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            44 - 9 * datasync + 44 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 88 - 9 * datasync + 88 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store: 1,
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn after_large_write_to_empty_file<const DATASYNC: bool, const CLOSE_AFTER: bool>(
    test_driver: impl TestDriver,
) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file
            fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap()
        })
        .setup_noflush(async |fixture, (file, mut fh)| {
            // Perform large write without flushing
            let data = vec![b'A'; 2 * NUM_BYTES_FOR_THREE_LEVEL_TREE as usize];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(0), data)
                .await
                .unwrap();
            (file, fh)
        })
        .test_noflush(async |fixture, (file, mut fh)| {
            fixture
                .filesystem
                .fsync(file.clone(), &mut fh, DATASYNC)
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let datasync = if DATASYNC { 1 } else { 0 };
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 - datasync + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            1 - datasync + 1 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 2 - datasync + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 - datasync + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: 1 + close_after,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            6 - datasync + 6 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 12 - datasync + 12 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_flush_block: 1 + close_after,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            44 - 9 * datasync + 44 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 88 - 9 * datasync + 88 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store: 1,
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn after_large_write_to_middle_of_large_file<const DATASYNC: bool, const CLOSE_AFTER: bool>(
    test_driver: impl TestDriver,
) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file, write some initial large data
            let (file, mut fh) = fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();

            // Write initial large data
            let initial_data = vec![b'X'; 3 * NUM_BYTES_FOR_THREE_LEVEL_TREE as usize];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(0), initial_data)
                .await
                .unwrap();

            (file, fh)
        })
        .setup_noflush(async |fixture, (file, mut fh)| {
            // Write large data in the middle
            let data = vec![b'A'; NUM_BYTES_FOR_THREE_LEVEL_TREE as usize];
            fixture
                .filesystem
                .write(
                    file.clone(),
                    &mut fh,
                    NumBytes::from(NUM_BYTES_FOR_THREE_LEVEL_TREE),
                    data,
                )
                .await
                .unwrap();
            (file, fh)
        })
        .test_noflush(async |fixture, (file, mut fh)| {
            fixture
                .filesystem
                .fsync(file.clone(), &mut fh, DATASYNC)
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let datasync = if DATASYNC { 1 } else { 0 };
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 - datasync + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            1 - datasync + close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 2 - datasync + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 - datasync + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: 1 + close_after,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            6 - datasync + 6 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 12 - datasync + 12 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_flush_block: 1 + close_after,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            44 - 9 * datasync + 44 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 88 - 9 * datasync + 88 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    // TODO Why no store? Shouldn't this store to flush the write?
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn after_large_write_beyond_end_of_large_file<const DATASYNC: bool, const CLOSE_AFTER: bool>(
    test_driver: impl TestDriver,
) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file, write some initial large data
            let (file, mut fh) = fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();

            // Write initial large data
            let initial_data = vec![b'X'; NUM_BYTES_FOR_THREE_LEVEL_TREE as usize];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(0), initial_data)
                .await
                .unwrap();

            (file, fh)
        })
        .setup_noflush(async |fixture, (file, mut fh)| {
            // Write large data beyond the end
            let data = vec![b'A'; NUM_BYTES_FOR_THREE_LEVEL_TREE as usize];
            fixture
                .filesystem
                .write(
                    file.clone(),
                    &mut fh,
                    NumBytes::from(2 * NUM_BYTES_FOR_THREE_LEVEL_TREE),
                    data,
                )
                .await
                .unwrap();
            (file, fh)
        })
        .test_noflush(async |fixture, (file, mut fh)| {
            fixture
                .filesystem
                .fsync(file.clone(), &mut fh, DATASYNC)
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let datasync = if DATASYNC { 1 } else { 0 };
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 - datasync + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            1 - datasync + close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 2 - datasync + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 - datasync + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: 1 + close_after,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            6 - datasync + 6 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 12 - datasync + 12 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_flush_block: 1 + close_after,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            44 - 9 * datasync + 44 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 88 - 9 * datasync + 88 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store: 1,
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn after_write_to_file_in_nested_dir<const DATASYNC: bool, const CLOSE_AFTER: bool>(
    test_driver: impl TestDriver,
) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create a nested directory and file
            let parent = fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap();

            fixture
                .filesystem
                .create_and_open_file(
                    Some(parent),
                    PathComponent::try_from_str("nestedfile.txt").unwrap(),
                )
                .await
                .unwrap()
        })
        .setup_noflush(async |fixture, (file, mut fh)| {
            // Write to the file
            let data = vec![b'A'; 1];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(0), data)
                .await
                .unwrap();
            (file, fh)
        })
        .test_noflush(async |fixture, (file, mut fh)| {
            fixture
                .filesystem
                .fsync(file.clone(), &mut fh, DATASYNC)
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let datasync = if DATASYNC { 1 } else { 0 };
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 6 - datasync + 6 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            1 - datasync + close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 - datasync + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 6 - datasync + 6 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: 1 + close_after,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 6 - datasync + 6 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_flush_block: 1 + close_after,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            16 - 9 * datasync + 16 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 50 - 9 * datasync + 50 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
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
        })
}

fn after_write_to_file_in_deeply_nested_dir<const DATASYNC: bool, const CLOSE_AFTER: bool>(
    test_driver: impl TestDriver,
) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create a deeply nested directory and file
            let deeply_nested = fixture
                .filesystem
                .mkdir_recursive(AbsolutePath::try_from_str("/deep/nested/dir").unwrap())
                .await
                .unwrap();

            fixture
                .filesystem
                .create_and_open_file(
                    Some(deeply_nested),
                    PathComponent::try_from_str("deepfile.txt").unwrap(),
                )
                .await
                .unwrap()
        })
        .setup_noflush(async |fixture, (file, mut fh)| {
            // Write to the file
            let data = vec![b'A'; 1];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(0), data)
                .await
                .unwrap();
            (file, fh)
        })
        .test_noflush(async |fixture, (file, mut fh)| {
            fixture
                .filesystem
                .fsync(file.clone(), &mut fh, DATASYNC)
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let datasync = if DATASYNC { 1 } else { 0 };
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 10 - datasync + 10 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            1 - datasync + close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 8 - datasync + 8 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 10 - datasync + 10 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: 1 + close_after,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 - datasync + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 10 - datasync + 10 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_flush_block: 1 + close_after,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            16 - 9 * datasync + 16 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 86 - 9 * datasync + 86 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store: 1,
                    ..LLActionCounts::ZERO
                },
            }
        })
}
