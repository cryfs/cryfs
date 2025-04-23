use crate::fixture::FilesystemFixture;
use crate::fixture::request_info;
use cryfs_blockstore::ActionCounts;
use cryfs_rustfs::AtimeUpdateBehavior;
use cryfs_rustfs::low_level_api::AsyncFilesystemLL as _;
use rstest::rstest;
use rstest_reuse::apply;

use crate::rstest::all_atime_behaviors;

#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn notexisting_from_rootdir(atime_behavior: AtimeUpdateBehavior) {
    let fixture = FilesystemFixture::create_uninitialized_filesystem(atime_behavior).await;

    let mut counts = fixture.totals();

    counts += fixture
        .run_operation(async |fs| {
            fs.init(&request_info()).await.unwrap();
        })
        .await;
    assert_eq!(
        counts,
        ActionCounts {
            exists: 1,
            loaded: 0,
            stored: 1,
            removed: 0,
            created: 0,
        }
    );
}
