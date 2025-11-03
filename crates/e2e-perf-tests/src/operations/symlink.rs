use crate::filesystem_driver::FilesystemDriver as _;
use crate::filesystem_fixture::ActionCounts;
use crate::filesystem_fixture::NUM_BYTES_FOR_THREE_LEVEL_TREE;
use crate::perf_test_macro::FixtureType;
use crate::test_driver::{TestDriver, TestReady};
use cryfs_blobstore::BlobStoreActionCounts;
use cryfs_blockstore::HLActionCounts;
use cryfs_blockstore::LLActionCounts;
use cryfs_rustfs::AbsolutePath;
use cryfs_rustfs::PathComponent;

crate::perf_test_macro::perf_test!(
    symlink,
    [
        notexisting_from_rootdir,
        existing_from_rootdir,
        notexisting_from_nesteddir,
        existing_from_nesteddir,
        notexisting_from_deeplynesteddir,
        existing_from_deeplynesteddir,
        long_target,
    ]
);

fn notexisting_from_rootdir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async move |_fixture| {
            let target = AbsolutePath::try_from_str("/target/path").unwrap();
            target
        })
        .test(async move |fixture, target| {
            fixture
                .filesystem
                .create_symlink(
                    None,
                    PathComponent::try_from_str("symlink").unwrap(),
                    &target,
                )
                .await
                .unwrap();
        })
        .expect_op_counts(|_fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_create: 1,
                store_load: 1,
                blob_resize: 1,
                blob_read_all: 1,
                blob_read: 1,
                blob_write: 2,
                blob_flush: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: 2,
                blob_data: 18,
                blob_data_mut: 4,
                store_create: 1,
                store_flush_block: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                exists: 1,
                store: 2,
                load: 1,
                ..LLActionCounts::ZERO
            },
        })
}

fn existing_from_rootdir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async move |fixture| {
            let target = AbsolutePath::try_from_str("/target/path").unwrap();

            // First create it so that it already exists
            fixture
                .filesystem
                .create_symlink(
                    None,
                    PathComponent::try_from_str("existing").unwrap(),
                    &target,
                )
                .await
                .unwrap();

            target
        })
        .test(async move |fixture, target| {
            // Try creating it again with the same name
            fixture
                .filesystem
                .create_symlink(
                    None,
                    PathComponent::try_from_str("existing").unwrap(),
                    &target,
                )
                .await
                .unwrap_err();
        })
        .expect_op_counts(|_fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: 1,
                store_create: 1,
                store_remove_by_id: 1,
                blob_read_all: 1,
                blob_read: 1,
                blob_write: 1,
                blob_flush: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: 3,
                blob_data: 19,
                blob_data_mut: 2,
                store_create: 1,
                store_remove: 1,
                store_flush_block: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                exists: 1,
                load: 1,
                remove: 1,
                store: 1,
                ..LLActionCounts::ZERO
            },
        })
}

fn notexisting_from_nesteddir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async move |fixture| {
            let target = AbsolutePath::try_from_str("/target/path").unwrap();

            // First create the nested dir
            let parent = fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap();

            (target, parent)
        })
        .test(async move |fixture, (target, parent)| {
            fixture
                .filesystem
                .create_symlink(
                    Some(parent),
                    PathComponent::try_from_str("symlink").unwrap(),
                    &target,
                )
                .await
                .unwrap();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_create: 1,
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 3,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_resize: 2,
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
                blob_write: 3,
                blob_flush: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 29,
                    FixtureType::Fusemt => 38,
                    FixtureType::FuserWithoutInodeCache => 47, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 5,
                store_create: 1,
                store_flush_block: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                exists: 1,
                load: 2,
                store: 3,
                ..LLActionCounts::ZERO
            },
        })
}

fn existing_from_nesteddir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async move |fixture| {
            let target = AbsolutePath::try_from_str("/target/path").unwrap();

            // First create the nested dir
            let parent = fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap();

            // Then create the symlink so it's already existing
            fixture
                .filesystem
                .create_symlink(
                    Some(parent.clone()),
                    PathComponent::try_from_str("existing").unwrap(),
                    &target,
                )
                .await
                .unwrap();

            (target, parent)
        })
        .test(async move |fixture, (target, parent)| {
            // Try creating it again with the same name
            fixture
                .filesystem
                .create_symlink(
                    Some(parent),
                    PathComponent::try_from_str("existing").unwrap(),
                    &target,
                )
                .await
                .unwrap_err();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_create: 1,
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 3,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_remove_by_id: 1,
                blob_resize: 1,
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
                blob_write: 2,
                blob_flush: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 4,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 6, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 30,
                    FixtureType::Fusemt => 39,
                    FixtureType::FuserWithoutInodeCache => 48, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 3,
                store_create: 1,
                store_remove: 1,
                store_flush_block: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                exists: 1,
                load: 2,
                remove: 1,
                store: 2,
                ..LLActionCounts::ZERO
            },
        })
}

fn notexisting_from_deeplynesteddir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async move |fixture| {
            let target = AbsolutePath::try_from_str("/target/path").unwrap();

            // First create the nested dir
            let parent = fixture
                .filesystem
                .mkdir_recursive(AbsolutePath::try_from_str("/nested1/nested2/nested3").unwrap())
                .await
                .unwrap();

            (target, parent)
        })
        .test(async move |fixture, (target, parent)| {
            fixture
                .filesystem
                .create_symlink(
                    Some(parent),
                    PathComponent::try_from_str("symlink").unwrap(),
                    &target,
                )
                .await
                .unwrap();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_create: 1,
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 8, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_resize: 2,
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
                blob_write: 3,
                blob_flush: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 6,
                    FixtureType::FuserWithoutInodeCache => 9, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 29,
                    FixtureType::Fusemt => 56,
                    FixtureType::FuserWithoutInodeCache => 83, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 5,
                store_create: 1,
                store_flush_block: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                exists: 1,
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 4,
                },
                store: 3,
                ..LLActionCounts::ZERO
            },
        })
}

fn existing_from_deeplynesteddir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async move |fixture| {
            let target = AbsolutePath::try_from_str("/target/path").unwrap();

            // First create the nested dir
            let parent = fixture
                .filesystem
                .mkdir_recursive(AbsolutePath::try_from_str("/nested1/nested2/nested3").unwrap())
                .await
                .unwrap();

            // Then create the symlink so it's already existing
            fixture
                .filesystem
                .create_symlink(
                    Some(parent.clone()),
                    PathComponent::try_from_str("existing").unwrap(),
                    &target,
                )
                .await
                .unwrap();

            (target, parent)
        })
        .test(async move |fixture, (target, parent)| {
            // Try creating it again with the same name
            fixture
                .filesystem
                .create_symlink(
                    Some(parent),
                    PathComponent::try_from_str("existing").unwrap(),
                    &target,
                )
                .await
                .unwrap_err();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                blob_resize: 1,
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
                blob_write: 2,
                store_create: 1,
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 8, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_remove_by_id: 1,
                blob_flush: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 4,
                    FixtureType::Fusemt => 7,
                    FixtureType::FuserWithoutInodeCache => 10, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 30,
                    FixtureType::Fusemt => 57,
                    FixtureType::FuserWithoutInodeCache => 84, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 3,
                store_create: 1,
                store_remove: 1,
                store_flush_block: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                exists: 1,
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 4,
                },
                store: 2,
                remove: 1,
                ..LLActionCounts::ZERO
            },
        })
}

fn long_target(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async move |_fixture| {
            // no setup needed
        })
        .test(async move |fixture, ()| {
            // Create a very long target path that spans multiple blocks
            let long_target =
                "/very/long".repeat(NUM_BYTES_FOR_THREE_LEVEL_TREE as usize / 10) + "/target/path";
            let target = AbsolutePath::try_from_str(&long_target).unwrap();
            fixture
                .filesystem
                .create_symlink(
                    None,
                    PathComponent::try_from_str("longlink").unwrap(),
                    &target,
                )
                .await
                .unwrap();
        })
        .expect_op_counts(|_fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_create: 1,
                store_load: 1,
                blob_resize: 1,
                blob_read_all: 1,
                blob_read: 1,
                blob_write: 2,
                blob_flush: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: 27,
                blob_data: 154,
                blob_data_mut: 36,
                store_create: 26,
                store_flush_block: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                exists: 26,
                store: 27,
                load: 1,
                ..LLActionCounts::ZERO
            },
        })
}
