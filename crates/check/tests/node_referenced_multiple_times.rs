use rand::{rngs::SmallRng, SeedableRng};

use cryfs_blockstore::RemoveResult;
use cryfs_check::CorruptedError;

mod common;
use common::entry_helpers::find_leaf_node_and_parent;
use common::fixture::FilesystemFixture;

#[tokio::test(flavor = "multi_thread")]
async fn file_with_leaf_node_referenced_multiple_times_from_same_file() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let node_id = fs_fixture
        .update_nodestore(|nodestore| {
            Box::pin(async move {
                let root = *some_blobs.large_file.to_root_block_id();
                let mut rng = SmallRng::seed_from_u64(0);

                // get two leaves, delete leaf1 and replace the entry in its parent with leaf2 so that there are now two references to leaf2.

                let (leaf1, mut parent1, leaf1_index) =
                    find_leaf_node_and_parent(nodestore, root, &mut rng).await;
                let (leaf2, parent2, _leaf2_index) =
                    find_leaf_node_and_parent(nodestore, root, &mut rng).await;
                let leaf1_id = *leaf1.block_id();
                let leaf2_id = *leaf2.block_id();
                assert_ne!(parent1.block_id(), parent2.block_id()); // Note: If this fails, it might actually not even get here but cause a deadlock above when loading the second node.
                assert_ne!(leaf1_id, leaf2_id);
                std::mem::drop(parent2);
                std::mem::drop(leaf1);
                std::mem::drop(leaf2);

                parent1.update_child(leaf1_index, &leaf2_id);
                std::mem::drop(parent1);
                let remove_result = nodestore.remove_by_id(&leaf1_id).await.unwrap();
                assert_eq!(RemoveResult::SuccessfullyRemoved, remove_result);

                leaf2_id
            })
        })
        .await;

    let errors = fs_fixture.run_cryfs_check().await;
    assert_eq!(
        vec![CorruptedError::NodeReferencedMultipleTimes { node_id }],
        errors
    );
}

// TODO test inner node referenced multiple times
// TODO test root node referenced from within the file
// TODO test referenced from different file

// TODO test same things for dirs and symlinks
