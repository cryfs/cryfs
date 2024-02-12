//! Tests where whole blobs are missing

use rstest::rstest;
use std::iter;

use cryfs_blockstore::RemoveResult;
use cryfs_check::{BlobInfo, CorruptedError};

use cryfs_utils::testutils::asserts::assert_unordered_vec_eq;

mod common;
use common::entry_helpers::SomeBlobs;
use common::fixture::FilesystemFixture;

#[rstest]
#[case::file(|some_blobs: &SomeBlobs| some_blobs.large_file_1.clone())]
#[case::dir_with_children(|some_blobs: &SomeBlobs| some_blobs.large_dir_1.clone())]
#[case::dir_without_children(|some_blobs: &SomeBlobs| some_blobs.empty_dir.clone())]
#[case::symlink(|some_blobs: &SomeBlobs| some_blobs.large_symlink_1.clone())]
#[case::rootdir_with_children(|some_blobs: &SomeBlobs| some_blobs.root.clone())]
#[tokio::test(flavor = "multi_thread")]
async fn blob_entirely_missing(#[case] blob: impl FnOnce(&SomeBlobs) -> BlobInfo) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_id = blob(&some_blobs).blob_id;
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
