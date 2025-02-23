//! Tests where a node is referenced multiple times, either from the same or from a different blob

use pretty_assertions::{assert_eq, assert_ne};
use rand::{SeedableRng, rngs::SmallRng};
use rstest::rstest;
use rstest_reuse::{self, *};
use std::collections::HashSet;
use std::hash::Hash;
use std::num::NonZeroU8;

use cryfs_blockstore::{BlockId, RemoveResult};
use cryfs_check::{
    BlobReferenceWithId, BlobUnreadableError, CorruptedError, MaybeBlobReferenceWithId,
    MaybeNodeInfoAsSeenByLookingAtNode, NodeAndBlobReference, NodeMissingError,
    NodeReferencedMultipleTimesError,
};

mod common;
use common::entry_helpers::{
    SomeBlobs, expect_blobs_to_have_unreferenced_root_nodes, find_inner_node_id_and_parent,
    find_leaf_id_and_parent, remove_subtree,
};
use common::fixture::FilesystemFixture;

#[derive(Debug)]
struct ReplaceInParentResult {
    node_id: BlockId,
    original_parent_id: BlockId,
    additional_parent_id: BlockId,
}

/// get two leaves, delete leaf1 and replace the entry in its parent with leaf2 so that there are now two references to leaf2.
async fn remove_leaf_and_replace_in_parent_with_another_existing_leaf(
    fs_fixture: &FilesystemFixture,
    root1: BlockId,
    root2: BlockId,
) -> ReplaceInParentResult {
    fs_fixture
        .update_nodestore(|nodestore| {
            Box::pin(async move {
                let mut rng = SmallRng::seed_from_u64(0);
                let (leaf1_id, mut parent1, leaf1_index) =
                    find_leaf_id_and_parent(nodestore, root1, &mut rng).await;
                let (leaf2_id, parent2, _leaf2_index) =
                    find_leaf_id_and_parent(nodestore, root2, &mut rng).await;
                let parent1_id = *parent1.block_id();
                let parent2_id = *parent2.block_id();
                assert_ne!(leaf1_id, leaf2_id);
                assert_ne!(parent1_id, parent2_id);
                std::mem::drop(parent2);

                parent1.update_child(leaf1_index, &leaf2_id);
                std::mem::drop(parent1);
                let remove_result = nodestore.remove_by_id(&leaf1_id).await.unwrap();
                assert_eq!(RemoveResult::SuccessfullyRemoved, remove_result);

                ReplaceInParentResult {
                    node_id: leaf2_id,
                    original_parent_id: parent2_id,
                    additional_parent_id: parent1_id,
                }
            })
        })
        .await
}

/// get two inner nodes at given depths, delete node1 and replace the entry in its parent with node2 so that there are now two references to node2.
async fn remove_inner_node_and_replace_in_parent_with_another_existing_inner_node(
    fs_fixture: &FilesystemFixture,
    root1: BlockId,
    depth_distance_from_root_1: u8,
    root2: BlockId,
    depth_distance_from_root_2: u8,
) -> ReplaceInParentResult {
    fs_fixture
        .update_nodestore(|nodestore| {
            Box::pin(async move {
                let mut rng = SmallRng::seed_from_u64(0);
                let (node1_id, mut parent1, node1_index) = find_inner_node_id_and_parent(
                    nodestore,
                    root1,
                    depth_distance_from_root_1,
                    &mut rng,
                )
                .await;
                let (node2_id, parent2, _node2_index) = find_inner_node_id_and_parent(
                    nodestore,
                    root2,
                    depth_distance_from_root_2,
                    &mut rng,
                )
                .await;
                let parent1_id = *parent1.block_id();
                let parent2_id = *parent2.block_id();
                assert_ne!(node2_id, node1_id);
                assert_ne!(node1_id, parent2_id);
                assert_ne!(node2_id, parent2_id);
                assert_ne!(node1_id, parent1_id);
                assert_ne!(node2_id, parent1_id);
                assert_ne!(parent1_id, parent2_id);
                std::mem::drop(parent2);

                parent1.update_child(node1_index, &node2_id);
                std::mem::drop(parent1);
                remove_subtree(nodestore, node1_id).await;

                ReplaceInParentResult {
                    node_id: node2_id,
                    original_parent_id: parent2_id,
                    additional_parent_id: parent1_id,
                }
            })
        })
        .await
}

#[derive(Debug)]
struct ReplaceInParentWithRootResult {
    node_id: BlockId,
    additional_parent_id: BlockId,
}

/// get an inner node at the given depth, delete it and replace the entry in its parent with root2 so that there are now two references to root2.
async fn remove_inner_node_and_replace_in_parent_with_root_node(
    fs_fixture: &FilesystemFixture,
    root1: BlockId,
    depth_distance_from_root: u8,
    root2: BlockId,
) -> ReplaceInParentWithRootResult {
    fs_fixture
        .update_nodestore(|nodestore| {
            Box::pin(async move {
                let mut rng = SmallRng::seed_from_u64(0);
                let (node1_id, mut parent1, node1_index) = find_inner_node_id_and_parent(
                    nodestore,
                    root1,
                    depth_distance_from_root,
                    &mut rng,
                )
                .await;
                assert_ne!(root1, node1_id);
                assert_ne!(root1, *parent1.block_id());
                assert_ne!(root2, node1_id);
                assert_ne!(root2, *parent1.block_id());

                parent1.update_child(node1_index, &root2);
                let parent1_id = *parent1.block_id();
                std::mem::drop(parent1);
                remove_subtree(nodestore, node1_id).await;

                ReplaceInParentWithRootResult {
                    node_id: root2,
                    additional_parent_id: parent1_id,
                }
            })
        })
        .await
}

#[template]
#[rstest]
// #[case::file_referenced_from_same_file(|some_blobs: &SomeBlobs| (some_blobs.large_file_1.clone(), some_blobs.large_file_1.clone()))]
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
// #[case::symlink_referenced_from_same_symlink(|some_blobs: &SomeBlobs| (some_blobs.large_symlink_1.clone(), some_blobs.large_symlink_1.clone()))]
#[case::symlink_referenced_from_different_symlink(|some_blobs: &SomeBlobs| (some_blobs.large_symlink_2.clone(), some_blobs.large_symlink_1.clone()))]
#[case::symlink_referenced_from_different_file(|some_blobs: &SomeBlobs| (some_blobs.large_file_1.clone(), some_blobs.large_symlink_1.clone()))]
#[case::symlink_referenced_from_different_dir(|some_blobs: &SomeBlobs| (some_blobs.large_dir_1.clone(), some_blobs.large_symlink_1.clone()))]
// TODO #[case::symlink_referenced_from_parent_dir(|some_blobs: &SomeBlobs| (some_blobs.dir2.clone(), some_blobs.dir2_large_symlink_1.clone()))]
// TODO #[case::symlink_referenced_from_grandparent_dir(|some_blobs: &SomeBlobs| (some_blobs.dir2.clone(), some_blobs.dir2_dir7_large_symlink_1.clone()))]
#[tokio::test(flavor = "multi_thread")]
fn test_case_with_multiple_reference_scenarios(
    #[case] blobs: impl FnOnce(&SomeBlobs) -> (BlobReferenceWithId, BlobReferenceWithId),
) {
}

async fn errors_allowed_from_blob_being_unreadable(
    fs_fixture: &FilesystemFixture,
    blob_info: BlobReferenceWithId,
) -> HashSet<CorruptedError> {
    let blob_errors = [
        BlobUnreadableError {
            blob_id: blob_info.blob_id,
            referenced_as: [blob_info.referenced_as.clone()].into_iter().collect(),
        }
        .into(),
        // TODO Why is NodeMissing necessary here? Without it, tests seem to become flaky because it seems to be sometimes thrown
        NodeMissingError {
            node_id: *blob_info.blob_id.to_root_block_id(),
            referenced_as: [NodeAndBlobReference::RootNode {
                belongs_to_blob: BlobReferenceWithId {
                    blob_id: blob_info.blob_id,
                    referenced_as: blob_info.referenced_as,
                },
            }]
            .into_iter()
            .collect(),
        }
        .into(),
    ]
    .into_iter();
    if fs_fixture.is_dir_blob(blob_info.blob_id).await {
        expect_blobs_to_have_unreferenced_root_nodes(
            fs_fixture,
            fs_fixture
                .get_descendants_of_dir_blob(blob_info.blob_id)
                .await,
        )
        .await
        .into_iter()
        .chain(blob_errors)
        .collect()
    } else {
        blob_errors.collect()
    }
}

#[apply(test_case_with_multiple_reference_scenarios)]
async fn leaf_node_referenced_multiple_times(
    #[case] blobs: impl FnOnce(&SomeBlobs) -> (BlobReferenceWithId, BlobReferenceWithId),
) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let (blob1, blob2) = blobs(&some_blobs);

    // Depending on how this modifies the dir blob, it could make it unreadable.
    // So let's ignore any errors that could be caused by that.
    // Note: This is indeterministic. Dir entries are ordered by blob id and in some test
    // runs this could make the blob unreadable while in others it wouldn't. So we have to
    // actually ignore these errors and allow for both cases to avoid test flakiness.
    let ignored_errors =
        errors_allowed_from_blob_being_unreadable(&fs_fixture, blob1.clone()).await;

    let replace_result = remove_leaf_and_replace_in_parent_with_another_existing_leaf(
        &fs_fixture,
        *blob1.blob_id.to_root_block_id(),
        *blob2.blob_id.to_root_block_id(),
    )
    .await;

    let errors = fs_fixture.run_cryfs_check().await;
    let errors = remove_all(errors, ignored_errors);
    assert_eq!(
        vec![CorruptedError::NodeReferencedMultipleTimes(
            NodeReferencedMultipleTimesError {
                node_id: replace_result.node_id,
                node_info: MaybeNodeInfoAsSeenByLookingAtNode::LeafNode,
                referenced_as: [
                    NodeAndBlobReference::NonRootLeafNode {
                        belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                            blob_id: blob1.blob_id,
                            referenced_as: blob1.referenced_as,
                        },
                        parent_id: replace_result.additional_parent_id,
                    },
                    NodeAndBlobReference::NonRootLeafNode {
                        belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                            blob_id: blob2.blob_id,
                            referenced_as: blob2.referenced_as,
                        },
                        parent_id: replace_result.original_parent_id,
                    }
                ]
                .into_iter()
                .collect(),
            }
        )],
        errors,
    );
}

#[apply(test_case_with_multiple_reference_scenarios)]
async fn inner_node_referenced_multiple_times(
    #[case] blobs: impl FnOnce(&SomeBlobs) -> (BlobReferenceWithId, BlobReferenceWithId),
    // with_same_depth and with_different_depth
    #[values((5, 5), (5, 7))] depths_distance_from_root: (u8, u8),
) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let (blob1, blob2) = blobs(&some_blobs);

    let expected_referenced_depth_1 = fs_fixture
        .get_node_depth(*blob1.blob_id.to_root_block_id())
        .await
        - depths_distance_from_root.0;
    let expected_referenced_depth_1 =
        NonZeroU8::new(expected_referenced_depth_1).expect("test invariant violated");
    let expected_referenced_depth_2 = fs_fixture
        .get_node_depth(*blob2.blob_id.to_root_block_id())
        .await
        - depths_distance_from_root.1;
    let expected_referenced_depth_2 =
        NonZeroU8::new(expected_referenced_depth_2).expect("test invariant violated");

    // Depending on how this modifies the dir blob, it could make it unreadable.
    // So let's ignore any errors that could be caused by that.
    // Note: This is indeterministic. Dir entries are ordered by blob id and in some test
    // runs this could make the blob unreadable while in others it wouldn't. So we have to
    // actually ignore these errors and allow for both cases to avoid test flakiness.
    let ignored_errors =
        errors_allowed_from_blob_being_unreadable(&fs_fixture, blob1.clone()).await;

    let replace_result = remove_inner_node_and_replace_in_parent_with_another_existing_inner_node(
        &fs_fixture,
        *blob1.blob_id.to_root_block_id(),
        depths_distance_from_root.0,
        *blob2.blob_id.to_root_block_id(),
        depths_distance_from_root.1,
    )
    .await;

    let expected_depth = fs_fixture.get_node_depth(replace_result.node_id).await;
    let expected_depth = NonZeroU8::new(expected_depth).expect("test invariant violated");

    let errors = fs_fixture.run_cryfs_check().await;
    let errors = remove_all(errors, ignored_errors.clone());
    assert_eq!(
        vec![CorruptedError::NodeReferencedMultipleTimes(
            NodeReferencedMultipleTimesError {
                node_id: replace_result.node_id,
                node_info: MaybeNodeInfoAsSeenByLookingAtNode::InnerNode {
                    depth: expected_depth,
                },
                referenced_as: [
                    NodeAndBlobReference::NonRootInnerNode {
                        belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                            blob_id: blob1.blob_id,
                            referenced_as: blob1.referenced_as,
                        },
                        parent_id: replace_result.additional_parent_id,
                        depth: expected_referenced_depth_1,
                    },
                    NodeAndBlobReference::NonRootInnerNode {
                        belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                            blob_id: blob2.blob_id,
                            referenced_as: blob2.referenced_as,
                        },
                        parent_id: replace_result.original_parent_id,
                        depth: expected_referenced_depth_2,
                    }
                ]
                .into_iter()
                .collect(),
            }
        ),],
        errors
    );
}

#[apply(test_case_with_multiple_reference_scenarios)]
async fn root_node_referenced(
    #[case] blobs: impl FnOnce(&SomeBlobs) -> (BlobReferenceWithId, BlobReferenceWithId),
) {
    let (fs_fixture, some_blobs) = FilesystemFixture::new_with_some_blobs().await;
    let (blob1, blob2) = blobs(&some_blobs);

    let expected_referenced_depth = fs_fixture
        .get_node_depth(*blob1.blob_id.to_root_block_id())
        .await
        - DEPTH_DISTANCE_FROM_ROOT;
    let expected_referenced_depth =
        NonZeroU8::new(expected_referenced_depth).expect("test invariant violated");

    // Depending on how this modifies the dir blob, it could make it unreadable.
    // So let's ignore any errors that could be caused by that.
    // Note: This is indeterministic. Dir entries are ordered by blob id and in some test
    // runs this could make the blob unreadable while in others it wouldn't. So we have to
    // actually ignore these errors and allow for both cases to avoid test flakiness.
    let ignored_errors =
        errors_allowed_from_blob_being_unreadable(&fs_fixture, blob1.clone()).await;

    const DEPTH_DISTANCE_FROM_ROOT: u8 = 5;
    let replace_result = remove_inner_node_and_replace_in_parent_with_root_node(
        &fs_fixture,
        *blob1.blob_id.to_root_block_id(),
        DEPTH_DISTANCE_FROM_ROOT,
        *blob2.blob_id.to_root_block_id(),
    )
    .await;

    let expected_depth = fs_fixture.get_node_depth(replace_result.node_id).await;
    let expected_node_info = if let Some(depth) = NonZeroU8::new(expected_depth) {
        MaybeNodeInfoAsSeenByLookingAtNode::InnerNode { depth }
    } else {
        MaybeNodeInfoAsSeenByLookingAtNode::LeafNode
    };

    let errors = fs_fixture.run_cryfs_check().await;
    let errors = remove_all(errors, ignored_errors);
    assert_eq!(
        vec![CorruptedError::NodeReferencedMultipleTimes(
            NodeReferencedMultipleTimesError {
                node_id: replace_result.node_id,
                node_info: expected_node_info,
                referenced_as: [
                    NodeAndBlobReference::NonRootInnerNode {
                        belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                            blob_id: blob1.blob_id,
                            referenced_as: blob1.referenced_as,
                        },
                        parent_id: replace_result.additional_parent_id,
                        depth: expected_referenced_depth,
                    },
                    NodeAndBlobReference::RootNode {
                        belongs_to_blob: BlobReferenceWithId {
                            blob_id: blob2.blob_id,
                            referenced_as: blob2.referenced_as,
                        }
                    },
                ]
                .into_iter()
                .collect(),
            }
        )],
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

// TODO Test node referenced multiple times but doesn't actually exist (check how we did that for blob_referenced_multiple_times by adding a parameter for blob_status)
// TODO tests where we get NodeReferencedMultipleTimes::node_info == NodeInfoAsExpectedByEntryInParent::Unreadable
// TODO Do the tests already cover cases where a (leaf/inner/root) node is referenced as a different node type or do we have to add it?
// TODO Tests where NodeReferencedMultipleTimes::referenced_as contains a `belongs_to_blob: Unreachable`
