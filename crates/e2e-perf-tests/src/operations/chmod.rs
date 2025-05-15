use pretty_assertions::assert_eq;
use rstest::rstest;
use rstest_reuse::apply;

use crate::filesystem_driver::FilesystemDriver as _;
use crate::fixture::ActionCounts;
use crate::rstest::FixtureFactory;
use crate::rstest::FixtureType;
use crate::rstest::{all_atime_behaviors, all_fixtures};
use cryfs_blobstore::BlobStoreActionCounts;
use cryfs_blockstore::HLActionCounts;
use cryfs_blockstore::LLActionCounts;
use cryfs_rustfs::AbsolutePath;
use cryfs_rustfs::AtimeUpdateBehavior;
use cryfs_rustfs::Mode;
use cryfs_rustfs::PathComponent;

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn chmod_file_in_rootdir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create a file so we have something to change permissions for
    let file = fixture
        .ops(async |fs| {
            fs.create_file(None, PathComponent::try_from_str("testfile.txt").unwrap())
                .await
                .unwrap()
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.chmod(Some(file), Mode::from(0o644).add_file_flag())
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
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: 1,
                blob_resize: 1,
                blob_num_bytes: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 18,
                    FixtureType::FuserWithoutInodeCache => 34, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 2,
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
async fn chmod_dir_in_rootdir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create a directory so we have something to change permissions for
    let dir = fixture
        .ops(async |fs| {
            fs.mkdir(None, PathComponent::try_from_str("testdir").unwrap())
                .await
                .unwrap()
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.chmod(Some(dir), Mode::from(0o755).add_dir_flag())
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
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: 1,
                blob_resize: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 20,
                    FixtureType::FuserWithoutInodeCache => 38, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 2,
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
async fn chmod_symlink_in_rootdir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create a symlink so we have something to change permissions for
    let symlink = fixture
        .ops(async |fs| {
            fs.create_symlink(
                None,
                PathComponent::try_from_str("link").unwrap(),
                AbsolutePath::try_from_str("/target/file.txt").unwrap(),
            )
            .await
            .unwrap()
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.chmod(Some(symlink), Mode::from(0o777).add_symlink_flag())
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
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: 1,
                blob_resize: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 20,
                    FixtureType::FuserWithoutInodeCache => 38, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 2,
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
async fn chmod_file_in_nesteddir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create a nested directory and a file in it
    let file = fixture
        .ops(async |fs| {
            let parent = fs
                .mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap();
            fs.create_file(
                Some(parent),
                PathComponent::try_from_str("testfile.txt").unwrap(),
            )
            .await
            .unwrap()
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.chmod(Some(file), Mode::from(0o644).add_file_flag())
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
                    FixtureType::Fusemt => 3,
                    FixtureType::FuserWithoutInodeCache => 6, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 2,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 3,
                    FixtureType::FuserWithoutInodeCache => 6, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: 1,
                blob_resize: 1,
                blob_num_bytes: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 3,
                    FixtureType::FuserWithoutInodeCache => 6, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 18,
                    FixtureType::Fusemt => 27,
                    FixtureType::FuserWithoutInodeCache => 52, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 3,
                },
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
async fn chmod_file_in_deeplynesteddir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory.create_filesystem(atime_behavior).await;

    // First create a deeply nested directory
    let nested_dir = fixture
        .ops(async |fs| {
            fs.mkdir_recursive(AbsolutePath::try_from_str("/nested1/nested2/nested3").unwrap())
                .await
                .unwrap()
        })
        .await;

    // Then create a file in that directory
    let file = fixture
        .ops(async |fs| {
            fs.create_file(
                Some(nested_dir.clone()),
                PathComponent::try_from_str("testfile.txt").unwrap(),
            )
            .await
            .unwrap()
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.chmod(Some(file), Mode::from(0o644).add_file_flag())
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
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 10, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 1,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 8, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 10, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_resize: 1,
                blob_num_bytes: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 1,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 10, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 18,
                    FixtureType::Fusemt => 45,
                    FixtureType::FuserWithoutInodeCache => 88, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 5,
                },
                store: 1,
                ..LLActionCounts::ZERO
            },
        }
    );
}
