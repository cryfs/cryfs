use crate::filesystem_driver::FilesystemDriver;
use crate::fixture::ActionCounts;
use crate::perf_test_macro::FixtureType;
use crate::test_driver::TestDriver;
use crate::test_driver::TestReady;
use cryfs_blobstore::BlobStoreActionCounts;
use cryfs_blockstore::HLActionCounts;
use cryfs_blockstore::LLActionCounts;
use cryfs_rustfs::AbsolutePath;
use cryfs_rustfs::PathComponent;

crate::perf_test_macro::perf_test!(
    getattr,
    [
        rootdir,
        file_in_rootdir,
        dir_in_rootdir,
        symlink_in_rootdir,
        file_in_nesteddir,
        file_in_deeplynesteddir,
    ]
);

fn rootdir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // No setup needed for root directory
        })
        .test(async |fixture, ()| {
            fixture.filesystem.getattr(None).await.unwrap();
        })
        .expect_op_counts(|_fixture_type, _atime_behavior| ActionCounts {
            // Counts are all zero because we don't store attributes for the root directory
            blobstore: BlobStoreActionCounts {
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                ..LLActionCounts::ZERO
            },
        })
}

fn file_in_rootdir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            fixture
                .filesystem
                .create_file(None, PathComponent::try_from_str("testfile.txt").unwrap())
                .await
                .unwrap()
        })
        .test(async |fixture, file| {
            fixture.filesystem.getattr(Some(file)).await.unwrap();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO: Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_num_bytes: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO: Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 16,
                    FixtureType::FuserWithoutInodeCache => 32, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO: Check if these counts are what we'd expect
                load: 2,
                ..LLActionCounts::ZERO
            },
        })
}

fn dir_in_rootdir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("testdir").unwrap())
                .await
                .unwrap()
        })
        .test(async |fixture, dir| {
            fixture.filesystem.getattr(Some(dir)).await.unwrap();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO: Check if these counts are what we'd expect
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
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO: Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 18,
                    FixtureType::FuserWithoutInodeCache => 36, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO: Check if these counts are what we'd expect
                load: 2,
                ..LLActionCounts::ZERO
            },
        })
}

fn symlink_in_rootdir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            fixture
                .filesystem
                .create_symlink(
                    None,
                    PathComponent::try_from_str("link").unwrap(),
                    AbsolutePath::try_from_str("/target/file.txt").unwrap(),
                )
                .await
                .unwrap()
        })
        .test(async |fixture, symlink| {
            fixture.filesystem.getattr(Some(symlink)).await.unwrap();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO: Check if these counts are what we'd expect
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
                blob_num_bytes: 0,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO: Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 18,
                    FixtureType::FuserWithoutInodeCache => 36, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO: Check if these counts are what we'd expect
                load: 2,
                ..LLActionCounts::ZERO
            },
        })
}

fn file_in_nesteddir(test_driver: impl TestDriver) -> impl TestReady {
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
                .create_file(
                    Some(parent),
                    PathComponent::try_from_str("testfile.txt").unwrap(),
                )
                .await
                .unwrap()
        })
        .test(async |fixture, file| {
            fixture.filesystem.getattr(Some(file)).await.unwrap();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO: Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 3,
                    FixtureType::FuserWithoutInodeCache => 6, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 3,
                    FixtureType::FuserWithoutInodeCache => 6, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_num_bytes: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO: Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 3,
                    FixtureType::FuserWithoutInodeCache => 6, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 16,
                    FixtureType::Fusemt => 25,
                    FixtureType::FuserWithoutInodeCache => 50, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO: Check if these counts are what we'd expect
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 3,
                },
                ..LLActionCounts::ZERO
            },
        })
}

fn file_in_deeplynesteddir(test_driver: impl TestDriver) -> impl TestReady {
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
                .create_file(
                    Some(nested_dir),
                    PathComponent::try_from_str("testfile.txt").unwrap(),
                )
                .await
                .unwrap()
        })
        .test(async |fixture, file| {
            fixture.filesystem.getattr(Some(file)).await.unwrap();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO: Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 10, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 8, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 10, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_num_bytes: match fixture_type {
                    FixtureType::Fusemt | FixtureType::FuserWithInodeCache => 1,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO: Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 10, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 16,
                    FixtureType::Fusemt => 43,
                    FixtureType::FuserWithoutInodeCache => 86, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO: Check if these counts are what we'd expect
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 5,
                },
                ..LLActionCounts::ZERO
            },
        })
}
