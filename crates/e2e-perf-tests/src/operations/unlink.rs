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
use cryfs_rustfs::NumBytes;
use cryfs_rustfs::PathComponent;

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn file_from_rootdir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    // First create a file to unlink
    fixture
        .ops(async |fs| {
            fs.create_file(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.unlink(None, PathComponent::try_from_str("file.txt").unwrap())
                .await
                .unwrap();
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
                blob_resize: 1,
                blob_write: 1,
                store_remove_by_id: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: 2,
                blob_data: 15,
                blob_data_mut: 1,
                store_remove: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 2,
                store: 1,
                remove: 1,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn symlink_from_rootdir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    // First create a symlink to unlink
    fixture
        .ops(async |fs| {
            fs.create_symlink(
                None,
                PathComponent::try_from_str("link.txt").unwrap(),
                AbsolutePath::try_from_str("/target").unwrap(),
            )
            .await
            .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.unlink(None, PathComponent::try_from_str("link.txt").unwrap())
                .await
                .unwrap();
        })
        .await;

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                store_load: 1,
                blob_read_all: 1,
                blob_read: 1,
                blob_resize: 1,
                blob_write: 1,
                store_remove_by_id: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                store_load: 2,
                blob_data: 15,
                blob_data_mut: 1,
                store_remove: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                load: 2,
                store: 1,
                remove: 1,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn file_not_existing(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.unlink(
                None,
                PathComponent::try_from_str("nonexistent.txt").unwrap(),
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
async fn file_from_nested_dir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    // First create the nested directory and file to unlink
    let parent = fixture
        .ops(async |fs| {
            let dir = fs
                .mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap();
            fs.create_file(
                Some(dir.clone()),
                PathComponent::try_from_str("file.txt").unwrap(),
            )
            .await
            .unwrap();
            dir
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.unlink(
                Some(parent),
                PathComponent::try_from_str("file.txt").unwrap(),
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
                blob_resize: 2,
                blob_write: 2,
                store_remove_by_id: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 26,
                    FixtureType::Fusemt => 35,
                    FixtureType::FuserWithoutInodeCache => 44, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 2,
                store_remove: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 3,
                store: 2,
                remove: 1,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn symlink_from_nested_dir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    // First create the nested directory and symlink to unlink
    let parent = fixture
        .ops(async |fs| {
            let dir = fs
                .mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap();
            fs.create_symlink(
                Some(dir.clone()),
                PathComponent::try_from_str("link.txt").unwrap(),
                AbsolutePath::try_from_str("/target").unwrap(),
            )
            .await
            .unwrap();
            dir
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.unlink(
                Some(parent),
                PathComponent::try_from_str("link.txt").unwrap(),
            )
            .await
            .unwrap();
        })
        .await;

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
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
                blob_resize: 2,
                blob_write: 2,
                store_remove_by_id: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 4,
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 26,
                    FixtureType::Fusemt => 35,
                    FixtureType::FuserWithoutInodeCache => 44, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 2,
                store_remove: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                load: 3,
                store: 2,
                remove: 1,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn file_from_deeply_nested_dir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    // Create a deeply nested directory structure with a file
    let parent = fixture
        .ops(async |fs| {
            let deeply_nested = fs
                .mkdir_recursive(AbsolutePath::try_from_str("/deep/nested/dir").unwrap())
                .await
                .unwrap();
            fs.create_file(
                Some(deeply_nested.clone()),
                PathComponent::try_from_str("file.txt").unwrap(),
            )
            .await
            .unwrap();
            deeply_nested
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.unlink(
                Some(parent),
                PathComponent::try_from_str("file.txt").unwrap(),
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
                blob_resize: 2,
                blob_write: 2,
                store_remove_by_id: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 6,
                    FixtureType::FuserWithoutInodeCache => 9, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 26,
                    FixtureType::Fusemt => 53,
                    FixtureType::FuserWithoutInodeCache => 80, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 2,
                store_remove: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 5,
                },
                store: 2,
                remove: 1,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn symlink_from_deeply_nested_dir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    // Create a deeply nested directory structure with a symlink
    let parent = fixture
        .ops(async |fs| {
            let deeply_nested = fs
                .mkdir_recursive(AbsolutePath::try_from_str("/deep/nested/dir").unwrap())
                .await
                .unwrap();
            fs.create_symlink(
                Some(deeply_nested.clone()),
                PathComponent::try_from_str("link.txt").unwrap(),
                AbsolutePath::try_from_str("/target/path").unwrap(),
            )
            .await
            .unwrap();
            deeply_nested
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.unlink(
                Some(parent),
                PathComponent::try_from_str("link.txt").unwrap(),
            )
            .await
            .unwrap();
        })
        .await;

    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
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
                blob_resize: 2,
                blob_write: 2,
                store_remove_by_id: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt => 6,
                    FixtureType::FuserWithoutInodeCache => 9, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 26,
                    FixtureType::Fusemt => 53,
                    FixtureType::FuserWithoutInodeCache => 80, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 2,
                store_remove: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 3,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 5,
                },
                store: 2,
                remove: 1,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn try_unlink_directory_in_rootdir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    // First create a directory that we'll try to unlink (which should fail)
    fixture
        .ops(async |fs| {
            fs.mkdir(None, PathComponent::try_from_str("directory").unwrap())
                .await
                .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.unlink(None, PathComponent::try_from_str("directory").unwrap())
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
                blob_write: 1,
                blob_resize: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: 1,
                blob_data: 11,
                blob_data_mut: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 1,
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
async fn try_unlink_directory_in_nested_dir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    // First create a nested directory structure with a subdirectory
    let parent = fixture
        .ops(async |fs| {
            let parent = fs
                .mkdir(None, PathComponent::try_from_str("parent").unwrap())
                .await
                .unwrap();
            fs.mkdir(
                Some(parent.clone()),
                PathComponent::try_from_str("subdir").unwrap(),
            )
            .await
            .unwrap();
            parent
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.unlink(Some(parent), PathComponent::try_from_str("subdir").unwrap())
                .await
                .unwrap_err(); // Should fail because target is a directory
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
                blob_write: 2,
                blob_resize: 2,
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
                    FixtureType::FuserWithInodeCache => 22,
                    FixtureType::Fusemt => 31,
                    FixtureType::FuserWithoutInodeCache => 40, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 2,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 2,
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
async fn try_unlink_directory_in_deeply_nested_dir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    // First create a deeply nested directory structure with a subdirectory
    let parent = fixture
        .ops(async |fs| {
            let deeply_nested = fs
                .mkdir_recursive(AbsolutePath::try_from_str("/deep/nested/dir").unwrap())
                .await
                .unwrap();
            fs.mkdir(
                Some(deeply_nested.clone()),
                PathComponent::try_from_str("subdir").unwrap(),
            )
            .await
            .unwrap();
            deeply_nested
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.unlink(Some(parent), PathComponent::try_from_str("subdir").unwrap())
                .await
                .unwrap_err(); // Should fail because target is a directory
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
                blob_write: 2,
                blob_resize: 2,
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
                    FixtureType::FuserWithInodeCache => 22,
                    FixtureType::Fusemt => 49,
                    FixtureType::FuserWithoutInodeCache => 76, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: 2,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 2,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 4,
                },
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
async fn large_file(fixture_factory: impl FixtureFactory, atime_behavior: AtimeUpdateBehavior) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    // First create a large file to unlink
    fixture
        .ops(async |fs| {
            let (file, mut fh) = fs
                .create_and_open_file(None, PathComponent::try_from_str("largefile.dat").unwrap())
                .await
                .unwrap();

            // Write a large amount of data to the file to ensure it spans multiple blocks
            let data = vec![0u8; NUM_BYTES_FOR_THREE_LEVEL_TREE as usize];
            fs.write(file.clone(), &mut fh, NumBytes::from(0), data)
                .await
                .unwrap();
            fs.release(file, fh).await.unwrap();
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.unlink(None, PathComponent::try_from_str("largefile.dat").unwrap())
                .await
                .unwrap();
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
                blob_resize: 1,
                blob_write: 1,
                store_remove_by_id: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: 6,
                blob_data: 45,
                blob_data_mut: 1,
                store_remove: 5,
                store_remove_by_id: 52,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 6,
                store: 1,
                remove: 57,
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn large_symlink(fixture_factory: impl FixtureFactory, atime_behavior: AtimeUpdateBehavior) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    // Create a very long target path which is stored across multiple nodes
    let long_target =
        "/very/long".repeat(NUM_BYTES_FOR_THREE_LEVEL_TREE as usize / 5) + "/target/path";

    // First create a symlink with a very long target path
    fixture
        .ops(async |fs| {
            fs.create_symlink(
                None,
                PathComponent::try_from_str("largesymlink").unwrap(),
                &AbsolutePath::try_from_str(&long_target).unwrap(),
            )
            .await
            .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.unlink(None, PathComponent::try_from_str("largesymlink").unwrap())
                .await
                .unwrap();
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
                blob_resize: 1,
                blob_write: 1,
                store_remove_by_id: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: 9,
                blob_data: 66,
                blob_data_mut: 1,
                store_remove: 8,
                store_remove_by_id: 104,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 9,
                store: 1,
                remove: 112,
                ..LLActionCounts::ZERO
            },
        }
    );
}
