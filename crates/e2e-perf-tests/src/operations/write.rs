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

crate::perf_test_macro::perf_test!(
    write,
    [
        small_write_to_empty_file::<false>,
        small_write_to_empty_file::<true>,
        small_write_to_middle_of_small_file::<false>,
        small_write_to_middle_of_small_file::<true>,
        small_write_beyond_end_of_small_file::<false>,
        small_write_beyond_end_of_small_file::<true>,
        small_write_to_middle_of_large_file::<false>,
        small_write_to_middle_of_large_file::<true>,
        small_write_beyond_end_of_large_file::<false>,
        small_write_beyond_end_of_large_file::<true>,
        large_write_to_empty_file::<false>,
        large_write_to_empty_file::<true>,
        large_write_to_middle_of_large_file::<false>,
        large_write_to_middle_of_large_file::<true>,
        large_write_beyond_end_of_large_file::<false>,
        large_write_beyond_end_of_large_file::<true>,
        write_to_file_in_nested_dir::<false>,
        write_to_file_in_nested_dir::<true>,
        small_write_to_file_in_deeply_nested_dir::<false>,
        small_write_to_file_in_deeply_nested_dir::<true>,
        multiple_writes_to_same_file::<false>,
        multiple_writes_to_same_file::<true>,
    ]
);

fn small_write_to_empty_file<const CLOSE_AFTER: bool>(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file to write to
            fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap()
        })
        .test(async |fixture, (file, mut fh)| {
            // Small write of 1 byte
            let data = vec![b'A'; 1];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(0), data)
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
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
                blob_resize: 1,
                blob_write: 2,
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
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + 2 * close_after,
                    FixtureType::FuserWithoutInodeCache => 4 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 20 + 16 * close_after,
                    FixtureType::FuserWithoutInodeCache => 36 + 32 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 3,
                store_flush_block: close_after,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 2,
                store: 2,
                ..LLActionCounts::ZERO
            },
        }})
}

fn small_write_to_middle_of_small_file<const CLOSE_AFTER: bool>(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file, write some initial data
            let (file, mut fh) = fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();
            let initial_data = vec![b'X'; 2 * BLOCKSIZE_BYTES as usize];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(0), initial_data)
                .await
                .unwrap();

            (file, fh)
        })
        .test(async |fixture, (file, mut fh)| {
            let data = vec![b'A'; 1];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(BLOCKSIZE_BYTES), data)
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
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
                blob_resize: 1,
                blob_write: 2,
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
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 5 + 4 * close_after,
                    FixtureType::FuserWithoutInodeCache => 9 + 8 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 41 + 30 * close_after,
                    FixtureType::FuserWithoutInodeCache => 71 + 60 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 2,
                store_flush_block: close_after,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 5,
                store: 2,
                ..LLActionCounts::ZERO
            },
        }})
}

fn small_write_beyond_end_of_small_file<const CLOSE_AFTER: bool>(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file, write some initial data
            let (file, mut fh) = fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();
            let initial_data = vec![b'X'; 2 * BLOCKSIZE_BYTES as usize];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(0), initial_data)
                .await
                .unwrap();

            (file, fh)
        })
        .test(async |fixture, (file, mut fh)| {
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
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
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
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                    FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_resize: 1,
                blob_write: 2,
                blob_flush: close_after,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 6 + 4 * close_after,
                    FixtureType::FuserWithoutInodeCache => 10 + 8 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_create: 1,
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 50 + 30 * close_after,
                    FixtureType::FuserWithoutInodeCache => 80 + 60 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 5,
                store_flush_block: close_after,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                exists: 1,
                load: 4,
                store: 4,
                ..LLActionCounts::ZERO
            },
        }})
}

fn small_write_to_middle_of_large_file<const CLOSE_AFTER: bool>(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file, write some initial data
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
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
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
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                    FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_resize: 1,
                blob_write: 2,
                blob_flush: close_after,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 8 + 6 * close_after,
                    FixtureType::FuserWithoutInodeCache => 14 + 12 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 63 + 44 * close_after,
                    FixtureType::FuserWithoutInodeCache => 107 + 88 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 2,
                store_flush_block: close_after,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 8,
                store: 2,
                ..LLActionCounts::ZERO
            },
        }})
}

fn small_write_beyond_end_of_large_file<const CLOSE_AFTER: bool>(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file, write some initial data
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
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
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
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                    FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_resize: 1,
                blob_write: 2,
                blob_flush: close_after,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 32 + 6 * close_after,
                    FixtureType::FuserWithoutInodeCache => 38 + 12 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_create: 24,
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 176 + 44 * close_after,
                    FixtureType::FuserWithoutInodeCache => 220 + 88 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 17,
                store_flush_block: close_after,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                exists: 24,
                load: 6,
                store: 28,
                ..LLActionCounts::ZERO
            },
        }})
}

fn large_write_to_empty_file<const CLOSE_AFTER: bool>(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file to write to
            fixture
                .filesystem
                .create_and_open_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap()
        })
        .test(async |fixture, (file, mut fh)| {
            let data = vec![b'A'; 2 * NUM_BYTES_FOR_THREE_LEVEL_TREE as usize];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(0), data)
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
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
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                    FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_resize: 1,
                blob_write: 2,
                blob_flush: close_after,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 50 + 6 * close_after,
                    FixtureType::FuserWithoutInodeCache => 52 + 12 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_create: 48,
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 254 + 44 * close_after,
                    FixtureType::FuserWithoutInodeCache => 270 + 88 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 41,
                store_flush_block: close_after,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                exists: 48,
                load: 2,
                store: 50,
                ..LLActionCounts::ZERO
            },
        }})
}

fn large_write_to_middle_of_large_file<const CLOSE_AFTER: bool>(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file, write some initial data
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
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
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
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                    FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_resize: 1,
                blob_write: 2,
                blob_flush: close_after,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 11 + 6 * close_after,
                    FixtureType::FuserWithoutInodeCache => 17 + 12 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_overwrite: 20,
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 104 + 44 * close_after,
                    FixtureType::FuserWithoutInodeCache => 148 + 88 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 3,
                store_flush_block: close_after,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                exists: 20,
                load: 11,
                store: 23,
                ..LLActionCounts::ZERO
            },
        }})
}

fn large_write_beyond_end_of_large_file<const CLOSE_AFTER: bool>(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create and open a file, write some initial data
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
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
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
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                    FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_resize: 1,
                blob_write: 2,
                blob_flush: close_after,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 55 + 6 * close_after,
                    FixtureType::FuserWithoutInodeCache => 61 + 12 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_create: 47,
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 280 + 44 * close_after,
                    FixtureType::FuserWithoutInodeCache => 324 + 88 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 32,
                store_flush_block: close_after,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                exists: 47,
                load: 6,
                store: 51,
                ..LLActionCounts::ZERO
            },
        }})
}

fn write_to_file_in_nested_dir<const CLOSE_AFTER: bool>(test_driver: impl TestDriver) -> impl TestReady {
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
        .test(async |fixture, (file, mut fh)| {
            let data = vec![b'A'; 1];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(0), data)
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
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
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                    FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_resize: 1,
                blob_write: 2,
                blob_flush: close_after,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + 2 * close_after,
                    FixtureType::FuserWithoutInodeCache => 6 + 6 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 20 + 16 * close_after,
                    FixtureType::FuserWithoutInodeCache => 54 + 50 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 3,
                store_flush_block: close_after,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 3, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store: 2,
                ..LLActionCounts::ZERO
            },
        }})
}

fn small_write_to_file_in_deeply_nested_dir<const CLOSE_AFTER: bool>(test_driver: impl TestDriver) -> impl TestReady {
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
        .test(async |fixture, (file, mut fh)| {
            let data = vec![b'A'; 1];
            fixture
                .filesystem
                .write(file.clone(), &mut fh, NumBytes::from(0), data)
                .await
                .unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
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
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 0,
                    FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_resize: 1,
                blob_write: 2,
                blob_flush: close_after,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2 + 2 * close_after,
                    FixtureType::FuserWithoutInodeCache => 10 + 10 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 20 + 16 * close_after,
                    FixtureType::FuserWithoutInodeCache => 90 + 86 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 3,
                store_flush_block: close_after,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store: 2,
                ..LLActionCounts::ZERO
            },
        }})
}

fn multiple_writes_to_same_file<const CLOSE_AFTER: bool>(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
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
            for i in 0..10 {
                let data = vec![b'A'; 1];
                fixture
                    .filesystem
                    .write(
                        file.clone(),
                        &mut fh,
                        NumBytes::from(NUM_BYTES_FOR_THREE_LEVEL_TREE + i),
                        data,
                    )
                    .await
                    .unwrap();
            }
            maybe_close::<CLOSE_AFTER, _>(fixture, file, fh).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 20 + 2 * close_after,
                    FixtureType::FuserWithoutInodeCache => 40 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 10 + close_after,
                    FixtureType::FuserWithoutInodeCache => 20 + 2 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 20 + 2 * close_after,
                    FixtureType::FuserWithoutInodeCache => 40 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_resize: 10,
                blob_write: 20,
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
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 80 + 6 * close_after,
                    FixtureType::FuserWithoutInodeCache => 140 + 12 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 630 + 44 * close_after,
                    FixtureType::FuserWithoutInodeCache => 1070 + 88 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 20,
                store_flush_block: close_after,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 8,
                store: 2,
                ..LLActionCounts::ZERO
            },
        }})
}
