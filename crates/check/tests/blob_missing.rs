//! Tests where whole blobs are missing

use rstest::rstest;
use std::iter;

use cryfs_blockstore::RemoveResult;
use cryfs_check::{BlobReference, BlobReferenceWithId, NodeAndBlobReference, NodeMissingError};
use cryfs_utils::testutils::asserts::assert_unordered_vec_eq;

mod common;
use common::entry_helpers::{SomeBlobs, expect_blobs_to_have_unreferenced_root_nodes};
use common::fixture::FilesystemFixture;

#[rstest]
#[case::file(|some_blobs: &SomeBlobs| some_blobs.large_file_1.clone())]
#[case::dir_with_children(|some_blobs: &SomeBlobs| some_blobs.large_dir_1.clone())]
#[case::dir_without_children(|some_blobs: &SomeBlobs| some_blobs.empty_dir.clone())]
#[case::symlink(|some_blobs: &SomeBlobs| some_blobs.large_symlink_1.clone())]
#[case::rootdir_with_children(|some_blobs: &SomeBlobs| some_blobs.root.clone())]
#[tokio::test(flavor = "multi_thread")]
async fn blob_entirely_missing(#[case] blob: impl FnOnce(&SomeBlobs) -> BlobReferenceWithId) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_info = blob(&some_blobs);
    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_if_dir_blob(blob_info.blob_id)
        .await;
    let expected_errors_from_orphaned_descendant_blobs =
        expect_blobs_to_have_unreferenced_root_nodes(&fs_fixture, orphaned_descendant_blobs).await;

    fs_fixture
        .update_fsblobstore(move |fsblobstore| {
            Box::pin(async move {
                let remove_result = fsblobstore.remove_by_id(&blob_info.blob_id).await.unwrap();
                assert_eq!(RemoveResult::SuccessfullyRemoved, remove_result);
            })
        })
        .await;

    let expected_errors = iter::once(
        NodeMissingError {
            node_id: *blob_info.blob_id.to_root_block_id(),
            referenced_as: [NodeAndBlobReference::RootNode {
                belongs_to_blob: BlobReferenceWithId {
                    blob_id: blob_info.blob_id,
                    referenced_as: blob_info.referenced_as,
                },
            }]
            .into_iter()
            .collect(),
        }
        .into(),
    )
    .chain(expected_errors_from_orphaned_descendant_blobs)
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

    let expected_errors = vec![
        NodeMissingError {
            node_id: *root_dir_id.to_root_block_id(),
            referenced_as: [NodeAndBlobReference::RootNode {
                belongs_to_blob: BlobReferenceWithId {
                    blob_id: root_dir_id,
                    referenced_as: BlobReference::root_dir(),
                },
            }]
            .into_iter()
            .collect(),
        }
        .into(),
    ];

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

// Test for blob_missing and referenced_multiple_times is in [super::blob_referenced_multiple_times]
