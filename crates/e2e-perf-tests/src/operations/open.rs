use crate::filesystem_driver::FilesystemDriver;
use crate::filesystem_fixture::ActionCounts;
use crate::perf_test_macro::FixtureType;
use crate::test_driver::TestDriver;
use crate::test_driver::TestReady;
use crate::utils::maybe_close;
use cryfs_blobstore::BlobStoreActionCounts;
use cryfs_blockstore::HLActionCounts;
use cryfs_blockstore::LLActionCounts;
use cryfs_rustfs::AbsolutePath;
use cryfs_rustfs::PathComponent;

crate::perf_test_macro::perf_test!(
    open,
    [
        in_rootdir::<false>,
        in_rootdir::<true>,
        in_nesteddir::<false>,
        in_nesteddir::<true>,
        in_deeplynesteddir::<false>,
        in_deeplynesteddir::<true>,
    ]
);

fn in_rootdir<const CLOSE_AFTER: bool>(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            fixture
                .filesystem
                .create_file(None, PathComponent::try_from_str("testfile.txt").unwrap())
                .await
                .unwrap()
        })
        .test(async |fixture, file| {
            let file_handle = fixture.filesystem.open(file.clone()).await.unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, file_handle).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 0 + 2 * close_after, // If the inode is already cached, opening a file doesn't need to do any ops
                    FixtureType::Fusemt => 1 + 2 * close_after,
                    FixtureType::FuserWithoutInodeCache => 2 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache => 0 + close_after, // If the inode is already cached, opening a file doesn't need to do any ops
                    FixtureType::Fusemt => 1 + close_after,
                    FixtureType::FuserWithoutInodeCache => 1 + 2 * close_after,
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache => 0 + 2 * close_after, // If the inode is already cached, opening a file doesn't need to do any ops
                    FixtureType::Fusemt => 1 + 2 * close_after,
                    FixtureType::FuserWithoutInodeCache => 2 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_num_bytes: match fixture_type {
                    FixtureType::FuserWithInodeCache => 0, // If the inode is already cached, opening a file doesn't need to do any ops
                    FixtureType::Fusemt => 0,
                    FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_flush: close_after,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 0 + 2 * close_after, // If the inode is already cached, opening a file doesn't need to do any ops
                    FixtureType::Fusemt => 1 + 2 * close_after,
                    FixtureType::FuserWithoutInodeCache => 2 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 0 + 16 * close_after, // If the inode is already cached, opening a file doesn't need to do any ops
                    FixtureType::Fusemt => 9 + 16 * close_after,
                    FixtureType::FuserWithoutInodeCache => 16 + 32 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_flush_block: close_after,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2 * close_after, // If the inode is already cached, opening a file doesn't need to do any ops
                    FixtureType::Fusemt => 1 + close_after,
                    FixtureType::FuserWithoutInodeCache => 2, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..LLActionCounts::ZERO
            },
        }})
}

fn in_nesteddir<const CLOSE_AFTER: bool>(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            let parent = fixture
                .filesystem
                .mkdir(None, PathComponent::try_from_str("nested").unwrap())
                .await
                .unwrap();
            fixture
                .filesystem
                .create_file(
                    Some(parent),
                    PathComponent::try_from_str("testfile.txt").unwrap(),
                )
                .await
                .unwrap()
        })
        .test(async |fixture, file| {
            let file_handle = fixture.filesystem.open(file.clone()).await.unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, file_handle).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 0 + 2 * close_after, // If the inode is already cached, opening a file doesn't need to do any ops
                    FixtureType::Fusemt => 2 + 2 * close_after,
                    FixtureType::FuserWithoutInodeCache => 4 + 6 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache => 0 + close_after, // If the inode is already cached, opening a file doesn't need to do any ops
                    FixtureType::Fusemt => 2 + close_after,
                    FixtureType::FuserWithoutInodeCache => 3 + 4 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache => 0 + 2 * close_after, // If the inode is already cached, opening a file doesn't need to do any ops
                    FixtureType::Fusemt => 2 + 2 * close_after,
                    FixtureType::FuserWithoutInodeCache => 4 + 6 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_num_bytes: match fixture_type {
                    FixtureType::FuserWithInodeCache => 0, // If the inode is already cached, opening a file doesn't need to do any ops
                    FixtureType::Fusemt => 0,
                    FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_flush: close_after,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 0 + 2 * close_after, // If the inode is already cached, opening a file doesn't need to do any ops
                    FixtureType::Fusemt => 2 + 2 * close_after,
                    FixtureType::FuserWithoutInodeCache => 4 + 6 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 0 + 16 * close_after, // If the inode is already cached, opening a file doesn't need to do any ops
                    FixtureType::Fusemt => 18 + 16 * close_after,
                    FixtureType::FuserWithoutInodeCache => 34 + 50 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_flush_block: close_after,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2 * close_after, // If the inode is already cached, opening a file doesn't need to do any ops
                    FixtureType::Fusemt => 2 + close_after,
                    FixtureType::FuserWithoutInodeCache => 3, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..LLActionCounts::ZERO
            },
        }})
}

fn in_deeplynesteddir<const CLOSE_AFTER: bool>(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async |fixture| {
            let parent = fixture
                .filesystem
                .mkdir_recursive(AbsolutePath::try_from_str("/nested1/nested2/nested3").unwrap())
                .await
                .unwrap();
            fixture
                .filesystem
                .create_file(
                    Some(parent),
                    PathComponent::try_from_str("testfile.txt").unwrap(),
                )
                .await
                .unwrap()
        })
        .test(async |fixture, file| {
            let file_handle = fixture.filesystem.open(file.clone()).await.unwrap();
            maybe_close::<CLOSE_AFTER, _>(fixture, file, file_handle).await;
        })
        .expect_op_counts(|fixture_type, _atime_behavior| {
            let close_after = if CLOSE_AFTER { 1 } else { 0 };
            ActionCounts {
            blobstore: BlobStoreActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 0 + 2 * close_after, // If the inode is already cached, opening a file doesn't need to do any ops
                    FixtureType::Fusemt => 4 + 2 * close_after,
                    FixtureType::FuserWithoutInodeCache => 8 + 10 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read_all: match fixture_type {
                    FixtureType::FuserWithInodeCache => 0 + close_after, // If the inode is already cached, opening a file doesn't need to do any ops
                    FixtureType::Fusemt => 4 + close_after,
                    FixtureType::FuserWithoutInodeCache => 7 + 8 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_read: match fixture_type {
                    FixtureType::FuserWithInodeCache => 0 + 2 * close_after, // If the inode is already cached, opening a file doesn't need to do any ops
                    FixtureType::Fusemt => 4 + 2 * close_after,
                    FixtureType::FuserWithoutInodeCache => 8 + 10 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_num_bytes: match fixture_type {
                    FixtureType::FuserWithInodeCache => 0, // If the inode is already cached, opening a file doesn't need to do any ops
                    FixtureType::Fusemt => 0,
                    FixtureType::FuserWithoutInodeCache => 1 + close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_flush: close_after,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 0 + 2 * close_after, // If the inode is already cached, opening a file doesn't need to do any ops
                    FixtureType::Fusemt => 4 + 2 * close_after,
                    FixtureType::FuserWithoutInodeCache => 8 + 10 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                blob_data: match fixture_type {
                    FixtureType::FuserWithInodeCache => 0 + 16 * close_after, // If the inode is already cached, opening a file doesn't need to do any ops
                    FixtureType::Fusemt => 36 + 16 * close_after,
                    FixtureType::FuserWithoutInodeCache => 70 + 86 * close_after, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                store_flush_block: close_after,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                // TODO Check if these counts are what we'd expect
                load: match fixture_type {
                    FixtureType::FuserWithInodeCache => 2 * close_after, // If the inode is already cached, opening a file doesn't need to do any ops
                    FixtureType::Fusemt => 4 + close_after,
                    FixtureType::FuserWithoutInodeCache => 5, // TODO Why more than fusemt? Maybe because our CryNode structs don't cache the node and only store the path, so we have to lookup for fuser and then lookup everythin again?
                },
                ..LLActionCounts::ZERO
            },
        }})
}
