//! Tests where a node is referenced multiple times, either from the same or from a different blob

use rand::{rngs::SmallRng, SeedableRng};
use rstest::rstest;
use rstest_reuse::{self, *};
use std::collections::HashSet;
use std::hash::Hash;

use cryfs_blockstore::{BlockId, RemoveResult};
use cryfs_check::CorruptedError;

mod common;
use common::entry_helpers::{
    find_inner_node_id_and_parent, find_leaf_id_and_parent, remove_subtree, CreatedBlobInfo,
    SomeBlobs,
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

#[template]
#[rstest]
#[case::file_referenced_from_same_file(|some_blobs: &SomeBlobs| (some_blobs.large_file_1.clone(), some_blobs.large_file_1.clone()))]
#[case::file_referenced_from_different_file(|some_blobs: &SomeBlobs| (some_blobs.large_file_2.clone(), some_blobs.large_file_1.clone()))]
#[case::file_referenced_from_different_dir(|some_blobs: &SomeBlobs| (some_blobs.large_dir_1.clone(), some_blobs.large_file_1.clone()))]
#[case::file_referenced_from_different_symlink(|some_blobs: &SomeBlobs| (some_blobs.large_symlink_1.clone(), some_blobs.large_file_1.clone()))]
// TODO #[case::file_referenced_from_parent_dir(|some_blobs: &SomeBlobs| (some_blobs.dir2.clone(), some_blobs.dir2_large_file_1.clone()))]
// TODO #[case::file_referenced_from_grandparent_dir(|some_blobs: &SomeBlobs| (some_blobs.dir2.clone(), some_blobs.dir2_dir7_large_file_1.clone()))]
// TODO For some reason this causes a deadlock. Maybe because it runs into an infinite loop loading that dir again and again?
// #[case::dir_referenced_from_same_dir(|some_blobs: &SomeBlobs| (some_blobs.large_dir_1.clone(), some_blobs.large_dir_1.clone()))]
// TODO #[case::dir_referenced_from_child_dir(|some_blobs: &SomeBlobs| (some_blobs.dir1_dir3.clone(), some_blobs.dir1.clone()))]
// TODO #[case::dir_referenced_from_child_file(|some_blobs: &SomeBlobs| (some_blobs.dir2_large_file_1.clone(), some_blobs.dir2.clone()))]
// TODO #[case::dir_referenced_from_child_symlink(|some_blobs: &SomeBlobs| (some_blobs.dir2_large_symlink_1.clone(), some_blobs.dir2.clone()))]
// TODO #[case::dir_referenced_from_grandchild_dir(|some_blobs: &SomeBlobs| (some_blobs.dir1_dir3_dir5.clone(), some_blobs.dir1.clone()))]
// TODO #[case::dir_referenced_from_grandchild_file(|some_blobs: &SomeBlobs| (some_blobs.dir2_dir7_large_file_1.clone(), some_blobs.dir2.clone()))]
// TODO #[case::dir_referenced_from_grandchild_symlink(|some_blobs: &SomeBlobs| (some_blobs.dir2_dir7_large_symlink_1.clone(), some_blobs.dir2.clone()))]
// TODO #[case::dir_referenced_from_parent_dir(|some_blobs: &SomeBlobs| (some_blobs.dir1.clone(), some_blobs.dir1_dir3.clone()))]
// TODO #[case::dir_referenced_from_grandparent_dir(|some_blobs: &SomeBlobs| (some_blobs.dir1.clone(), some_blobs.dir1_dir3_dir5.clone()))]
// TODO leaf_node_referenced_multiple_times::case_05_dir_referenced_from_different_dir is flaky. Probably because sometimes, it aligns just right so that the blob ids from the other dir blob remain valid.
// Repro:
// ```fish
// cargo t --release --test node_referenced_multiple_times root_node_referenced_from_same_file::case_05_dir_referenced_from_same_dir                                                                                                                                                                                                    (base)
// set iter 1
// while RUST_BACKTRACE=1 RUST_LOG=debug /home/heinzi/projects/cryfs/target/release/deps/node_referenced_multiple_times-b97db7071c4492dd --nocapture leaf_node_referenced_multiple_times::case_05_dir_referenced_from_different_dir
//    echo again $iter
//    set iter (math $iter + 1)
// end
// ````
// #[case::dir_referenced_from_different_dir(|some_blobs: &SomeBlobs| (some_blobs.large_dir_2.clone(), some_blobs.large_dir_1.clone()))]
#[case::dir_referenced_from_different_file(|some_blobs: &SomeBlobs| (some_blobs.large_file_1.clone(), some_blobs.large_dir_1.clone()))]
#[case::dir_referenced_from_different_symlink(|some_blobs: &SomeBlobs| (some_blobs.large_symlink_1.clone(), some_blobs.large_dir_1.clone()))]
#[case::symlink_referenced_from_same_symlink(|some_blobs: &SomeBlobs| (some_blobs.large_symlink_1.clone(), some_blobs.large_symlink_1.clone()))]
#[case::symlink_referenced_from_different_symlink(|some_blobs: &SomeBlobs| (some_blobs.large_symlink_2.clone(), some_blobs.large_symlink_1.clone()))]
#[case::symlink_referenced_from_different_file(|some_blobs: &SomeBlobs| (some_blobs.large_file_1.clone(), some_blobs.large_symlink_1.clone()))]
#[case::symlink_referenced_from_different_dir(|some_blobs: &SomeBlobs| (some_blobs.large_dir_1.clone(), some_blobs.large_symlink_1.clone()))]
// TODO #[case::symlink_referenced_from_parent_dir(|some_blobs: &SomeBlobs| (some_blobs.dir2.clone(), some_blobs.dir2_large_symlink_1.clone()))]
// TODO #[case::symlink_referenced_from_grandparent_dir(|some_blobs: &SomeBlobs| (some_blobs.dir2.clone(), some_blobs.dir2_dir7_large_symlink_1.clone()))]
#[tokio::test(flavor = "multi_thread")]
fn test_case_with_multiple_reference_scenarios(
    #[case] blobs: impl FnOnce(&SomeBlobs) -> (CreatedBlobInfo, CreatedBlobInfo),
) {
}

async fn errors_allowed_from_dir_blob_being_unreadable(
    fs_fixture: &FilesystemFixture,
    blob_info: CreatedBlobInfo,
) -> HashSet<CorruptedError> {
    if fs_fixture.is_dir_blob(blob_info.blob_id).await {
        fs_fixture
            .get_descendants_of_dir_blob(blob_info.blob_id)
            .await
            .into_iter()
            .map(|descendant| CorruptedError::NodeUnreferenced {
                node_id: *descendant.to_root_block_id(),
            })
            .chain(
                [
                    CorruptedError::BlobUnreadable {
                        blob_id: blob_info.blob_id,
                        expected_blob_info: blob_info.blob_info.clone(),
                    },
                    // TODO Why is BlobMissing necessary here? Without it, tests seem to become flaky because it is sometimes thrown
                    CorruptedError::BlobMissing {
                        blob_id: blob_info.blob_id,
                        expected_blob_info: blob_info.blob_info,
                    },
                ]
                .into_iter(),
            )
            .collect()
    } else {
        HashSet::new()
    }
}

#[apply(test_case_with_multiple_reference_scenarios)]
async fn leaf_node_referenced_multiple_times(
    #[case] blobs: impl FnOnce(&SomeBlobs) -> (CreatedBlobInfo, CreatedBlobInfo),
) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let (blob1, blob2) = blobs(&some_blobs);

    // Depending on how this modifies the dir blob, it could make it unreadable.
    // So let's ignore any errors that could be caused by that.
    // Note: This is indeterministic. Dir entries are ordered by blob id and in some test
    // runs this could make the blob unreadable while in others it wouldn't. So we have to
    // actually ignore these errors and allow for both cases to avoid test flakiness.
    let ignored_errors =
        errors_allowed_from_dir_blob_being_unreadable(&fs_fixture, blob1.clone()).await;

    let node_id = remove_leaf_and_replace_in_parent_with_another_existing_leaf(
        &fs_fixture,
        *blob1.blob_id.to_root_block_id(),
        *blob2.blob_id.to_root_block_id(),
    )
    .await;

    let errors = fs_fixture.run_cryfs_check().await;
    let errors = remove_all(errors, ignored_errors);
    assert_eq!(
        vec![CorruptedError::NodeReferencedMultipleTimes { node_id }],
        errors,
    );
}

#[apply(test_case_with_multiple_reference_scenarios)]
async fn inner_node_referenced_multiple_times(
    #[case] blobs: impl FnOnce(&SomeBlobs) -> (CreatedBlobInfo, CreatedBlobInfo),
    // with_same_depth and with_different_depth
    #[values((5, 5), (5, 7))] depths: (u8, u8),
) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let (blob1, blob2) = blobs(&some_blobs);

    // Depending on how this modifies the dir blob, it could make it unreadable.
    // So let's ignore any errors that could be caused by that.
    // Note: This is indeterministic. Dir entries are ordered by blob id and in some test
    // runs this could make the blob unreadable while in others it wouldn't. So we have to
    // actually ignore these errors and allow for both cases to avoid test flakiness.
    let ignored_errors =
        errors_allowed_from_dir_blob_being_unreadable(&fs_fixture, blob1.clone()).await;

    let node_id = remove_inner_node_and_replace_in_parent_with_another_existing_inner_node(
        &fs_fixture,
        *blob1.blob_id.to_root_block_id(),
        depths.0,
        *blob2.blob_id.to_root_block_id(),
        depths.1,
    )
    .await;

    let errors = fs_fixture.run_cryfs_check().await;
    let errors = remove_all(errors, ignored_errors);
    assert_eq!(
        vec![CorruptedError::NodeReferencedMultipleTimes { node_id }],
        errors
    );
}

#[apply(test_case_with_multiple_reference_scenarios)]
async fn root_node_referenced(
    #[case] blobs: impl FnOnce(&SomeBlobs) -> (CreatedBlobInfo, CreatedBlobInfo),
) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let (blob1, blob2) = blobs(&some_blobs);

    // Depending on how this modifies the dir blob, it could make it unreadable.
    // So let's ignore any errors that could be caused by that.
    // Note: This is indeterministic. Dir entries are ordered by blob id and in some test
    // runs this could make the blob unreadable while in others it wouldn't. So we have to
    // actually ignore these errors and allow for both cases to avoid test flakiness.
    let ignored_errors =
        errors_allowed_from_dir_blob_being_unreadable(&fs_fixture, blob1.clone()).await;

    let node_id = remove_inner_node_and_replace_in_parent_with_root_node(
        &fs_fixture,
        *blob1.blob_id.to_root_block_id(),
        5,
        *blob2.blob_id.to_root_block_id(),
    )
    .await;

    let errors = fs_fixture.run_cryfs_check().await;
    let errors = remove_all(errors, ignored_errors);
    assert_eq!(
        vec![CorruptedError::NodeReferencedMultipleTimes { node_id }],
        errors
    );
}

fn remove_all<T>(mut source: Vec<T>, to_remove: HashSet<T>) -> Vec<T>
where
    T: PartialEq + Eq + Hash,
{
    source.retain(|item| !to_remove.contains(item));
    source
}
