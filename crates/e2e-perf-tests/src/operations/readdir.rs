use crate::filesystem_driver::FilesystemDriver as _;
use crate::filesystem_fixture::ActionCounts;
use crate::filesystem_fixture::NUM_BYTES_FOR_THREE_LEVEL_TREE;
use crate::perf_test_macro::FixtureType;
use crate::test_driver::TestDriver;
use crate::test_driver::TestReady;
use cryfs_blobstore::BlobStoreActionCounts;
use cryfs_blockstore::BLOCKID_LEN;
use cryfs_blockstore::HLActionCounts;
use cryfs_blockstore::LLActionCounts;
use cryfs_rustfs::AbsolutePath;
use cryfs_rustfs::AtimeUpdateBehavior;
use cryfs_rustfs::PathComponent;

crate::perf_test_macro::perf_test!(
    readdir,
    [
        empty_rootdir,
        rootdir_with_entries,
        nesteddir,
        deeplynesteddir,
        large_directory,
    ]
);

fn empty_rootdir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |_fixture| {
            // No setup needed for empty root directory
        })
        .test(async |fixture, ()| {
            fixture.filesystem.readdir(None).await.unwrap();
        })
        .expect_op_counts(|_fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: 1,
                blob_read_all: 1,
                blob_read: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: 1,
                blob_data: 9,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 1,
                ..LLActionCounts::ZERO
            },
        })
}

fn rootdir_with_entries(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create some entries in the root directory
            fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("dir1").unwrap())
                .await
                .unwrap();
            fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("dir2").unwrap())
                .await
                .unwrap();
            fixture
                .filesystem
                .create_file(None, PathComponent::try_from_str("file1").unwrap())
                .await
                .unwrap();
            fixture
                .filesystem
                .create_file(None, PathComponent::try_from_str("file2").unwrap())
                .await
                .unwrap();
            fixture
                .filesystem
                .create_symlink(
                    None,
                    PathComponent::try_from_str("link1").unwrap(),
                    AbsolutePath::try_from_str("/target/path").unwrap(),
                )
                .await
                .unwrap();
        })
        .test(async |fixture, ()| {
            fixture.filesystem.readdir(None).await.unwrap();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::FuserWithoutInodeCache => 2,
                    FixtureType::Fusemt => 1,
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::FuserWithoutInodeCache => 2,
                    FixtureType::Fusemt => 1,
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::FuserWithoutInodeCache => 2,
                    FixtureType::Fusemt => 1,
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::FuserWithoutInodeCache => 12,
                    FixtureType::Fusemt => 6,
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::FuserWithoutInodeCache => 88,
                    FixtureType::Fusemt => 44,
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 4,
                ..LLActionCounts::ZERO
            },
        })
}

fn nesteddir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create a nested directory with some entries
            let dir = fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap();
            fixture
                .filesystem
                .mkdir(
                    Some(dir.clone()),
                    PathComponent::try_from_str("dir1").unwrap(),
                )
                .await
                .unwrap();
            fixture
                .filesystem
                .create_file(
                    Some(dir.clone()),
                    PathComponent::try_from_str("file1").unwrap(),
                )
                .await
                .unwrap();
            fixture
                .filesystem
                .create_symlink(
                    Some(dir.clone()),
                    PathComponent::try_from_str("link1").unwrap(),
                    AbsolutePath::try_from_str("/target/path").unwrap(),
                )
                .await
                .unwrap();

            dir
        })
        .test(async |fixture, nested_dir| {
            fixture.filesystem.readdir(Some(nested_dir)).await.unwrap();
        })
        .expect_op_counts(|fixture_type, atime_behavior| {
            let expect_atime_update = match atime_behavior {
                AtimeUpdateBehavior::Noatime
                | AtimeUpdateBehavior::NodiratimeRelatime
                | AtimeUpdateBehavior::NodiratimeStrictatime => 0,
                AtimeUpdateBehavior::Relatime | AtimeUpdateBehavior::Strictatime => 1,
            };

            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                        FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                        FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                        FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_write: expect_atime_update,
                    blob_resize: expect_atime_update,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache => 10,
                        FixtureType::Fusemt => 6,
                        FixtureType::FuserWithoutInodeCache => 16, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache => 76,
                        FixtureType::Fusemt => 47,
                        FixtureType::FuserWithoutInodeCache => 123, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    } + 2 * expect_atime_update,
                    blob_data_mut: expect_atime_update,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: match fixture_type {
                        FixtureType::FuserWithInodeCache => 3,
                        FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 4,
                    },
                    store: expect_atime_update,
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn deeplynesteddir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create a deeply nested directory with some entries
            let dir = fixture
                .filesystem
                .mkdir_recursive(AbsolutePath::try_from_str("/deep/nested/dir").unwrap())
                .await
                .unwrap();
            fixture
                .filesystem
                .mkdir(
                    Some(dir.clone()),
                    PathComponent::try_from_str("dir1").unwrap(),
                )
                .await
                .unwrap();
            fixture
                .filesystem
                .create_file(
                    Some(dir.clone()),
                    PathComponent::try_from_str("file1").unwrap(),
                )
                .await
                .unwrap();
            fixture
                .filesystem
                .create_symlink(
                    Some(dir.clone()),
                    PathComponent::try_from_str("link1").unwrap(),
                    AbsolutePath::try_from_str("/target/path").unwrap(),
                )
                .await
                .unwrap();

            dir
        })
        .test(async |fixture, deeply_nested_dir| {
            fixture
                .filesystem
                .readdir(Some(deeply_nested_dir))
                .await
                .unwrap();
        })
        .expect_op_counts(|fixture_type, atime_behavior| {
            let expect_atime_update = match atime_behavior {
                AtimeUpdateBehavior::Noatime
                | AtimeUpdateBehavior::NodiratimeRelatime
                | AtimeUpdateBehavior::NodiratimeStrictatime => 0,
                AtimeUpdateBehavior::Relatime | AtimeUpdateBehavior::Strictatime => 1,
            };

            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache => 2,
                        FixtureType::Fusemt => 4,
                        FixtureType::FuserWithoutInodeCache => 8, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache => 2,
                        FixtureType::Fusemt => 4,
                        FixtureType::FuserWithoutInodeCache => 8, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache => 2,
                        FixtureType::Fusemt => 4,
                        FixtureType::FuserWithoutInodeCache => 8, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_write: expect_atime_update,
                    blob_resize: expect_atime_update,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache => 10,
                        FixtureType::Fusemt => 8,
                        FixtureType::FuserWithoutInodeCache => 20, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache => 76,
                        FixtureType::Fusemt => 65,
                        FixtureType::FuserWithoutInodeCache => 159, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    } + 2 * expect_atime_update,
                    blob_data_mut: expect_atime_update,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: match fixture_type {
                        FixtureType::FuserWithInodeCache => 3,
                        FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 6,
                    },
                    store: expect_atime_update,
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn large_directory(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // Create a directory with many entries to test readdir performance with large directories
            let dir = fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("large_dir").unwrap())
                .await
                .unwrap();

            // Create many entries that will definitely require multiple blocks to store
            let num_entries = NUM_BYTES_FOR_THREE_LEVEL_TREE / BLOCKID_LEN as u64;
            for i in 0..num_entries {
                fixture
                    .filesystem
                    .create_file(
                        Some(dir.clone()),
                        PathComponent::try_from_str(&format!("file{}", i)).unwrap(),
                    )
                    .await
                    .unwrap();
            }

            dir
        })
        .test(async |fixture, large_dir| {
            fixture.filesystem.readdir(Some(large_dir)).await.unwrap();
        })
        .expect_op_counts(|fixture_type, atime_behavior| {
            let expect_atime_update = match atime_behavior {
                AtimeUpdateBehavior::Noatime
                | AtimeUpdateBehavior::NodiratimeRelatime
                | AtimeUpdateBehavior::NodiratimeStrictatime => 0,
                AtimeUpdateBehavior::Relatime | AtimeUpdateBehavior::Strictatime => 1,
            };

            ActionCounts {
                blobstore: BlobStoreActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                        FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                        FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                        FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_write: expect_atime_update,
                    blob_resize: expect_atime_update,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache => 232,
                        FixtureType::Fusemt => 117,
                        FixtureType::FuserWithoutInodeCache => 349, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache => 1468,
                        FixtureType::Fusemt => 743,
                        FixtureType::FuserWithoutInodeCache => 2211, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    } + 2 * expect_atime_update,
                    blob_data_mut: expect_atime_update,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: match fixture_type {
                        FixtureType::FuserWithInodeCache => 110,
                        FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 111,
                    },
                    store: expect_atime_update,
                    ..LLActionCounts::ZERO
                },
            }
        })
}
