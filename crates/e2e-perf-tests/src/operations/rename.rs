use pretty_assertions::assert_eq;
use rstest::rstest;
use rstest_reuse::apply;

use crate::filesystem_driver::FilesystemDriver as _;
use crate::fixture::ActionCounts;
use crate::rstest::FixtureFactory;
use crate::rstest::FixtureType;
use crate::rstest::all_fusemt_fixtures;
use crate::rstest::{all_atime_behaviors, all_fixtures};
use cryfs_blobstore::BlobStoreActionCounts;
use cryfs_blockstore::HLActionCounts;
use cryfs_blockstore::LLActionCounts;
use cryfs_rustfs::AbsolutePath;
use cryfs_rustfs::AtimeUpdateBehavior;
use cryfs_rustfs::PathComponent;

// TODO Some rename operations in here run into deadlocks. Fix them.

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn within_rootdir(fixture_factory: impl FixtureFactory, atime_behavior: AtimeUpdateBehavior) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    // First create a file to rename
    fixture
        .ops(async |fs| {
            fs.create_file(None, PathComponent::try_from_str("original.txt").unwrap())
                .await
                .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.rename(
                None,
                PathComponent::try_from_str("original.txt").unwrap(),
                None,
                PathComponent::try_from_str("renamed.txt").unwrap(),
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
                blob_data: 12,
                blob_data_mut: 2,
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

// TODO Use #[apply(all_fixtures)] but that currently deadlocks
#[apply(all_fusemt_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn from_rootdir_to_nesteddir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    // First create a file to rename and a destination directory
    let dest_dir = fixture
        .ops(async |fs| {
            fs.create_file(None, PathComponent::try_from_str("source.txt").unwrap())
                .await
                .unwrap();
            fs.mkdir(None, PathComponent::try_from_str("destdir").unwrap())
                .await
                .unwrap()
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.rename(
                None,
                PathComponent::try_from_str("source.txt").unwrap(),
                Some(dest_dir),
                PathComponent::try_from_str("moved.txt").unwrap(),
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
                store_load: 3,
                blob_read_all: 2,
                blob_read: 3,
                blob_write: 3,
                blob_resize: 2,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: 3,
                blob_data: 32,
                blob_data_mut: 5,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 3,
                store: 3,
                ..LLActionCounts::ZERO
            },
        }
    );
}

// TODO Use #[apply(all_fixtures)] but that currently deadlocks
#[apply(all_fusemt_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn from_nesteddir_to_rootdir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    // Create a source directory with a file in it
    let source_dir = fixture
        .ops(async |fs| {
            fs.mkdir(None, PathComponent::try_from_str("sourcedir").unwrap())
                .await
                .unwrap()
        })
        .await;

    // Create the file in the source directory
    fixture
        .ops(async |fs| {
            fs.create_file(
                Some(source_dir.clone()),
                PathComponent::try_from_str("nested.txt").unwrap(),
            )
            .await
            .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.rename(
                Some(source_dir),
                PathComponent::try_from_str("nested.txt").unwrap(),
                None,
                PathComponent::try_from_str("moved_to_root.txt").unwrap(),
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
                store_load: 3,
                blob_read_all: 2,
                blob_read: 3,
                blob_write: 3,
                blob_resize: 2,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: 3,
                blob_data: 31,
                blob_data_mut: 4,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 3,
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
async fn between_nested_dirs(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    // Create source and destination directories
    let (source_dir, dest_dir) = fixture
        .ops(async |fs| {
            let source = fs
                .mkdir(None, PathComponent::try_from_str("sourcedir").unwrap())
                .await
                .unwrap();
            let dest = fs
                .mkdir(None, PathComponent::try_from_str("destdir").unwrap())
                .await
                .unwrap();
            (source, dest)
        })
        .await;

    // Create a file in the source directory
    fixture
        .ops(async |fs| {
            fs.create_file(
                Some(source_dir.clone()),
                PathComponent::try_from_str("to_move.txt").unwrap(),
            )
            .await
            .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.rename(
                Some(source_dir),
                PathComponent::try_from_str("to_move.txt").unwrap(),
                Some(dest_dir),
                PathComponent::try_from_str("moved.txt").unwrap(),
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
                    FixtureType::FuserWithInodeCache => 6,
                    FixtureType::Fusemt => 4, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 9, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 5,
                    FixtureType::Fusemt => 3, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 8, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 6,
                    FixtureType::Fusemt => 4, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 9, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 5,
                    FixtureType::Fusemt => 3, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_resize: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 4,
                    FixtureType::Fusemt => 2, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 6,
                    FixtureType::Fusemt => 4, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 9, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 62,
                    FixtureType::Fusemt => 40, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 89, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 6,
                    FixtureType::Fusemt => 4, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 6, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 4,
                store: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 4,
                    FixtureType::Fusemt => 3, // TODO Why less than fuser with cache???
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..LLActionCounts::ZERO
            },
        }
    );
}

// TODO Use #[apply(all_fixtures)] but that currently deadlocks
#[apply(all_fusemt_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn within_nested_dir(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    // Create a nested directory
    let nested_dir = fixture
        .ops(async |fs| {
            fs.mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap()
        })
        .await;

    // Create a file in the nested directory
    fixture
        .ops(async |fs| {
            fs.create_file(
                Some(nested_dir.clone()),
                PathComponent::try_from_str("original.txt").unwrap(),
            )
            .await
            .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.rename(
                Some(nested_dir.clone()),
                PathComponent::try_from_str("original.txt").unwrap(),
                Some(nested_dir),
                PathComponent::try_from_str("renamed.txt").unwrap(),
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
                store_load: 2,
                blob_read_all: 2,
                blob_read: 2,
                blob_write: 1,
                blob_resize: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: 2,
                blob_data: 21,
                blob_data_mut: 2,
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
async fn between_deeply_nested_dirs(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    // Create deeply nested source and destination directories
    let (source_dir, dest_dir) = fixture
        .ops(async |fs| {
            let source = fs
                .mkdir_recursive(AbsolutePath::try_from_str("/source/path/deep").unwrap())
                .await
                .unwrap();
            let dest = fs
                .mkdir_recursive(AbsolutePath::try_from_str("/dest/another/path").unwrap())
                .await
                .unwrap();
            (source, dest)
        })
        .await;

    // Create a file in the source directory
    fixture
        .ops(async |fs| {
            fs.create_file(
                Some(source_dir.clone()),
                PathComponent::try_from_str("to_move.txt").unwrap(),
            )
            .await
            .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.rename(
                Some(source_dir),
                PathComponent::try_from_str("to_move.txt").unwrap(),
                Some(dest_dir),
                PathComponent::try_from_str("moved.txt").unwrap(),
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
                    FixtureType::FuserWithInodeCache => 6,
                    FixtureType::Fusemt => 8,
                    FixtureType::FuserWithoutInodeCache => 17, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 5,
                    FixtureType::Fusemt => 7,
                    FixtureType::FuserWithoutInodeCache => 16, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 6,
                    FixtureType::Fusemt => 8,
                    FixtureType::FuserWithoutInodeCache => 17, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 5,
                    FixtureType::Fusemt => 3, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_resize: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 4,
                    FixtureType::Fusemt => 2, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 6,
                    FixtureType::Fusemt => 8,
                    FixtureType::FuserWithoutInodeCache => 17, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 62,
                    FixtureType::Fusemt => 76,
                    FixtureType::FuserWithoutInodeCache => 161, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 6,
                    FixtureType::Fusemt => 4, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 6, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 5,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 8,
                },
                store: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 5,
                    FixtureType::Fusemt => 3, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn to_existing_target(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    // Create source and destination files
    fixture
        .ops(async |fs| {
            fs.create_file(None, PathComponent::try_from_str("source.txt").unwrap())
                .await
                .unwrap();
            fs.create_file(None, PathComponent::try_from_str("target.txt").unwrap())
                .await
                .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.rename(
                None,
                PathComponent::try_from_str("source.txt").unwrap(),
                None,
                PathComponent::try_from_str("target.txt").unwrap(),
            )
            .await
            .unwrap(); // TODO Shouldn't this be unwrap_err, i.e. the op should fail since the target already exists?
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
                store_remove_by_id: 1,
                blob_resize: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: 2,
                blob_data: 16,
                blob_data_mut: 2,
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
async fn directory(fixture_factory: impl FixtureFactory, atime_behavior: AtimeUpdateBehavior) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    // Create a directory to rename
    fixture
        .ops(async |fs| {
            fs.mkdir(None, PathComponent::try_from_str("olddir").unwrap())
                .await
                .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.rename(
                None,
                PathComponent::try_from_str("olddir").unwrap(),
                None,
                PathComponent::try_from_str("newdir").unwrap(),
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
async fn symlink(fixture_factory: impl FixtureFactory, atime_behavior: AtimeUpdateBehavior) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    // Create a symlink to rename
    fixture
        .ops(async |fs| {
            fs.create_symlink(
                None,
                PathComponent::try_from_str("oldlink").unwrap(),
                AbsolutePath::try_from_str("/target/path").unwrap(),
            )
            .await
            .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.rename(
                None,
                PathComponent::try_from_str("oldlink").unwrap(),
                None,
                PathComponent::try_from_str("newlink").unwrap(),
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
async fn from_nested_to_deeply_nested(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    // Create a simple nested directory and a deeply nested directory
    let (nested_dir, deeply_nested_dir) = fixture
        .ops(async |fs| {
            let nested = fs
                .mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap();
            let deeply = fs
                .mkdir_recursive(AbsolutePath::try_from_str("/deep/path/structure").unwrap())
                .await
                .unwrap();
            (nested, deeply)
        })
        .await;

    // Create a file in the nested directory
    fixture
        .ops(async |fs| {
            fs.create_file(
                Some(nested_dir.clone()),
                PathComponent::try_from_str("source.txt").unwrap(),
            )
            .await
            .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.rename(
                Some(nested_dir),
                PathComponent::try_from_str("source.txt").unwrap(),
                Some(deeply_nested_dir),
                PathComponent::try_from_str("moved.txt").unwrap(),
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
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 6,
                    FixtureType::FuserWithoutInodeCache => 13, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 12, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 6,
                    FixtureType::FuserWithoutInodeCache => 13, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 5,
                    FixtureType::Fusemt => 3, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_resize: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 4,
                    FixtureType::Fusemt => 2, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 6,
                    FixtureType::FuserWithoutInodeCache => 13, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 62,
                    FixtureType::Fusemt => 58,
                    FixtureType::FuserWithoutInodeCache => 125, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 6,
                    FixtureType::Fusemt => 4, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 6, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 5,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 6,
                },
                store: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 5,
                    FixtureType::Fusemt => 3, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..LLActionCounts::ZERO
            },
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn from_deeply_nested_to_nested(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    // Create a simple nested directory and a deeply nested directory
    let (nested_dir, deeply_nested_dir) = fixture
        .ops(async |fs| {
            let nested = fs
                .mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap();
            let deeply = fs
                .mkdir_recursive(AbsolutePath::try_from_str("/deep/path/structure").unwrap())
                .await
                .unwrap();
            (nested, deeply)
        })
        .await;

    // Create a file in the deeply nested directory
    fixture
        .ops(async |fs| {
            fs.create_file(
                Some(deeply_nested_dir.clone()),
                PathComponent::try_from_str("source.txt").unwrap(),
            )
            .await
            .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.rename(
                Some(deeply_nested_dir),
                PathComponent::try_from_str("source.txt").unwrap(),
                Some(nested_dir),
                PathComponent::try_from_str("moved.txt").unwrap(),
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
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 6,
                    FixtureType::FuserWithoutInodeCache => 13, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 12, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 6,
                    FixtureType::FuserWithoutInodeCache => 13, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 5,
                    FixtureType::Fusemt => 3, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_resize: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 4,
                    FixtureType::Fusemt => 2, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 6,
                    FixtureType::FuserWithoutInodeCache => 13, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 62,
                    FixtureType::Fusemt => 58, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 125, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 6,
                    FixtureType::Fusemt => 4, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 6, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 5,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 6,
                },
                store: match fixture_factory.fixture_type() {
                    FixtureType::FuserWithInodeCache => 5,
                    FixtureType::Fusemt => 3, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..LLActionCounts::ZERO
            },
        }
    );
}
