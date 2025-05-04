use pretty_assertions::assert_eq;
use rstest::rstest;
use rstest_reuse::apply;

use crate::filesystem_driver::FilesystemDriver as _;
use crate::fixture::ActionCounts;
use crate::rstest::FixtureFactory;
use crate::rstest::FixtureType;
use crate::rstest::{all_atime_behaviors, all_fuser_fixtures};
use cryfs_blobstore::BlobStoreActionCounts;
use cryfs_blockstore::HLActionCounts;
use cryfs_blockstore::LLActionCounts;
use cryfs_rustfs::AbsolutePath;
use cryfs_rustfs::AtimeUpdateBehavior;
use cryfs_rustfs::PathComponent;

#[apply(all_fuser_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn existing_from_rootdir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create a file so that it exists
    fixture
        .ops(async |fs| {
            fs.create_file(None, PathComponent::try_from_str("existing.txt").unwrap())
                .await
                .unwrap()
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.lookup(None, PathComponent::try_from_str("existing.txt").unwrap())
                .await
                .unwrap();
        })
        .await;
    assert_eq!(
        counts,
        ActionCounts {
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
        }
    );
}

#[apply(all_fuser_fixtures)]
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
            fs.lookup(
                None,
                PathComponent::try_from_str("notexisting.txt").unwrap(),
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
        }
    );
}

#[apply(all_fuser_fixtures)]
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

    // Create a file inside the nested dir
    fixture
        .ops(async |fs| {
            fs.create_file(
                Some(parent.clone()),
                PathComponent::try_from_str("existing.txt").unwrap(),
            )
            .await
            .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.lookup(
                Some(parent),
                PathComponent::try_from_str("existing.txt").unwrap(),
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
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 4,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::FuserWithoutInodeCache => 3,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                blob_read: match fixture_factory.fixture_type() {
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
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 4,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                blob_data: match fixture_factory.fixture_type() {
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
                load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 3,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fuser_fixtures)]
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
            fs.lookup(
                Some(parent),
                PathComponent::try_from_str("notexisting.txt").unwrap(),
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
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 3,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 3,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                blob_read: match fixture_factory.fixture_type() {
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
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 3,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                blob_data: match fixture_factory.fixture_type() {
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
        }
    );
}

#[apply(all_fuser_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn existing_from_deeplynesteddir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create the deeply nested dir
    let parent = fixture
        .ops(async |fs| {
            fs.mkdir_recursive(AbsolutePath::try_from_str("/nested1/nested2/nested3").unwrap())
                .await
                .unwrap()
        })
        .await;

    // Create a file inside the deeply nested dir
    fixture
        .ops(async |fs| {
            fs.create_file(
                Some(parent.clone()),
                PathComponent::try_from_str("existing.txt").unwrap(),
            )
            .await
            .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.lookup(
                Some(parent),
                PathComponent::try_from_str("existing.txt").unwrap(),
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
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 8,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::FuserWithoutInodeCache => 7,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                blob_read: match fixture_factory.fixture_type() {
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
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 8,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                blob_data: match fixture_factory.fixture_type() {
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
                load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 5,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fuser_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn notexisting_from_deeplynesteddir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create the deeply nested dir
    let parent = fixture
        .ops(async |fs| {
            fs.mkdir_recursive(AbsolutePath::try_from_str("/nested1/nested2/nested3").unwrap())
                .await
                .unwrap()
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.lookup(
                Some(parent),
                PathComponent::try_from_str("notexisting.txt").unwrap(),
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
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 7,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 7,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                blob_read: match fixture_factory.fixture_type() {
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
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 7,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                blob_data: match fixture_factory.fixture_type() {
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
                load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::FuserWithoutInodeCache => 4,
                    FixtureType::Fusemt => unreachable!(
                        "Fusemt isn't enabled for this test because it doesn't have a lookup operation"
                    ),
                },
                ..LLActionCounts::ZERO
            },
        }
    );
}
