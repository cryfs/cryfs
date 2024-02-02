//! Tests where there are blobs that aren't referenced from anywhere

use futures::future::BoxFuture;
use rstest::rstest;
use std::iter;
use std::time::SystemTime;

use cryfs_blobstore::BlobId;
use cryfs_check::CorruptedError;
use cryfs_cryfs::utils::fs_types::{Gid, Uid};
use cryfs_utils::{
    data::Data, testutils::asserts::assert_unordered_vec_eq, testutils::data_fixture::DataFixture,
};

mod common;
use common::fixture::FilesystemFixture;

fn parent_id() -> BlobId {
    blob_id(123456)
}

fn blob_id(seed: u64) -> BlobId {
    BlobId::from_slice(&DataFixture::new(seed).get(16)).unwrap()
}

fn data(size: usize, seed: u64) -> Data {
    DataFixture::new(seed).get(size).into()
}

fn make_single_node_file_blob(fs_fixture: &FilesystemFixture) -> BoxFuture<'_, BlobId> {
    Box::pin(async move {
        fs_fixture
            .update_fsblobstore(|fsblobstore| {
                Box::pin(async move {
                    fsblobstore
                        .create_file_blob(&parent_id())
                        .await
                        .unwrap()
                        .blob_id()
                })
            })
            .await
    })
}

fn make_large_file_blob(fs_fixture: &FilesystemFixture) -> BoxFuture<'_, BlobId> {
    Box::pin(async move {
        fs_fixture
            .update_fsblobstore(|fsblobstore| {
                Box::pin(async move {
                    let mut file = fsblobstore.create_file_blob(&parent_id()).await.unwrap();
                    file.write(&data(common::entry_helpers::LARGE_FILE_SIZE, 0), 0)
                        .await
                        .unwrap();
                    assert!(
                        file.num_nodes().await.unwrap() > 1_000,
                        "If this fails, we need to make the data larger so it uses enough nodes."
                    );
                    file.blob_id()
                })
            })
            .await
    })
}

fn make_single_node_dir_blob(fs_fixture: &FilesystemFixture) -> BoxFuture<'_, BlobId> {
    Box::pin(async move {
        fs_fixture
            .update_fsblobstore(|fsblobstore| {
                Box::pin(async move {
                    let mut dir_blob = fsblobstore.create_dir_blob(&parent_id()).await.unwrap();
                    let blob_id = dir_blob.blob_id();
                    dir_blob.async_drop().await.unwrap();
                    blob_id
                })
            })
            .await
    })
}

fn make_large_dir_blob(fs_fixture: &FilesystemFixture) -> BoxFuture<'_, BlobId> {
    Box::pin(async move {
        fs_fixture
            .update_fsblobstore(|fsblobstore| {
                Box::pin(async move {
                    let mut dir_blob = fsblobstore.create_dir_blob(&parent_id()).await.unwrap();
                    for i in 0..400 {
                        dir_blob
                            .add_entry_symlink(
                                format!("symlink_{i}").try_into().unwrap(),
                                blob_id(i),
                                Uid::from(1000),
                                Gid::from(1000),
                                SystemTime::now(),
                                SystemTime::now(),
                            )
                            .unwrap();
                    }
                    assert!(
                        dir_blob.num_nodes().await.unwrap() > 1_000,
                        "If this fails, we need to make the data larger so it uses enough nodes."
                    );
                    let blob_id = dir_blob.blob_id();
                    dir_blob.async_drop().await.unwrap();
                    blob_id
                })
            })
            .await
    })
}

fn make_dir_blob_with_children(fs_fixture: &FilesystemFixture) -> BoxFuture<'_, BlobId> {
    Box::pin(async move {
        fs_fixture
            .update_fsblobstore(|fsblobstore| {
                Box::pin(async move {
                    let mut dir_blob = fsblobstore.create_dir_blob(&parent_id()).await.unwrap();
                    common::entry_helpers::add_entries_to_make_dir_large(
                        fsblobstore,
                        &mut dir_blob,
                    )
                    .await;
                    assert!(
                        dir_blob.num_nodes().await.unwrap() > 1_000,
                        "If this fails, we need to make the data larger so it uses enough nodes."
                    );
                    let blob_id = dir_blob.blob_id();
                    dir_blob.async_drop().await.unwrap();
                    blob_id
                })
            })
            .await
    })
}

fn make_single_node_symlink_blob(fs_fixture: &FilesystemFixture) -> BoxFuture<'_, BlobId> {
    Box::pin(async move {
        fs_fixture
            .update_fsblobstore(|fsblobstore| {
                Box::pin(async move {
                    fsblobstore
                        .create_symlink_blob(&parent_id(), "target")
                        .await
                        .unwrap()
                        .blob_id()
                })
            })
            .await
    })
}

fn make_large_symlink_blob(fs_fixture: &FilesystemFixture) -> BoxFuture<'_, BlobId> {
    Box::pin(async move {
        fs_fixture
            .update_fsblobstore(|fsblobstore| {
                Box::pin(async move {
                    let mut symlink = fsblobstore
                        .create_symlink_blob(
                            &parent_id(),
                            &common::entry_helpers::large_symlink_target(),
                        )
                        .await
                        .unwrap();
                    assert!(
                        symlink.num_nodes().await.unwrap() > 1_000,
                        "If this fails, we need to make the data larger so it uses enough nodes."
                    );
                    symlink.blob_id()
                })
            })
            .await
    })
}

#[rstest]
#[tokio::test(flavor = "multi_thread")]
async fn blob_unreferenced(
    #[values(
        make_single_node_file_blob,
        make_large_file_blob,
        make_single_node_dir_blob,
        make_large_dir_blob,
        make_single_node_symlink_blob,
        make_large_symlink_blob
    )]
    make_blob: impl for<'a> FnOnce(&'a FilesystemFixture) -> BoxFuture<'a, BlobId>,
) {
    let (fs_fixture, _some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_id = make_blob(&fs_fixture).await;

    let expected_errors: Vec<_> = vec![CorruptedError::NodeUnreferenced {
        node_id: *blob_id.to_root_block_id(),
    }];

    let errors = fs_fixture.run_cryfs_check().await;
    assert_eq!(expected_errors, errors,);
}

#[tokio::test(flavor = "multi_thread")]
async fn dir_blob_with_children_unreferenced() {
    let (fs_fixture, _some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_id = make_dir_blob_with_children(&fs_fixture).await;
    let orphaned_descendant_blobs = fs_fixture.get_descendants_of_dir_blob(blob_id).await;

    let expected_errors: Vec<_> =
        iter::once(CorruptedError::NodeUnreferenced {
            node_id: *blob_id.to_root_block_id(),
        })
        .chain(orphaned_descendant_blobs.into_iter().map(|child| {
            CorruptedError::NodeUnreferenced {
                node_id: *child.to_root_block_id(),
            }
        }))
        .collect();

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}
