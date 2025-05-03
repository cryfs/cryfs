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
            fs.create_and_open_file(AbsolutePath::try_from_str("/newfile.txt").unwrap())
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
                exists: 1,
                store: 2,
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
        .run_operation(async |fs| {
            fs.create_and_open_file(AbsolutePath::try_from_str("/existing.txt").unwrap())
                .await
                .unwrap();
        })
        .await;

    let counts = fixture
        .run_operation(async |fs| {
            fs.create_and_open_file(AbsolutePath::try_from_str("/existing.txt").unwrap())
                .await
                .unwrap_err();
        })
        .await;
    assert_eq!(
        counts,
        ActionCounts {
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
            fs.create_and_open_file(AbsolutePath::try_from_str("/nested/newfile.txt").unwrap())
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
                    FixtureType::Fuser => 4, // TODO Why more than Fusemt?
                    FixtureType::Fusemt => 3,
                },
                blob_resize: 2,
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 4, // TODO Why more than Fusemt?
                    FixtureType::Fusemt => 3,
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 4, // TODO Why more than Fusemt?
                    FixtureType::Fusemt => 3,
                },
                blob_write: 3,
                blob_flush: 1,
                ..BlobStoreActionCounts::ZERO
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

    // Then create the file so it's already existing
    fixture
        .run_operation(async |fs| {
            fs.create_and_open_file(AbsolutePath::try_from_str("/nested/existing.txt").unwrap())
                .await
                .unwrap();
        })
        .await;

    let counts = fixture
        .run_operation(async |fs| {
            fs.create_and_open_file(AbsolutePath::try_from_str("/nested/existing.txt").unwrap())
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
                    FixtureType::Fuser => 4, // TODO Why one more than Fusemt?
                    FixtureType::Fusemt => 3,
                },
                store_remove_by_id: 1,
                blob_resize: 1,
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 4, // TODO Why one more than Fusemt?
                    FixtureType::Fusemt => 3,
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 4, // TODO Why one more than Fusemt?
                    FixtureType::Fusemt => 3,
                },
                blob_write: 2,
                blob_flush: 1,
                ..BlobStoreActionCounts::ZERO
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
            fs.mkdir_recursive(AbsolutePath::try_from_str("/nested1/nested2/nested3").unwrap())
                .await
                .unwrap();
        })
        .await;

    let counts = fixture
        .run_operation(async |fs| {
            fs.create_and_open_file(
                AbsolutePath::try_from_str("/nested1/nested2/nested3/newfile.txt").unwrap(),
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
                    FixtureType::Fuser => 8, // TODO Why more than Fusemt?
                    FixtureType::Fusemt => 5,
                },
                blob_resize: 2,
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 8, // TODO Why more than Fusemt?
                    FixtureType::Fusemt => 5,
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 8, // TODO Why more than Fusemt?
                    FixtureType::Fusemt => 5,
                },
                blob_write: 3,
                blob_flush: 1,
                ..BlobStoreActionCounts::ZERO
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
                store_flush_block: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                exists: 1,
                load: 4,
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
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create the nested dir
    fixture
        .run_operation(async |fs| {
            fs.mkdir_recursive(AbsolutePath::try_from_str("/nested1/nested2/nested3").unwrap())
                .await
                .unwrap();
        })
        .await;

    // Then create the file so it's already existing
    fixture
        .run_operation(async |fs| {
            fs.create_and_open_file(
                AbsolutePath::try_from_str("/nested1/nested2/nested3/existing.txt").unwrap(),
            )
            .await
            .unwrap();
        })
        .await;

    let counts = fixture
        .run_operation(async |fs| {
            fs.create_and_open_file(
                AbsolutePath::try_from_str("/nested1/nested2/nested3/existing.txt").unwrap(),
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
                    FixtureType::Fuser => 8, // TODO Why more than Fusemt?
                    FixtureType::Fusemt => 5,
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 8, // TODO Why more than Fusemt?
                    FixtureType::Fusemt => 5,
                },
                blob_write: 2,
                store_create: 1,
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::Fuser => 8, // TODO Why more than Fusemt?
                    FixtureType::Fusemt => 5,
                },
                store_remove_by_id: 1,
                blob_flush: 1,
                ..BlobStoreActionCounts::ZERO
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
                store_flush_block: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                exists: 1,
                load: 4,
                store: 2,
                remove: 1,
                ..LLActionCounts::ZERO
            },
        }
    );
}
