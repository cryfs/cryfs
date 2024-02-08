//! Tests where all individual nodes are readable but there is something wrong in the blob data.

use rstest::rstest;
use std::iter;

use cryfs_blobstore::BlobId;
use cryfs_check::CorruptedError;
use cryfs_utils::testutils::asserts::assert_unordered_vec_eq;

mod common;
use common::entry_helpers::SomeBlobs;
use common::fixture::FilesystemFixture;

#[rstest]
#[case::file(|some_blobs: &SomeBlobs| some_blobs.large_file_1)]
#[case::dir(|some_blobs: &SomeBlobs| some_blobs.large_dir_1)]
#[case::symlink(|some_blobs: &SomeBlobs| some_blobs.large_symlink_1)]
#[case::rootdir(|some_blobs: &SomeBlobs| some_blobs.root)]
#[tokio::test(flavor = "multi_thread")]
async fn unreadable_blob_bad_format_version(#[case] blob_id: impl FnOnce(&SomeBlobs) -> BlobId) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_id = blob_id(&some_blobs);
    let orphaned_descendant_blobs = fs_fixture.get_descendants_if_dir_blob(blob_id).await;

    fs_fixture.increment_format_version_of_blob(blob_id).await;

    let expected_errors = iter::once(CorruptedError::BlobUnreadable { blob_id })
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
async fn unreadable_file_blob_bad_blob_type(#[case] blob_id: impl FnOnce(&SomeBlobs) -> BlobId) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_id = blob_id(&some_blobs);
    let orphaned_descendant_blobs = fs_fixture.get_descendants_if_dir_blob(blob_id).await;

    fs_fixture.corrupt_blob_type(blob_id).await;

    let expected_errors = iter::once(CorruptedError::BlobUnreadable { blob_id })
        .chain(orphaned_descendant_blobs.into_iter().map(|child| {
            CorruptedError::NodeUnreferenced {
                node_id: *child.to_root_block_id(),
            }
        }))
        .collect();

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}
