use std::iter;

use cryfs_blobstore::{BlobId, BlobStoreOnBlocks};
use cryfs_blockstore::DynBlockStore;
use cryfs_check::CorruptedError;
use cryfs_cryfs::filesystem::fsblobstore::FsBlobStore;

use cryfs_utils::testutils::asserts::assert_unordered_vec_eq;

mod common;
use common::entry_helpers::{add_dir_entry, add_file_entry, add_symlink_entry, load_dir_blob};
use common::fixture::{FilesystemFixture, RemoveInnerNodeResult};

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
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let RemoveInnerNodeResult {
        removed_node,
        orphaned_children_nodes,
    } = fs_fixture
        .remove_an_inner_node_of_a_large_blob(some_blobs.large_file)
        .await;

    let expected_errors = iter::once(CorruptedError::NodeMissing {
        node_id: removed_node,
    })
    .chain(
        orphaned_children_nodes
            .into_iter()
            .map(|child| CorruptedError::NodeUnreferenced { node_id: child }),
    )
    .collect();

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
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
    // TODO In this test, make sure that some dir entries have more than just one node
    // TODO In this test, make sure that a dir entry has its own entries, i.e. 2 dir levels removed from the missing node

    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let orphaned_children_blobs = fs_fixture
        .get_children_of_dir_blob(some_blobs.large_dir)
        .await;
    let RemoveInnerNodeResult {
        removed_node,
        orphaned_children_nodes,
    } = fs_fixture
        .remove_an_inner_node_of_a_large_blob(some_blobs.large_dir)
        .await;

    let expected_errors = [
        CorruptedError::NodeMissing {
            node_id: removed_node,
        },
        CorruptedError::BlobUnreadable {
            blob_id: some_blobs.large_dir,
        },
    ]
    .into_iter()
    .chain(
        orphaned_children_nodes
            .into_iter()
            .map(|child| CorruptedError::NodeUnreferenced { node_id: child }),
    )
    .chain(
        orphaned_children_blobs
            .into_iter()
            .map(|child| CorruptedError::NodeUnreferenced {
                node_id: *child.to_root_block_id(),
            }),
    )
    .collect();

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
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
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let RemoveInnerNodeResult {
        removed_node,
        orphaned_children_nodes,
    } = fs_fixture
        .remove_an_inner_node_of_a_large_blob(some_blobs.large_symlink)
        .await;

    let expected_errors = iter::once(CorruptedError::NodeMissing {
        node_id: removed_node,
    })
    .chain(
        orphaned_children_nodes
            .into_iter()
            .map(|child| CorruptedError::NodeUnreferenced { node_id: child }),
    )
    .collect();

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
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

#[tokio::test(flavor = "multi_thread")]
async fn root_dir_with_missing_root_node() {
    // TODO
}

#[tokio::test(flavor = "multi_thread")]
async fn root_dir_with_missing_inner_node() {
    // TODO
}

#[tokio::test(flavor = "multi_thread")]
async fn root_dir_with_missing_leaf_node() {
    // TODO
}

#[tokio::test(flavor = "multi_thread")]
async fn root_dir_with_missing_inner_node_and_own_leaf_node() {
    // TODO
}

#[tokio::test(flavor = "multi_thread")]
async fn root_dir_with_missing_inner_node_and_foreign_leaf_node() {
    // TODO
}
