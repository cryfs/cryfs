//! Tests where blobs have wrong parent pointers set

use futures::future::BoxFuture;
use rstest::rstest;
use std::collections::BTreeSet;
use std::iter;
use std::num::NonZeroU8;

use cryfs_blobstore::BlobId;
use cryfs_check::{
    BlobReference, BlobReferenceWithId, BlobReferencedMultipleTimesError, CorruptedError,
    MaybeBlobInfoAsSeenByLookingAtBlob, MaybeNodeInfoAsSeenByLookingAtNode, NodeAndBlobReference,
    NodeReferencedMultipleTimesError, WrongParentPointerError,
};
use cryfs_filesystem::filesystem::fsblobstore::{BlobType, FsBlob};
use cryfs_utils::testutils::asserts::assert_unordered_vec_eq;

mod common;
use common::{
    entry_helpers::{self, CreatedDirBlob},
    fixture::FilesystemFixture,
};

fn make_file(
    fs_fixture: &FilesystemFixture,
    parent: BlobReferenceWithId,
) -> BoxFuture<'_, BlobReferenceWithId> {
    assert!(parent.referenced_as.blob_type == BlobType::Dir);
    Box::pin(async move {
        fs_fixture
            .create_empty_file_in_parent(parent, "filename")
            .await
    })
}

fn make_symlink(
    fs_fixture: &FilesystemFixture,
    parent: BlobReferenceWithId,
) -> BoxFuture<'_, BlobReferenceWithId> {
    assert!(parent.referenced_as.blob_type == BlobType::Dir);
    Box::pin(async move {
        fs_fixture
            .create_symlink_in_parent(parent, "symlinkname", "target")
            .await
    })
}

fn make_empty_dir<'a>(
    fs_fixture: &'a FilesystemFixture,
    parent: BlobReferenceWithId,
) -> BoxFuture<'a, BlobReferenceWithId> {
    assert!(parent.referenced_as.blob_type == BlobType::Dir);
    Box::pin(async move {
        fs_fixture
            .create_empty_dir_in_parent(parent, "my_empty_dir")
            .await
    })
}

fn make_large_dir<'a>(
    fs_fixture: &'a FilesystemFixture,
    parent_info: BlobReferenceWithId,
) -> BoxFuture<'a, BlobReferenceWithId> {
    assert!(parent_info.referenced_as.blob_type == BlobType::Dir);
    Box::pin(async move {
        fs_fixture
            .update_fsblobstore(move |fsblobstore| {
                Box::pin(async move {
                    let parent = FsBlob::into_dir(
                        fsblobstore
                            .load(&parent_info.blob_id)
                            .await
                            .unwrap()
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                    let mut parent = CreatedDirBlob::new(parent, parent_info.referenced_as.path);
                    let mut dir =
                        entry_helpers::create_large_dir(fsblobstore, &mut *parent, "dirname").await;
                    let result = (&*dir).into();
                    dir.async_drop().await.unwrap();
                    parent.async_drop().await.unwrap();
                    result
                })
            })
            .await
    })
}

async fn set_parent(fs_fixture: &FilesystemFixture, blob_id: BlobId, new_parent: BlobId) {
    fs_fixture
        .update_fsblobstore(|fsblobstore| {
            Box::pin(async move {
                let mut blob = fsblobstore.load(&blob_id).await.unwrap().unwrap();
                blob.set_parent(&new_parent).await.unwrap();
                blob.async_drop().await.unwrap();
            })
        })
        .await;
}

#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn blob_with_wrong_parent_pointer_referenced_from_one_dir(
    #[values(make_empty_dir, make_large_dir)] make_old_parent: fn(
        &FilesystemFixture,
        BlobReferenceWithId,
    ) -> BoxFuture<
        '_,
        BlobReferenceWithId,
    >,
    #[values(make_empty_dir, make_large_dir)] make_new_parent: fn(
        &FilesystemFixture,
        BlobReferenceWithId,
    ) -> BoxFuture<
        '_,
        BlobReferenceWithId,
    >,
    #[values(make_file, make_empty_dir, make_symlink)] make_blob: fn(
        &FilesystemFixture,
        BlobReferenceWithId,
    ) -> BoxFuture<
        '_,
        BlobReferenceWithId,
    >,
) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;

    let old_parent = make_old_parent(&fs_fixture, some_blobs.dir1).await;
    let blob_info = make_blob(&fs_fixture, old_parent.clone()).await;
    let new_parent = make_new_parent(&fs_fixture, some_blobs.dir2).await;

    set_parent(&fs_fixture, blob_info.blob_id, new_parent.blob_id).await;

    let expected_errors: Vec<CorruptedError> = vec![WrongParentPointerError {
        blob_id: blob_info.blob_id,
        blob_type: blob_info.referenced_as.blob_type,
        parent_pointer: new_parent.blob_id,
        referenced_as: iter::once(blob_info.referenced_as).collect(),
    }
    .into()];

    let errors = fs_fixture.run_cryfs_check().await;
    assert_eq!(expected_errors, errors);
}

#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn blob_with_wrong_parent_pointer_referenced_from_two_dirs(
    #[values(make_empty_dir, make_large_dir)] make_old_parent: fn(
        &FilesystemFixture,
        BlobReferenceWithId,
    ) -> BoxFuture<
        '_,
        BlobReferenceWithId,
    >,
    #[values(make_empty_dir, make_large_dir)] make_new_parent: fn(
        &FilesystemFixture,
        BlobReferenceWithId,
    ) -> BoxFuture<
        '_,
        BlobReferenceWithId,
    >,
    #[values(make_file, make_empty_dir, make_symlink)] make_blob: fn(
        &FilesystemFixture,
        BlobReferenceWithId,
    ) -> BoxFuture<
        '_,
        BlobReferenceWithId,
    >,
) {
    const SECOND_NAME: &str = "dirname";
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;

    let old_parent = make_old_parent(&fs_fixture, some_blobs.dir1).await;
    let blob_info = make_blob(&fs_fixture, old_parent.clone()).await;
    let expected_depth = fs_fixture
        .get_node_depth(*blob_info.blob_id.to_root_block_id())
        .await;
    let expected_node_info = if let Some(depth) = NonZeroU8::new(expected_depth) {
        MaybeNodeInfoAsSeenByLookingAtNode::InnerNode { depth }
    } else {
        MaybeNodeInfoAsSeenByLookingAtNode::LeafNode
    };

    let old_parent_2 = make_old_parent(&fs_fixture, some_blobs.dir2.clone()).await;
    fs_fixture
        .add_dir_entry_to_dir(old_parent_2.blob_id, SECOND_NAME, blob_info.blob_id)
        .await;
    let new_parent = make_new_parent(&fs_fixture, some_blobs.dir1_dir3).await;

    set_parent(&fs_fixture, blob_info.blob_id, new_parent.blob_id).await;

    let expected_blob_references: BTreeSet<BlobReference> = [
        blob_info.referenced_as.clone(),
        BlobReference {
            blob_type: BlobType::Dir,
            parent_id: old_parent_2.blob_id,
            path: old_parent_2
                .referenced_as
                .path
                .join(SECOND_NAME.try_into().unwrap()),
        },
    ]
    .into_iter()
    .collect();
    let expected_errors: Vec<_> = vec![
        WrongParentPointerError {
            blob_id: blob_info.blob_id,
            blob_type: blob_info.referenced_as.blob_type,
            parent_pointer: new_parent.blob_id,
            referenced_as: expected_blob_references.clone(),
        }
        .into(),
        NodeReferencedMultipleTimesError {
            node_id: *blob_info.blob_id.to_root_block_id(),
            node_info: expected_node_info,
            referenced_as: expected_blob_references
                .iter()
                .map(|blob_reference| NodeAndBlobReference::RootNode {
                    belongs_to_blob: BlobReferenceWithId {
                        blob_id: blob_info.blob_id,
                        referenced_as: blob_reference.clone(),
                    },
                })
                .collect(),
        }
        .into(),
        BlobReferencedMultipleTimesError {
            blob_id: blob_info.blob_id,
            blob_info: MaybeBlobInfoAsSeenByLookingAtBlob::Readable {
                blob_type: blob_info.referenced_as.blob_type,
                parent_pointer: new_parent.blob_id,
            },
            referenced_as: expected_blob_references,
        }
        .into(),
    ];

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn blob_with_wrong_parent_pointer_referenced_from_four_dirs(
    #[values(make_empty_dir, make_large_dir)] make_old_parent: fn(
        &FilesystemFixture,
        BlobReferenceWithId,
    ) -> BoxFuture<
        '_,
        BlobReferenceWithId,
    >,
    #[values(make_empty_dir, make_large_dir)] make_new_parent: fn(
        &FilesystemFixture,
        BlobReferenceWithId,
    ) -> BoxFuture<
        '_,
        BlobReferenceWithId,
    >,
    #[values(make_file, make_empty_dir, make_symlink)] make_blob: fn(
        &FilesystemFixture,
        BlobReferenceWithId,
    ) -> BoxFuture<
        '_,
        BlobReferenceWithId,
    >,
) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;

    let old_parent = make_old_parent(&fs_fixture, some_blobs.dir1).await;
    let blob_info = make_blob(&fs_fixture, old_parent.clone()).await;
    let expected_depth = fs_fixture
        .get_node_depth(*blob_info.blob_id.to_root_block_id())
        .await;
    let expected_node_info = if let Some(depth) = NonZeroU8::new(expected_depth) {
        MaybeNodeInfoAsSeenByLookingAtNode::InnerNode { depth }
    } else {
        MaybeNodeInfoAsSeenByLookingAtNode::LeafNode
    };

    let old_parent_2 = make_old_parent(&fs_fixture, some_blobs.dir2).await;
    fs_fixture
        .add_dir_entry_to_dir(old_parent_2.blob_id, "dirname", blob_info.blob_id)
        .await;
    let old_parent_3 = make_old_parent(&fs_fixture, some_blobs.dir1_dir4).await;
    fs_fixture
        .add_file_entry_to_dir(old_parent_3.blob_id, "filename", blob_info.blob_id)
        .await;
    let old_parent_4 = make_old_parent(&fs_fixture, some_blobs.dir2_dir6).await;
    fs_fixture
        .add_symlink_entry_to_dir(old_parent_4.blob_id, "symlinkname", blob_info.blob_id)
        .await;
    let new_parent = make_new_parent(&fs_fixture, some_blobs.dir1_dir3).await;

    set_parent(&fs_fixture, blob_info.blob_id, new_parent.blob_id).await;

    let expected_blob_references: BTreeSet<BlobReference> = [
        blob_info.referenced_as.clone(),
        BlobReference {
            blob_type: BlobType::Dir,
            parent_id: old_parent_2.blob_id,
            path: old_parent_2
                .referenced_as
                .path
                .join("dirname".try_into().unwrap()),
        },
        BlobReference {
            blob_type: BlobType::File,
            parent_id: old_parent_3.blob_id,
            path: old_parent_3
                .referenced_as
                .path
                .join("filename".try_into().unwrap()),
        },
        BlobReference {
            blob_type: BlobType::Symlink,
            parent_id: old_parent_4.blob_id,
            path: old_parent_4
                .referenced_as
                .path
                .join("symlinkname".try_into().unwrap()),
        },
    ]
    .into_iter()
    .collect();
    let expected_errors: Vec<_> = vec![
        WrongParentPointerError {
            blob_id: blob_info.blob_id,
            blob_type: blob_info.referenced_as.blob_type,
            parent_pointer: new_parent.blob_id,
            referenced_as: expected_blob_references.clone(),
        }
        .into(),
        NodeReferencedMultipleTimesError {
            node_id: *blob_info.blob_id.to_root_block_id(),
            node_info: expected_node_info,
            referenced_as: expected_blob_references
                .iter()
                .map(|blob_reference| NodeAndBlobReference::RootNode {
                    belongs_to_blob: BlobReferenceWithId {
                        blob_id: blob_info.blob_id,
                        referenced_as: blob_reference.clone(),
                    },
                })
                .collect(),
        }
        .into(),
        BlobReferencedMultipleTimesError {
            blob_id: blob_info.blob_id,
            blob_info: MaybeBlobInfoAsSeenByLookingAtBlob::Readable {
                blob_type: blob_info.referenced_as.blob_type,
                parent_pointer: new_parent.blob_id,
            },
            referenced_as: expected_blob_references,
        }
        .into(),
    ];

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}
