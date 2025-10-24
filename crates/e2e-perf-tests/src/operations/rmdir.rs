use crate::filesystem_driver::FilesystemDriver;
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
use cryfs_rustfs::PathComponent;

crate::perf_test_macro::perf_test!(
    rmdir,
    [
        existing_empty_dir_from_rootdir,
        not_existing_from_rootdir,
        non_empty_directory_from_rootdir,
        empty_dir_from_nested_dir,
        non_empty_dir_from_nested_dir,
        not_existing_from_nested_dir,
        empty_dir_from_deeply_nested_dir,
        non_empty_dir_from_deeply_nested_dir,
        not_existing_from_deeply_nested_dir,
        try_rmdir_file_in_rootdir,
        try_rmdir_file_in_nested_dir,
        try_rmdir_file_in_deeply_nested_dir,
        try_rmdir_symlink_in_root_dir,
        try_rmdir_symlink_in_nested_dir,
        try_rmdir_symlink_in_deeply_nested_dir,
        rmdir_large_directory,
    ]
);

fn existing_empty_dir_from_rootdir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create an empty directory to remove
            fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("emptydir").unwrap())
                .await
                .unwrap();
        })
        .test(async |fixture, ()| {
            fixture
                .filesystem
                .rmdir(None, PathComponent::try_from_str("emptydir").unwrap())
                .await
                .unwrap();
        })
        .expect_op_counts(|_fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: 2,
                blob_read_all: 2,
                blob_read: 2,
                blob_resize: 1,
                blob_write: 1,
                blob_remove: 1,
                blob_flush: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: 2,
                blob_data: 20,
                blob_data_mut: 1,
                store_remove: 1,
                store_flush_block: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 2,
                store: 1,
                remove: 1,
                ..LLActionCounts::ZERO
            },
        })
}

fn not_existing_from_rootdir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |_fixture| {
            // No setup needed
        })
        .test(async |fixture, ()| {
            fixture
                .filesystem
                .rmdir(None, PathComponent::try_from_str("nonexistent").unwrap())
                .await
                .unwrap_err();
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

fn non_empty_directory_from_rootdir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // Create a directory with a file in it
            let dir = fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("nonemptydir").unwrap())
                .await
                .unwrap();
            fixture
                .filesystem
                .create_file(Some(dir), PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();
        })
        .test(async |fixture, ()| {
            fixture
                .filesystem
                .rmdir(None, PathComponent::try_from_str("nonemptydir").unwrap())
                .await
                .unwrap_err(); // Should fail because directory is not empty
        })
        .expect_op_counts(|_fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: 2, // Load root dir and the non-empty dir
                blob_read_all: 2,
                blob_read: 2,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: 2,
                blob_data: 18,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 2,
                ..LLActionCounts::ZERO
            },
        })
}

fn empty_dir_from_nested_dir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create the nested directory structure
            let parent = fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap();
            fixture
                .filesystem
                .mkdir(
                    Some(parent.clone()),
                    PathComponent::try_from_str("emptydir").unwrap(),
                )
                .await
                .unwrap();
            parent
        })
        .test(async |fixture, parent| {
            fixture
                .filesystem
                .rmdir(
                    Some(parent),
                    PathComponent::try_from_str("emptydir").unwrap(),
                )
                .await
                .unwrap();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 3,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 3,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 3,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_resize: 2,
                blob_write: 2,
                blob_remove: 1,
                blob_flush: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 3,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 22,
                    FixtureType::Fusemt => 31,
                    FixtureType::FuserWithoutInodeCache => 40, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 2,
                store_remove: 1,
                store_flush_block: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 3,
                },
                store: 2,
                remove: 1,
                ..LLActionCounts::ZERO
            },
        })
}

fn non_empty_dir_from_nested_dir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // Create a nested directory with a non-empty subdirectory
            let parent = fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("parent").unwrap())
                .await
                .unwrap();
            let nonempty = fixture
                .filesystem
                .mkdir(
                    Some(parent.clone()),
                    PathComponent::try_from_str("nonempty").unwrap(),
                )
                .await
                .unwrap();
            // Add a file to make the directory non-empty
            fixture
                .filesystem
                .create_file(
                    Some(nonempty),
                    PathComponent::try_from_str("file.txt").unwrap(),
                )
                .await
                .unwrap();
            parent
        })
        .test(async |fixture, parent| {
            fixture
                .filesystem
                .rmdir(
                    Some(parent),
                    PathComponent::try_from_str("nonempty").unwrap(),
                )
                .await
                .unwrap_err(); // Should fail because directory is not empty
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 3,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 3,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 3,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: 1,
                blob_resize: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 3,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 20,
                    FixtureType::Fusemt => 29,
                    FixtureType::FuserWithoutInodeCache => 38, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 3,
                },
                store: 1,
                ..LLActionCounts::ZERO
            },
        })
}

fn not_existing_from_nested_dir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create a parent directory
            fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("parent").unwrap())
                .await
                .unwrap()
        })
        .test(async |fixture, parent| {
            fixture
                .filesystem
                .rmdir(
                    Some(parent),
                    PathComponent::try_from_str("nonexistent").unwrap(),
                )
                .await
                .unwrap_err();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
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
                blob_write: 1,
                blob_resize: 1,
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
                    FixtureType::FuserWithInodeCache => 11,
                    FixtureType::Fusemt => 20,
                    FixtureType::FuserWithoutInodeCache => 29, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 2,
                },
                store: 1,
                ..LLActionCounts::ZERO
            },
        })
}

fn empty_dir_from_deeply_nested_dir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // Create a deeply nested directory structure with an empty dir to remove
            let parent = fixture
                .filesystem
                .mkdir_recursive(AbsolutePath::try_from_str("/deep/nested/dir").unwrap())
                .await
                .unwrap();
            fixture
                .filesystem
                .mkdir(
                    Some(parent.clone()),
                    PathComponent::try_from_str("emptydir").unwrap(),
                )
                .await
                .unwrap();
            parent
        })
        .test(async |fixture, parent| {
            fixture
                .filesystem
                .rmdir(
                    Some(parent),
                    PathComponent::try_from_str("emptydir").unwrap(),
                )
                .await
                .unwrap();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 8, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 8, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 8, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_resize: 2,
                blob_write: 2,
                blob_remove: 1,
                blob_flush: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 8, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 22,
                    FixtureType::Fusemt => 49,
                    FixtureType::FuserWithoutInodeCache => 76, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 2,
                store_remove: 1,
                store_flush_block: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 5,
                },
                store: 2,
                remove: 1,
                ..LLActionCounts::ZERO
            },
        })
}

fn non_empty_dir_from_deeply_nested_dir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // Create a deeply nested directory with a non-empty directory
            let deeply_nested = fixture
                .filesystem
                .mkdir_recursive(AbsolutePath::try_from_str("/deep/nested/dir").unwrap())
                .await
                .unwrap();
            let nonempty = fixture
                .filesystem
                .mkdir(
                    Some(deeply_nested.clone()),
                    PathComponent::try_from_str("nonempty").unwrap(),
                )
                .await
                .unwrap();
            // Add a file to make the directory non-empty
            fixture
                .filesystem
                .create_file(
                    Some(nonempty),
                    PathComponent::try_from_str("file.txt").unwrap(),
                )
                .await
                .unwrap();
            deeply_nested
        })
        .test(async |fixture, deeply_nested| {
            fixture
                .filesystem
                .rmdir(
                    Some(deeply_nested),
                    PathComponent::try_from_str("nonempty").unwrap(),
                )
                .await
                .unwrap_err(); // Should fail because directory is not empty
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 8, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 8, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 8, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: 1,
                blob_resize: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 8, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 20,
                    FixtureType::Fusemt => 47,
                    FixtureType::FuserWithoutInodeCache => 74, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 5,
                },
                store: 1,
                ..LLActionCounts::ZERO
            },
        })
}

fn not_existing_from_deeply_nested_dir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create a deeply nested directory
            fixture
                .filesystem
                .mkdir_recursive(AbsolutePath::try_from_str("/deep/nested/dir").unwrap())
                .await
                .unwrap()
        })
        .test(async |fixture, deeply_nested| {
            fixture
                .filesystem
                .rmdir(
                    Some(deeply_nested),
                    PathComponent::try_from_str("nonexistent").unwrap(),
                )
                .await
                .unwrap_err();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 7, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 7, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 7, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: 1,
                blob_resize: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 7, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 11,
                    FixtureType::Fusemt => 38,
                    FixtureType::FuserWithoutInodeCache => 65, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 4,
                },
                store: 1,
                ..LLActionCounts::ZERO
            },
        })
}

fn try_rmdir_file_in_rootdir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create a file that we'll try to rmdir (which should fail)
            fixture
                .filesystem
                .create_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();
        })
        .test(async |fixture, ()| {
            fixture
                .filesystem
                .rmdir(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap_err();
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

fn try_rmdir_file_in_nested_dir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // Create a nested directory structure with a file
            let parent = fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap();
            fixture
                .filesystem
                .create_file(
                    Some(parent.clone()),
                    PathComponent::try_from_str("file.txt").unwrap(),
                )
                .await
                .unwrap();
            parent
        })
        .test(async |fixture, parent| {
            fixture
                .filesystem
                .rmdir(
                    Some(parent),
                    PathComponent::try_from_str("file.txt").unwrap(),
                )
                .await
                .unwrap_err();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
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
                blob_write: 1,
                blob_resize: 1,
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
                    FixtureType::FuserWithInodeCache => 11,
                    FixtureType::Fusemt => 20,
                    FixtureType::FuserWithoutInodeCache => 29, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 2,
                },
                store: 1,
                ..LLActionCounts::ZERO
            },
        })
}

fn try_rmdir_file_in_deeply_nested_dir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // Create a deeply nested directory structure with a file
            let deeply_nested = fixture
                .filesystem
                .mkdir_recursive(AbsolutePath::try_from_str("/deep/nested/dir").unwrap())
                .await
                .unwrap();
            fixture
                .filesystem
                .create_file(
                    Some(deeply_nested.clone()),
                    PathComponent::try_from_str("file.txt").unwrap(),
                )
                .await
                .unwrap();
            deeply_nested
        })
        .test(async |fixture, parent| {
            fixture
                .filesystem
                .rmdir(
                    Some(parent),
                    PathComponent::try_from_str("file.txt").unwrap(),
                )
                .await
                .unwrap_err();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 7, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 7, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 7, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: 1,
                blob_resize: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 7, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 11,
                    FixtureType::Fusemt => 38,
                    FixtureType::FuserWithoutInodeCache => 65, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 4,
                },
                store: 1,
                ..LLActionCounts::ZERO
            },
        })
}

fn try_rmdir_symlink_in_root_dir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create a symlink in the root directory that we'll try to rmdir (which should fail)
            fixture
                .filesystem
                .create_symlink(
                    None,
                    PathComponent::try_from_str("link.txt").unwrap(),
                    AbsolutePath::try_from_str("/target/path").unwrap(),
                )
                .await
                .unwrap();
        })
        .test(async |fixture, ()| {
            fixture
                .filesystem
                .rmdir(None, PathComponent::try_from_str("link.txt").unwrap())
                .await
                .unwrap_err();
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

fn try_rmdir_symlink_in_nested_dir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // Create a nested directory structure with a symlink
            let parent = fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap();
            fixture
                .filesystem
                .create_symlink(
                    Some(parent.clone()),
                    PathComponent::try_from_str("link.txt").unwrap(),
                    AbsolutePath::try_from_str("/target/path").unwrap(),
                )
                .await
                .unwrap();
            parent
        })
        .test(async |fixture, parent| {
            fixture
                .filesystem
                .rmdir(
                    Some(parent),
                    PathComponent::try_from_str("link.txt").unwrap(),
                )
                .await
                .unwrap_err();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
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
                blob_write: 1,
                blob_resize: 1,
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
                    FixtureType::FuserWithInodeCache => 11,
                    FixtureType::Fusemt => 20,
                    FixtureType::FuserWithoutInodeCache => 29, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 2,
                },
                store: 1,
                ..LLActionCounts::ZERO
            },
        })
}

fn try_rmdir_symlink_in_deeply_nested_dir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // Create a deeply nested directory structure with a symlink
            let deeply_nested = fixture
                .filesystem
                .mkdir_recursive(AbsolutePath::try_from_str("/deep/nested/dir").unwrap())
                .await
                .unwrap();
            fixture
                .filesystem
                .create_symlink(
                    Some(deeply_nested.clone()),
                    PathComponent::try_from_str("link.txt").unwrap(),
                    AbsolutePath::try_from_str("/target/path").unwrap(),
                )
                .await
                .unwrap();
            deeply_nested
        })
        .test(async |fixture, parent| {
            fixture
                .filesystem
                .rmdir(
                    Some(parent),
                    PathComponent::try_from_str("link.txt").unwrap(),
                )
                .await
                .unwrap_err();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 7, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 7, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 7, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: 1,
                blob_resize: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 7, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 11,
                    FixtureType::Fusemt => 38,
                    FixtureType::FuserWithoutInodeCache => 65, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 4,
                },
                store: 1,
                ..LLActionCounts::ZERO
            },
        })
}

fn rmdir_large_directory(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // Create a large directory
            let dir = fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("large_dir").unwrap())
                .await
                .unwrap();

            // Create many subdirectories that will require multiple blocks to store
            let num_entries = NUM_BYTES_FOR_THREE_LEVEL_TREE / BLOCKID_LEN as u64;
            for i in 0..num_entries {
                fixture
                    .filesystem
                    .mkdir(
                        Some(dir.clone()),
                        PathComponent::try_from_str(&format!("subdir{}", i)).unwrap(),
                    )
                    .await
                    .unwrap();
            }

            dir
        })
        .test(async |fixture, large_dir| {
            // Determine how many subdirectories we created
            let num_entries = NUM_BYTES_FOR_THREE_LEVEL_TREE / BLOCKID_LEN as u64;

            // Remove each subdirectory
            for i in 0..num_entries {
                fixture
                    .filesystem
                    .rmdir(
                        Some(large_dir.clone()),
                        PathComponent::try_from_str(&format!("subdir{}", i)).unwrap(),
                    )
                    .await
                    .unwrap();
            }

            // Finally remove the empty large directory itself
            fixture
                .filesystem
                .rmdir(None, PathComponent::try_from_str("large_dir").unwrap())
                .await
                .unwrap();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // Performance numbers for removing many directories
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 401,
                    FixtureType::Fusemt => 602,
                    FixtureType::FuserWithoutInodeCache => 802, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache => 401,
                    FixtureType::Fusemt => 602,
                    FixtureType::FuserWithoutInodeCache => 802, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache => 401,
                    FixtureType::Fusemt => 602,
                    FixtureType::FuserWithoutInodeCache => 802, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_resize: match fixture_type {
                    FixtureType::FuserWithInodeCache => 201,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 401,
                },
                blob_write: match fixture_type {
                    FixtureType::FuserWithInodeCache => 201,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 401,
                },
                blob_remove: 201,
                blob_flush: 201,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // Performance numbers for removing many directories
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 14575,
                    FixtureType::Fusemt => 14776,
                    FixtureType::FuserWithoutInodeCache => 27078, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 107912,
                    FixtureType::Fusemt => 110121,
                    FixtureType::FuserWithoutInodeCache => 188480, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1018,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 1218,
                },
                store_remove_by_id: 98,
                store_remove: 215,
                store_overwrite: 9514,
                store_flush_block: 201,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // Performance numbers for removing many directories
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 313,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 314,
                },
                store: 201,
                remove: 313,
                ..LLActionCounts::ZERO
            },
        })
}
