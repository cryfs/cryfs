use pretty_assertions::assert_eq;
use rstest::rstest;
use rstest_reuse::apply;

use crate::filesystem_driver::FilesystemDriver as _;
use crate::fixture::ActionCounts;
use crate::rstest::FixtureFactory;
use crate::rstest::{all_atime_behaviors, all_fixtures};
use cryfs_blobstore::BlobStoreActionCounts;
use cryfs_blockstore::HLActionCounts;
use cryfs_blockstore::LLActionCounts;
use cryfs_rustfs::AtimeUpdateBehavior;

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn empty_filesystem(
    fixture_factory: impl FixtureFactory,
    atime_behavior: AtimeUpdateBehavior,
) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.statfs().await.unwrap();
        })
        .await;

    assert_eq!(
        counts,
        ActionCounts {
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
        }
    );
}

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn with_content(fixture_factory: impl FixtureFactory, atime_behavior: AtimeUpdateBehavior) {
    let fixture = fixture_factory
        .create_filesystem_deprecated(atime_behavior)
        .await;

    fixture
        .ops(async |fs| {
            let dir = fs
                .mkdir(
                    None,
                    cryfs_rustfs::PathComponent::try_from_str("testdir").unwrap(),
                )
                .await
                .unwrap();

            fs.create_file(
                None,
                cryfs_rustfs::PathComponent::try_from_str("rootfile.txt").unwrap(),
            )
            .await
            .unwrap();

            fs.create_file(
                Some(dir),
                cryfs_rustfs::PathComponent::try_from_str("subfile.txt").unwrap(),
            )
            .await
            .unwrap();
        })
        .await;

    let counts = fixture
        .count_ops(async |fs| {
            fs.statfs().await.unwrap();
        })
        .await;

    assert_eq!(
        counts,
        ActionCounts {
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
        }
    );
}
