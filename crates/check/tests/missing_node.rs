//! Tests where individual nodes are missing

use std::iter;

use cryfs_blobstore::{BlobId, BlobStoreOnBlocks};
use cryfs_blockstore::{DynBlockStore, RemoveResult};
use cryfs_check::CorruptedError;
use cryfs_cryfs::filesystem::fsblobstore::FsBlobStore;

use cryfs_utils::testutils::asserts::assert_unordered_vec_eq;

mod common;
use common::entry_helpers::{add_dir_entry, add_file_entry, add_symlink_entry, load_dir_blob};
use common::fixture::{FilesystemFixture, RemoveInnerNodeResult, RemoveSomeNodesResult};

fn blobid1() -> BlobId {
    BlobId::from_hex("ad977bad7882ede765bc3ef88f95c040").unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn file_entirely_missing() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    fs_fixture
        .update_fsblobstore(
            |blobstore: &FsBlobStore<BlobStoreOnBlocks<DynBlockStore>>| {
                Box::pin(async move {
                    let mut parent = load_dir_blob(blobstore, &some_blobs.dir1_dir3_dir5).await;
                    // Add a file entry but don't add the corresponding blob
                    add_file_entry(&mut parent, "filename", blobid1());
                    parent.async_drop().await.unwrap();
                })
            },
        )
        .await;

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(
        vec![CorruptedError::BlobMissing { blob_id: blobid1() }],
        errors,
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_missing_root_node() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let RemoveInnerNodeResult {
        removed_node,
        orphaned_nodes,
    } = fs_fixture
        .remove_root_node_of_blob(some_blobs.large_file)
        .await;

    let expected_errors = iter::once(CorruptedError::BlobMissing {
        blob_id: BlobId::from_root_block_id(removed_node),
    })
    .chain(
        orphaned_nodes
            .into_iter()
            .map(|child| CorruptedError::NodeUnreferenced { node_id: child }),
    )
    .collect();

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_missing_inner_node() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let RemoveInnerNodeResult {
        removed_node,
        orphaned_nodes,
    } = fs_fixture
        .remove_an_inner_node_of_a_large_blob(some_blobs.large_file)
        .await;

    let expected_errors = iter::once(CorruptedError::NodeMissing {
        node_id: removed_node,
    })
    .chain(
        orphaned_nodes
            .into_iter()
            .map(|child| CorruptedError::NodeUnreferenced { node_id: child }),
    )
    .collect();

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_missing_leaf_node() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let removed_node = fs_fixture.remove_a_leaf_node(some_blobs.large_file).await;

    let expected_errors = vec![CorruptedError::NodeMissing {
        node_id: removed_node,
    }];

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_missing_some_nodes() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let RemoveSomeNodesResult {
        removed_nodes,
        orphaned_nodes,
    } = fs_fixture
        .remove_some_nodes_of_a_large_blob(some_blobs.large_file)
        .await;

    let expected_errors = removed_nodes
        .into_iter()
        .map(|node_id| CorruptedError::NodeMissing { node_id })
        .chain(
            orphaned_nodes
                .into_iter()
                .map(|child| CorruptedError::NodeUnreferenced { node_id: child }),
        )
        .collect();

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

#[tokio::test(flavor = "multi_thread")]
async fn dir_entirely_missing_without_children() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    fs_fixture
        .update_fsblobstore(
            |blobstore: &FsBlobStore<BlobStoreOnBlocks<DynBlockStore>>| {
                Box::pin(async move {
                    let mut parent = load_dir_blob(blobstore, &some_blobs.dir2_dir6).await;
                    // Add a dir entry but don't add the corresponding blob
                    add_dir_entry(&mut parent, "dirname", blobid1());
                    parent.async_drop().await.unwrap();
                })
            },
        )
        .await;

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(
        vec![CorruptedError::BlobMissing { blob_id: blobid1() }],
        errors,
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn dir_entirely_missing_with_children() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_of_dir_blob(some_blobs.large_dir)
        .await;
    fs_fixture
        .update_fsblobstore(|fsblobstore| {
            Box::pin(async move {
                let remove_result = fsblobstore
                    .remove_by_id(&some_blobs.large_dir)
                    .await
                    .unwrap();
                assert_eq!(RemoveResult::SuccessfullyRemoved, remove_result);
            })
        })
        .await;

    let expected_errors =
        [CorruptedError::BlobMissing {
            blob_id: some_blobs.large_dir,
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

#[tokio::test(flavor = "multi_thread")]
async fn dir_with_missing_root_node() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_of_dir_blob(some_blobs.large_dir)
        .await;
    let RemoveInnerNodeResult {
        removed_node,
        orphaned_nodes,
    } = fs_fixture
        .remove_root_node_of_blob(some_blobs.large_dir)
        .await;

    let expected_errors =
        [CorruptedError::BlobMissing {
            blob_id: BlobId::from_root_block_id(removed_node),
        }]
        .into_iter()
        .chain(
            orphaned_nodes
                .into_iter()
                .map(|child| CorruptedError::NodeUnreferenced { node_id: child }),
        )
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
async fn dir_with_missing_inner_node() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_of_dir_blob(some_blobs.large_dir)
        .await;
    let RemoveInnerNodeResult {
        removed_node,
        orphaned_nodes,
    } = fs_fixture
        .remove_an_inner_node_of_a_large_blob(some_blobs.large_dir)
        .await;

    let expected_errors =
        [
            CorruptedError::NodeMissing {
                node_id: removed_node,
            },
            CorruptedError::BlobUnreadable {
                blob_id: some_blobs.large_dir,
            },
        ]
        .into_iter()
        .chain(
            orphaned_nodes
                .into_iter()
                .map(|child| CorruptedError::NodeUnreferenced { node_id: child }),
        )
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
async fn dir_with_missing_leaf_node() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_of_dir_blob(some_blobs.large_dir)
        .await;
    let removed_node = fs_fixture.remove_a_leaf_node(some_blobs.large_dir).await;

    let expected_errors =
        [
            CorruptedError::NodeMissing {
                node_id: removed_node,
            },
            CorruptedError::BlobUnreadable {
                blob_id: some_blobs.large_dir,
            },
        ]
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

#[tokio::test(flavor = "multi_thread")]
async fn dir_with_missing_some_nodes() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_of_dir_blob(some_blobs.large_dir)
        .await;
    let RemoveSomeNodesResult {
        removed_nodes,
        orphaned_nodes,
    } = fs_fixture
        .remove_some_nodes_of_a_large_blob(some_blobs.large_dir)
        .await;

    let expected_errors =
        iter::once(CorruptedError::BlobUnreadable {
            blob_id: some_blobs.large_dir,
        })
        .chain(
            removed_nodes
                .into_iter()
                .map(|node_id| CorruptedError::NodeMissing { node_id }),
        )
        .chain(
            orphaned_nodes
                .into_iter()
                .map(|child| CorruptedError::NodeUnreferenced { node_id: child }),
        )
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
async fn symlink_entirely_missing() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    fs_fixture
        .update_fsblobstore(
            |blobstore: &FsBlobStore<BlobStoreOnBlocks<DynBlockStore>>| {
                Box::pin(async move {
                    let mut parent = load_dir_blob(blobstore, &some_blobs.root).await;
                    // Add a symlink entry but don't add the corresponding blob
                    add_symlink_entry(&mut parent, "symlinkname", blobid1());
                    parent.async_drop().await.unwrap();
                })
            },
        )
        .await;

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(
        vec![CorruptedError::BlobMissing { blob_id: blobid1() }],
        errors,
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn symlink_with_missing_root_node() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let RemoveInnerNodeResult {
        removed_node,
        orphaned_nodes,
    } = fs_fixture
        .remove_root_node_of_blob(some_blobs.large_symlink)
        .await;

    let expected_errors = iter::once(CorruptedError::BlobMissing {
        blob_id: BlobId::from_root_block_id(removed_node),
    })
    .chain(
        orphaned_nodes
            .into_iter()
            .map(|child| CorruptedError::NodeUnreferenced { node_id: child }),
    )
    .collect();

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

#[tokio::test(flavor = "multi_thread")]
async fn symlink_with_missing_inner_node() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let RemoveInnerNodeResult {
        removed_node,
        orphaned_nodes,
    } = fs_fixture
        .remove_an_inner_node_of_a_large_blob(some_blobs.large_symlink)
        .await;

    let expected_errors = iter::once(CorruptedError::NodeMissing {
        node_id: removed_node,
    })
    .chain(
        orphaned_nodes
            .into_iter()
            .map(|child| CorruptedError::NodeUnreferenced { node_id: child }),
    )
    .collect();

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

#[tokio::test(flavor = "multi_thread")]
async fn symlink_with_missing_leaf_node() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let removed_node = fs_fixture
        .remove_a_leaf_node(some_blobs.large_symlink)
        .await;

    let expected_errors = vec![CorruptedError::NodeMissing {
        node_id: removed_node,
    }];

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

#[tokio::test(flavor = "multi_thread")]
async fn symlink_with_missing_some_nodes() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let RemoveSomeNodesResult {
        removed_nodes,
        orphaned_nodes,
    } = fs_fixture
        .remove_some_nodes_of_a_large_blob(some_blobs.large_symlink)
        .await;

    let expected_errors = removed_nodes
        .into_iter()
        .map(|node_id| CorruptedError::NodeMissing { node_id })
        .chain(
            orphaned_nodes
                .into_iter()
                .map(|child| CorruptedError::NodeUnreferenced { node_id: child }),
        )
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

#[tokio::test(flavor = "multi_thread")]
async fn root_dir_entirely_missing_with_children() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_of_dir_blob(some_blobs.root)
        .await;
    fs_fixture
        .update_fsblobstore(|fsblobstore| {
            Box::pin(async move {
                let remove_result = fsblobstore.remove_by_id(&some_blobs.root).await.unwrap();
                assert_eq!(RemoveResult::SuccessfullyRemoved, remove_result);
            })
        })
        .await;

    let expected_errors =
        [CorruptedError::BlobMissing {
            blob_id: some_blobs.root,
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

#[tokio::test(flavor = "multi_thread")]
async fn root_dir_with_missing_root_node() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_of_dir_blob(some_blobs.root)
        .await;
    let RemoveInnerNodeResult {
        removed_node,
        orphaned_nodes,
    } = fs_fixture.remove_root_node_of_blob(some_blobs.root).await;

    let expected_errors =
        [CorruptedError::BlobMissing {
            blob_id: BlobId::from_root_block_id(removed_node),
        }]
        .into_iter()
        .chain(
            orphaned_nodes
                .into_iter()
                .map(|child| CorruptedError::NodeUnreferenced { node_id: child }),
        )
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
async fn root_dir_with_missing_inner_node() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_of_dir_blob(some_blobs.root)
        .await;
    let RemoveInnerNodeResult {
        removed_node,
        orphaned_nodes,
    } = fs_fixture
        .remove_an_inner_node_of_a_small_blob(some_blobs.root)
        .await;

    let expected_errors =
        [
            CorruptedError::NodeMissing {
                node_id: removed_node,
            },
            CorruptedError::BlobUnreadable {
                blob_id: some_blobs.root,
            },
        ]
        .into_iter()
        .chain(
            orphaned_nodes
                .into_iter()
                .map(|child| CorruptedError::NodeUnreferenced { node_id: child }),
        )
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
async fn root_dir_with_missing_leaf_node() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_of_dir_blob(some_blobs.root)
        .await;
    let removed_node = fs_fixture.remove_a_leaf_node(some_blobs.root).await;

    let expected_errors =
        [
            CorruptedError::NodeMissing {
                node_id: removed_node,
            },
            CorruptedError::BlobUnreadable {
                blob_id: some_blobs.root,
            },
        ]
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

#[tokio::test(flavor = "multi_thread")]
async fn root_dir_with_missing_some_nodes() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;
    // Make the root dir large enough that we can remove some nodes
    fs_fixture
        .add_entries_to_make_dir_large(some_blobs.root)
        .await;

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_of_dir_blob(some_blobs.root)
        .await;
    let RemoveSomeNodesResult {
        removed_nodes,
        orphaned_nodes,
    } = fs_fixture
        .remove_some_nodes_of_a_large_blob(some_blobs.root)
        .await;

    let expected_errors =
        iter::once(CorruptedError::BlobUnreadable {
            blob_id: some_blobs.root,
        })
        .chain(
            removed_nodes
                .into_iter()
                .map(|node_id| CorruptedError::NodeMissing { node_id }),
        )
        .chain(
            orphaned_nodes
                .into_iter()
                .map(|child| CorruptedError::NodeUnreferenced { node_id: child }),
        )
        .chain(orphaned_descendant_blobs.into_iter().map(|child| {
            CorruptedError::NodeUnreferenced {
                node_id: *child.to_root_block_id(),
            }
        }))
        .collect();

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}
