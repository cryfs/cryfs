use cryfs_blockstore::BLOCKID_LEN;
use pretty_assertions::assert_eq;
use rstest::rstest;
use rstest_reuse::apply;

use crate::filesystem_driver::FilesystemDriver as _;
use crate::fixture::ActionCounts;
use crate::fixture::NUM_BYTES_FOR_THREE_LEVEL_TREE;
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
async fn readdir_empty_rootdir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.readdir(None).await.unwrap();
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
                load: 1,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn readdir_rootdir_with_entries(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    // First create some entries in the root directory
    fixture
        .ops(async |fs| {
            fs.mkdir(None, PathComponent::try_from_str("dir1").unwrap())
                .await
                .unwrap();
            fs.mkdir(None, PathComponent::try_from_str("dir2").unwrap())
                .await
                .unwrap();
            fs.create_file(None, PathComponent::try_from_str("file1").unwrap())
                .await
                .unwrap();
            fs.create_file(None, PathComponent::try_from_str("file2").unwrap())
                .await
                .unwrap();
            fs.create_symlink(
                None,
                PathComponent::try_from_str("link1").unwrap(),
                AbsolutePath::try_from_str("/target/path").unwrap(),
            )
            .await
            .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.readdir(None).await.unwrap();
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
                store_load: 5,
                blob_data: 38,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 3,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn readdir_nesteddir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    // First create a nested directory with some entries
    let nested_dir = fixture
        .ops(async |fs| {
            let dir = fs
                .mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap();
            fs.mkdir(
                Some(dir.clone()),
                PathComponent::try_from_str("dir1").unwrap(),
            )
            .await
            .unwrap();
            fs.create_file(
                Some(dir.clone()),
                PathComponent::try_from_str("file1").unwrap(),
            )
            .await
            .unwrap();
            fs.create_symlink(
                Some(dir.clone()),
                PathComponent::try_from_str("link1").unwrap(),
                AbsolutePath::try_from_str("/target/path").unwrap(),
            )
            .await
            .unwrap();

            dir
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.readdir(Some(nested_dir)).await.unwrap();
        })
        .await;

    let expect_atime_update = match atime_behavior {
        AtimeUpdateBehavior::Noatime
        | AtimeUpdateBehavior::NodiratimeRelatime
        | AtimeUpdateBehavior::NodiratimeStrictatime => 0,
        AtimeUpdateBehavior::Relatime | AtimeUpdateBehavior::Strictatime => 1,
    };

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 3,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 3,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 3,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: expect_atime_update,
                blob_resize: expect_atime_update,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 3,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 18,
                    FixtureType::Fusemt => 27,
                    FixtureType::FuserWithoutInodeCache => 36, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                } + 2 * expect_atime_update,
                blob_data_mut: expect_atime_update,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 2,
                store: expect_atime_update,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn readdir_deeplynesteddir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    // First create a deeply nested directory with some entries
    let deeply_nested_dir = fixture
        .ops(async |fs| {
            let dir = fs
                .mkdir_recursive(AbsolutePath::try_from_str("/deep/nested/dir").unwrap())
                .await
                .unwrap();
            fs.mkdir(
                Some(dir.clone()),
                PathComponent::try_from_str("dir1").unwrap(),
            )
            .await
            .unwrap();
            fs.create_file(
                Some(dir.clone()),
                PathComponent::try_from_str("file1").unwrap(),
            )
            .await
            .unwrap();
            fs.create_symlink(
                Some(dir.clone()),
                PathComponent::try_from_str("link1").unwrap(),
                AbsolutePath::try_from_str("/target/path").unwrap(),
            )
            .await
            .unwrap();

            dir
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.readdir(Some(deeply_nested_dir)).await.unwrap();
        })
        .await;

    let expect_atime_update = match atime_behavior {
        AtimeUpdateBehavior::Noatime
        | AtimeUpdateBehavior::NodiratimeRelatime
        | AtimeUpdateBehavior::NodiratimeStrictatime => 0,
        AtimeUpdateBehavior::Relatime | AtimeUpdateBehavior::Strictatime => 1,
    };

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 8, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 8, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 8, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: expect_atime_update,
                blob_resize: expect_atime_update,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 8, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 18,
                    FixtureType::Fusemt => 45,
                    FixtureType::FuserWithoutInodeCache => 72, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                } + 2 * expect_atime_update,
                blob_data_mut: expect_atime_update,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 4,
                },
                store: expect_atime_update,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn readdir_large_directory(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    // Create a directory with many entries to test readdir performance with large directories
    let large_dir = fixture
        .ops(async |fs| {
            let dir = fs
                .mkdir(None, PathComponent::try_from_str("large_dir").unwrap())
                .await
                .unwrap();

            // Create many entries that will definitely require multiple blocks to store
            let num_entries = NUM_BYTES_FOR_THREE_LEVEL_TREE / BLOCKID_LEN as u64;
            for i in 0..num_entries {
                fs.create_file(
                    Some(dir.clone()),
                    PathComponent::try_from_str(&format!("file{}", i)).unwrap(),
                )
                .await
                .unwrap();
            }

            dir
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.readdir(Some(large_dir)).await.unwrap();
        })
        .await;

    let expect_atime_update = match atime_behavior {
        AtimeUpdateBehavior::Noatime
        | AtimeUpdateBehavior::NodiratimeRelatime
        | AtimeUpdateBehavior::NodiratimeStrictatime => 0,
        AtimeUpdateBehavior::Relatime | AtimeUpdateBehavior::Strictatime => 1,
    };

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 3,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 3,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt => 3,
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: expect_atime_update,
                blob_resize: expect_atime_update,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 262,
                    FixtureType::Fusemt => 263,
                    FixtureType::FuserWithoutInodeCache => 524, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 1623,
                    FixtureType::Fusemt => 1632,
                    FixtureType::FuserWithoutInodeCache => 3246, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                } + 2 * expect_atime_update,
                blob_data_mut: expect_atime_update,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 256,
                store: expect_atime_update,
                ..LLActionCounts::ZERO
            },
        }
    );
}
