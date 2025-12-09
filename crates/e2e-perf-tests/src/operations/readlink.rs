use crate::filesystem_driver::FilesystemDriver as _;
use crate::filesystem_fixture::ActionCounts;
use crate::filesystem_fixture::NUM_BYTES_FOR_THREE_LEVEL_TREE;
use crate::perf_test_macro::FixtureType;
use crate::test_driver::TestDriver;
use crate::test_driver::TestReady;
use cryfs_blobstore::BlobStoreActionCounts;
use cryfs_blockstore::HLActionCounts;
use cryfs_blockstore::LLActionCounts;
use cryfs_rustfs::AtimeUpdateBehavior;
use cryfs_utils::path::{AbsolutePath, PathComponent};

crate::perf_test_macro::perf_test!(
    readlink,
    [
        from_rootdir,
        from_nesteddir,
        from_deeplynesteddir,
        long_target,
    ]
);

fn from_rootdir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            fixture
                .filesystem
                .create_symlink(
                    None,
                    PathComponent::try_from_str("mysymlink").unwrap(),
                    AbsolutePath::try_from_str("/target/path").unwrap(),
                )
                .await
                .unwrap()
        })
        .test(async |fixture, symlink| {
            fixture.filesystem.readlink(symlink).await.unwrap();
        })
        .expect_op_counts(|fixture_type, atime_behavior| {
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
                        FixtureType::FuserWithInodeCache => 1,
                        FixtureType::Fusemt => 2,
                        FixtureType::FuserWithoutInodeCache => 3, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache => 1,
                        FixtureType::Fusemt => 2,
                        FixtureType::FuserWithoutInodeCache => 3, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache => 1,
                        FixtureType::Fusemt => 2,
                        FixtureType::FuserWithoutInodeCache => 3, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_write: expect_atime_update,
                    blob_resize: expect_atime_update,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache => 1,
                        FixtureType::Fusemt => 2,
                        FixtureType::FuserWithoutInodeCache => 3, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache => 9,
                        FixtureType::Fusemt => 18,
                        FixtureType::FuserWithoutInodeCache => 27, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    } + 2 * expect_atime_update,
                    blob_data_mut: expect_atime_update,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: match fixture_type {
                        FixtureType::FuserWithInodeCache => 1,
                        FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 2,
                    },
                    store: expect_atime_update,
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn from_nesteddir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create the nested dir and a symlink in it
            let parent = fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap();
            fixture
                .filesystem
                .create_symlink(
                    Some(parent),
                    PathComponent::try_from_str("mysymlink").unwrap(),
                    AbsolutePath::try_from_str("/target/path").unwrap(),
                )
                .await
                .unwrap()
        })
        .test(async |fixture, symlink| {
            fixture.filesystem.readlink(symlink).await.unwrap();
        })
        .expect_op_counts(|fixture_type, atime_behavior| {
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
                        FixtureType::FuserWithInodeCache => 1,
                        FixtureType::Fusemt => 3,
                        FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache => 1,
                        FixtureType::Fusemt => 3,
                        FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache => 1,
                        FixtureType::Fusemt => 3,
                        FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_write: expect_atime_update,
                    blob_resize: expect_atime_update,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache => 1,
                        FixtureType::Fusemt => 3,
                        FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    },
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache => 9,
                        FixtureType::Fusemt => 27,
                        FixtureType::FuserWithoutInodeCache => 45, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    } + 2 * expect_atime_update,
                    blob_data_mut: expect_atime_update,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: match fixture_type {
                        FixtureType::FuserWithInodeCache => 1,
                        FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 3,
                    },
                    store: expect_atime_update,
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn from_deeplynesteddir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create the deeply nested dir
            let parent = fixture
                .filesystem
                .mkdir_recursive(AbsolutePath::try_from_str("/nested1/nested2/nested3").unwrap())
                .await
                .unwrap();

            // Then create the symlink in the deeply nested dir
            fixture
                .filesystem
                .create_symlink(
                    Some(parent),
                    PathComponent::try_from_str("mysymlink").unwrap(),
                    AbsolutePath::try_from_str("/target/path").unwrap(),
                )
                .await
                .unwrap()
        })
        .test(async |fixture, symlink| {
            fixture.filesystem.readlink(symlink).await.unwrap();
        })
        .expect_op_counts(|fixture_type, atime_behavior| {
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
                        FixtureType::FuserWithInodeCache => 1,
                        FixtureType::Fusemt => 5,
                        FixtureType::FuserWithoutInodeCache => 9,
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache => 1,
                        FixtureType::Fusemt => 5,
                        FixtureType::FuserWithoutInodeCache => 9,
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache => 1,
                        FixtureType::Fusemt => 5,
                        FixtureType::FuserWithoutInodeCache => 9,
                    },
                    blob_write: expect_atime_update,
                    blob_resize: expect_atime_update,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache => 1,
                        FixtureType::Fusemt => 5,
                        FixtureType::FuserWithoutInodeCache => 9,
                    },
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache => 9,
                        FixtureType::Fusemt => 45,
                        FixtureType::FuserWithoutInodeCache => 81,
                    } + 2 * expect_atime_update,
                    blob_data_mut: expect_atime_update,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: match fixture_type {
                        FixtureType::FuserWithInodeCache => 1,
                        FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 5,
                    },
                    store: expect_atime_update,
                    ..LLActionCounts::ZERO
                },
            }
        })
}

fn long_target(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // Create a very long target path which is stored across multiple nodes
            let long_target =
                "/very/long".repeat(NUM_BYTES_FOR_THREE_LEVEL_TREE as usize / 10) + "/target/path";

            // First create a symlink with the long target
            fixture
                .filesystem
                .create_symlink(
                    None,
                    PathComponent::try_from_str("longlink").unwrap(),
                    &AbsolutePath::try_from_str(&long_target).unwrap(),
                )
                .await
                .unwrap()
        })
        .test(async |fixture, symlink| {
            fixture.filesystem.readlink(symlink).await.unwrap();
        })
        .expect_op_counts(|fixture_type, atime_behavior| {
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
                        FixtureType::FuserWithInodeCache => 1,
                        FixtureType::Fusemt => 2,
                        FixtureType::FuserWithoutInodeCache => 3,
                    },
                    blob_read_all: match fixture_type {
                        FixtureType::FuserWithInodeCache => 1,
                        FixtureType::Fusemt => 2,
                        FixtureType::FuserWithoutInodeCache => 3,
                    },
                    blob_read: match fixture_type {
                        FixtureType::FuserWithInodeCache => 1,
                        FixtureType::Fusemt => 2,
                        FixtureType::FuserWithoutInodeCache => 3,
                    },
                    blob_write: expect_atime_update,
                    blob_resize: expect_atime_update,
                    ..BlobStoreActionCounts::ZERO
                },
                high_level: HLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    store_load: match fixture_type {
                        FixtureType::FuserWithInodeCache => 30,
                        FixtureType::Fusemt => 31,
                        FixtureType::FuserWithoutInodeCache => 61,
                    },
                    blob_data: match fixture_type {
                        FixtureType::FuserWithInodeCache => 196,
                        FixtureType::Fusemt => 205,
                        FixtureType::FuserWithoutInodeCache => 401,
                    } + 2 * expect_atime_update,
                    blob_data_mut: expect_atime_update,
                    ..HLActionCounts::ZERO
                },
                low_level: LLActionCounts {
                    // TODO Check if these counts are what we'd expect
                    load: match fixture_type {
                        FixtureType::FuserWithInodeCache => 26,
                        FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 27,
                    },
                    store: expect_atime_update,
                    ..LLActionCounts::ZERO
                },
            }
        })
}
