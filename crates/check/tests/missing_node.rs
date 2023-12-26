use cryfs_blobstore::{BlobId, BlobStoreOnBlocks};
use cryfs_blockstore::{BlockStore, DynBlockStore};
use cryfs_check::CorruptedError;
use cryfs_cryfs::filesystem::fsblobstore::{FsBlob, FsBlobStore};
use cryfs_cryfs::utils::fs_types::{Gid, Mode, Uid};
use cryfs_utils::testutils::asserts::assert_unordered_vec_eq;
use std::time::SystemTime;

mod common;
use common::entry_helpers::{
    add_dir_entry, add_file_entry, add_symlink_entry, create_dir, create_some_blobs, load_dir_blob,
};
use common::fixture::FilesystemFixture;

fn blobid1() -> BlobId {
    BlobId::from_hex("ad977bad7882ede765bc3ef88f95c040").unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_missing_root_node() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    fs_fixture
        .update_fsblobstore(
            |blobstore: &FsBlobStore<BlobStoreOnBlocks<DynBlockStore>>| {
                Box::pin(async move {
                    let mut parent = load_dir_blob(blobstore, &some_blobs.dir1_dir3_dir5).await;
                    // Add a file entry but don't add the corresponding blob
                    add_file_entry(&mut parent, "filename", blobid1());
                    parent.async_drop().await.unwrap();
                })
            },
        )
        .await;

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(
        vec![CorruptedError::BlobMissing { blob_id: blobid1() }],
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
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    fs_fixture
        .update_fsblobstore(
            |blobstore: &FsBlobStore<BlobStoreOnBlocks<DynBlockStore>>| {
                Box::pin(async move {
                    let mut parent = load_dir_blob(blobstore, &some_blobs.dir2_dir6).await;
                    // Add a dir entry but don't add the corresponding blob
                    add_dir_entry(&mut parent, "dirname", blobid1());
                    parent.async_drop().await.unwrap();
                })
            },
        )
        .await;

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(
        vec![CorruptedError::BlobMissing { blob_id: blobid1() }],
        errors,
    );
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
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    fs_fixture
        .update_fsblobstore(
            |blobstore: &FsBlobStore<BlobStoreOnBlocks<DynBlockStore>>| {
                Box::pin(async move {
                    let mut parent = load_dir_blob(blobstore, &some_blobs.root).await;
                    // Add a symlink entry but don't add the corresponding blob
                    add_symlink_entry(&mut parent, "symlinkname", blobid1());
                    parent.async_drop().await.unwrap();
                })
            },
        )
        .await;

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(
        vec![CorruptedError::BlobMissing { blob_id: blobid1() }],
        errors,
    );
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
