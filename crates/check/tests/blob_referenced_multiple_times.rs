//! Tests where a blob is referenced multiple times, either from the same or from a different directory
//! Note: Tests for the blob being referenced from an inner node is in [super::node_referenced_multiple_times::root_node_referenced]

use futures::future::BoxFuture;
use rstest::rstest;
use std::collections::BTreeSet;
use std::num::NonZeroU8;

use cryfs_blobstore::BlobId;
use cryfs_check::{
    BlobReference, BlobReferenceWithId, BlobReferencedMultipleTimesError, BlobUnreadableError,
    CorruptedError, MaybeBlobInfoAsSeenByLookingAtBlob, MaybeNodeInfoAsSeenByLookingAtNode,
    NodeAndBlobReference, NodeMissingError, NodeReferencedMultipleTimesError, NodeUnreadableError,
};
use cryfs_filesystem::filesystem::fsblobstore::BlobType;
use cryfs_utils::testutils::asserts::assert_unordered_vec_eq;

mod common;

use common::{entry_helpers::SomeBlobs, fixture::FilesystemFixture};

fn make_file(
    fs_fixture: &FilesystemFixture,
    parent: BlobReferenceWithId,
) -> BoxFuture<'_, BlobReferenceWithId> {
    Box::pin(async move {
        fs_fixture
            .create_empty_file_in_parent(parent, "my_filename1")
            .await
    })
}

fn make_dir(
    fs_fixture: &FilesystemFixture,
    parent: BlobReferenceWithId,
) -> BoxFuture<'_, BlobReferenceWithId> {
    Box::pin(async move {
        fs_fixture
            .create_empty_dir_in_parent(parent, "my_dirname1")
            .await
    })
}

fn make_symlink(
    fs_fixture: &FilesystemFixture,
    parent: BlobReferenceWithId,
) -> BoxFuture<'_, BlobReferenceWithId> {
    Box::pin(async move {
        fs_fixture
            .create_symlink_in_parent(parent, "my_symlink1", "target1")
            .await
    })
}

fn add_as_file_entry<'a>(
    fs_fixture: &'a FilesystemFixture,
    parent: BlobReferenceWithId,
    blob_id: BlobId,
) -> BoxFuture<'a, BlobReference> {
    const NAME: &str = "my_filename2";
    Box::pin(async move {
        fs_fixture
            .add_file_entry_to_dir(parent.blob_id, NAME, blob_id)
            .await;
        BlobReference {
            blob_type: BlobType::File,
            parent_id: parent.blob_id,
            path: parent.referenced_as.path.join(NAME.try_into().unwrap()),
        }
    })
}

fn add_as_dir_entry<'a>(
    fs_fixture: &'a FilesystemFixture,
    parent: BlobReferenceWithId,
    blob_id: BlobId,
) -> BoxFuture<'a, BlobReference> {
    const NAME: &str = "my_dirname2";
    Box::pin(async move {
        fs_fixture
            .add_dir_entry_to_dir(parent.blob_id, NAME, blob_id)
            .await;
        BlobReference {
            blob_type: BlobType::Dir,
            parent_id: parent.blob_id,
            path: parent.referenced_as.path.join(NAME.try_into().unwrap()),
        }
    })
}

fn add_as_symlink_entry<'a>(
    fs_fixture: &'a FilesystemFixture,
    parent: BlobReferenceWithId,
    blob_id: BlobId,
) -> BoxFuture<'a, BlobReference> {
    const NAME: &str = "my_symlink2";
    Box::pin(async move {
        fs_fixture
            .add_symlink_entry_to_dir(parent.blob_id, NAME, blob_id)
            .await;
        BlobReference {
            blob_type: BlobType::Symlink,
            parent_id: parent.blob_id,
            path: parent.referenced_as.path.join(NAME.try_into().unwrap()),
        }
    })
}

fn same_dir(some_blobs: &SomeBlobs) -> (BlobReferenceWithId, BlobReferenceWithId) {
    (
        some_blobs.large_dir_1.clone(),
        some_blobs.large_dir_1.clone(),
    )
}

fn different_dirs(some_blobs: &SomeBlobs) -> (BlobReferenceWithId, BlobReferenceWithId) {
    (
        some_blobs.large_dir_1.clone(),
        some_blobs.large_dir_2.clone(),
    )
}

enum BlobStatus {
    Readable,
    Unreadable,
    Missing,
}

#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn blob_referenced_multiple_times(
    #[values(same_dir, different_dirs)] parents: impl FnOnce(
        &SomeBlobs,
    ) -> (
        BlobReferenceWithId,
        BlobReferenceWithId,
    ),
    #[values(make_file, make_dir, make_symlink)] make_first_blob: impl for<'a> FnOnce(
        &'a FilesystemFixture,
        BlobReferenceWithId,
    ) -> BoxFuture<
        'a,
        BlobReferenceWithId,
    >,
    #[values(add_as_file_entry, add_as_dir_entry, add_as_symlink_entry)]
    add_to_second_parent: impl for<'a> FnOnce(
        &'a FilesystemFixture,
        BlobReferenceWithId,
        BlobId,
    ) -> BoxFuture<'a, BlobReference>,
    #[values(BlobStatus::Readable, BlobStatus::Unreadable, BlobStatus::Missing)]
    blob_status: BlobStatus,
) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let (parent1_info, parent2_info) = parents(&some_blobs);

    let blob_info = make_first_blob(&fs_fixture, parent1_info).await;

    let expected_depth = fs_fixture
        .get_node_depth(*blob_info.blob_id.to_root_block_id())
        .await;

    let second_blob_info = add_to_second_parent(&fs_fixture, parent2_info, blob_info.blob_id).await;

    let expected_blob_referenced_as: BTreeSet<_> =
        [blob_info.referenced_as.clone(), second_blob_info.clone()]
            .into_iter()
            .collect();
    let expected_node_referenced_as: BTreeSet<_> = [
        NodeAndBlobReference::RootNode {
            belongs_to_blob: BlobReferenceWithId {
                blob_id: blob_info.blob_id,
                referenced_as: blob_info.referenced_as.clone(),
            },
        },
        NodeAndBlobReference::RootNode {
            belongs_to_blob: BlobReferenceWithId {
                blob_id: blob_info.blob_id,
                referenced_as: second_blob_info.clone(),
            },
        },
    ]
    .into_iter()
    .collect();

    let mut expected_errors = vec![];

    // Depending on whether the blob_status is readable,unreadable or missing, set up the file system correspondingly.
    let (expected_node_info, expected_blob_info) = match blob_status {
        BlobStatus::Readable => {
            let node_info = if let Some(depth) = NonZeroU8::new(expected_depth) {
                MaybeNodeInfoAsSeenByLookingAtNode::InnerNode { depth }
            } else {
                MaybeNodeInfoAsSeenByLookingAtNode::LeafNode
            };
            let blob_info = MaybeBlobInfoAsSeenByLookingAtBlob::Readable {
                blob_type: blob_info.referenced_as.blob_type,
                parent_pointer: blob_info.referenced_as.parent_id,
            };
            (node_info, blob_info)
        }
        BlobStatus::Unreadable => {
            fs_fixture
                .corrupt_root_node_of_blob(blob_info.clone())
                .await;
            expected_errors.extend(
                [
                    BlobUnreadableError {
                        blob_id: blob_info.blob_id,
                        referenced_as: expected_blob_referenced_as.clone(),
                    }
                    .into(),
                    NodeUnreadableError {
                        node_id: *blob_info.blob_id.to_root_block_id(),
                        referenced_as: expected_node_referenced_as.clone(),
                    }
                    .into(),
                ]
                .into_iter(),
            );
            (
                MaybeNodeInfoAsSeenByLookingAtNode::Unreadable,
                MaybeBlobInfoAsSeenByLookingAtBlob::Unreadable,
            )
        }
        BlobStatus::Missing => {
            fs_fixture.remove_blob(blob_info.clone()).await;
            expected_errors.push(
                NodeMissingError {
                    node_id: *blob_info.blob_id.to_root_block_id(),
                    referenced_as: expected_node_referenced_as.clone(),
                }
                .into(),
            );
            (
                MaybeNodeInfoAsSeenByLookingAtNode::Missing,
                MaybeBlobInfoAsSeenByLookingAtBlob::Missing,
            )
        }
    };

    expected_errors.extend(
        [
            // TODO Do we want to report `NodeReferencedMultipleTimes` or only report `BlobReferencedMultipleTimes`?
            CorruptedError::NodeReferencedMultipleTimes(NodeReferencedMultipleTimesError {
                node_id: *blob_info.blob_id.to_root_block_id(),
                node_info: expected_node_info,
                referenced_as: expected_node_referenced_as,
            }),
            CorruptedError::BlobReferencedMultipleTimes(BlobReferencedMultipleTimesError {
                blob_id: blob_info.blob_id,
                blob_info: expected_blob_info,
                referenced_as: expected_blob_referenced_as,
            }),
        ]
        .into_iter(),
    );

    let errors: Vec<CorruptedError> = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
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

// TODO Also test where they are referenced as both a blob and as a node within a blob
