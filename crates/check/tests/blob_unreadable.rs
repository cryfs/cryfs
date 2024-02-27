//! Tests where all individual nodes are readable but there is something wrong in the blob data.

use rstest::rstest;
use std::iter;

use cryfs_check::{BlobReferenceWithId, CorruptedError};
use cryfs_utils::testutils::asserts::assert_unordered_vec_eq;

mod common;
use common::entry_helpers::{expect_blobs_to_have_unreferenced_root_nodes, SomeBlobs};
use common::fixture::FilesystemFixture;

#[rstest]
#[case::file(|some_blobs: &SomeBlobs| some_blobs.large_file_1.clone())]
#[case::dir(|some_blobs: &SomeBlobs| some_blobs.large_dir_1.clone())]
#[case::symlink(|some_blobs: &SomeBlobs| some_blobs.large_symlink_1.clone())]
#[case::rootdir(|some_blobs: &SomeBlobs| some_blobs.root.clone())]
#[tokio::test(flavor = "multi_thread")]
async fn unreadable_blob_bad_format_version(
    #[case] blob: impl FnOnce(&SomeBlobs) -> BlobReferenceWithId,
) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_info = blob(&some_blobs);
    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_if_dir_blob(blob_info.blob_id)
        .await;
    let expected_errors_from_orphaned_descendant_blobs =
        expect_blobs_to_have_unreferenced_root_nodes(&fs_fixture, orphaned_descendant_blobs).await;

    fs_fixture
        .increment_format_version_of_blob(blob_info.blob_id)
        .await;

    let expected_errors = iter::once(CorruptedError::BlobUnreadable {
        blob_id: blob_info.blob_id,
        referenced_as: blob_info.referenced_as,
    })
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
async fn unreadable_file_blob_bad_blob_type(
    #[case] blob: impl FnOnce(&SomeBlobs) -> BlobReferenceWithId,
) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_info = blob(&some_blobs);
    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_if_dir_blob(blob_info.blob_id)
        .await;
    let expected_errors_from_orphaned_descendant_blobs =
        expect_blobs_to_have_unreferenced_root_nodes(&fs_fixture, orphaned_descendant_blobs).await;

    fs_fixture.corrupt_blob_type(blob_info.blob_id).await;

    let expected_errors = iter::once(CorruptedError::BlobUnreadable {
        blob_id: blob_info.blob_id,
        referenced_as: blob_info.referenced_as,
    })
    .chain(expected_errors_from_orphaned_descendant_blobs)
    .collect();

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}
