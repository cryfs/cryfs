use cryfs_blobstore::BlobId;
use cryfs_check::CorruptedError;
use cryfs_cryfs::filesystem::fsblobstore::FsBlob;
use cryfs_cryfs::utils::fs_types::{Gid, Mode, Uid};
use cryfs_utils::testutils::asserts::assert_unordered_vec_eq;
use std::time::SystemTime;

mod common;
use common::entry_helpers::{create_dir, create_some_blobs};
use common::fixture::FilesystemFixture;

#[tokio::test(flavor = "multi_thread")]
async fn file_with_missing_root_node() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let blob_id = BlobId::from_hex("ad977bad7882ede765bc3ef88f95c040").unwrap();

    fs_fixture
        .update_fsblobstore(|blobstore| async move {
            let mut parent = FsBlob::into_dir(
                blobstore
                    .load(&some_blobs.dir1_dir3_dir5)
                    .await
                    .unwrap()
                    .unwrap(),
            )
            .await
            .unwrap();
            // Add a file entry but don't add the corresponding blob
            parent
                .add_entry_file(
                    "filename".to_string().try_into().unwrap(),
                    blob_id,
                    Mode::zero().add_file_flag(),
                    Uid::from(1000),
                    Gid::from(1000),
                    SystemTime::now(),
                    SystemTime::now(),
                )
                .unwrap();
            parent.async_drop().await.unwrap();
        })
        .await;

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(
        vec![
            // TODO Why is this reported both as a missing blob **and** as a missing node? Can we only report it as a blob? Maybe during the error deduplication step.
            CorruptedError::BlobMissing { blob_id },
            CorruptedError::NodeMissing {
                node_id: *blob_id.to_root_block_id(),
            },
        ],
        errors,
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_missing_inner_node() {
    // TODO
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_missing_leaf_node() {
    // TODO
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_missing_inner_node_and_own_leaf_node() {
    // TODO
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_missing_inner_node_and_foreign_leaf_node() {
    // TODO
}

#[tokio::test(flavor = "multi_thread")]
async fn dir_with_missing_root_node() {
    // TODO
}

#[tokio::test(flavor = "multi_thread")]
async fn dir_with_missing_inner_node() {
    // TODO
}

#[tokio::test(flavor = "multi_thread")]
async fn dir_with_missing_leaf_node() {
    // TODO
}

#[tokio::test(flavor = "multi_thread")]
async fn dir_with_missing_inner_node_and_own_leaf_node() {
    // TODO
}

#[tokio::test(flavor = "multi_thread")]
async fn dir_with_missing_inner_node_and_foreign_leaf_node() {
    // TODO
}

#[tokio::test(flavor = "multi_thread")]
async fn symlink_with_missing_root_node() {
    // TODO
}

#[tokio::test(flavor = "multi_thread")]
async fn symlink_with_missing_inner_node() {
    // TODO
}

#[tokio::test(flavor = "multi_thread")]
async fn symlink_with_missing_leaf_node() {
    // TODO
}

#[tokio::test(flavor = "multi_thread")]
async fn symlink_with_missing_inner_node_and_own_leaf_node() {
    // TODO
}

#[tokio::test(flavor = "multi_thread")]
async fn symlink_with_missing_inner_node_and_foreign_leaf_node() {
    // TODO
}
