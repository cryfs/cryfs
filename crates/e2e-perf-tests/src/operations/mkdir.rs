use pretty_assertions::assert_eq;
use rstest::rstest;
use rstest_reuse::apply;

use crate::filesystem_test_ext::FilesystemTestExt as _;
use crate::fixture::ActionCounts;
use crate::rstest::FixtureFactory;
use crate::rstest::FixtureType;
use crate::rstest::{all_atime_behaviors, all_fixtures};
use cryfs_blobstore::BlobStoreActionCounts;
use cryfs_blockstore::HLActionCounts;
use cryfs_blockstore::LLActionCounts;
use cryfs_rustfs::AbsolutePath;
use cryfs_rustfs::AtimeUpdateBehavior;
use cryfs_rustfs::PathComponent;

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn notexisting_from_rootdir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.mkdir(None, PathComponent::try_from_str("notexisting").unwrap())
                .await
                .unwrap();
        })
        .await;
    assert_eq!(
        counts,
        ActionCounts {
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
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn existing_from_rootdir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create it so that it already exists
    fixture
        .ops(async |fs| {
            fs.mkdir(None, PathComponent::try_from_str("existing").unwrap())
                .await
                .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.mkdir(None, PathComponent::try_from_str("existing").unwrap())
                .await
                .unwrap_err();
        })
        .await;
    assert_eq!(
        counts,
        ActionCounts {
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
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn notexisting_from_nesteddir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create the nested dir
    let parent = fixture
        .ops(async |fs| {
            fs.mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap()
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.mkdir(
                Some(parent),
                PathComponent::try_from_str("notexisting").unwrap(),
            )
            .await
            .unwrap();
        })
        .await;
    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_create: 1,
                store_load: 3,
                blob_resize: 2,
                blob_read_all: 3,
                blob_read: 3,
                blob_write: 3,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: 4,
                blob_data: 38,
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
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn existing_from_nesteddir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create the nested dir
    let parent = fixture
        .ops(async |fs| {
            fs.mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap()
        })
        .await;

    // Then create the dir so it's already existing
    fixture
        // TODO Combine with mkdir_recursive above
        .ops(async |fs| {
            fs.mkdir(
                Some(parent.clone()),
                PathComponent::try_from_str("existing").unwrap(),
            )
            .await
            .unwrap()
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.mkdir(
                Some(parent),
                PathComponent::try_from_str("existing").unwrap(),
            )
            .await
            .unwrap_err();
        })
        .await;
    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_create: 1,
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 2,
                    FixtureType::Fusemt => 3,
                },
                store_remove_by_id: 1,
                blob_resize: 1,
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 2,
                    FixtureType::Fusemt => 3,
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 2,
                    FixtureType::Fusemt => 3,
                },
                blob_write: 2,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 4,
                    FixtureType::Fusemt => 5,
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 30,
                    FixtureType::Fusemt => 39,
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
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn notexisting_from_deeplynesteddir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create the nested dir
    let parent = fixture
        .ops(async |fs| {
            fs.mkdir_recursive(AbsolutePath::try_from_str("/nested1/nested2/nested3").unwrap())
                .await
                .unwrap()
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.mkdir(
                Some(parent),
                PathComponent::try_from_str("notexisting").unwrap(),
            )
            .await
            .unwrap();
        })
        .await;
    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_create: 1,
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 3,
                    FixtureType::Fusemt => 5,
                },
                blob_resize: 2,
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 3,
                    FixtureType::Fusemt => 5,
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 3,
                    FixtureType::Fusemt => 5,
                },
                blob_write: 3,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 4,
                    FixtureType::Fusemt => 6,
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 38,
                    FixtureType::Fusemt => 56,
                },
                blob_data_mut: 5,
                store_create: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                exists: 1,
                load: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 2,
                    FixtureType::Fusemt => 4,
                },
                store: 3,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn existing_from_deeplynesteddir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    use cryfs_rustfs::PathComponent;

    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create the nested dir
    let parent = fixture
        .ops(async |fs| {
            fs.mkdir_recursive(AbsolutePath::try_from_str("/nested1/nested2/nested3").unwrap())
                .await
                .unwrap()
        })
        .await;

    // Then create the dir so it's already existing
    fixture
        // TODO Combine with mkdir_recursive above
        .ops(async |fs| {
            fs.mkdir(
                Some(parent.clone()),
                PathComponent::try_from_str("existing").unwrap(),
            )
            .await
            .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.mkdir(
                Some(parent),
                PathComponent::try_from_str("existing").unwrap(),
            )
            .await
            .unwrap_err();
        })
        .await;
    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                blob_resize: 1,
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 2,
                    FixtureType::Fusemt => 5,
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 2,
                    FixtureType::Fusemt => 5,
                },
                blob_write: 2,
                store_create: 1,
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 2,
                    FixtureType::Fusemt => 5,
                },
                store_remove_by_id: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 4,
                    FixtureType::Fusemt => 7,
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 30,
                    FixtureType::Fusemt => 57,
                },
                blob_data_mut: 3,
                store_create: 1,
                store_remove: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                exists: 1,
                load: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 2,
                    FixtureType::Fusemt => 4,
                },
                store: 1,
                ..LLActionCounts::ZERO
            },
        }
    );
}
