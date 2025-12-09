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
use cryfs_rustfs::AtimeUpdateBehavior;
use cryfs_rustfs::NumBytes;
use cryfs_utils::path::{AbsolutePath, PathComponent};

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
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_try_read: 1,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_write: expect_atime_update * close_after,
                    blob_resize: expect_atime_update * close_after,
                    blob_flush: close_after + expect_atime_update * close_after,
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
                            7 + 7 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 14 + 14 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    } + 2 * expect_atime_update * close_after,
                    blob_data_mut: expect_atime_update * close_after,
                    store_flush_block: close_after + expect_atime_update * close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: 1,
                    store: expect_atime_update * close_after,
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
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            1 + 1 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_try_read: 1,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_write: expect_atime_update * close_after,
                    blob_resize: expect_atime_update * close_after,
                    blob_flush: close_after + expect_atime_update * close_after,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            4 + 3 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 7 + 6 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            31 + 21 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 52 + 42 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    } + 2 * expect_atime_update * close_after,
                    blob_data_mut: expect_atime_update * close_after,
                    store_flush_block: close_after + expect_atime_update * close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: 3,
                    store: expect_atime_update * close_after,
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
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_try_read: 1,
                    blob_resize: expect_atime_update * close_after,
                    blob_write: expect_atime_update * close_after,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: close_after + expect_atime_update * close_after,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            3 + 3 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 6 + 6 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            21 + 21 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 42 + 42 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    } + 2 * expect_atime_update * close_after,
                    blob_data_mut: expect_atime_update * close_after,
                    store_flush_block: close_after + expect_atime_update * close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: 3,
                    store: expect_atime_update * close_after,
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
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_try_read: 1,
                    blob_resize: expect_atime_update * close_after,
                    blob_write: expect_atime_update * close_after,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: close_after + expect_atime_update * close_after,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            7 + 5 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 12 + 10 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_flush_block: close_after + expect_atime_update * close_after,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            53 + 35 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 88 + 70 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    } + 2 * expect_atime_update * close_after,
                    blob_data_mut: expect_atime_update * close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: 7,
                    store: expect_atime_update * close_after,
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
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_try_read: 1,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_resize: expect_atime_update * close_after,
                    blob_write: expect_atime_update * close_after,
                    blob_flush: close_after + expect_atime_update * close_after,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            5 + 5 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 10 + 10 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_flush_block: close_after + expect_atime_update * close_after,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            35 + 35 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 70 + 70 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    } + 2 * expect_atime_update * close_after,
                    blob_data_mut: expect_atime_update * close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: 5,
                    store: expect_atime_update * close_after,
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
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_try_read: 1,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_write: expect_atime_update * close_after,
                    blob_resize: expect_atime_update * close_after,
                    blob_flush: close_after + expect_atime_update * close_after,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_flush_block: close_after + expect_atime_update * close_after,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            7 + 7 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 14 + 14 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    } + 2 * expect_atime_update * close_after,
                    blob_data_mut: expect_atime_update * close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: 1,
                    store: expect_atime_update * close_after,
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
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_try_read: 1,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_resize: expect_atime_update * close_after,
                    blob_write: expect_atime_update * close_after,
                    blob_flush: close_after + expect_atime_update * close_after,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            30 + 5 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 35 + 10 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_flush_block: close_after + expect_atime_update * close_after,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            195 + 35 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 230 + 70 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    } + 2 * expect_atime_update * close_after,
                    blob_data_mut: expect_atime_update * close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: 30,
                    store: expect_atime_update * close_after,
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
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 2 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_try_read: 1,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_resize: expect_atime_update * close_after,
                    blob_write: expect_atime_update * close_after,
                    blob_flush: close_after + expect_atime_update * close_after,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            5 + 5 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 10 + 10 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_flush_block: close_after + expect_atime_update * close_after,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            35 + 35 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 70 + 70 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    } + 2 * expect_atime_update * close_after,
                    blob_data_mut: expect_atime_update * close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: 5,
                    store: expect_atime_update * close_after,
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
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 3 + 3 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 3 + 3 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_try_read: 1,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_resize: expect_atime_update * close_after,
                    blob_write: expect_atime_update * close_after,
                    blob_flush: close_after + expect_atime_update * close_after,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 3 + 3 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_flush_block: close_after + expect_atime_update * close_after,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            9 + 7 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 25 + 23 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    } + 2 * expect_atime_update * close_after,
                    blob_data_mut: expect_atime_update * close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                        FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store: expect_atime_update * close_after,
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
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 7 + 7 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 5 + 5 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 7 + 7 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_try_read: 1,
                    blob_resize: expect_atime_update * close_after,
                    blob_write: expect_atime_update * close_after,
                    blob_flush: close_after + expect_atime_update * close_after,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1 + close_after,
                        FixtureType::FuserWithoutInodeCache => 7 + 7 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_flush_block: close_after + expect_atime_update * close_after,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            9 + 7 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 61 + 59 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    } + 2 * expect_atime_update * close_after,
                    blob_data_mut: expect_atime_update * close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                        FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store: expect_atime_update * close_after,
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
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 10 + close_after,
                        FixtureType::FuserWithoutInodeCache => 20 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 10 + close_after,
                        FixtureType::FuserWithoutInodeCache => 20 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_try_read: 10,
                    blob_resize: expect_atime_update_1 * close_after,
                    blob_write: expect_atime_update_1 * close_after,
                    blob_num_bytes: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                        FixtureType::FuserWithoutInodeCache => 10 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_flush: close_after + expect_atime_update_1 * close_after,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            70 + 5 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 120 + 10 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    store_flush_block: close_after + expect_atime_update_1 * close_after,
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => {
                            530 + 35 * close_after
                        }
                        FixtureType::FuserWithoutInodeCache => 880 + 70 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    } + 2 * expect_atime_update_1 * close_after,
                    blob_data_mut: expect_atime_update_1 * close_after,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: 7,
                    store: expect_atime_update_1 * close_after,
                    ..LLActionCounts::ZERO
                },
            }
        })
}
