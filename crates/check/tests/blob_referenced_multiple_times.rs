//! Tests where a blob is referenced multiple times, either from the same or from a different directory
//! Note: Tests for the blob being referenced from an inner node is in [super::node_referenced_multiple_times::root_node_referenced]

use futures::future::BoxFuture;
use rstest::rstest;

use cryfs_blobstore::BlobId;
use cryfs_check::{BlobInfo, CorruptedError};

mod common;

use common::{entry_helpers::SomeBlobs, fixture::FilesystemFixture};

fn make_file(fs_fixture: &FilesystemFixture, parent: BlobId) -> BoxFuture<'_, BlobId> {
    Box::pin(async move {
        fs_fixture
            .create_empty_file_in_parent(parent, "my_filename1")
            .await
    })
}

fn make_dir(fs_fixture: &FilesystemFixture, parent: BlobId) -> BoxFuture<'_, BlobId> {
    Box::pin(async move {
        fs_fixture
            .create_empty_dir_in_parent(parent, "my_dirname1")
            .await
    })
}

fn make_symlink(fs_fixture: &FilesystemFixture, parent: BlobId) -> BoxFuture<'_, BlobId> {
    Box::pin(async move {
        fs_fixture
            .create_symlink_in_parent(parent, "my_symlink1", "target1")
            .await
    })
}

fn add_as_file_entry<'a>(
    fs_fixture: &'a FilesystemFixture,
    parent: BlobId,
    blob_id: BlobId,
) -> BoxFuture<'a, ()> {
    Box::pin(async move {
        fs_fixture
            .add_file_entry_to_dir(parent, "my_filename2", blob_id)
            .await;
    })
}

fn add_as_dir_entry<'a>(
    fs_fixture: &'a FilesystemFixture,
    parent: BlobId,
    blob_id: BlobId,
) -> BoxFuture<'a, ()> {
    Box::pin(async move {
        fs_fixture
            .add_dir_entry_to_dir(parent, "my_dirname2", blob_id)
            .await;
    })
}

fn add_as_symlink_entry<'a>(
    fs_fixture: &'a FilesystemFixture,
    parent: BlobId,
    blob_id: BlobId,
) -> BoxFuture<'a, ()> {
    Box::pin(async move {
        fs_fixture
            .add_symlink_entry_to_dir(parent, "my_symlink2", blob_id)
            .await;
    })
}

fn same_dir(some_blobs: &SomeBlobs) -> (BlobInfo, BlobInfo) {
    (
        some_blobs.large_dir_1.clone(),
        some_blobs.large_dir_1.clone(),
    )
}

fn different_dirs(some_blobs: &SomeBlobs) -> (BlobInfo, BlobInfo) {
    (
        some_blobs.large_dir_1.clone(),
        some_blobs.large_dir_2.clone(),
    )
}

#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn blob_referenced_multiple_times(
    #[values(same_dir, different_dirs)] parents: impl FnOnce(&SomeBlobs) -> (BlobInfo, BlobInfo),
    #[values(make_file, make_dir, make_symlink)] make_first_blob: impl for<'a> FnOnce(
        &'a FilesystemFixture,
        BlobId,
    ) -> BoxFuture<
        'a,
        BlobId,
    >,
    #[values(add_as_file_entry, add_as_dir_entry, add_as_symlink_entry)]
    add_to_second_parent: impl for<'a> FnOnce(
        &'a FilesystemFixture,
        BlobId,
        BlobId,
    ) -> BoxFuture<'a, ()>,
) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let (parent1_info, parent2_info) = parents(&some_blobs);
    let parent1_id = parent1_info.blob_id;
    let parent2_id = parent2_info.blob_id;

    let blob_id = make_first_blob(&fs_fixture, parent1_id).await;
    add_to_second_parent(&fs_fixture, parent2_id, blob_id).await;

    let errors = fs_fixture.run_cryfs_check().await;
    assert_eq!(
        vec![
            // TODO Do we want to report `NodeReferencedMultipleTimes` or only report `BlobReferencedMultipleTimes`?
            CorruptedError::NodeReferencedMultipleTimes {
                node_id: *blob_id.to_root_block_id()
            },
            CorruptedError::BlobReferencedMultipleTimes { blob_id }
        ],
        errors,
    );
}

// TODO Test
//  - dir blob referenced from child dir
//  - dir blob referenced from grandchild dir
//  - dir blob referenced from parent dir (i.e. 2x from the same dir)
//  - dir blob referenced from grandparent dir
//  - file blob referenced from parent dir (i.e. 2x from the same dir)
//  - file blob referenced from grandparent dir
//  - symlink blob referenced from parent dir (i.e. 2x from the same dir)
//  - symlink blob referenced from grandparent dir
