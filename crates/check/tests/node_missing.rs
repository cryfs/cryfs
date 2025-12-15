//! Tests where individual nodes in a blob are missing

use rstest::rstest;
use std::iter;

use cryfs_check::{
    BlobReferenceWithId, BlobUnreadableError, NodeAndBlobReference, NodeMissingError,
};
use cryfs_fsblobstore::fsblobstore::BlobType;
use cryfs_utils::testutils::asserts::assert_unordered_vec_eq;

mod common;
use common::entry_helpers::{
    SomeBlobs, expect_blobs_to_have_unreferenced_root_nodes, expect_nodes_to_be_unreferenced,
};
use common::fixture::{FilesystemFixture, RemoveInnerNodeResult, RemoveSomeNodesResult};

#[rstest]
#[case::file(|some_blobs: &SomeBlobs| some_blobs.large_file_1.clone())]
#[case::dir(|some_blobs: &SomeBlobs| some_blobs.large_dir_1.clone())]
#[case::symlink(|some_blobs: &SomeBlobs| some_blobs.large_symlink_1.clone())]
#[case::rootdir(|some_blobs: &SomeBlobs| some_blobs.root.clone())]
#[tokio::test(flavor = "multi_thread")]
async fn blob_with_missing_root_node(#[case] blob: impl FnOnce(&SomeBlobs) -> BlobReferenceWithId) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_info = blob(&some_blobs);

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_if_dir_blob(blob_info.blob_id)
        .await;
    let expected_errors_from_orphaned_descendant_blobs =
        expect_blobs_to_have_unreferenced_root_nodes(&fs_fixture, orphaned_descendant_blobs).await;

    let RemoveInnerNodeResult {
        removed_node,
        removed_node_info: _,
        orphaned_nodes,
    } = fs_fixture.remove_root_node_of_blob(blob_info.clone()).await;
    assert_eq!(
        removed_node,
        *blob_info.blob_id.to_root_block_id(),
        "test invariant"
    );
    let expected_errors_from_orphaned_nodes =
        expect_nodes_to_be_unreferenced(&fs_fixture, orphaned_nodes).await;

    let expected_errors = iter::once(
        NodeMissingError {
            node_id: *blob_info.blob_id.to_root_block_id(),
            referenced_as: [NodeAndBlobReference::RootNode {
                belongs_to_blob: BlobReferenceWithId {
                    blob_id: blob_info.blob_id,
                    referenced_as: blob_info.referenced_as,
                },
            }
            .into()]
            .into_iter()
            .collect(),
        }
        .into(),
    )
    .chain(expected_errors_from_orphaned_nodes)
    .chain(expected_errors_from_orphaned_descendant_blobs)
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
async fn blob_with_missing_inner_node(
    #[case] blob: impl FnOnce(&SomeBlobs) -> BlobReferenceWithId,
) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_info = blob(&some_blobs);
    if blob_info.blob_id == some_blobs.root.blob_id {
        // If we're testing the root dir, we need to make it large enough that we can remove some nodes
        fs_fixture
            .add_entries_to_make_dir_large(blob_info.clone())
            .await;
    }

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_if_dir_blob(blob_info.blob_id)
        .await;
    assert_eq!(
        orphaned_descendant_blobs.is_empty(),
        blob_info.referenced_as.blob_type != BlobType::Dir,
        "test invariant"
    );
    let expected_errors_from_orphaned_descendant_blobs =
        expect_blobs_to_have_unreferenced_root_nodes(&fs_fixture, orphaned_descendant_blobs).await;

    let RemoveInnerNodeResult {
        removed_node,
        removed_node_info,
        orphaned_nodes,
    } = fs_fixture
        .remove_an_inner_node_of_a_large_blob(blob_info.clone())
        .await;
    let expected_errors_from_orphaned_nodes =
        expect_nodes_to_be_unreferenced(&fs_fixture, orphaned_nodes).await;

    let mut expected_errors = vec![
        NodeMissingError {
            node_id: removed_node,
            referenced_as: [removed_node_info.into()].into_iter().collect(),
        }
        .into(),
    ];
    if blob_info.referenced_as.blob_type == BlobType::Dir {
        // Dirs are reported as unreadable because we try to read them when checking the file system.
        expected_errors.push(
            BlobUnreadableError {
                blob_id: blob_info.blob_id,
                referenced_as: [blob_info.referenced_as].into_iter().collect(),
            }
            .into(),
        );
    }
    expected_errors.extend(
        expected_errors_from_orphaned_nodes.chain(expected_errors_from_orphaned_descendant_blobs),
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
async fn blob_with_missing_leaf_node(#[case] blob: impl FnOnce(&SomeBlobs) -> BlobReferenceWithId) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_info = blob(&some_blobs);

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_if_dir_blob(blob_info.blob_id)
        .await;
    assert_eq!(
        orphaned_descendant_blobs.is_empty(),
        blob_info.referenced_as.blob_type != BlobType::Dir,
        "test invariant"
    );
    let expected_errors_from_orphaned_descendant_blobs =
        expect_blobs_to_have_unreferenced_root_nodes(&fs_fixture, orphaned_descendant_blobs).await;

    let removed_node = fs_fixture.remove_a_leaf_node(blob_info.clone()).await;

    let mut expected_errors = vec![
        NodeMissingError {
            node_id: removed_node.removed_node,
            referenced_as: [removed_node.removed_node_info.into()]
                .into_iter()
                .collect(),
        }
        .into(),
    ];
    if blob_info.referenced_as.blob_type == BlobType::Dir {
        // Dirs are reported as unreadable because we try to read them when checking the file system.
        expected_errors.push(
            BlobUnreadableError {
                blob_id: blob_info.blob_id,
                referenced_as: [blob_info.referenced_as].into_iter().collect(),
            }
            .into(),
        );
    }
    expected_errors.extend(expected_errors_from_orphaned_descendant_blobs);

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

#[rstest]
#[case::file(|some_blobs: &SomeBlobs| some_blobs.large_file_1.clone())]
#[case::dir(|some_blobs: &SomeBlobs| some_blobs.large_dir_1.clone())]
#[case::symlink(|some_blobs: &SomeBlobs| some_blobs.large_symlink_1.clone())]
#[case::rootdir(|some_blobs: &SomeBlobs| some_blobs.root.clone())]
#[tokio::test(flavor = "multi_thread")]
async fn blob_with_missing_some_nodes(
    #[case] blob: impl FnOnce(&SomeBlobs) -> BlobReferenceWithId,
) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_info = blob(&some_blobs);
    if blob_info.blob_id == some_blobs.root.blob_id {
        // If we're testing the root dir, we need to make it large enough that we can remove some nodes
        fs_fixture
            .add_entries_to_make_dir_large(blob_info.clone())
            .await;
    }

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_if_dir_blob(blob_info.blob_id)
        .await;
    assert_eq!(
        orphaned_descendant_blobs.is_empty(),
        blob_info.referenced_as.blob_type != BlobType::Dir,
        "test invariant"
    );
    let expected_errors_from_orphaned_descendant_blobs =
        expect_blobs_to_have_unreferenced_root_nodes(&fs_fixture, orphaned_descendant_blobs).await;

    let RemoveSomeNodesResult {
        removed_nodes,
        orphaned_nodes,
    } = fs_fixture
        .remove_some_nodes_of_a_large_blob(blob_info.clone())
        .await;
    let expected_errors_from_orphaned_nodes =
        expect_nodes_to_be_unreferenced(&fs_fixture, orphaned_nodes).await;

    let mut expected_errors = vec![];
    if blob_info.referenced_as.blob_type == BlobType::Dir {
        // Dirs are reported as unreadable because we try to read them when checking the file system.
        expected_errors.push(
            BlobUnreadableError {
                blob_id: blob_info.blob_id,
                referenced_as: [blob_info.referenced_as].into_iter().collect(),
            }
            .into(),
        );
    }
    expected_errors.extend(
        removed_nodes
            .into_iter()
            .map(|(node_id, referenced_as)| {
                NodeMissingError {
                    node_id,
                    referenced_as: [referenced_as].into_iter().collect(),
                }
                .into()
            })
            .chain(expected_errors_from_orphaned_nodes)
            .chain(expected_errors_from_orphaned_descendant_blobs),
    );

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

// TODO Test NodeMissing with multiple referenced_as
