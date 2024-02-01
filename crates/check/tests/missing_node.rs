//! Tests where individual nodes are missing

use rstest::rstest;
use std::iter;

use cryfs_blobstore::BlobId;
use cryfs_blockstore::RemoveResult;
use cryfs_check::CorruptedError;

use cryfs_utils::testutils::asserts::assert_unordered_vec_eq;

mod common;
use common::entry_helpers::SomeBlobs;
use common::fixture::{FilesystemFixture, RemoveInnerNodeResult, RemoveSomeNodesResult};

#[rstest]
#[case::file(|some_blobs: &SomeBlobs| some_blobs.large_file_1)]
#[case::dir_with_children(|some_blobs: &SomeBlobs| some_blobs.large_dir_1)]
#[case::dir_without_children(|some_blobs: &SomeBlobs| some_blobs.empty_dir)]
#[case::symlink(|some_blobs: &SomeBlobs| some_blobs.large_symlink_1)]
#[case::rootdir_with_children(|some_blobs: &SomeBlobs| some_blobs.root)]
#[tokio::test(flavor = "multi_thread")]
async fn blob_entirely_missing(#[case] blob_id: impl FnOnce(&SomeBlobs) -> BlobId) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_id = blob_id(&some_blobs);
    let orphaned_descendant_blobs = fs_fixture.get_descendants_if_dir_blob(blob_id).await;

    fs_fixture
        .update_fsblobstore(move |fsblobstore| {
            Box::pin(async move {
                let remove_result = fsblobstore.remove_by_id(&blob_id).await.unwrap();
                assert_eq!(RemoveResult::SuccessfullyRemoved, remove_result);
            })
        })
        .await;

    let expected_errors = iter::once(CorruptedError::BlobMissing { blob_id })
        .chain(orphaned_descendant_blobs.into_iter().map(|child| {
            CorruptedError::NodeUnreferenced {
                node_id: *child.to_root_block_id(),
            }
        }))
        .collect();

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

#[tokio::test(flavor = "multi_thread")]
async fn root_dir_entirely_missing_without_children() {
    let fs_fixture = FilesystemFixture::new().await;
    let root_dir_id = fs_fixture.root_blob_id();

    let orphaned_descendant_blobs = fs_fixture.get_descendants_of_dir_blob(root_dir_id).await;
    assert_eq!(
        0,
        orphaned_descendant_blobs.len(),
        "Test precondition failed"
    );

    fs_fixture
        .update_fsblobstore(|fsblobstore| {
            Box::pin(async move {
                let remove_result = fsblobstore.remove_by_id(&root_dir_id).await.unwrap();
                assert_eq!(RemoveResult::SuccessfullyRemoved, remove_result);
            })
        })
        .await;

    let expected_errors = vec![CorruptedError::BlobMissing {
        blob_id: root_dir_id,
    }];

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

#[rstest]
#[case::file(|some_blobs: &SomeBlobs| some_blobs.large_file_1)]
#[case::dir(|some_blobs: &SomeBlobs| some_blobs.large_dir_1)]
#[case::symlink(|some_blobs: &SomeBlobs| some_blobs.large_symlink_1)]
#[case::rootdir(|some_blobs: &SomeBlobs| some_blobs.root)]
#[tokio::test(flavor = "multi_thread")]
async fn blob_with_missing_root_node(#[case] blob_id: impl FnOnce(&SomeBlobs) -> BlobId) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_id = blob_id(&some_blobs);

    let orphaned_descendant_blobs = fs_fixture.get_descendants_if_dir_blob(blob_id).await;

    let RemoveInnerNodeResult {
        removed_node,
        orphaned_nodes,
    } = fs_fixture.remove_root_node_of_blob(blob_id).await;

    let expected_errors =
        iter::once(CorruptedError::BlobMissing {
            blob_id: BlobId::from_root_block_id(removed_node),
        })
        .chain(
            orphaned_nodes
                .into_iter()
                .map(|child| CorruptedError::NodeUnreferenced { node_id: child }),
        )
        .chain(orphaned_descendant_blobs.into_iter().map(|child| {
            CorruptedError::NodeUnreferenced {
                node_id: *child.to_root_block_id(),
            }
        }))
        .collect();

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

#[rstest]
#[case::file(|some_blobs: &SomeBlobs| some_blobs.large_file_1)]
#[case::dir(|some_blobs: &SomeBlobs| some_blobs.large_dir_1)]
#[case::symlink(|some_blobs: &SomeBlobs| some_blobs.large_symlink_1)]
#[case::rootdir(|some_blobs: &SomeBlobs| some_blobs.root)]
#[tokio::test(flavor = "multi_thread")]
async fn blob_with_missing_inner_node(#[case] blob_id: impl FnOnce(&SomeBlobs) -> BlobId) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_id = blob_id(&some_blobs);
    if blob_id == some_blobs.root {
        // If we're testing the root dir, we need to make it large enough that we can remove some nodes
        fs_fixture
            .add_entries_to_make_dir_large(some_blobs.root)
            .await;
    }

    let orphaned_descendant_blobs = fs_fixture.get_descendants_if_dir_blob(blob_id).await;
    let RemoveInnerNodeResult {
        removed_node,
        orphaned_nodes,
    } = fs_fixture
        .remove_an_inner_node_of_a_large_blob(blob_id)
        .await;

    let mut expected_errors = vec![CorruptedError::NodeMissing {
        node_id: removed_node,
    }];
    if !orphaned_descendant_blobs.is_empty() {
        // It was a dir. Dirs are reported as unreadable because we try to read them when checking the file system.
        expected_errors.push(CorruptedError::BlobUnreadable { blob_id });
    }
    expected_errors.extend(
        orphaned_nodes
            .into_iter()
            .map(|child| CorruptedError::NodeUnreferenced { node_id: child })
            .chain(orphaned_descendant_blobs.into_iter().map(|child| {
                CorruptedError::NodeUnreferenced {
                    node_id: *child.to_root_block_id(),
                }
            })),
    );

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

#[rstest]
#[case::file(|some_blobs: &SomeBlobs| some_blobs.large_file_1)]
#[case::dir(|some_blobs: &SomeBlobs| some_blobs.large_dir_1)]
#[case::symlink(|some_blobs: &SomeBlobs| some_blobs.large_symlink_1)]
#[case::rootdir(|some_blobs: &SomeBlobs| some_blobs.root)]
#[tokio::test(flavor = "multi_thread")]
async fn blob_with_missing_leaf_node(#[case] blob_id: impl FnOnce(&SomeBlobs) -> BlobId) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_id = blob_id(&some_blobs);

    let orphaned_descendant_blobs = fs_fixture.get_descendants_if_dir_blob(blob_id).await;

    let removed_node = fs_fixture.remove_a_leaf_node(blob_id).await;

    let mut expected_errors = vec![CorruptedError::NodeMissing {
        node_id: removed_node,
    }];
    if !orphaned_descendant_blobs.is_empty() {
        // It was a dir. Dirs are reported as unreadable because we try to read them when checking the file system.
        expected_errors.push(CorruptedError::BlobUnreadable { blob_id });
    }
    expected_errors.extend(orphaned_descendant_blobs.into_iter().map(|child| {
        CorruptedError::NodeUnreferenced {
            node_id: *child.to_root_block_id(),
        }
    }));

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

#[rstest]
#[case::file(|some_blobs: &SomeBlobs| some_blobs.large_file_1)]
#[case::dir(|some_blobs: &SomeBlobs| some_blobs.large_dir_1)]
#[case::symlink(|some_blobs: &SomeBlobs| some_blobs.large_symlink_1)]
#[case::rootdir(|some_blobs: &SomeBlobs| some_blobs.root)]
#[tokio::test(flavor = "multi_thread")]
async fn blob_with_missing_some_nodes(#[case] blob_id: impl FnOnce(&SomeBlobs) -> BlobId) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_id = blob_id(&some_blobs);
    if blob_id == some_blobs.root {
        // If we're testing the root dir, we need to make it large enough that we can remove some nodes
        fs_fixture
            .add_entries_to_make_dir_large(some_blobs.root)
            .await;
    }

    let orphaned_descendant_blobs = fs_fixture.get_descendants_if_dir_blob(blob_id).await;

    let RemoveSomeNodesResult {
        removed_nodes,
        orphaned_nodes,
    } = fs_fixture.remove_some_nodes_of_a_large_blob(blob_id).await;

    let mut expected_errors = vec![];
    if !orphaned_descendant_blobs.is_empty() {
        // It was a dir. Dirs are reported as unreadable because we try to read them when checking the file system.
        expected_errors.push(CorruptedError::BlobUnreadable { blob_id });
    }
    expected_errors.extend(
        removed_nodes
            .into_iter()
            .map(|node_id| CorruptedError::NodeMissing { node_id })
            .chain(
                orphaned_nodes
                    .into_iter()
                    .map(|child| CorruptedError::NodeUnreferenced { node_id: child }),
            )
            .chain(orphaned_descendant_blobs.into_iter().map(|child| {
                CorruptedError::NodeUnreferenced {
                    node_id: *child.to_root_block_id(),
                }
            })),
    );

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}
