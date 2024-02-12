//! Tests where individual nodes in a blob are missing

use rstest::rstest;
use std::iter;

use cryfs_blobstore::BlobId;
use cryfs_check::{BlobInfo, CorruptedError};
use cryfs_cryfs::filesystem::fsblobstore::BlobType;
use cryfs_utils::testutils::asserts::assert_unordered_vec_eq;

mod common;
use common::entry_helpers::SomeBlobs;
use common::fixture::{FilesystemFixture, RemoveInnerNodeResult, RemoveSomeNodesResult};

#[rstest]
#[case::file(|some_blobs: &SomeBlobs| some_blobs.large_file_1.clone())]
#[case::dir(|some_blobs: &SomeBlobs| some_blobs.large_dir_1.clone())]
#[case::symlink(|some_blobs: &SomeBlobs| some_blobs.large_symlink_1.clone())]
#[case::rootdir(|some_blobs: &SomeBlobs| some_blobs.root.clone())]
#[tokio::test(flavor = "multi_thread")]
async fn blob_with_missing_root_node(#[case] blob: impl FnOnce(&SomeBlobs) -> BlobInfo) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_id = blob(&some_blobs).blob_id;

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
#[case::file(|some_blobs: &SomeBlobs| some_blobs.large_file_1.clone())]
#[case::dir(|some_blobs: &SomeBlobs| some_blobs.large_dir_1.clone())]
#[case::symlink(|some_blobs: &SomeBlobs| some_blobs.large_symlink_1.clone())]
#[case::rootdir(|some_blobs: &SomeBlobs| some_blobs.root.clone())]
#[tokio::test(flavor = "multi_thread")]
async fn blob_with_missing_inner_node(#[case] blob: impl FnOnce(&SomeBlobs) -> BlobInfo) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_info = blob(&some_blobs);
    if blob_info.blob_id == some_blobs.root.blob_id {
        // If we're testing the root dir, we need to make it large enough that we can remove some nodes
        fs_fixture
            .add_entries_to_make_dir_large(blob_info.blob_id)
            .await;
    }

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_if_dir_blob(blob_info.blob_id)
        .await;
    let RemoveInnerNodeResult {
        removed_node,
        orphaned_nodes,
    } = fs_fixture
        .remove_an_inner_node_of_a_large_blob(blob_info.blob_id)
        .await;
    assert_eq!(
        orphaned_descendant_blobs.is_empty(),
        blob_info.blob_type != BlobType::Dir,
        "test invariant"
    );

    let mut expected_errors = vec![CorruptedError::NodeMissing {
        node_id: removed_node,
    }];
    if blob_info.blob_type == BlobType::Dir {
        // Dirs are reported as unreadable because we try to read them when checking the file system.
        expected_errors.push(CorruptedError::BlobUnreadable {
            expected_blob_info: blob_info,
        });
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
#[case::file(|some_blobs: &SomeBlobs| some_blobs.large_file_1.clone())]
#[case::dir(|some_blobs: &SomeBlobs| some_blobs.large_dir_1.clone())]
#[case::symlink(|some_blobs: &SomeBlobs| some_blobs.large_symlink_1.clone())]
#[case::rootdir(|some_blobs: &SomeBlobs| some_blobs.root.clone())]
#[tokio::test(flavor = "multi_thread")]
async fn blob_with_missing_leaf_node(#[case] blob: impl FnOnce(&SomeBlobs) -> BlobInfo) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_info = blob(&some_blobs);

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_if_dir_blob(blob_info.blob_id)
        .await;
    assert_eq!(
        orphaned_descendant_blobs.is_empty(),
        blob_info.blob_type != BlobType::Dir,
        "test invariant"
    );

    let removed_node = fs_fixture.remove_a_leaf_node(blob_info.blob_id).await;

    let mut expected_errors = vec![CorruptedError::NodeMissing {
        node_id: removed_node,
    }];
    if blob_info.blob_type == BlobType::Dir {
        // Dirs are reported as unreadable because we try to read them when checking the file system.
        expected_errors.push(CorruptedError::BlobUnreadable {
            expected_blob_info: blob_info,
        });
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
#[case::file(|some_blobs: &SomeBlobs| some_blobs.large_file_1.clone())]
#[case::dir(|some_blobs: &SomeBlobs| some_blobs.large_dir_1.clone())]
#[case::symlink(|some_blobs: &SomeBlobs| some_blobs.large_symlink_1.clone())]
#[case::rootdir(|some_blobs: &SomeBlobs| some_blobs.root.clone())]
#[tokio::test(flavor = "multi_thread")]
async fn blob_with_missing_some_nodes(#[case] blob: impl FnOnce(&SomeBlobs) -> BlobInfo) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_info = blob(&some_blobs);
    if blob_info.blob_id == some_blobs.root.blob_id {
        // If we're testing the root dir, we need to make it large enough that we can remove some nodes
        fs_fixture
            .add_entries_to_make_dir_large(blob_info.blob_id)
            .await;
    }

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_if_dir_blob(blob_info.blob_id)
        .await;

    let RemoveSomeNodesResult {
        removed_nodes,
        orphaned_nodes,
    } = fs_fixture
        .remove_some_nodes_of_a_large_blob(blob_info.blob_id)
        .await;
    assert_eq!(
        orphaned_descendant_blobs.is_empty(),
        blob_info.blob_type != BlobType::Dir,
        "test invariant"
    );

    let mut expected_errors = vec![];
    if blob_info.blob_type == BlobType::Dir {
        // Dirs are reported as unreadable because we try to read them when checking the file system.
        expected_errors.push(CorruptedError::BlobUnreadable {
            expected_blob_info: blob_info,
        });
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
