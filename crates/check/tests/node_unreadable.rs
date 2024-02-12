//! Tests where individual nodes are unreadable

use rstest::rstest;

use cryfs_check::{BlobInfo, CorruptedError};
use cryfs_cryfs::filesystem::fsblobstore::BlobType;
use cryfs_rustfs::AbsolutePathBuf;
use cryfs_utils::testutils::asserts::assert_unordered_vec_eq;

mod common;
use common::entry_helpers::SomeBlobs;
use common::fixture::{CorruptInnerNodeResult, CorruptSomeNodesResult, FilesystemFixture};

#[rstest]
#[case::file(|some_blobs: &SomeBlobs| some_blobs.empty_file.clone())]
#[case::dir(|some_blobs: &SomeBlobs| some_blobs.empty_dir.clone())]
#[case::symlink(|some_blobs: &SomeBlobs| some_blobs.empty_symlink.clone())]
#[tokio::test(flavor = "multi_thread")]
async fn blob_with_unreadable_single_node(#[case] blob: impl FnOnce(&SomeBlobs) -> BlobInfo) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_info = blob(&some_blobs);

    let CorruptInnerNodeResult {
        corrupted_node,
        orphaned_nodes,
    } = fs_fixture
        .corrupt_root_node_of_blob(blob_info.blob_id)
        .await;
    assert_eq!(0, orphaned_nodes.len(), "test precondition violated");
    assert_eq!(&corrupted_node, blob_info.blob_id.to_root_block_id());

    let expected_errors = vec![
        CorruptedError::BlobUnreadable {
            expected_blob_info: blob_info,
        },
        CorruptedError::NodeUnreadable {
            node_id: corrupted_node,
        },
    ];

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
    assert_eq!(&corrupted_node, root.to_root_block_id());

    let expected_errors =
        [
            CorruptedError::BlobUnreadable {
                expected_blob_info: BlobInfo {
                    blob_id: root,
                    blob_type: BlobType::Dir,
                    path: AbsolutePathBuf::root(),
                },
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

#[rstest]
#[case::file(|some_blobs: &SomeBlobs| some_blobs.large_file_1.clone())]
#[case::dir(|some_blobs: &SomeBlobs| some_blobs.large_dir_1.clone())]
#[case::symlink(|some_blobs: &SomeBlobs| some_blobs.large_symlink_1.clone())]
#[case::rootdir(|some_blobs: &SomeBlobs| some_blobs.root.clone())]
#[tokio::test(flavor = "multi_thread")]
async fn blob_with_unreadable_root_node(#[case] blob: impl FnOnce(&SomeBlobs) -> BlobInfo) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_info = blob(&some_blobs);
    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_if_dir_blob(blob_info.blob_id)
        .await;

    let CorruptInnerNodeResult {
        corrupted_node,
        orphaned_nodes,
    } = fs_fixture
        .corrupt_root_node_of_blob(blob_info.blob_id)
        .await;
    assert_eq!(&corrupted_node, blob_info.blob_id.to_root_block_id());

    let expected_errors =
        [
            CorruptedError::BlobUnreadable {
                expected_blob_info: blob_info,
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

#[rstest]
#[case::file(|some_blobs: &SomeBlobs| some_blobs.large_file_1.clone())]
#[case::dir(|some_blobs: &SomeBlobs| some_blobs.large_dir_1.clone())]
#[case::symlink(|some_blobs: &SomeBlobs| some_blobs.large_symlink_1.clone())]
#[case::rootdir(|some_blobs: &SomeBlobs| some_blobs.root.clone())]
#[tokio::test(flavor = "multi_thread")]
async fn blob_with_unreadable_inner_node(#[case] blob: impl FnOnce(&SomeBlobs) -> BlobInfo) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_info = blob(&some_blobs);
    if blob_info.blob_id == some_blobs.root.blob_id {
        // If we're testing the root dir, we need to make it large enough that we can remove some nodes
        fs_fixture
            .add_entries_to_make_dir_large(blob_info.blob_id)
            .await;
    }

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_if_dir_blob(blob_info.blob_id)
        .await;
    let CorruptInnerNodeResult {
        corrupted_node,
        orphaned_nodes,
    } = fs_fixture
        .corrupt_an_inner_node_of_a_large_blob(blob_info.blob_id)
        .await;
    assert_eq!(
        orphaned_descendant_blobs.is_empty(),
        blob_info.blob_type != BlobType::Dir,
        "test invariant"
    );

    let mut expected_errors = vec![CorruptedError::NodeUnreadable {
        node_id: corrupted_node,
    }];
    if blob_info.blob_type == BlobType::Dir {
        // Dirs are reported as unreadable because we try to read them when checking the file system.
        expected_errors.push(CorruptedError::BlobUnreadable {
            expected_blob_info: blob_info,
        });
    }
    expected_errors.extend(
        orphaned_nodes
            .into_iter()
            .map(|child| CorruptedError::NodeUnreferenced { node_id: child })
            .chain(orphaned_descendant_blobs.into_iter().map(|child| {
                CorruptedError::NodeUnreferenced {
                    node_id: *child.to_root_block_id(),
                }
            })),
    );

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

#[rstest]
#[case::file(|some_blobs: &SomeBlobs| some_blobs.large_file_1.clone())]
#[case::dir(|some_blobs: &SomeBlobs| some_blobs.large_dir_1.clone())]
#[case::symlink(|some_blobs: &SomeBlobs| some_blobs.large_symlink_1.clone())]
#[case::rootdir(|some_blobs: &SomeBlobs| some_blobs.root.clone())]
#[tokio::test(flavor = "multi_thread")]
async fn blob_with_unreadable_leaf_node(#[case] blob: impl FnOnce(&SomeBlobs) -> BlobInfo) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_info = blob(&some_blobs);

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_if_dir_blob(blob_info.blob_id)
        .await;
    assert_eq!(
        orphaned_descendant_blobs.is_empty(),
        blob_info.blob_type != BlobType::Dir,
        "test invariant"
    );
    let removed_node = fs_fixture.corrupt_a_leaf_node(blob_info.blob_id).await;

    let mut expected_errors = vec![CorruptedError::NodeUnreadable {
        node_id: removed_node,
    }];
    if blob_info.blob_type == BlobType::Dir {
        // Dirs are reported as unreadable because we try to read them when checking the file system.
        expected_errors.push(CorruptedError::BlobUnreadable {
            expected_blob_info: blob_info,
        });
    }
    expected_errors.extend(orphaned_descendant_blobs.into_iter().map(|child| {
        CorruptedError::NodeUnreferenced {
            node_id: *child.to_root_block_id(),
        }
    }));

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

#[rstest]
#[case::file(|some_blobs: &SomeBlobs| some_blobs.large_file_1.clone())]
#[case::dir(|some_blobs: &SomeBlobs| some_blobs.large_dir_1.clone())]
#[case::symlink(|some_blobs: &SomeBlobs| some_blobs.large_symlink_1.clone())]
#[case::rootdir(|some_blobs: &SomeBlobs| some_blobs.root.clone())]
#[tokio::test(flavor = "multi_thread")]
async fn blob_with_corrupted_some_nodes(#[case] blob: impl FnOnce(&SomeBlobs) -> BlobInfo) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_info = blob(&some_blobs);
    if blob_info.blob_id == some_blobs.root.blob_id {
        // If we're testing the root dir, we need to make it large enough that we can remove some nodes
        fs_fixture
            .add_entries_to_make_dir_large(blob_info.blob_id)
            .await;
    }

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_if_dir_blob(blob_info.blob_id)
        .await;
    let CorruptSomeNodesResult {
        corrupted_nodes,
        orphaned_nodes,
    } = fs_fixture
        .corrupt_some_nodes_of_a_large_blob(blob_info.blob_id)
        .await;
    assert_eq!(
        orphaned_descendant_blobs.is_empty(),
        blob_info.blob_type != BlobType::Dir,
        "test invariant"
    );

    let mut expected_errors = vec![];
    if blob_info.blob_type == BlobType::Dir {
        // Dirs are reported as unreadable because we try to read them when checking the file system.
        expected_errors.push(CorruptedError::BlobUnreadable {
            expected_blob_info: blob_info,
        });
    }

    expected_errors.extend(
        corrupted_nodes
            .into_iter()
            .map(|node_id| CorruptedError::NodeUnreadable { node_id })
            .chain(
                orphaned_nodes
                    .into_iter()
                    .map(|child| CorruptedError::NodeUnreferenced { node_id: child }),
            )
            .chain(orphaned_descendant_blobs.into_iter().map(|child| {
                CorruptedError::NodeUnreferenced {
                    node_id: *child.to_root_block_id(),
                }
            })),
    );

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}
