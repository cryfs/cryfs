//! Tests where individual nodes are unreadable

use rstest::rstest;

use cryfs_check::{BlobReference, BlobReferenceWithId, BlobUnreadableError, NodeUnreadableError};
use cryfs_cryfs::filesystem::fsblobstore::BlobType;
use cryfs_utils::testutils::asserts::assert_unordered_vec_eq;

mod common;
use common::entry_helpers::{
    expect_blobs_to_have_unreferenced_root_nodes, expect_nodes_to_be_unreferenced, SomeBlobs,
};
use common::fixture::{
    CorruptInnerNodeResult, CorruptLeafNodeResult, CorruptSomeNodesResult, FilesystemFixture,
};

#[rstest]
#[case::file(|some_blobs: &SomeBlobs| some_blobs.empty_file.clone())]
#[case::dir(|some_blobs: &SomeBlobs| some_blobs.empty_dir.clone())]
#[case::symlink(|some_blobs: &SomeBlobs| some_blobs.empty_symlink.clone())]
#[tokio::test(flavor = "multi_thread")]
async fn blob_with_unreadable_single_node(
    #[case] blob: impl FnOnce(&SomeBlobs) -> BlobReferenceWithId,
) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_info = blob(&some_blobs);

    let CorruptInnerNodeResult {
        corrupted_node,
        corrupted_node_info,
        orphaned_nodes,
    } = fs_fixture
        .corrupt_root_node_of_blob(blob_info.clone())
        .await;
    assert_eq!(0, orphaned_nodes.len(), "test precondition violated");
    assert_eq!(&corrupted_node, blob_info.blob_id.to_root_block_id());

    let expected_errors = vec![
        BlobUnreadableError {
            blob_id: blob_info.blob_id,
            referenced_as: [blob_info.referenced_as].into_iter().collect(),
        }
        .into(),
        NodeUnreadableError {
            node_id: corrupted_node,
            referenced_as: [corrupted_node_info.into()].into_iter().collect(),
        }
        .into(),
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
        corrupted_node_info,
        orphaned_nodes,
    } = fs_fixture
        .corrupt_root_node_of_blob(BlobReferenceWithId {
            blob_id: root,
            referenced_as: BlobReference::root_dir(),
        })
        .await;
    assert_eq!(0, orphaned_nodes.len(), "test precondition violated");
    assert_eq!(&corrupted_node, root.to_root_block_id());

    let expected_errors = vec![
        BlobUnreadableError {
            blob_id: root,
            referenced_as: [BlobReference::root_dir()].into_iter().collect(),
        }
        .into(),
        NodeUnreadableError {
            node_id: corrupted_node,
            referenced_as: [corrupted_node_info.into()].into_iter().collect(),
        }
        .into(),
    ];

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

#[rstest]
#[case::file(|some_blobs: &SomeBlobs| some_blobs.large_file_1.clone())]
#[case::dir(|some_blobs: &SomeBlobs| some_blobs.large_dir_1.clone())]
#[case::symlink(|some_blobs: &SomeBlobs| some_blobs.large_symlink_1.clone())]
#[case::rootdir(|some_blobs: &SomeBlobs| some_blobs.root.clone())]
#[tokio::test(flavor = "multi_thread")]
async fn blob_with_unreadable_root_node(
    #[case] blob: impl FnOnce(&SomeBlobs) -> BlobReferenceWithId,
) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_info = blob(&some_blobs);
    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_if_dir_blob(blob_info.blob_id)
        .await;
    let expected_errors_from_orphaned_descendant_blobs =
        expect_blobs_to_have_unreferenced_root_nodes(&fs_fixture, orphaned_descendant_blobs).await;

    let CorruptInnerNodeResult {
        corrupted_node,
        corrupted_node_info,
        orphaned_nodes,
    } = fs_fixture
        .corrupt_root_node_of_blob(blob_info.clone())
        .await;
    assert_eq!(&corrupted_node, blob_info.blob_id.to_root_block_id());
    let expected_errors_from_orphaned_nodes =
        expect_nodes_to_be_unreferenced(&fs_fixture, orphaned_nodes).await;

    let expected_errors = [
        BlobUnreadableError {
            blob_id: blob_info.blob_id,
            referenced_as: [blob_info.referenced_as].into_iter().collect(),
        }
        .into(),
        NodeUnreadableError {
            node_id: corrupted_node,
            referenced_as: [corrupted_node_info.into()].into_iter().collect(),
        }
        .into(),
    ]
    .into_iter()
    .chain(expected_errors_from_orphaned_nodes)
    .chain(expected_errors_from_orphaned_descendant_blobs)
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
async fn blob_with_unreadable_inner_node(
    #[case] blob: impl FnOnce(&SomeBlobs) -> BlobReferenceWithId,
) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_info = blob(&some_blobs);
    if blob_info.blob_id == some_blobs.root.blob_id {
        // If we're testing the root dir, we need to make it large enough that we can remove some nodes
        fs_fixture
            .add_entries_to_make_dir_large(blob_info.clone())
            .await;
    }

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_if_dir_blob(blob_info.blob_id)
        .await;
    assert_eq!(
        orphaned_descendant_blobs.is_empty(),
        blob_info.referenced_as.blob_type != BlobType::Dir,
        "test invariant"
    );
    let expected_errors_from_orphaned_descendant_blobs =
        expect_blobs_to_have_unreferenced_root_nodes(&fs_fixture, orphaned_descendant_blobs).await;

    let CorruptInnerNodeResult {
        corrupted_node,
        corrupted_node_info,
        orphaned_nodes,
    } = fs_fixture
        .corrupt_an_inner_node_of_a_large_blob(blob_info.clone())
        .await;
    let expected_errors_from_orphaned_nodes =
        expect_nodes_to_be_unreferenced(&fs_fixture, orphaned_nodes).await;

    let mut expected_errors = vec![NodeUnreadableError {
        node_id: corrupted_node,
        referenced_as: [corrupted_node_info.into()].into_iter().collect(),
    }
    .into()];
    if blob_info.referenced_as.blob_type == BlobType::Dir {
        // Dirs are reported as unreadable because we try to read them when checking the file system.
        expected_errors.push(
            BlobUnreadableError {
                blob_id: blob_info.blob_id,
                referenced_as: [blob_info.referenced_as].into_iter().collect(),
            }
            .into(),
        );
    }
    expected_errors.extend(
        expected_errors_from_orphaned_nodes.chain(expected_errors_from_orphaned_descendant_blobs),
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
async fn blob_with_unreadable_leaf_node(
    #[case] blob: impl FnOnce(&SomeBlobs) -> BlobReferenceWithId,
) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_info = blob(&some_blobs);

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_if_dir_blob(blob_info.blob_id)
        .await;
    assert_eq!(
        orphaned_descendant_blobs.is_empty(),
        blob_info.referenced_as.blob_type != BlobType::Dir,
        "test invariant"
    );
    let expected_errors_from_orphaned_descendant_blobs =
        expect_blobs_to_have_unreferenced_root_nodes(&fs_fixture, orphaned_descendant_blobs).await;

    let CorruptLeafNodeResult {
        corrupted_node,
        corrupted_node_info,
    } = fs_fixture.corrupt_a_leaf_node(blob_info.clone()).await;

    let mut expected_errors = vec![NodeUnreadableError {
        node_id: corrupted_node,
        referenced_as: [corrupted_node_info.into()].into_iter().collect(),
    }
    .into()];
    if blob_info.referenced_as.blob_type == BlobType::Dir {
        // Dirs are reported as unreadable because we try to read them when checking the file system.
        expected_errors.push(
            BlobUnreadableError {
                blob_id: blob_info.blob_id,
                referenced_as: [blob_info.referenced_as].into_iter().collect(),
            }
            .into(),
        );
    }
    expected_errors.extend(expected_errors_from_orphaned_descendant_blobs);

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

#[rstest]
#[case::file(|some_blobs: &SomeBlobs| some_blobs.large_file_1.clone())]
#[case::dir(|some_blobs: &SomeBlobs| some_blobs.large_dir_1.clone())]
#[case::symlink(|some_blobs: &SomeBlobs| some_blobs.large_symlink_1.clone())]
#[case::rootdir(|some_blobs: &SomeBlobs| some_blobs.root.clone())]
#[tokio::test(flavor = "multi_thread")]
async fn blob_with_corrupted_some_nodes(
    #[case] blob: impl FnOnce(&SomeBlobs) -> BlobReferenceWithId,
) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let blob_info = blob(&some_blobs);
    if blob_info.blob_id == some_blobs.root.blob_id {
        // If we're testing the root dir, we need to make it large enough that we can remove some nodes
        fs_fixture
            .add_entries_to_make_dir_large(blob_info.clone())
            .await;
    }

    let orphaned_descendant_blobs = fs_fixture
        .get_descendants_if_dir_blob(blob_info.blob_id)
        .await;
    assert_eq!(
        orphaned_descendant_blobs.is_empty(),
        blob_info.referenced_as.blob_type != BlobType::Dir,
        "test invariant"
    );
    let expected_errors_from_orphaned_descendant_blobs =
        expect_blobs_to_have_unreferenced_root_nodes(&fs_fixture, orphaned_descendant_blobs).await;

    let CorruptSomeNodesResult {
        corrupted_nodes,
        orphaned_nodes,
    } = fs_fixture
        .corrupt_some_nodes_of_a_large_blob(blob_info.clone())
        .await;
    let expected_errors_from_orphaned_nodes =
        expect_nodes_to_be_unreferenced(&fs_fixture, orphaned_nodes).await;

    let mut expected_errors = vec![];
    if blob_info.referenced_as.blob_type == BlobType::Dir {
        // Dirs are reported as unreadable because we try to read them when checking the file system.
        expected_errors.push(
            BlobUnreadableError {
                blob_id: blob_info.blob_id,
                referenced_as: [blob_info.referenced_as].into_iter().collect(),
            }
            .into(),
        );
    }

    expected_errors.extend(
        corrupted_nodes
            .into_iter()
            .map(|(node_id, referenced_as)| {
                NodeUnreadableError {
                    node_id,
                    referenced_as,
                }
                .into()
            })
            .chain(expected_errors_from_orphaned_nodes)
            .chain(expected_errors_from_orphaned_descendant_blobs),
    );

    let errors = fs_fixture.run_cryfs_check().await;
    assert_unordered_vec_eq(expected_errors, errors);
}

// TODO Tests where CorruptedError::NodeUnreadable::referenced_as has zero or multiple references
