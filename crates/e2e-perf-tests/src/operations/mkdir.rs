use pretty_assertions::assert_eq;
use rstest::rstest;
use rstest_reuse::apply;

use crate::filesystem_test_ext::FilesystemTestExt as _;
use crate::fixture::ActionCounts;
use crate::rstest::FixtureFactory;
use crate::rstest::FixtureType;
use crate::rstest::{all_atime_behaviors, all_fixtures};
use cryfs_blockstore::HLActionCounts;
use cryfs_blockstore::LLActionCounts;
use cryfs_rustfs::AbsolutePath;
use cryfs_rustfs::AtimeUpdateBehavior;

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
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/notexisting").unwrap())
                .await
                .unwrap();
        })
        .await;
    assert_eq!(
        counts,
        ActionCounts {
            low_level: LLActionCounts {
                exists: 1, // Check if a blob with the new blob id already exists before creating it.
                store: 2,  // Create new directory blob and add an entry for it to the root blob.
                ..LLActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: 2,
                blob_data: 18,
                blob_data_mut: 4,
                store_create: 1,
                ..HLActionCounts::ZERO
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
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/existing").unwrap())
                .await
                .unwrap();
        })
        .await;

    let counts = fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/existing").unwrap())
                .await
                .unwrap_err();
        })
        .await;
    assert_eq!(
        counts,
        ActionCounts {
            low_level: LLActionCounts {
                exists: 1, // Check if a blob with the new blob id already exists before creating it.
                load: 1, // TODO What are we loading here? The root dir should already be cached in the device.
                ..LLActionCounts::ZERO
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
    fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested").unwrap())
                .await
                .unwrap();
        })
        .await;

    let counts = fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested/notexisting").unwrap())
                .await
                .unwrap();
        })
        .await;
    assert_eq!(
        counts,
        ActionCounts {
            low_level: LLActionCounts {
                exists: 1, // Check if a blob with the new blob id already exists before creating it.
                load: 2, // TODO Shouldn't we only have to load one less? Root blob is already cached in the device.
                store: 3, // Create new directory blob and add an entry for it to the parent dir and update parent dir timestamps in the root blob.
                ..LLActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 5, // TODO Why one more than Fusemt?
                    FixtureType::Fusemt => 4,
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 47, // TODO Why more than Fusemt?
                    FixtureType::Fusemt => 38,
                },
                blob_data_mut: 5,
                store_create: 1,
                ..HLActionCounts::ZERO
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
    fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested").unwrap())
                .await
                .unwrap();
        })
        .await;

    // Then create the dir so it's already existing
    fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested/existing").unwrap())
                .await
                .unwrap();
        })
        .await;

    let counts = fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested/existing").unwrap())
                .await
                .unwrap_err();
        })
        .await;
    assert_eq!(
        counts,
        ActionCounts {
            low_level: LLActionCounts {
                exists: 1, // Check if a blob with the new blob id already exists before creating it.
                load: 2, // TODO Shouldn't we only have to load one less? Root blob is already cached in the device.
                store: 1, // TODO What are we storing here? We didn't make any changes.
                ..LLActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 6, // TODO Why one more than Fusemt?
                    FixtureType::Fusemt => 5,
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 48, // TODO Why more than Fusemt?
                    FixtureType::Fusemt => 39,
                },
                blob_data_mut: 3,
                store_create: 1,
                store_remove: 1,
                ..HLActionCounts::ZERO
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
    fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested1").unwrap())
                .await
                .unwrap();
        })
        .await;
    fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested1/nested2").unwrap())
                .await
                .unwrap();
        })
        .await;
    fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested1/nested2/nested3").unwrap())
                .await
                .unwrap();
        })
        .await;

    let counts = fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested1/nested2/nested3/notexisting").unwrap())
                .await
                .unwrap();
        })
        .await;
    assert_eq!(
        counts,
        ActionCounts {
            low_level: LLActionCounts {
                exists: 1, // Check if a blob with the new blob id already exists before creating it.
                load: 4, // TODO Shouldn't we only have to load one less? Root blob is already cached in the device.
                store: 3, // Create new directory blob and add an entry for it to the parent dir and update parent dir timestamps in the root blob.
                ..LLActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 9, // TODO Why more than Fusemt?
                    FixtureType::Fusemt => 6,
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 83, // TODO Why more than Fusemt?
                    FixtureType::Fusemt => 56,
                },
                blob_data_mut: 5,
                store_create: 1,
                ..HLActionCounts::ZERO
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
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create the nested dir
    fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested1").unwrap())
                .await
                .unwrap();
        })
        .await;
    fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested1/nested2").unwrap())
                .await
                .unwrap();
        })
        .await;
    fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested1/nested2/nested3").unwrap())
                .await
                .unwrap();
        })
        .await;

    // Then create the dir so it's already existing
    fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested1/nested2/nested3/existing").unwrap())
                .await
                .unwrap();
        })
        .await;

    let counts = fixture
        .run_operation(async |fs| {
            fs.mkdir(AbsolutePath::try_from_str("/nested1/nested2/nested3/existing").unwrap())
                .await
                .unwrap_err();
        })
        .await;
    assert_eq!(
        counts,
        ActionCounts {
            low_level: LLActionCounts {
                exists: 1, // Check if a blob with the new blob id already exists before creating it.
                load: 4, // TODO Shouldn't we only have to load one less? Root blob is already cached in the device.
                store: 1, // TODO What are we storing here? We didn't make any changes.
                ..LLActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 10, // TODO Why more than Fusemt?
                    FixtureType::Fusemt => 7,
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 84, // TODO Why more than Fusemt?
                    FixtureType::Fusemt => 57,
                },
                blob_data_mut: 3,
                store_create: 1,
                store_remove: 1,
                ..HLActionCounts::ZERO
            },
        }
    );
}
