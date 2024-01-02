use std::{fs, iter};

use cryfs_blobstore::{BlobId, BlobStoreOnBlocks};
use cryfs_blockstore::{DynBlockStore, RemoveResult};
use cryfs_check::CorruptedError;
use cryfs_cryfs::filesystem::fsblobstore::FsBlobStore;

use cryfs_utils::testutils::asserts::assert_unordered_vec_eq;

mod common;
use common::entry_helpers::{add_dir_entry, add_file_entry, add_symlink_entry, load_dir_blob};
use common::fixture::{CorruptInnerNodeResult, CorruptSomeNodesResult, FilesystemFixture};

fn blobid1() -> BlobId {
    BlobId::from_hex("2ede765bc3ef88f95c040ad977bad788").unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_unreadable_single_node() {
    let fs_fixture = FilesystemFixture::new().await;
    fs_fixture.create_some_blobs().await;
    let file = fs_fixture.create_empty_file().await;

    let CorruptInnerNodeResult {
        corrupted_node,
        orphaned_nodes,
    } = fs_fixture.corrupt_root_node_of_blob(file).await;
    assert_eq!(0, orphaned_nodes.len(), "test precondition violated");

    let expected_errors = vec![
        CorruptedError::BlobUnreadable {
            blob_id: BlobId::from_root_block_id(corrupted_node),
        },
        CorruptedError::NodeUnreadable {
            node_id: corrupted_node,
        },
    ];

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_unreadable_root_node() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let CorruptInnerNodeResult {
        corrupted_node,
        orphaned_nodes,
    } = fs_fixture
        .corrupt_root_node_of_blob(some_blobs.large_file)
        .await;

    let expected_errors = [
        CorruptedError::BlobUnreadable {
            blob_id: BlobId::from_root_block_id(corrupted_node),
        },
        CorruptedError::NodeUnreadable {
            node_id: corrupted_node,
        },
    ]
    .into_iter()
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
async fn file_with_corrupted_inner_node() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let CorruptInnerNodeResult {
        corrupted_node,
        orphaned_nodes,
    } = fs_fixture
        .corrupt_an_inner_node_of_a_large_blob(some_blobs.large_file)
        .await;

    let expected_errors = iter::once(CorruptedError::NodeUnreadable {
        node_id: corrupted_node,
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
async fn file_with_corrupted_leaf_node() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let removed_node = fs_fixture.corrupt_a_leaf_node(some_blobs.large_file).await;

    let expected_errors = vec![CorruptedError::NodeUnreadable {
        node_id: removed_node,
    }];

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_corrupted_some_nodes() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let CorruptSomeNodesResult {
        corrupted_nodes,
        orphaned_nodes,
    } = fs_fixture
        .corrupt_some_nodes_of_a_large_blob(some_blobs.large_file)
        .await;

    let expected_errors = corrupted_nodes
        .into_iter()
        .map(|node_id| CorruptedError::NodeUnreadable { node_id })
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
async fn dir_with_unreadable_single_node_without_children() {
    let fs_fixture = FilesystemFixture::new().await;
    fs_fixture.create_some_blobs().await;
    let dir = fs_fixture.create_empty_dir().await;

    let orphaned_descendant_blobs = fs_fixture.get_descendants_of_dir_blob(dir).await;
    assert_eq!(
        0,
        orphaned_descendant_blobs.len(),
        "test precondition violated"
    );
    let CorruptInnerNodeResult {
        corrupted_node,
        orphaned_nodes,
    } = fs_fixture.corrupt_root_node_of_blob(dir).await;
    assert_eq!(0, orphaned_nodes.len(), "test precondition violated");

    let expected_errors = vec![
        CorruptedError::BlobUnreadable {
            blob_id: BlobId::from_root_block_id(corrupted_node),
        },
        CorruptedError::NodeUnreadable {
            node_id: corrupted_node,
        },
    ];

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

#[tokio::test(flavor = "multi_thread")]
async fn dir_with_unreadable_root_node() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_of_dir_blob(some_blobs.large_dir)
        .await;
    let CorruptInnerNodeResult {
        corrupted_node,
        orphaned_nodes,
    } = fs_fixture
        .corrupt_root_node_of_blob(some_blobs.large_dir)
        .await;

    let expected_errors =
        [
            CorruptedError::BlobUnreadable {
                blob_id: BlobId::from_root_block_id(corrupted_node),
            },
            CorruptedError::NodeUnreadable {
                node_id: corrupted_node,
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
async fn dir_with_unreadable_inner_node() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_of_dir_blob(some_blobs.large_dir)
        .await;
    let CorruptInnerNodeResult {
        corrupted_node,
        orphaned_nodes,
    } = fs_fixture
        .corrupt_an_inner_node_of_a_large_blob(some_blobs.large_dir)
        .await;

    let expected_errors =
        [
            CorruptedError::NodeUnreadable {
                node_id: corrupted_node,
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
async fn dir_with_unreadable_leaf_node() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_of_dir_blob(some_blobs.large_dir)
        .await;
    let removed_node = fs_fixture.corrupt_a_leaf_node(some_blobs.large_dir).await;

    let expected_errors =
        [
            CorruptedError::NodeUnreadable {
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
async fn dir_with_unreadable_some_nodes() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_of_dir_blob(some_blobs.large_dir)
        .await;
    let CorruptSomeNodesResult {
        corrupted_nodes,
        orphaned_nodes,
    } = fs_fixture
        .corrupt_some_nodes_of_a_large_blob(some_blobs.large_dir)
        .await;

    let expected_errors =
        iter::once(CorruptedError::BlobUnreadable {
            blob_id: some_blobs.large_dir,
        })
        .chain(
            corrupted_nodes
                .into_iter()
                .map(|node_id| CorruptedError::NodeUnreadable { node_id }),
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
async fn symlink_with_unreadable_single_node() {
    let fs_fixture = FilesystemFixture::new().await;
    fs_fixture.create_some_blobs().await;
    let symlink = fs_fixture.create_symlink("/t").await;

    let CorruptInnerNodeResult {
        corrupted_node,
        orphaned_nodes,
    } = fs_fixture.corrupt_root_node_of_blob(symlink).await;
    assert_eq!(0, orphaned_nodes.len(), "test precondition violated");

    let expected_errors = vec![
        CorruptedError::NodeUnreadable {
            node_id: corrupted_node,
        },
        CorruptedError::BlobUnreadable {
            blob_id: BlobId::from_root_block_id(corrupted_node),
        },
    ];

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

#[tokio::test(flavor = "multi_thread")]
async fn symlink_with_unreadable_root_node() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let CorruptInnerNodeResult {
        corrupted_node,
        orphaned_nodes,
    } = fs_fixture
        .corrupt_root_node_of_blob(some_blobs.large_symlink)
        .await;

    let expected_errors = [
        CorruptedError::NodeUnreadable {
            node_id: corrupted_node,
        },
        CorruptedError::BlobUnreadable {
            blob_id: BlobId::from_root_block_id(corrupted_node),
        },
    ]
    .into_iter()
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
async fn symlink_with_unreadable_inner_node() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let CorruptInnerNodeResult {
        corrupted_node,
        orphaned_nodes,
    } = fs_fixture
        .corrupt_an_inner_node_of_a_large_blob(some_blobs.large_symlink)
        .await;

    let expected_errors = iter::once(CorruptedError::NodeUnreadable {
        node_id: corrupted_node,
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
async fn symlink_with_unreadable_leaf_node() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let corrupted_node = fs_fixture
        .corrupt_a_leaf_node(some_blobs.large_symlink)
        .await;

    let expected_errors = vec![CorruptedError::NodeUnreadable {
        node_id: corrupted_node,
    }];

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

#[tokio::test(flavor = "multi_thread")]
async fn symlink_with_unreadable_some_nodes() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let CorruptSomeNodesResult {
        corrupted_nodes,
        orphaned_nodes,
    } = fs_fixture
        .corrupt_some_nodes_of_a_large_blob(some_blobs.large_symlink)
        .await;

    let expected_errors = corrupted_nodes
        .into_iter()
        .map(|node_id| CorruptedError::NodeUnreadable { node_id })
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
async fn root_dir_with_unreadable_single_node_without_children() {
    let fs_fixture = FilesystemFixture::new().await;
    let root = fs_fixture.root_blob_id();

    let orphaned_descendant_blobs = fs_fixture.get_descendants_of_dir_blob(root).await;
    assert_eq!(
        0,
        orphaned_descendant_blobs.len(),
        "test precondition violated"
    );
    let CorruptInnerNodeResult {
        corrupted_node,
        orphaned_nodes,
    } = fs_fixture.corrupt_root_node_of_blob(root).await;
    assert_eq!(0, orphaned_nodes.len(), "test precondition violated");

    let expected_errors =
        [
            CorruptedError::BlobUnreadable {
                blob_id: BlobId::from_root_block_id(corrupted_node),
            },
            CorruptedError::NodeUnreadable {
                node_id: corrupted_node,
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
async fn root_dir_with_unreadable_root_node() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_of_dir_blob(some_blobs.root)
        .await;
    let CorruptInnerNodeResult {
        corrupted_node,
        orphaned_nodes,
    } = fs_fixture.corrupt_root_node_of_blob(some_blobs.root).await;

    let expected_errors =
        [
            CorruptedError::BlobUnreadable {
                blob_id: BlobId::from_root_block_id(corrupted_node),
            },
            CorruptedError::NodeUnreadable {
                node_id: corrupted_node,
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
async fn root_dir_with_unreadable_inner_node() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_of_dir_blob(some_blobs.root)
        .await;
    let CorruptInnerNodeResult {
        corrupted_node,
        orphaned_nodes,
    } = fs_fixture
        .corrupt_an_inner_node_of_a_small_blob(some_blobs.root)
        .await;

    let expected_errors =
        [
            CorruptedError::NodeUnreadable {
                node_id: corrupted_node,
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
async fn root_dir_with_unreadable_leaf_node() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_of_dir_blob(some_blobs.root)
        .await;
    let corrupted_node = fs_fixture.corrupt_a_leaf_node(some_blobs.root).await;

    let expected_errors =
        [
            CorruptedError::NodeUnreadable {
                node_id: corrupted_node,
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
async fn root_dir_with_unreadable_some_nodes() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;
    // Make the root dir large enough that we can remove some nodes
    fs_fixture
        .add_entries_to_make_dir_large(some_blobs.root)
        .await;

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_of_dir_blob(some_blobs.root)
        .await;
    let CorruptSomeNodesResult {
        corrupted_nodes,
        orphaned_nodes,
    } = fs_fixture
        .corrupt_some_nodes_of_a_large_blob(some_blobs.root)
        .await;

    let expected_errors =
        iter::once(CorruptedError::BlobUnreadable {
            blob_id: some_blobs.root,
        })
        .chain(
            corrupted_nodes
                .into_iter()
                .map(|node_id| CorruptedError::NodeUnreadable { node_id }),
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
