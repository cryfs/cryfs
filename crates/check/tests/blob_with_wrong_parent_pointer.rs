//! Tests where blobs have wrong parent pointers set

use futures::future::BoxFuture;
use rstest::rstest;

use cryfs_blobstore::BlobId;
use cryfs_check::{BlobInfo, CorruptedError};
use cryfs_cryfs::filesystem::fsblobstore::{BlobType, FsBlob};
use cryfs_utils::testutils::asserts::assert_unordered_vec_eq;

mod common;
use common::{
    entry_helpers::{self, CreatedDirBlob},
    fixture::FilesystemFixture,
};

fn make_file(fs_fixture: &FilesystemFixture, parent: BlobInfo) -> BoxFuture<'_, BlobInfo> {
    assert!(parent.blob_type == BlobType::Dir);
    Box::pin(async move {
        fs_fixture
            .create_empty_file_in_parent(parent, "filename")
            .await
    })
}

fn make_symlink(fs_fixture: &FilesystemFixture, parent: BlobInfo) -> BoxFuture<'_, BlobInfo> {
    assert!(parent.blob_type == BlobType::Dir);
    Box::pin(async move {
        fs_fixture
            .create_symlink_in_parent(parent, "symlinkname", "target")
            .await
    })
}

fn make_empty_dir<'a>(
    fs_fixture: &'a FilesystemFixture,
    parent: BlobInfo,
) -> BoxFuture<'a, BlobInfo> {
    assert!(parent.blob_type == BlobType::Dir);
    Box::pin(async move {
        fs_fixture
            .create_empty_dir_in_parent(parent, "my_empty_dir")
            .await
    })
}

fn make_large_dir<'a>(
    fs_fixture: &'a FilesystemFixture,
    parent_info: BlobInfo,
) -> BoxFuture<'a, BlobInfo> {
    assert!(parent_info.blob_type == BlobType::Dir);
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
                    let mut parent = CreatedDirBlob::new(parent, parent_info.path);
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
        BlobInfo,
    ) -> BoxFuture<'_, BlobInfo>,
    #[values(make_empty_dir, make_large_dir)] make_new_parent: fn(
        &FilesystemFixture,
        BlobInfo,
    ) -> BoxFuture<'_, BlobInfo>,
    #[values(make_file, make_empty_dir, make_symlink)] make_blob: fn(
        &FilesystemFixture,
        BlobInfo,
    )
        -> BoxFuture<'_, BlobInfo>,
) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;

    let old_parent = make_old_parent(&fs_fixture, some_blobs.dir1).await;
    let blob_info = make_blob(&fs_fixture, old_parent.clone()).await;
    let new_parent = make_new_parent(&fs_fixture, some_blobs.dir2).await;

    set_parent(&fs_fixture, blob_info.blob_id, new_parent.blob_id).await;

    let expected_errors: Vec<_> = vec![CorruptedError::WrongParentPointer {
        blob_id: blob_info.blob_id,
        referenced_by: [old_parent.blob_id].into_iter().collect(),
        parent_pointer: new_parent.blob_id,
    }];

    let errors = fs_fixture.run_cryfs_check().await;
    assert_eq!(expected_errors, errors);
}

#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn blob_with_wrong_parent_pointer_referenced_from_two_dirs(
    #[values(make_empty_dir, make_large_dir)] make_old_parent: fn(
        &FilesystemFixture,
        BlobInfo,
    ) -> BoxFuture<'_, BlobInfo>,
    #[values(make_empty_dir, make_large_dir)] make_new_parent: fn(
        &FilesystemFixture,
        BlobInfo,
    ) -> BoxFuture<'_, BlobInfo>,
    #[values(make_file, make_empty_dir, make_symlink)] make_blob: fn(
        &FilesystemFixture,
        BlobInfo,
    )
        -> BoxFuture<'_, BlobInfo>,
) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;

    let old_parent = make_old_parent(&fs_fixture, some_blobs.dir1).await;
    let blob_info = make_blob(&fs_fixture, old_parent.clone()).await;
    let old_parent_2 = make_old_parent(&fs_fixture, some_blobs.dir2).await;
    fs_fixture
        .add_dir_entry_to_dir(old_parent_2.blob_id, "name", blob_info.blob_id)
        .await;
    let new_parent = make_new_parent(&fs_fixture, some_blobs.dir1_dir3).await;

    set_parent(&fs_fixture, blob_info.blob_id, new_parent.blob_id).await;

    let expected_errors: Vec<_> = vec![
        CorruptedError::WrongParentPointer {
            blob_id: blob_info.blob_id,
            referenced_by: [old_parent.blob_id, old_parent_2.blob_id]
                .into_iter()
                .collect(),
            parent_pointer: new_parent.blob_id,
        },
        CorruptedError::NodeReferencedMultipleTimes {
            node_id: *blob_info.blob_id.to_root_block_id(),
        },
        CorruptedError::BlobReferencedMultipleTimes {
            blob_id: blob_info.blob_id,
        },
    ];

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn blob_with_wrong_parent_pointer_referenced_from_three_dirs(
    #[values(make_empty_dir, make_large_dir)] make_old_parent: fn(
        &FilesystemFixture,
        BlobInfo,
    ) -> BoxFuture<'_, BlobInfo>,
    #[values(make_empty_dir, make_large_dir)] make_new_parent: fn(
        &FilesystemFixture,
        BlobInfo,
    ) -> BoxFuture<'_, BlobInfo>,
    #[values(make_file, make_empty_dir, make_symlink)] make_blob: fn(
        &FilesystemFixture,
        BlobInfo,
    )
        -> BoxFuture<'_, BlobInfo>,
) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;

    let old_parent = make_old_parent(&fs_fixture, some_blobs.dir1).await;
    let blob_info = make_blob(&fs_fixture, old_parent.clone()).await;
    let old_parent_2 = make_old_parent(&fs_fixture, some_blobs.dir2).await;
    fs_fixture
        .add_dir_entry_to_dir(old_parent_2.blob_id, "name", blob_info.blob_id)
        .await;
    let old_parent_3 = make_old_parent(&fs_fixture, some_blobs.dir1_dir4).await;
    fs_fixture
        .add_dir_entry_to_dir(old_parent_3.blob_id, "name", blob_info.blob_id)
        .await;
    let new_parent = make_new_parent(&fs_fixture, some_blobs.dir1_dir3).await;

    set_parent(&fs_fixture, blob_info.blob_id, new_parent.blob_id).await;

    let expected_errors: Vec<_> = vec![
        CorruptedError::WrongParentPointer {
            blob_id: blob_info.blob_id,
            referenced_by: [
                old_parent.blob_id,
                old_parent_2.blob_id,
                old_parent_3.blob_id,
            ]
            .into_iter()
            .collect(),
            parent_pointer: new_parent.blob_id,
        },
        CorruptedError::NodeReferencedMultipleTimes {
            node_id: *blob_info.blob_id.to_root_block_id(),
        },
        CorruptedError::BlobReferencedMultipleTimes {
            blob_id: blob_info.blob_id,
        },
    ];

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}
