use crate::filesystem_driver::FilesystemDriver;
use crate::fixture::ActionCounts;
use crate::rstest::FixtureType;
use crate::test_driver::TestDriver;
use crate::test_driver::TestReady;
use cryfs_blobstore::BlobStoreActionCounts;
use cryfs_blockstore::HLActionCounts;
use cryfs_blockstore::LLActionCounts;
use cryfs_rustfs::AbsolutePath;
use cryfs_rustfs::PathComponent;

// only run fuser tests since fuse-mt doesn't have a lookup operation
crate::rstest::perf_test_only_fuser!(
    lookup,
    [
        existing_from_rootdir,
        notexisting_from_rootdir,
        existing_from_nesteddir,
        notexisting_from_nesteddir,
        existing_from_deeplynesteddir,
        notexisting_from_deeplynesteddir,
    ]
);

fn existing_from_rootdir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create a file so that it exists
            fixture
                .filesystem
                .create_file(None, PathComponent::try_from_str("existing.txt").unwrap())
                .await
                .unwrap()
        })
        .test(async |fixture, file| {
            fixture
                .filesystem
                .lookup(None, PathComponent::try_from_str("existing.txt").unwrap())
                .await
                .unwrap();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: 2,
                blob_read_all: 1,
                blob_read: 2,
                blob_num_bytes: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: 2,
                blob_data: 16,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 2,
                ..LLActionCounts::ZERO
            },
        })
}

fn notexisting_from_rootdir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // No setup needed for non-existing file
        })
        .test(async |fixture, ()| {
            fixture
                .filesystem
                .lookup(
                    None,
                    PathComponent::try_from_str("notexisting.txt").unwrap(),
                )
                .await
                .unwrap_err();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
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
                load: 1, // TODO What are we loading here? The root dir?
                ..LLActionCounts::ZERO
            },
        })
}

fn existing_from_nesteddir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create the nested dir
            let parent = fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap();
            // Create a file inside the nested dir
            fixture
                .filesystem
                .create_file(
                    Some(parent.clone()),
                    PathComponent::try_from_str("existing.txt").unwrap(),
                )
                .await
                .unwrap();
            parent
        })
        .test(async |fixture, parent| {
            fixture
                .filesystem
                .lookup(
                    Some(parent),
                    PathComponent::try_from_str("existing.txt").unwrap(),
                )
                .await
                .unwrap();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 4,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::FuserWithoutInodeCache => 3,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 4,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                blob_num_bytes: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 4,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 16,
                    FixtureType::FuserWithoutInodeCache => 34,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 3,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                ..LLActionCounts::ZERO
            },
        })
}

fn notexisting_from_nesteddir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create the nested dir
            fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap()
        })
        .test(async |fixture, parent| {
            fixture
                .filesystem
                .lookup(
                    Some(parent),
                    PathComponent::try_from_str("notexisting.txt").unwrap(),
                )
                .await
                .unwrap_err();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 3,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 3,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 3,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 3,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 18,
                    FixtureType::FuserWithoutInodeCache => 27,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 2,
                ..LLActionCounts::ZERO
            },
        })
}

fn existing_from_deeplynesteddir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create the deeply nested dir
            let parent = fixture
                .filesystem
                .mkdir_recursive(AbsolutePath::try_from_str("/nested1/nested2/nested3").unwrap())
                .await
                .unwrap();
            // Create a file inside the deeply nested dir
            fixture
                .filesystem
                .create_file(
                    Some(parent.clone()),
                    PathComponent::try_from_str("existing.txt").unwrap(),
                )
                .await
                .unwrap();
            parent
        })
        .test(async |fixture, parent| {
            fixture
                .filesystem
                .lookup(
                    Some(parent),
                    PathComponent::try_from_str("existing.txt").unwrap(),
                )
                .await
                .unwrap();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 8,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::FuserWithoutInodeCache => 7,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 8,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                blob_num_bytes: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 8,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 16,
                    FixtureType::FuserWithoutInodeCache => 70,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 5,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                ..LLActionCounts::ZERO
            },
        })
}

fn notexisting_from_deeplynesteddir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create the deeply nested dir
            fixture
                .filesystem
                .mkdir_recursive(AbsolutePath::try_from_str("/nested1/nested2/nested3").unwrap())
                .await
                .unwrap()
        })
        .test(async |fixture, parent| {
            fixture
                .filesystem
                .lookup(
                    Some(parent),
                    PathComponent::try_from_str("notexisting.txt").unwrap(),
                )
                .await
                .unwrap_err();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 7,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 7,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 7,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 7,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 18,
                    FixtureType::FuserWithoutInodeCache => 63,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 4,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                ..LLActionCounts::ZERO
            },
        })
}
