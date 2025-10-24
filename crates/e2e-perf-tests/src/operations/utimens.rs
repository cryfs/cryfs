use crate::filesystem_driver::FilesystemDriver as _;
use crate::filesystem_fixture::ActionCounts;
use crate::perf_test_macro::FixtureType;
use crate::test_driver::TestDriver;
use crate::test_driver::TestReady;
use cryfs_blobstore::BlobStoreActionCounts;
use cryfs_blockstore::HLActionCounts;
use cryfs_blockstore::LLActionCounts;
use cryfs_rustfs::AbsolutePath;
use cryfs_rustfs::PathComponent;
use std::time::{Duration, SystemTime};

crate::perf_test_macro::perf_test!(
    utimens,
    [
        file_in_rootdir,
        dir_in_rootdir,
        symlink_in_rootdir,
        file_in_nesteddir,
        file_in_deeplynesteddir,
    ]
);

fn file_in_rootdir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create a file so we have something to update timestamps for
            fixture
                .filesystem
                .create_file(None, PathComponent::try_from_str("testfile.txt").unwrap())
                .await
                .unwrap()
        })
        .test(async |fixture, file| {
            let atime = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);
            let mtime = SystemTime::UNIX_EPOCH + Duration::from_secs(2000);

            fixture
                .filesystem
                .utimens(Some(file), Some(atime), Some(mtime))
                .await
                .unwrap();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 3, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache => 0,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 1,
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 3, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: 1,
                blob_resize: 1,
                blob_num_bytes: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 3, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 9,
                    FixtureType::Fusemt => 18,
                    FixtureType::FuserWithoutInodeCache => 25, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 2,
                },
                store: 1,
                ..LLActionCounts::ZERO
            },
        })
}

fn dir_in_rootdir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create a directory so we have something to update timestamps for
            fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("testdir").unwrap())
                .await
                .unwrap()
        })
        .test(async |fixture, dir| {
            let atime = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);
            let mtime = SystemTime::UNIX_EPOCH + Duration::from_secs(2000);

            fixture
                .filesystem
                .utimens(Some(dir), Some(atime), Some(mtime))
                .await
                .unwrap();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 3, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 3,
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 3, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: 1,
                blob_resize: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 3, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 11,
                    FixtureType::Fusemt => 20,
                    FixtureType::FuserWithoutInodeCache => 29, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 2,
                },
                store: 1,
                ..LLActionCounts::ZERO
            },
        })
}

fn symlink_in_rootdir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create a symlink so we have something to update timestamps for
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
            let atime = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);
            let mtime = SystemTime::UNIX_EPOCH + Duration::from_secs(2000);

            fixture
                .filesystem
                .utimens(Some(symlink), Some(atime), Some(mtime))
                .await
                .unwrap();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 3, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 3,
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 3, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: 1,
                blob_resize: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 3, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 11,
                    FixtureType::Fusemt => 20,
                    FixtureType::FuserWithoutInodeCache => 29, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 2,
                },
                store: 1,
                ..LLActionCounts::ZERO
            },
        })
}

fn file_in_nesteddir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create a nested directory and a file in it
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
            let atime = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);
            let mtime = SystemTime::UNIX_EPOCH + Duration::from_secs(2000);

            fixture
                .filesystem
                .utimens(Some(file), Some(atime), Some(mtime))
                .await
                .unwrap();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 3,
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache => 0,
                    FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 3,
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 3,
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: 1,
                blob_resize: 1,
                blob_num_bytes: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 3,
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 9,
                    FixtureType::Fusemt => 27,
                    FixtureType::FuserWithoutInodeCache => 43, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 3,
                },
                store: 1,
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
                    Some(nested_dir.clone()),
                    PathComponent::try_from_str("testfile.txt").unwrap(),
                )
                .await
                .unwrap()
        })
        .test(async |fixture, file| {
            let atime = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);
            let mtime = SystemTime::UNIX_EPOCH + Duration::from_secs(2000);

            fixture
                .filesystem
                .utimens(Some(file), Some(atime), Some(mtime))
                .await
                .unwrap();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 9, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache => 0,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 7,
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 9, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_resize: 1,
                blob_num_bytes: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 9, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 9,
                    FixtureType::Fusemt => 45,
                    FixtureType::FuserWithoutInodeCache => 79, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 5,
                },
                store: 1,
                ..LLActionCounts::ZERO
            },
        })
}
