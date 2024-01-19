use rand::{rngs::SmallRng, SeedableRng};

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
    root: BlockId,
) -> BlockId {
    fs_fixture
        .update_nodestore(|nodestore| {
            Box::pin(async move {
                let mut rng = SmallRng::seed_from_u64(0);
                let (leaf1_id, parent1, _leaf1_index) =
                    find_leaf_id_and_parent(nodestore, root, &mut rng).await;
                let (leaf2_id, mut parent2, leaf2_index) =
                    find_leaf_id_and_parent(nodestore, root, &mut rng).await;
                assert_ne!(leaf2_id, leaf1_id);
                assert_ne!(*parent1.block_id(), *parent2.block_id());
                std::mem::drop(parent1);

                parent2.update_child(leaf2_index, &leaf1_id);
                std::mem::drop(parent2);
                let remove_result = nodestore.remove_by_id(&leaf2_id).await.unwrap();
                assert_eq!(RemoveResult::SuccessfullyRemoved, remove_result);

                leaf1_id
            })
        })
        .await
}

/// get two inner nodes at given depths, delete node1 and replace the entry in its parent with node2 so that there are now two references to node2.
async fn remove_inner_node_and_replace_in_parent_with_another_existing_inner_node(
    fs_fixture: &FilesystemFixture,
    root: BlockId,
    depth1: u8,
    depth2: u8,
) -> BlockId {
    fs_fixture
        .update_nodestore(|nodestore| {
            Box::pin(async move {
                let mut rng = SmallRng::seed_from_u64(0);
                let (node1_id, parent1, _node1_index) =
                    find_inner_node_id_and_parent(nodestore, root, depth1, &mut rng).await;
                let (node2_id, mut parent2, node2_index) =
                    find_inner_node_id_and_parent(nodestore, root, depth2, &mut rng).await;
                assert_ne!(node2_id, node1_id);
                assert_ne!(node1_id, *parent2.block_id());
                assert_ne!(node2_id, *parent2.block_id());
                assert_ne!(node1_id, *parent1.block_id());
                assert_ne!(node2_id, *parent1.block_id());
                assert_ne!(*parent1.block_id(), *parent2.block_id());
                std::mem::drop(parent1);

                parent2.update_child(node2_index, &node1_id);
                std::mem::drop(parent2);
                remove_subtree(nodestore, node2_id).await;

                node1_id
            })
        })
        .await
}

async fn remove_inner_node_and_replace_in_parent_with_another_existing_inner_node_of_same_depth(
    fs_fixture: &FilesystemFixture,
    root: BlockId,
) -> BlockId {
    const DEPTH: u8 = 5;
    remove_inner_node_and_replace_in_parent_with_another_existing_inner_node(
        fs_fixture, root, DEPTH, DEPTH,
    )
    .await
}

async fn remove_inner_node_and_replace_in_parent_with_another_existing_inner_node_of_different_depth(
    fs_fixture: &FilesystemFixture,
    root: BlockId,
) -> BlockId {
    const DEPTH1: u8 = 5;
    const DEPTH2: u8 = 7;
    remove_inner_node_and_replace_in_parent_with_another_existing_inner_node(
        fs_fixture, root, DEPTH1, DEPTH2,
    )
    .await
}

async fn remove_inner_node_and_replace_in_parent_with_root_node(
    fs_fixture: &FilesystemFixture,
    root: BlockId,
) -> BlockId {
    const DEPTH: u8 = 6;
    fs_fixture
        .update_nodestore(|nodestore| {
            Box::pin(async move {
                let mut rng = SmallRng::seed_from_u64(0);
                let (node1_id, mut parent1, node1_index) =
                    find_inner_node_id_and_parent(nodestore, root, DEPTH, &mut rng).await;
                assert_ne!(root, node1_id);
                assert_ne!(root, *parent1.block_id());

                parent1.update_child(node1_index, &root);
                std::mem::drop(parent1);
                remove_subtree(nodestore, node1_id).await;

                root
            })
        })
        .await
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_leaf_node_referenced_multiple_times_from_same_file() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let blob_id = some_blobs.large_file;
    let node_id = remove_leaf_and_replace_in_parent_with_another_existing_leaf(
        &fs_fixture,
        *blob_id.to_root_block_id(),
    )
    .await;

    let errors = fs_fixture.run_cryfs_check().await;
    assert_eq!(
        vec![CorruptedError::NodeReferencedMultipleTimes { node_id }],
        errors
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_inner_node_referenced_multiple_times_from_same_file_with_same_depth() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let blob_id = some_blobs.large_file;
    let node_id =
        remove_inner_node_and_replace_in_parent_with_another_existing_inner_node_of_same_depth(
            &fs_fixture,
            *blob_id.to_root_block_id(),
        )
        .await;

    let errors = fs_fixture.run_cryfs_check().await;
    assert_eq!(
        vec![CorruptedError::NodeReferencedMultipleTimes { node_id }],
        errors
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_inner_node_referenced_multiple_times_from_same_file_with_different_depth() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let blob_id = some_blobs.large_file;
    let node_id =
        remove_inner_node_and_replace_in_parent_with_another_existing_inner_node_of_different_depth(
            &fs_fixture,
            *blob_id.to_root_block_id(),
        )
        .await;

    let errors = fs_fixture.run_cryfs_check().await;
    assert_eq!(
        vec![CorruptedError::NodeReferencedMultipleTimes { node_id }],
        errors
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_root_node_referenced_from_same_file() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let blob_id = some_blobs.large_file;
    let node_id = remove_inner_node_and_replace_in_parent_with_root_node(
        &fs_fixture,
        *blob_id.to_root_block_id(),
    )
    .await;

    let errors = fs_fixture.run_cryfs_check().await;
    assert_eq!(
        vec![CorruptedError::NodeReferencedMultipleTimes { node_id }],
        errors
    );
}

// TODO test referenced from different file

// TODO test same things for dirs and symlinks
