//! Tests where all individual nodes are readable but there is something wrong in the blob data.

use cryfs_check::CorruptedError;
use cryfs_utils::testutils::asserts::assert_unordered_vec_eq;

mod common;
use common::fixture::FilesystemFixture;

#[tokio::test(flavor = "multi_thread")]
async fn unreadable_file_blob_bad_format_version() {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;

    fs_fixture
        .increment_format_version_of_blob(some_blobs.large_file_1)
        .await;

    let expected_errors = vec![CorruptedError::BlobUnreadable {
        blob_id: some_blobs.large_file_1,
    }];

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

// TODO
// #[tokio::test(flavor = "multi_thread")]
// async fn unreadable_file_blob_bad_blob_type() {
//     let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;

//     fs_fixture.corrupt_blob_type(some_blobs.large_file_1).await;

//     let expected_errors = vec![CorruptedError::BlobUnreadable {
//         blob_id: some_blobs.large_file_1,
//     }];

//     let errors = fs_fixture.run_cryfs_check().await;
//     assert_unordered_vec_eq(expected_errors, errors);
// }

#[tokio::test(flavor = "multi_thread")]
async fn unreadable_dir_blob_bad_format_version() {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_of_dir_blob(some_blobs.large_dir_1)
        .await;
    fs_fixture
        .increment_format_version_of_blob(some_blobs.large_dir_1)
        .await;

    let expected_errors =
        [CorruptedError::BlobUnreadable {
            blob_id: some_blobs.large_dir_1,
        }]
        .into_iter()
        .chain(orphaned_descendant_blobs.into_iter().map(|child| {
            CorruptedError::NodeUnreferenced {
                node_id: *child.to_root_block_id(),
            }
        }))
        .collect();

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

// TODO
// #[tokio::test(flavor = "multi_thread")]
// async fn unreadable_dir_blob_bad_blob_type() {
//     let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;

//     let orphaned_descendant_blobs = fs_fixture
//         .get_descendants_of_dir_blob(some_blobs.large_dir_1)
//         .await;
//     fs_fixture.corrupt_blob_type(some_blobs.large_dir_1).await;

//     let expected_errors =
//         [CorruptedError::BlobUnreadable {
//             blob_id: some_blobs.large_dir_1,
//         }]
//         .into_iter()
//         .chain(orphaned_descendant_blobs.into_iter().map(|child| {
//             CorruptedError::NodeUnreferenced {
//                 node_id: *child.to_root_block_id(),
//             }
//         }))
//         .collect();

//     let errors = fs_fixture.run_cryfs_check().await;
//     assert_unordered_vec_eq(expected_errors, errors);
// }

#[tokio::test(flavor = "multi_thread")]
async fn unreadable_symlink_blob_bad_format_version() {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;

    fs_fixture
        .increment_format_version_of_blob(some_blobs.large_symlink_1)
        .await;

    let expected_errors = vec![CorruptedError::BlobUnreadable {
        blob_id: some_blobs.large_symlink_1,
    }];

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

// TODO
// #[tokio::test(flavor = "multi_thread")]
// async fn unreadable_symlink_blob_bad_blob_type() {
//     let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;

//     fs_fixture.corrupt_blob_type(some_blobs.large_symlink_1).await;

//     let expected_errors = vec![CorruptedError::BlobUnreadable {
//         blob_id: some_blobs.large_symlink_1,
//     }];

//     let errors = fs_fixture.run_cryfs_check().await;
//     assert_unordered_vec_eq(expected_errors, errors);
// }
