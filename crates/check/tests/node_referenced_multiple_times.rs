use rand::{rngs::SmallRng, SeedableRng};

use cryfs_blobstore::DataNodeStore;
use cryfs_blockstore::{BlockId, BlockStore, RemoveResult};
use cryfs_check::CorruptedError;

mod common;
use common::entry_helpers::{find_leaf_id, find_leaf_id_and_parent};
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
                let leaf1_id = find_leaf_id(nodestore, root, &mut rng).await;
                let (leaf2_id, mut parent2, leaf2_index) =
                    find_leaf_id_and_parent(nodestore, root, &mut rng).await;
                assert_ne!(leaf2_id, leaf1_id);

                parent2.update_child(leaf2_index, &leaf1_id);
                std::mem::drop(parent2);
                let remove_result = nodestore.remove_by_id(&leaf2_id).await.unwrap();
                assert_eq!(RemoveResult::SuccessfullyRemoved, remove_result);

                leaf1_id
            })
        })
        .await
}

#[tokio::test(flavor = "multi_thread")]
async fn file_with_leaf_node_referenced_multiple_times_from_same_file() {
    let fs_fixture = FilesystemFixture::new().await;
    let some_blobs = fs_fixture.create_some_blobs().await;

    let root = *some_blobs.large_file.to_root_block_id();
    let node_id =
        remove_leaf_and_replace_in_parent_with_another_existing_leaf(&fs_fixture, root).await;

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
