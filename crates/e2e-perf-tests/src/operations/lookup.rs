use cryfs_blockstore::ActionCounts;
use cryfs_rustfs::{
    AtimeUpdateBehavior, PathComponent, low_level_api::AsyncFilesystemLL as _,
    object_based_api::FUSE_ROOT_ID,
};

use crate::fixture::{FilesystemFixture, request_info};

#[tokio::test(flavor = "multi_thread")]
async fn notexisting_from_rootdir() {
    let fixture = FilesystemFixture::create_filesystem(AtimeUpdateBehavior::Noatime).await;

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
