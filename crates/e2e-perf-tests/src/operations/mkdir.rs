use crate::filesystem_driver::FilesystemDriver;
use crate::filesystem_fixture::ActionCounts;
use crate::perf_test_macro::FixtureType;
use crate::test_driver::TestDriver;
use crate::test_driver::TestReady;
use cryfs_blobstore::BlobStoreActionCounts;
use cryfs_blockstore::HLActionCounts;
use cryfs_blockstore::LLActionCounts;
use cryfs_rustfs::AbsolutePath;
use cryfs_rustfs::PathComponent;

crate::perf_test_macro::perf_test!(
    mkdir,
    [
        notexisting_from_rootdir,
        existing_from_rootdir,
        notexisting_from_nesteddir,
        existing_from_nesteddir,
        notexisting_from_deeplynesteddir,
        existing_from_deeplynesteddir,
    ]
);

fn notexisting_from_rootdir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |_fixture| {
            // No setup needed
        })
        .test(async |fixture, ()| {
            fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("notexisting").unwrap())
                .await
                .unwrap();
        })
        .expect_op_counts(|_fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                store_create: 1,  // create new dir blob
                store_load: 1,    // load root dir blob
                blob_resize: 1,   // add new entry to root dir blob
                blob_read_all: 1, // deserialize root dir blob
                blob_read: 1,     // read header of root dir blob
                blob_write: 2,    // write to new dir blob + add hew entry to root dir blob
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: 2,
                blob_data: 18,
                blob_data_mut: 4,
                store_create: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                exists: 1, // Check if a blob with the new blob id already exists before creating it.
                store: 2,  // Create new directory blob and add an entry for it to the root blob.
                load: 1,   // TODO What are we loading here? The root dir?
                ..LLActionCounts::ZERO
            },
        })
}

fn existing_from_rootdir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create it so that it already exists
            fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("existing").unwrap())
                .await
                .unwrap()
        })
        .test(async |fixture, _dir| {
            fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("existing").unwrap())
                .await
                .unwrap_err();
        })
        .expect_op_counts(|_fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                store_load: 1,         // load root dir blob
                store_create: 1,       // create new dir blob
                store_remove_by_id: 1, // remove new dir blob after we notice that we can't add it to the root dir because it already exists
                blob_read_all: 1,      // deserialize root dir blob
                blob_read: 1,          // read header of root dir blob
                blob_write: 1,         // write to new dir blob
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: 3,
                blob_data: 19,
                blob_data_mut: 2,
                store_create: 1,
                store_remove: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                exists: 1, // Check if a blob with the new blob id already exists before creating it.
                load: 1, // TODO What are we loading here? The root dir should already be cached in the device.
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
                .mkdir(
                    Some(parent),
                    PathComponent::try_from_str("notexisting").unwrap(),
                )
                .await
                .unwrap();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_create: 1,
                store_load: match fixture_type {
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 3,
                },
                blob_resize: 2,
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 3,
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 3,
                },
                blob_write: 3,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 4,
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithoutInodeCache => 47, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 38,
                },
                blob_data_mut: 5,
                store_create: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                exists: 1, // Check if a blob with the new blob id already exists before creating it.
                load: 2, // TODO Shouldn't we only have to load one less? Root blob is already cached in the device.
                store: 3, // Create new directory blob and add an entry for it to the parent dir and update parent dir timestamps in the root blob.
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
            // Then create the dir so it's already existing
            fixture
                .filesystem
                .mkdir(
                    Some(parent.clone()),
                    PathComponent::try_from_str("existing").unwrap(),
                )
                .await
                .unwrap();
            parent
        })
        .test(async |fixture, parent| {
            fixture
                .filesystem
                .mkdir(
                    Some(parent),
                    PathComponent::try_from_str("existing").unwrap(),
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
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                exists: 1, // Check if a blob with the new blob id already exists before creating it.
                load: 2, // TODO Shouldn't we only have to load one less? Root blob is already cached in the device.
                store: 1, // TODO What are we storing here? We didn't make any changes.
                ..LLActionCounts::ZERO
            },
        })
}

fn notexisting_from_deeplynesteddir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create the nested dir
            fixture
                .filesystem
                .mkdir_recursive(AbsolutePath::try_from_str("/nested1/nested2/nested3").unwrap())
                .await
                .unwrap()
        })
        .test(async |fixture, parent| {
            fixture
                .filesystem
                .mkdir(
                    Some(parent),
                    PathComponent::try_from_str("notexisting").unwrap(),
                )
                .await
                .unwrap();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_create: 1,
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 8, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_resize: 2,
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 8, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 8, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: 3,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 4,
                    FixtureType::Fusemt => 6,
                    FixtureType::FuserWithoutInodeCache => 9, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 38,
                    FixtureType::Fusemt => 56,
                    FixtureType::FuserWithoutInodeCache => 83, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 5,
                store_create: 1,
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
        .setup(async |fixture| {
            // First create the nested dir
            let parent = fixture
                .filesystem
                .mkdir_recursive(AbsolutePath::try_from_str("/nested1/nested2/nested3").unwrap())
                .await
                .unwrap();
            // Then create the dir so it's already existing
            fixture
                .filesystem
                .mkdir(
                    Some(parent.clone()),
                    PathComponent::try_from_str("existing").unwrap(),
                )
                .await
                .unwrap();
            parent
        })
        .test(async |fixture, parent| {
            fixture
                .filesystem
                .mkdir(
                    Some(parent),
                    PathComponent::try_from_str("existing").unwrap(),
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
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                exists: 1,
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 4,
                },
                store: 1,
                ..LLActionCounts::ZERO
            },
        })
}
