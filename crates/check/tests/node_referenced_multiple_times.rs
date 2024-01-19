use rand::{rngs::SmallRng, SeedableRng};

use cryfs_blobstore::BlobId;
use cryfs_blockstore::{BlockId, RemoveResult};
use cryfs_check::CorruptedError;

mod common;
use common::entry_helpers::{
    find_inner_node_id_and_parent, find_leaf_id_and_parent, remove_subtree,
};
use common::fixture::FilesystemFixture;

/// get two leaves, delete leaf1 and replace the entry in its parent with leaf2 so that there are now two references to leaf2.
async fn remove_leaf_and_replace_in_parent_with_another_existing_leaf(
    fs_fixture: &FilesystemFixture,
    root1: BlockId,
    root2: BlockId,
) -> BlockId {
    fs_fixture
        .update_nodestore(|nodestore| {
            Box::pin(async move {
                let mut rng = SmallRng::seed_from_u64(0);
                let (leaf1_id, mut parent1, leaf1_index) =
                    find_leaf_id_and_parent(nodestore, root1, &mut rng).await;
                let (leaf2_id, parent2, _leaf2_index) =
                    find_leaf_id_and_parent(nodestore, root2, &mut rng).await;
                assert_ne!(leaf1_id, leaf2_id);
                assert_ne!(*parent1.block_id(), *parent2.block_id());
                std::mem::drop(parent2);

                parent1.update_child(leaf1_index, &leaf2_id);
                std::mem::drop(parent1);
                let remove_result = nodestore.remove_by_id(&leaf1_id).await.unwrap();
                assert_eq!(RemoveResult::SuccessfullyRemoved, remove_result);

                leaf2_id
            })
        })
        .await
}

/// get two inner nodes at given depths, delete node1 and replace the entry in its parent with node2 so that there are now two references to node2.
async fn remove_inner_node_and_replace_in_parent_with_another_existing_inner_node(
    fs_fixture: &FilesystemFixture,
    root1: BlockId,
    depth1: u8,
    root2: BlockId,
    depth2: u8,
) -> BlockId {
    fs_fixture
        .update_nodestore(|nodestore| {
            Box::pin(async move {
                let mut rng = SmallRng::seed_from_u64(0);
                let (node1_id, mut parent1, node1_index) =
                    find_inner_node_id_and_parent(nodestore, root1, depth1, &mut rng).await;
                let (node2_id, parent2, _node2_index) =
                    find_inner_node_id_and_parent(nodestore, root2, depth2, &mut rng).await;
                assert_ne!(node2_id, node1_id);
                assert_ne!(node1_id, *parent2.block_id());
                assert_ne!(node2_id, *parent2.block_id());
                assert_ne!(node1_id, *parent1.block_id());
                assert_ne!(node2_id, *parent1.block_id());
                assert_ne!(*parent1.block_id(), *parent2.block_id());
                std::mem::drop(parent2);

                parent1.update_child(node1_index, &node2_id);
                std::mem::drop(parent1);
                remove_subtree(nodestore, node1_id).await;

                node2_id
            })
        })
        .await
}

/// get an inner node at the given depth, delete it and replace the entry in its parent with root2 so that there are now two references to root2.
async fn remove_inner_node_and_replace_in_parent_with_root_node(
    fs_fixture: &FilesystemFixture,
    root1: BlockId,
    depth: u8,
    root2: BlockId,
) -> BlockId {
    fs_fixture
        .update_nodestore(|nodestore| {
            Box::pin(async move {
                let mut rng = SmallRng::seed_from_u64(0);
                let (node1_id, mut parent1, node1_index) =
                    find_inner_node_id_and_parent(nodestore, root1, depth, &mut rng).await;
                assert_ne!(root1, node1_id);
                assert_ne!(root1, *parent1.block_id());
                assert_ne!(root2, node1_id);
                assert_ne!(root2, *parent1.block_id());

                parent1.update_child(node1_index, &root2);
                std::mem::drop(parent1);
                remove_subtree(nodestore, node1_id).await;

                root2
            })
        })
        .await
}

async fn test_blob_with_leaf_node_referenced_multiple_times(
    fs_fixture: FilesystemFixture,
    blob1: BlobId,
    blob2: BlobId,
) {
    let node_id = remove_leaf_and_replace_in_parent_with_another_existing_leaf(
        &fs_fixture,
        *blob1.to_root_block_id(),
        *blob2.to_root_block_id(),
    )
    .await;

    let errors = fs_fixture.run_cryfs_check().await;
    assert_eq!(
        vec![CorruptedError::NodeReferencedMultipleTimes { node_id }],
        errors
    );
}

async fn test_blob_with_inner_node_referenced_multiple_times(
    fs_fixture: FilesystemFixture,
    blob1: BlobId,
    depth1: u8,
    blob2: BlobId,
    depth2: u8,
) {
    let node_id = remove_inner_node_and_replace_in_parent_with_another_existing_inner_node(
        &fs_fixture,
        *blob1.to_root_block_id(),
        depth1,
        *blob2.to_root_block_id(),
        depth2,
    )
    .await;

    let errors = fs_fixture.run_cryfs_check().await;
    assert_eq!(
        vec![CorruptedError::NodeReferencedMultipleTimes { node_id }],
        errors
    );
}

async fn test_blob_with_root_node_referenced_multiple_times(
    fs_fixture: FilesystemFixture,
    blob1: BlobId,
    blob2: BlobId,
) {
    let node_id = remove_inner_node_and_replace_in_parent_with_root_node(
        &fs_fixture,
        *blob1.to_root_block_id(),
        5,
        *blob2.to_root_block_id(),
    )
    .await;

    let errors = fs_fixture.run_cryfs_check().await;
    assert_eq!(
        vec![CorruptedError::NodeReferencedMultipleTimes { node_id }],
        errors
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_leaf_node_referenced_multiple_times_from_same_file() {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    test_blob_with_leaf_node_referenced_multiple_times(
        fs_fixture,
        some_blobs.large_file_1,
        some_blobs.large_file_1,
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_inner_node_referenced_multiple_times_from_same_file_with_same_depth() {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    test_blob_with_inner_node_referenced_multiple_times(
        fs_fixture,
        some_blobs.large_file_1,
        5,
        some_blobs.large_file_1,
        5,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_inner_node_referenced_multiple_times_from_same_file_with_different_depth() {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    test_blob_with_inner_node_referenced_multiple_times(
        fs_fixture,
        some_blobs.large_file_1,
        5,
        some_blobs.large_file_1,
        7,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_root_node_referenced_from_same_file() {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    test_blob_with_root_node_referenced_multiple_times(
        fs_fixture,
        some_blobs.large_file_1,
        some_blobs.large_file_1,
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_leaf_node_referenced_multiple_times_from_different_file() {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;

    test_blob_with_leaf_node_referenced_multiple_times(
        fs_fixture,
        some_blobs.large_file_1,
        some_blobs.large_file_2,
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_inner_node_referenced_multiple_times_from_different_file_with_same_depth() {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    test_blob_with_inner_node_referenced_multiple_times(
        fs_fixture,
        some_blobs.large_file_1,
        5,
        some_blobs.large_file_2,
        5,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_inner_node_referenced_multiple_times_from_different_file_with_different_depth() {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    test_blob_with_inner_node_referenced_multiple_times(
        fs_fixture,
        some_blobs.large_file_1,
        5,
        some_blobs.large_file_2,
        7,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_root_node_referenced_from_different_file() {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    test_blob_with_root_node_referenced_multiple_times(
        fs_fixture,
        some_blobs.large_file_1,
        some_blobs.large_file_2,
    )
    .await;
}

// TODO These currently don't work because the dir gets corrupted and it's children become unreferenced.
// #[tokio::test(flavor = "multi_thread")]
// async fn file_with_leaf_node_referenced_multiple_times_from_different_dir() {
//     let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
//     test_blob_with_leaf_node_referenced_multiple_times(
//         fs_fixture,
//         some_blobs.large_dir_1,
//         some_blobs.large_file_1,
//     )
//     .await;
// }
// #[tokio::test(flavor = "multi_thread")]
// async fn file_with_inner_node_referenced_multiple_times_from_different_dir_with_same_depth() {
//     let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
//     test_blob_with_inner_node_referenced_multiple_times(
//         fs_fixture,
//         some_blobs.large_dir_1,
//         5,
//         some_blobs.large_file_1,
//         5,
//     )
//     .await
// }
// #[tokio::test(flavor = "multi_thread")]
// async fn file_with_inner_node_referenced_multiple_times_from_different_dir_with_different_depth() {
//     let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
//     test_blob_with_inner_node_referenced_multiple_times(
//         fs_fixture,
//         some_blobs.large_dir_1,
//         5,
//         some_blobs.large_file_1,
//         7,
//     )
//     .await
// }
// #[tokio::test(flavor = "multi_thread")]
// async fn file_with_root_node_referenced_from_different_dir() {
//     let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
//     test_blob_with_root_node_referenced_multiple_times(
//         fs_fixture,
//         some_blobs.large_dir_1,
//         some_blobs.large_file_1,
//     )
//     .await;
// }

#[tokio::test(flavor = "multi_thread")]
async fn file_with_leaf_node_referenced_multiple_times_from_different_symlink() {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    test_blob_with_leaf_node_referenced_multiple_times(
        fs_fixture,
        some_blobs.large_symlink_1,
        some_blobs.large_file_1,
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_inner_node_referenced_multiple_times_from_different_symlink_with_same_depth() {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    test_blob_with_inner_node_referenced_multiple_times(
        fs_fixture,
        some_blobs.large_symlink_1,
        5,
        some_blobs.large_file_1,
        5,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_inner_node_referenced_multiple_times_from_different_symlink_with_different_depth(
) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    test_blob_with_inner_node_referenced_multiple_times(
        fs_fixture,
        some_blobs.large_symlink_1,
        5,
        some_blobs.large_file_1,
        7,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_root_node_referenced_from_different_symlink() {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    test_blob_with_root_node_referenced_multiple_times(
        fs_fixture,
        some_blobs.large_symlink_1,
        some_blobs.large_file_1,
    )
    .await;
}

// TODO test same things for dirs and symlinks
