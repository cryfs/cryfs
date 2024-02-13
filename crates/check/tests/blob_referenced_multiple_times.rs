//! Tests where a blob is referenced multiple times, either from the same or from a different directory
//! Note: Tests for the blob being referenced from an inner node is in [super::node_referenced_multiple_times::root_node_referenced]

use futures::future::BoxFuture;
use rstest::rstest;

use cryfs_blobstore::BlobId;
use cryfs_check::{
    BlobInfoAsExpectedByEntryInParent, BlobInfoAsSeenByLookingAtBlob, BlobReference,
    CorruptedError, NodeInfoAsSeenByLookingAtNode,
};
use cryfs_cryfs::filesystem::fsblobstore::BlobType;

mod common;

use common::{
    entry_helpers::{CreatedBlobInfo, SomeBlobs},
    fixture::FilesystemFixture,
};

fn make_file(
    fs_fixture: &FilesystemFixture,
    parent: CreatedBlobInfo,
) -> BoxFuture<'_, CreatedBlobInfo> {
    Box::pin(async move {
        fs_fixture
            .create_empty_file_in_parent(parent, "my_filename1")
            .await
    })
}

fn make_dir(
    fs_fixture: &FilesystemFixture,
    parent: CreatedBlobInfo,
) -> BoxFuture<'_, CreatedBlobInfo> {
    Box::pin(async move {
        fs_fixture
            .create_empty_dir_in_parent(parent, "my_dirname1")
            .await
    })
}

fn make_symlink(
    fs_fixture: &FilesystemFixture,
    parent: CreatedBlobInfo,
) -> BoxFuture<'_, CreatedBlobInfo> {
    Box::pin(async move {
        fs_fixture
            .create_symlink_in_parent(parent, "my_symlink1", "target1")
            .await
    })
}

fn add_as_file_entry<'a>(
    fs_fixture: &'a FilesystemFixture,
    parent: CreatedBlobInfo,
    blob_id: BlobId,
) -> BoxFuture<'a, BlobInfoAsExpectedByEntryInParent> {
    const NAME: &str = "my_filename2";
    Box::pin(async move {
        fs_fixture
            .add_file_entry_to_dir(parent.blob_id, NAME, blob_id)
            .await;
        BlobInfoAsExpectedByEntryInParent {
            blob_type: BlobType::File,
            parent_id: parent.blob_id,
            path: parent.blob_info.path.join(NAME.try_into().unwrap()),
        }
    })
}

fn add_as_dir_entry<'a>(
    fs_fixture: &'a FilesystemFixture,
    parent: CreatedBlobInfo,
    blob_id: BlobId,
) -> BoxFuture<'a, BlobInfoAsExpectedByEntryInParent> {
    const NAME: &str = "my_dirname2";
    Box::pin(async move {
        fs_fixture
            .add_dir_entry_to_dir(parent.blob_id, NAME, blob_id)
            .await;
        BlobInfoAsExpectedByEntryInParent {
            blob_type: BlobType::Dir,
            parent_id: parent.blob_id,
            path: parent.blob_info.path.join(NAME.try_into().unwrap()),
        }
    })
}

fn add_as_symlink_entry<'a>(
    fs_fixture: &'a FilesystemFixture,
    parent: CreatedBlobInfo,
    blob_id: BlobId,
) -> BoxFuture<'a, BlobInfoAsExpectedByEntryInParent> {
    const NAME: &str = "my_symlink2";
    Box::pin(async move {
        fs_fixture
            .add_symlink_entry_to_dir(parent.blob_id, NAME, blob_id)
            .await;
        BlobInfoAsExpectedByEntryInParent {
            blob_type: BlobType::Symlink,
            parent_id: parent.blob_id,
            path: parent.blob_info.path.join(NAME.try_into().unwrap()),
        }
    })
}

fn same_dir(some_blobs: &SomeBlobs) -> (CreatedBlobInfo, CreatedBlobInfo) {
    (
        some_blobs.large_dir_1.clone(),
        some_blobs.large_dir_1.clone(),
    )
}

fn different_dirs(some_blobs: &SomeBlobs) -> (CreatedBlobInfo, CreatedBlobInfo) {
    (
        some_blobs.large_dir_1.clone(),
        some_blobs.large_dir_2.clone(),
    )
}

#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn blob_referenced_multiple_times(
    #[values(same_dir, different_dirs)] parents: impl FnOnce(
        &SomeBlobs,
    ) -> (CreatedBlobInfo, CreatedBlobInfo),
    #[values(make_file, make_dir, make_symlink)] make_first_blob: impl for<'a> FnOnce(
        &'a FilesystemFixture,
        CreatedBlobInfo,
    ) -> BoxFuture<
        'a,
        CreatedBlobInfo,
    >,
    #[values(add_as_file_entry, add_as_dir_entry, add_as_symlink_entry)]
    add_to_second_parent: impl for<'a> FnOnce(
        &'a FilesystemFixture,
        CreatedBlobInfo,
        BlobId,
    )
        -> BoxFuture<'a, BlobInfoAsExpectedByEntryInParent>,
) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let (parent1_info, parent2_info) = parents(&some_blobs);

    let blob_info = make_first_blob(&fs_fixture, parent1_info).await;
    let second_blob_info = add_to_second_parent(&fs_fixture, parent2_info, blob_info.blob_id).await;

    let expected_depth = fs_fixture
        .get_node_depth(*blob_info.blob_id.to_root_block_id())
        .await;

    let errors = fs_fixture.run_cryfs_check().await;
    assert_eq!(
        vec![
            // TODO Do we want to report `NodeReferencedMultipleTimes` or only report `BlobReferencedMultipleTimes`?
            CorruptedError::NodeReferencedMultipleTimes {
                node_id: *blob_info.blob_id.to_root_block_id(),
                node_info: Some(NodeInfoAsSeenByLookingAtNode {
                    depth: expected_depth
                }),
            },
            CorruptedError::BlobReferencedMultipleTimes {
                blob_id: blob_info.blob_id,
                blob_info: Some(BlobInfoAsSeenByLookingAtBlob {
                    blob_type: blob_info.blob_info.blob_type,
                    parent_pointer: blob_info.blob_info.parent_id,
                }),
                referenced_as: [
                    BlobReference {
                        expected_child_info: blob_info.blob_info.clone(),
                    },
                    BlobReference {
                        expected_child_info: second_blob_info,
                    }
                ]
                .into_iter()
                .collect(),
            }
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

// - Blob referenced multiple times but doesn't actually exist
