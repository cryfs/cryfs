use crate::filesystem_driver::FilesystemDriver as _;
use crate::fixture::ActionCounts;
use crate::test_driver::{TestDriver, TestReady};
use cryfs_blobstore::BlobStoreActionCounts;
use cryfs_blockstore::HLActionCounts;
use cryfs_blockstore::LLActionCounts;

crate::rstest::perf_test!(statfs, [empty_filesystem, with_content]);

fn empty_filesystem(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async move |_fixture| {
            // No setup needed
        })
        .test(async move |fixture, ()| {
            fixture.filesystem.statfs().await.unwrap();
        })
        .expect_op_counts(|_fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                store_num_nodes: 1,
                store_estimate_space_for_num_blocks_left: 1,
                store_virtual_block_size_bytes: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                store_num_blocks: 1,
                store_estimate_num_free_bytes: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                estimate_num_free_bytes: 1,
                num_blocks: 1,
                ..LLActionCounts::ZERO
            },
        })
}

fn with_content(test_driver: impl TestDriver) -> impl TestReady {
    test_driver
        .create_filesystem()
        .setup(async move |fixture| {
            let dir = fixture
                .filesystem
                .mkdir(
                    None,
                    cryfs_rustfs::PathComponent::try_from_str("testdir").unwrap(),
                )
                .await
                .unwrap();

            fixture
                .filesystem
                .create_file(
                    None,
                    cryfs_rustfs::PathComponent::try_from_str("rootfile.txt").unwrap(),
                )
                .await
                .unwrap();

            fixture
                .filesystem
                .create_file(
                    Some(dir),
                    cryfs_rustfs::PathComponent::try_from_str("subfile.txt").unwrap(),
                )
                .await
                .unwrap();
        })
        .test(async move |fixture, ()| {
            fixture.filesystem.statfs().await.unwrap();
        })
        .expect_op_counts(|_fixture_type, _atime_behavior| ActionCounts {
            blobstore: BlobStoreActionCounts {
                store_num_nodes: 1,
                store_estimate_space_for_num_blocks_left: 1,
                store_virtual_block_size_bytes: 1,
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                store_num_blocks: 1,
                store_estimate_num_free_bytes: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                estimate_num_free_bytes: 1,
                num_blocks: 1,
                ..LLActionCounts::ZERO
            },
        })
}
