use crate::filesystem_driver::FilesystemDriver;
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
use cryfs_rustfs::AtimeUpdateBehavior;
use cryfs_rustfs::NumBytes;
use cryfs_rustfs::PathComponent;

crate::perf_test_macro::perf_test!(
    read,
    [
        small_read_from_empty_file::<false>,
        small_read_from_empty_file::<true>,
        small_read_from_middle_of_small_file::<false>,
        small_read_from_middle_of_small_file::<true>,
        small_read_beyond_end_of_small_file::<false>,
        small_read_beyond_end_of_small_file::<true>,
        small_read_from_middle_of_large_file::<false>,
        small_read_from_middle_of_large_file::<true>,
        small_read_from_beyond_end_of_large_file::<false>,
        small_read_from_beyond_end_of_large_file::<true>,
        large_read_from_empty_file::<false>,
        large_read_from_empty_file::<true>,
        large_read_from_middle_of_large_file::<false>,
        large_read_from_middle_of_large_file::<true>,
        large_read_from_beyond_end_of_large_file::<false>,
        large_read_from_beyond_end_of_large_file::<true>,
        read_from_file_in_nested_dir::<false>,
        read_from_file_in_nested_dir::<true>,
        read_from_file_in_deeply_nested_dir::<false>,
        read_from_file_in_deeply_nested_dir::<true>,
        multiple_reads_from_same_file::<false>,
        multiple_reads_from_same_file::<true>,
    ]
);

// TODO Why are the expect_atime_update calculations here different than in other operations , e.g. readlink? Also, some tests in this file here have a different formula than others

fn small_read_from_empty_file<const CLOSE_AFTER: bool>(
    test_driver: impl TestDriver,
) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open an empty file to read from
            fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap()
        })
        .test(async |fixture, (file, mut fh)| {
            // Attempt to read 1 byte from empty file
            fixture
                .filesystem
                .read(file.clone(), &mut fh, NumBytes::from(0), NumBytes::from(1))
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            let expect_atime_update = match atime_behavior {
                AtimeUpdateBehavior::Noatime
                | AtimeUpdateBehavior::Relatime
                | AtimeUpdateBehavior::NodiratimeRelatime => 0,
                AtimeUpdateBehavior::Strictatime | AtimeUpdateBehavior::NodiratimeStrictatime => 1,
            };

            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_try_read: 1,
                    blob_write: expect_atime_update,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_resize: expect_atime_update,
                    blob_flush: close_after,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            16 + 16 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 32 + 32 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    } + 2 * expect_atime_update,
                    blob_data_mut: expect_atime_update,
                    store_flush_block: close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: 2,
                    store: expect_atime_update,
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn small_read_from_middle_of_small_file<const CLOSE_AFTER: bool>(
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
            let initial_data = vec![b'X'; BLOCKSIZE_BYTES as usize];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(0), initial_data)
                .await
                .unwrap();

            (file, fh)
        })
        .test(async |fixture, (file, mut fh)| {
            // Read 1 byte from file
            fixture
                .filesystem
                .read(file.clone(), &mut fh, NumBytes::from(10), NumBytes::from(1))
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            let expect_atime_update = match atime_behavior {
                AtimeUpdateBehavior::Noatime => 0,
                AtimeUpdateBehavior::Relatime
                | AtimeUpdateBehavior::NodiratimeRelatime
                | AtimeUpdateBehavior::Strictatime
                | AtimeUpdateBehavior::NodiratimeStrictatime => 1,
            };

            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_write: expect_atime_update,
                    blob_resize: expect_atime_update,
                    blob_try_read: 1,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: close_after,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            5 + 4 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 9 + 8 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            40 + 30 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 70 + 60 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    } + 2 * expect_atime_update,
                    blob_data_mut: expect_atime_update,
                    store_flush_block: close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: 4,
                    store: expect_atime_update,
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn small_read_beyond_end_of_small_file<const CLOSE_AFTER: bool>(
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
            let initial_data = vec![b'X'; BLOCKSIZE_BYTES as usize];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(0), initial_data)
                .await
                .unwrap();

            (file, fh)
        })
        .test(async |fixture, (file, mut fh)| {
            // Try to read beyond the end of the file
            let data = fixture
                .filesystem
                .read(
                    file.clone(),
                    &mut fh,
                    NumBytes::from(2 * BLOCKSIZE_BYTES),
                    NumBytes::from(1),
                )
                .await
                .unwrap();
            assert_eq!(data.len(), 0); // Should be empty since we're reading beyond EOF
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            let expect_atime_update = match atime_behavior {
                AtimeUpdateBehavior::Noatime => 0,
                AtimeUpdateBehavior::Relatime
                | AtimeUpdateBehavior::NodiratimeRelatime
                | AtimeUpdateBehavior::Strictatime
                | AtimeUpdateBehavior::NodiratimeStrictatime => 1,
            };

            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_try_read: 1,
                    blob_write: expect_atime_update,
                    blob_resize: expect_atime_update,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: close_after,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            4 + 4 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 8 + 8 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            30 + 30 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 60 + 60 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    } + 2 * expect_atime_update,
                    blob_data_mut: expect_atime_update,
                    store_flush_block: close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: 4,
                    store: expect_atime_update,
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn small_read_from_middle_of_large_file<const CLOSE_AFTER: bool>(
    test_driver: impl TestDriver,
) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file, write a large amount of data
            let (file, mut fh) = fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();
            let initial_data = vec![b'X'; 2 * NUM_BYTES_FOR_THREE_LEVEL_TREE as usize];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(0), initial_data)
                .await
                .unwrap();

            (file, fh)
        })
        .test(async |fixture, (file, mut fh)| {
            fixture
                .filesystem
                .read(
                    file.clone(),
                    &mut fh,
                    NumBytes::from(NUM_BYTES_FOR_THREE_LEVEL_TREE),
                    NumBytes::from(1),
                )
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            let expect_atime_update = match atime_behavior {
                AtimeUpdateBehavior::Noatime => 0,
                AtimeUpdateBehavior::Relatime
                | AtimeUpdateBehavior::NodiratimeRelatime
                | AtimeUpdateBehavior::Strictatime
                | AtimeUpdateBehavior::NodiratimeStrictatime => 1,
            };

            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_write: expect_atime_update,
                    blob_resize: expect_atime_update,
                    blob_try_read: 1,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: close_after,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            8 + 6 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 14 + 12 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            62 + 44 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 106 + 88 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    } + 2 * expect_atime_update,
                    blob_data_mut: expect_atime_update,
                    store_flush_block: close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: 8,
                    store: expect_atime_update,
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn small_read_from_beyond_end_of_large_file<const CLOSE_AFTER: bool>(
    test_driver: impl TestDriver,
) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file, write a large amount of data
            let (file, mut fh) = fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();
            let initial_data = vec![b'X'; 2 * NUM_BYTES_FOR_THREE_LEVEL_TREE as usize];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(0), initial_data)
                .await
                .unwrap();

            (file, fh)
        })
        .test(async |fixture, (file, mut fh)| {
            fixture
                .filesystem
                .read(
                    file.clone(),
                    &mut fh,
                    NumBytes::from(3 * NUM_BYTES_FOR_THREE_LEVEL_TREE),
                    NumBytes::from(1),
                )
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            let expect_atime_update = match atime_behavior {
                AtimeUpdateBehavior::Noatime => 0,
                AtimeUpdateBehavior::Relatime
                | AtimeUpdateBehavior::NodiratimeRelatime
                | AtimeUpdateBehavior::Strictatime
                | AtimeUpdateBehavior::NodiratimeStrictatime => 1,
            };

            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_try_read: 1,
                    blob_write: expect_atime_update,
                    blob_resize: expect_atime_update,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: close_after,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            6 + 6 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 12 + 12 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            44 + 44 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 88 + 88 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    } + 2 * expect_atime_update,
                    blob_data_mut: expect_atime_update,
                    store_flush_block: close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: 6,
                    store: expect_atime_update,
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn large_read_from_empty_file<const CLOSE_AFTER: bool>(
    test_driver: impl TestDriver,
) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open an empty file to read from
            fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap()
        })
        .test(async |fixture, (file, mut fh)| {
            // Attempt to read 1 byte from empty file
            fixture
                .filesystem
                .read(
                    file.clone(),
                    &mut fh,
                    NumBytes::from(0),
                    NumBytes::from(NUM_BYTES_FOR_THREE_LEVEL_TREE),
                )
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            let expect_atime_update = match atime_behavior {
                AtimeUpdateBehavior::Noatime
                | AtimeUpdateBehavior::Relatime
                | AtimeUpdateBehavior::NodiratimeRelatime => 0,
                AtimeUpdateBehavior::Strictatime | AtimeUpdateBehavior::NodiratimeStrictatime => 1,
            };

            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_try_read: 1,
                    blob_write: expect_atime_update,
                    blob_resize: expect_atime_update,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: close_after,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            16 + 16 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 32 + 32 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    } + 2 * expect_atime_update,
                    blob_data_mut: expect_atime_update,
                    store_flush_block: close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: 2,
                    store: expect_atime_update,
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn large_read_from_middle_of_large_file<const CLOSE_AFTER: bool>(
    test_driver: impl TestDriver,
) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file, write a large amount of data
            let (file, mut fh) = fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();
            let initial_data = vec![b'X'; 3 * NUM_BYTES_FOR_THREE_LEVEL_TREE as usize];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(0), initial_data)
                .await
                .unwrap();

            (file, fh)
        })
        .test(async |fixture, (file, mut fh)| {
            // Read a large amount of data
            fixture
                .filesystem
                .read(
                    file.clone(),
                    &mut fh,
                    NumBytes::from(NUM_BYTES_FOR_THREE_LEVEL_TREE),
                    NumBytes::from(NUM_BYTES_FOR_THREE_LEVEL_TREE),
                )
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            let expect_atime_update = match atime_behavior {
                AtimeUpdateBehavior::Noatime => 0,
                AtimeUpdateBehavior::Relatime
                | AtimeUpdateBehavior::NodiratimeRelatime
                | AtimeUpdateBehavior::Strictatime
                | AtimeUpdateBehavior::NodiratimeStrictatime => 1,
            };

            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_try_read: 1,
                    blob_write: expect_atime_update,
                    blob_resize: expect_atime_update,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: close_after,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            31 + 6 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 37 + 12 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            204 + 44 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 248 + 88 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    } + 2 * expect_atime_update,
                    blob_data_mut: expect_atime_update,
                    store_flush_block: close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: 31,
                    store: expect_atime_update,
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn large_read_from_beyond_end_of_large_file<const CLOSE_AFTER: bool>(
    test_driver: impl TestDriver,
) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file, write a large amount of data
            let (file, mut fh) = fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();
            let initial_data = vec![b'X'; NUM_BYTES_FOR_THREE_LEVEL_TREE as usize];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(0), initial_data)
                .await
                .unwrap();

            (file, fh)
        })
        .test(async |fixture, (file, mut fh)| {
            // Read a large amount of data
            fixture
                .filesystem
                .read(
                    file.clone(),
                    &mut fh,
                    NumBytes::from(2 * NUM_BYTES_FOR_THREE_LEVEL_TREE),
                    NumBytes::from(NUM_BYTES_FOR_THREE_LEVEL_TREE),
                )
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            let expect_atime_update = match atime_behavior {
                AtimeUpdateBehavior::Noatime => 0,
                AtimeUpdateBehavior::Relatime
                | AtimeUpdateBehavior::NodiratimeRelatime
                | AtimeUpdateBehavior::Strictatime
                | AtimeUpdateBehavior::NodiratimeStrictatime => 1,
            };

            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 4 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_try_read: 1,
                    blob_write: expect_atime_update,
                    blob_resize: expect_atime_update,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: close_after,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            6 + 6 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 12 + 12 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            44 + 44 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 88 + 88 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    } + 2 * expect_atime_update,
                    blob_data_mut: expect_atime_update,
                    store_flush_block: close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: 6,
                    store: expect_atime_update,
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn read_from_file_in_nested_dir<const CLOSE_AFTER: bool>(
    test_driver: impl TestDriver,
) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create a nested directory and file with data
            let parent = fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap();

            let (file, mut fh) = fixture
                .filesystem
                .create_and_open_file(
                    Some(parent),
                    PathComponent::try_from_str("nestedfile.txt").unwrap(),
                )
                .await
                .unwrap();

            let initial_data = vec![b'Y'; 100];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(0), initial_data)
                .await
                .unwrap();

            (file, fh)
        })
        .test(async |fixture, (file, mut fh)| {
            // Read data from the nested file
            fixture
                .filesystem
                .read(file.clone(), &mut fh, NumBytes::from(0), NumBytes::from(1))
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            let expect_atime_update = match atime_behavior {
                AtimeUpdateBehavior::Noatime => 0,
                AtimeUpdateBehavior::Relatime
                | AtimeUpdateBehavior::NodiratimeRelatime
                | AtimeUpdateBehavior::Strictatime
                | AtimeUpdateBehavior::NodiratimeStrictatime => 1,
            };

            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 6 + 6 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 4 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 6 + 6 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_try_read: 1,
                    blob_write: expect_atime_update,
                    blob_resize: expect_atime_update,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: close_after,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 6 + 6 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            18 + 16 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 52 + 50 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    } + 2 * expect_atime_update,
                    blob_data_mut: expect_atime_update,
                    store_flush_block: close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                        FixtureType::FuserWithoutInodeCache => 3, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store: expect_atime_update,
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn read_from_file_in_deeply_nested_dir<const CLOSE_AFTER: bool>(
    test_driver: impl TestDriver,
) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create a deeply nested directory and file with data
            let deeply_nested = fixture
                .filesystem
                .mkdir_recursive(AbsolutePath::try_from_str("/deep/nested/dir").unwrap())
                .await
                .unwrap();

            let (file, mut fh) = fixture
                .filesystem
                .create_and_open_file(
                    Some(deeply_nested),
                    PathComponent::try_from_str("deepfile.txt").unwrap(),
                )
                .await
                .unwrap();

            let initial_data = vec![b'Z'; 100];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(0), initial_data)
                .await
                .unwrap();

            (file, fh)
        })
        .test(async |fixture, (file, mut fh)| {
            // Read data from the deeply nested file
            fixture
                .filesystem
                .read(file.clone(), &mut fh, NumBytes::from(0), NumBytes::from(1))
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            let expect_atime_update = match atime_behavior {
                AtimeUpdateBehavior::Noatime => 0,
                AtimeUpdateBehavior::Relatime
                | AtimeUpdateBehavior::NodiratimeRelatime
                | AtimeUpdateBehavior::Strictatime
                | AtimeUpdateBehavior::NodiratimeStrictatime => 1,
            };

            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 10 + 10 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 8 + 8 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 10 + 10 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_try_read: 1,
                    blob_write: expect_atime_update,
                    blob_resize: expect_atime_update,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: close_after,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            2 + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 10 + 10 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            18 + 16 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 88 + 86 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    } + 2 * expect_atime_update,
                    blob_data_mut: expect_atime_update,
                    store_flush_block: close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                        FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store: expect_atime_update,
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn multiple_reads_from_same_file<const CLOSE_AFTER: bool>(
    test_driver: impl TestDriver,
) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file, write a large amount of data
            let (file, mut fh) = fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();
            let initial_data = vec![b'X'; 2 * NUM_BYTES_FOR_THREE_LEVEL_TREE as usize];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(0), initial_data)
                .await
                .unwrap();

            (file, fh)
        })
        .test(async |fixture, (file, mut fh)| {
            // Perform multiple small reads from different positions
            for i in 0..10 {
                fixture
                    .filesystem
                    .read(
                        file.clone(),
                        &mut fh,
                        NumBytes::from(NUM_BYTES_FOR_THREE_LEVEL_TREE + i),
                        NumBytes::from(1),
                    )
                    .await
                    .unwrap();
            }
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            let (expect_atime_update_1, expect_atime_update_2) = match atime_behavior {
                AtimeUpdateBehavior::Noatime => (0, 0),
                AtimeUpdateBehavior::Relatime | AtimeUpdateBehavior::NodiratimeRelatime => (1, 0),
                AtimeUpdateBehavior::Strictatime | AtimeUpdateBehavior::NodiratimeStrictatime => {
                    (1, 1)
                }
            };

            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            20 + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 40 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 10 + close_after,
                        FixtureType::FuserWithoutInodeCache => 20 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            20 + 2 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 40 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_try_read: 10,
                    blob_write: expect_atime_update_1 + 9 * expect_atime_update_2,
                    blob_resize: expect_atime_update_1 + 9 * expect_atime_update_2,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 10 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: close_after,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            80 + 6 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 140 + 12 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            620 + 44 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 1060 + 88 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    } + 2 * (expect_atime_update_1 + 9 * expect_atime_update_2),
                    blob_data_mut: expect_atime_update_1 + 9 * expect_atime_update_2,
                    store_flush_block: close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: 8,
                    store: expect_atime_update_1,
                    ..LLActionCounts::ZERO
                },
            }
        })
}
