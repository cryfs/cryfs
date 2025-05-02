use pretty_assertions::assert_eq;
use rstest::rstest;
use rstest_reuse::apply;

use crate::filesystem_test_ext::FilesystemTestExt as _;
use crate::rstest::FixtureFactory;
use crate::rstest::all_atime_behaviors;
use crate::rstest::all_fixtures;
use cryfs_blockstore::{HLActionCounts, LLActionCounts};
use cryfs_rustfs::AtimeUpdateBehavior;

#[apply(all_fixtures)]
#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn init(fixture_factory: impl FixtureFactory, atime_behavior: AtimeUpdateBehavior) {
    use cryfs_blobstore::BlobStoreActionCounts;

    use crate::fixture::ActionCounts;

    let fixture = fixture_factory
        .create_uninitialized_filesystem(atime_behavior)
        .await;

    let mut counts = fixture.totals();

    counts += fixture
        .run_operation(async |fs| fs.init().await.unwrap())
        .await;
    assert_eq!(
        counts,
        ActionCounts {
            blobstore: BlobStoreActionCounts {
                store_try_create: 1, // create root dir blob
                store_load: 1,       // load root dir blob for sanity checking the file system
                blob_read_all: 1, // read content of root dir blob for sanity checking the file system
                blob_read: 1, // read header of root dir blob for sanity checking the file system
                blob_write: 1, // write to root dir blob. TODO Why don't we directly create it with the data?
                blob_flush: 1, // flush root dir blob
                ..BlobStoreActionCounts::ZERO
            },
            high_level: HLActionCounts {
                // TODO Check if these counts are what we'd expect
                store_load: 2,
                blob_data: 15,
                blob_data_mut: 2,
                store_try_create: 1,
                store_flush_block: 1,
                store_block_size_from_physical_block_size: 1,
                ..HLActionCounts::ZERO
            },
            low_level: LLActionCounts {
                exists: 1,
                store: 1,
                block_size_from_physical_block_size: 1,
                ..LLActionCounts::ZERO
            },
        }
    );
}
