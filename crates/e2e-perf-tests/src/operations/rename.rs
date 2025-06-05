use crate::filesystem_driver::FilesystemDriver as _;
use crate::fixture::ActionCounts;
use crate::perf_test_macro::FixtureType;
use crate::test_driver::TestDriver;
use crate::test_driver::TestReady;
use cryfs_blobstore::BlobStoreActionCounts;
use cryfs_blockstore::HLActionCounts;
use cryfs_blockstore::LLActionCounts;
use cryfs_rustfs::AbsolutePath;
use cryfs_rustfs::PathComponent;

crate::perf_test_macro::perf_test!(
    rename,
    [
        within_rootdir,
        between_nested_dirs,
        between_deeply_nested_dirs,
        to_existing_target,
        directory,
        symlink,
        from_nested_to_deeply_nested,
        from_deeply_nested_to_nested,
    ]
);

// TODO Move these to the `perf_test!` above, but that currently deadlocks
crate::perf_test_macro::perf_test_only_fusemt!(
    rename_fusemt,
    [
        from_rootdir_to_nesteddir,
        from_nesteddir_to_rootdir,
        within_nested_dir,
    ]
);

fn within_rootdir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create a file to rename
            fixture
                .filesystem
                .create_file(None, PathComponent::try_from_str("original.txt").unwrap())
                .await
                .unwrap();
        })
        .test(async |fixture, ()| {
            fixture
                .filesystem
                .rename(
                    None,
                    PathComponent::try_from_str("original.txt").unwrap(),
                    None,
                    PathComponent::try_from_str("renamed.txt").unwrap(),
                )
                .await
                .unwrap();
        })
        .expect_op_counts(|_fixture_type, _atime_behavior| ActionCounts {
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
        })
}

// TODO Use #[apply(all_fixtures)] but that currently deadlocks
fn from_rootdir_to_nesteddir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // First create a file to rename and a destination directory
            fixture
                .filesystem
                .create_file(None, PathComponent::try_from_str("source.txt").unwrap())
                .await
                .unwrap();
            let dest_dir = fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("destdir").unwrap())
                .await
                .unwrap();
            dest_dir
        })
        .test(async |fixture, dest_dir| {
            fixture
                .filesystem
                .rename(
                    None,
                    PathComponent::try_from_str("source.txt").unwrap(),
                    Some(dest_dir),
                    PathComponent::try_from_str("moved.txt").unwrap(),
                )
                .await
                .unwrap();
        })
        .expect_op_counts(|_fixture_type, _atime_behavior| ActionCounts {
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
        })
}

// TODO Use #[apply(all_fixtures)] but that currently deadlocks
fn from_nesteddir_to_rootdir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // Create a source directory with a file in it
            let source_dir = fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("sourcedir").unwrap())
                .await
                .unwrap();

            // Create the file in the source directory
            fixture
                .filesystem
                .create_file(
                    Some(source_dir.clone()),
                    PathComponent::try_from_str("nested.txt").unwrap(),
                )
                .await
                .unwrap();

            source_dir
        })
        .test(async |fixture, source_dir| {
            fixture
                .filesystem
                .rename(
                    Some(source_dir),
                    PathComponent::try_from_str("nested.txt").unwrap(),
                    None,
                    PathComponent::try_from_str("moved_to_root.txt").unwrap(),
                )
                .await
                .unwrap();
        })
        .expect_op_counts(|_fixture_type, _atime_behavior| ActionCounts {
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
        })
}

fn between_nested_dirs(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // Create source and destination directories
            let source_dir = fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("sourcedir").unwrap())
                .await
                .unwrap();
            let dest_dir = fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("destdir").unwrap())
                .await
                .unwrap();

            // Create a file in the source directory
            fixture
                .filesystem
                .create_file(
                    Some(source_dir.clone()),
                    PathComponent::try_from_str("to_move.txt").unwrap(),
                )
                .await
                .unwrap();

            (source_dir, dest_dir)
        })
        .test(async |fixture, (source_dir, dest_dir)| {
            fixture
                .filesystem
                .rename(
                    Some(source_dir),
                    PathComponent::try_from_str("to_move.txt").unwrap(),
                    Some(dest_dir),
                    PathComponent::try_from_str("moved.txt").unwrap(),
                )
                .await
                .unwrap();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 6,
                    FixtureType::Fusemt => 4, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 9, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache => 5,
                    FixtureType::Fusemt => 3, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 8, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache => 6,
                    FixtureType::Fusemt => 4, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 9, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: match fixture_type {
                    FixtureType::FuserWithInodeCache => 5,
                    FixtureType::Fusemt => 3, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_resize: match fixture_type {
                    FixtureType::FuserWithInodeCache => 4,
                    FixtureType::Fusemt => 2, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 6,
                    FixtureType::Fusemt => 4, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 9, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 62,
                    FixtureType::Fusemt => 40, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 89, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: match fixture_type {
                    FixtureType::FuserWithInodeCache => 6,
                    FixtureType::Fusemt => 4, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 6, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: 4,
                store: match fixture_type {
                    FixtureType::FuserWithInodeCache => 4,
                    FixtureType::Fusemt => 3, // TODO Why less than fuser with cache???
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..LLActionCounts::ZERO
            },
        })
}

// TODO Use #[apply(all_fixtures)] but that currently deadlocks
fn within_nested_dir(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // Create a nested directory
            let nested_dir = fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap();

            // Create a file in the nested directory
            fixture
                .filesystem
                .create_file(
                    Some(nested_dir.clone()),
                    PathComponent::try_from_str("original.txt").unwrap(),
                )
                .await
                .unwrap();

            nested_dir
        })
        .test(async |fixture, nested_dir| {
            fixture
                .filesystem
                .rename(
                    Some(nested_dir.clone()),
                    PathComponent::try_from_str("original.txt").unwrap(),
                    Some(nested_dir),
                    PathComponent::try_from_str("renamed.txt").unwrap(),
                )
                .await
                .unwrap();
        })
        .expect_op_counts(|_fixture_type, _atime_behavior| ActionCounts {
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
        })
}

fn between_deeply_nested_dirs(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // Create deeply nested source and destination directories
            let source_dir = fixture
                .filesystem
                .mkdir_recursive(AbsolutePath::try_from_str("/source/path/deep").unwrap())
                .await
                .unwrap();
            let dest_dir = fixture
                .filesystem
                .mkdir_recursive(AbsolutePath::try_from_str("/dest/another/path").unwrap())
                .await
                .unwrap();

            // Create a file in the source directory
            fixture
                .filesystem
                .create_file(
                    Some(source_dir.clone()),
                    PathComponent::try_from_str("to_move.txt").unwrap(),
                )
                .await
                .unwrap();

            (source_dir, dest_dir)
        })
        .test(async |fixture, (source_dir, dest_dir)| {
            fixture
                .filesystem
                .rename(
                    Some(source_dir),
                    PathComponent::try_from_str("to_move.txt").unwrap(),
                    Some(dest_dir),
                    PathComponent::try_from_str("moved.txt").unwrap(),
                )
                .await
                .unwrap();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 6,
                    FixtureType::Fusemt => 8,
                    FixtureType::FuserWithoutInodeCache => 17, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache => 5,
                    FixtureType::Fusemt => 7,
                    FixtureType::FuserWithoutInodeCache => 16, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache => 6,
                    FixtureType::Fusemt => 8,
                    FixtureType::FuserWithoutInodeCache => 17, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: match fixture_type {
                    FixtureType::FuserWithInodeCache => 5,
                    FixtureType::Fusemt => 3, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_resize: match fixture_type {
                    FixtureType::FuserWithInodeCache => 4,
                    FixtureType::Fusemt => 2, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 6,
                    FixtureType::Fusemt => 8,
                    FixtureType::FuserWithoutInodeCache => 17, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 62,
                    FixtureType::Fusemt => 76,
                    FixtureType::FuserWithoutInodeCache => 161, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: match fixture_type {
                    FixtureType::FuserWithInodeCache => 6,
                    FixtureType::Fusemt => 4, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 6, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 5,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 8,
                },
                store: match fixture_type {
                    FixtureType::FuserWithInodeCache => 5,
                    FixtureType::Fusemt => 3, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..LLActionCounts::ZERO
            },
        })
}

fn to_existing_target(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // Create source and destination files
            fixture
                .filesystem
                .create_file(None, PathComponent::try_from_str("source.txt").unwrap())
                .await
                .unwrap();
            fixture
                .filesystem
                .create_file(None, PathComponent::try_from_str("target.txt").unwrap())
                .await
                .unwrap();
        })
        .test(async |fixture, ()| {
            fixture
                .filesystem
                .rename(
                    None,
                    PathComponent::try_from_str("source.txt").unwrap(),
                    None,
                    PathComponent::try_from_str("target.txt").unwrap(),
                )
                .await
                .unwrap(); // TODO Shouldn't this be unwrap_err, i.e. the op should fail since the target already exists?
        })
        .expect_op_counts(|_fixture_type, _atime_behavior| ActionCounts {
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
        })
}

fn directory(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // Create a directory to rename
            fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("olddir").unwrap())
                .await
                .unwrap();
        })
        .test(async |fixture, ()| {
            fixture
                .filesystem
                .rename(
                    None,
                    PathComponent::try_from_str("olddir").unwrap(),
                    None,
                    PathComponent::try_from_str("newdir").unwrap(),
                )
                .await
                .unwrap();
        })
        .expect_op_counts(|_fixture_type, _atime_behavior| ActionCounts {
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
        })
}

fn symlink(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // Create a symlink to rename
            fixture
                .filesystem
                .create_symlink(
                    None,
                    PathComponent::try_from_str("oldlink").unwrap(),
                    AbsolutePath::try_from_str("/target/path").unwrap(),
                )
                .await
                .unwrap();
        })
        .test(async |fixture, ()| {
            fixture
                .filesystem
                .rename(
                    None,
                    PathComponent::try_from_str("oldlink").unwrap(),
                    None,
                    PathComponent::try_from_str("newlink").unwrap(),
                )
                .await
                .unwrap();
        })
        .expect_op_counts(|_fixture_type, _atime_behavior| ActionCounts {
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
        })
}

fn from_nested_to_deeply_nested(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // Create a simple nested directory and a deeply nested directory
            let nested_dir = fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap();
            let deeply_nested_dir = fixture
                .filesystem
                .mkdir_recursive(AbsolutePath::try_from_str("/deep/path/structure").unwrap())
                .await
                .unwrap();

            // Create a file in the nested directory
            fixture
                .filesystem
                .create_file(
                    Some(nested_dir.clone()),
                    PathComponent::try_from_str("source.txt").unwrap(),
                )
                .await
                .unwrap();

            (nested_dir, deeply_nested_dir)
        })
        .test(async |fixture, (nested_dir, deeply_nested_dir)| {
            fixture
                .filesystem
                .rename(
                    Some(nested_dir),
                    PathComponent::try_from_str("source.txt").unwrap(),
                    Some(deeply_nested_dir),
                    PathComponent::try_from_str("moved.txt").unwrap(),
                )
                .await
                .unwrap();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 6,
                    FixtureType::FuserWithoutInodeCache => 13, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 12, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 6,
                    FixtureType::FuserWithoutInodeCache => 13, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: match fixture_type {
                    FixtureType::FuserWithInodeCache => 5,
                    FixtureType::Fusemt => 3, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_resize: match fixture_type {
                    FixtureType::FuserWithInodeCache => 4,
                    FixtureType::Fusemt => 2, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 6,
                    FixtureType::FuserWithoutInodeCache => 13, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 62,
                    FixtureType::Fusemt => 58,
                    FixtureType::FuserWithoutInodeCache => 125, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: match fixture_type {
                    FixtureType::FuserWithInodeCache => 6,
                    FixtureType::Fusemt => 4, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 6, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 5,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 6,
                },
                store: match fixture_type {
                    FixtureType::FuserWithInodeCache => 5,
                    FixtureType::Fusemt => 3, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..LLActionCounts::ZERO
            },
        })
}

fn from_deeply_nested_to_nested(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            // Create a simple nested directory and a deeply nested directory
            let nested_dir = fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap();
            let deeply_nested_dir = fixture
                .filesystem
                .mkdir_recursive(AbsolutePath::try_from_str("/deep/path/structure").unwrap())
                .await
                .unwrap();

            // Create a file in the deeply nested directory
            fixture
                .filesystem
                .create_file(
                    Some(deeply_nested_dir.clone()),
                    PathComponent::try_from_str("source.txt").unwrap(),
                )
                .await
                .unwrap();

            (nested_dir, deeply_nested_dir)
        })
        .test(async |fixture, (nested_dir, deeply_nested_dir)| {
            fixture
                .filesystem
                .rename(
                    Some(deeply_nested_dir),
                    PathComponent::try_from_str("source.txt").unwrap(),
                    Some(nested_dir),
                    PathComponent::try_from_str("moved.txt").unwrap(),
                )
                .await
                .unwrap();
        })
        .expect_op_counts(|fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 6,
                    FixtureType::FuserWithoutInodeCache => 13, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 5,
                    FixtureType::FuserWithoutInodeCache => 12, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 6,
                    FixtureType::FuserWithoutInodeCache => 13, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_write: match fixture_type {
                    FixtureType::FuserWithInodeCache => 5,
                    FixtureType::Fusemt => 3, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_resize: match fixture_type {
                    FixtureType::FuserWithInodeCache => 4,
                    FixtureType::Fusemt => 2, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 4, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache | FixtureType::Fusemt => 6,
                    FixtureType::FuserWithoutInodeCache => 13, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 62,
                    FixtureType::Fusemt => 58, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 125, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data_mut: match fixture_type {
                    FixtureType::FuserWithInodeCache => 6,
                    FixtureType::Fusemt => 4, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 6, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 5,
                    FixtureType::Fusemt | FixtureType::FuserWithoutInodeCache => 6,
                },
                store: match fixture_type {
                    FixtureType::FuserWithInodeCache => 5,
                    FixtureType::Fusemt => 3, // TODO Why less than fuser with cache?
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..LLActionCounts::ZERO
            },
        })
}
