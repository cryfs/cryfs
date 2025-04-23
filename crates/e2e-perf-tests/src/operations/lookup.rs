use cryfs_blockstore::ActionCounts;
use cryfs_rustfs::{
    AtimeUpdateBehavior, PathComponent, low_level_api::AsyncFilesystemLL as _,
    object_based_api::FUSE_ROOT_ID,
};
use rstest::rstest;
use rstest_reuse::apply;

use crate::{
    fixture::{FilesystemFixture, request_info},
    rstest::all_atime_behaviors,
};

#[apply(all_atime_behaviors)]
#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn notexisting_from_rootdir(atime_behavior: AtimeUpdateBehavior) {
    let fixture = FilesystemFixture::create_filesystem(atime_behavior).await;

    let counts = fixture
        .run_operation(async |fs| {
            let _ = fs
                .lookup(
                    &request_info(),
                    FUSE_ROOT_ID,
                    PathComponent::try_from_str("notexisting").unwrap(),
                )
                .await;
        })
        .await;
    assert_eq!(counts, ActionCounts::ZERO);
}
