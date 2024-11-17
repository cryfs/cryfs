//! This module contains tests that ensure that tree operations only access the minimal required number of nodes.

use anyhow::Result;
use byte_unit::Byte;
use divrem::DivCeil;
use futures::future::BoxFuture;

use cryfs_blockstore::{
    ActionCounts, BlockId, BlockStore, InMemoryBlockStore, LockingBlockStore, SharedBlockStore,
    TrackingBlockStore,
};

use super::super::testutils::*;
use crate::on_blocks::{
    data_node_store::NodeLayout,
    data_tree_store::{DataTree, DataTreeStore},
};

const LAYOUT: NodeLayout = NodeLayout {
    block_size: Byte::from_u64(40),
};
const NUM_LEAVES: u64 = 100;
const DEPTH: u8 = expected_depth_for_num_leaves(NUM_LEAVES, LAYOUT);
const NUM_NODES: u64 = expected_num_nodes_for_num_leaves(NUM_LEAVES, LAYOUT);
const NUM_BYTES: u64 = NUM_LEAVES * LAYOUT.max_bytes_per_leaf() as u64;

mod testutils {
    use super::*;

    pub async fn with_treestore_and_tracking_blockstore(
        f: impl for<'a> FnOnce(
            &'a DataTreeStore<SharedBlockStore<TrackingBlockStore<InMemoryBlockStore>>>,
            &'a SharedBlockStore<TrackingBlockStore<InMemoryBlockStore>>,
        ) -> BoxFuture<'a, ()>,
    ) {
        let mut blockstore =
            SharedBlockStore::new(TrackingBlockStore::new(InMemoryBlockStore::new()));
        let mut treestore = DataTreeStore::new(
            LockingBlockStore::new(SharedBlockStore::clone(&blockstore)),
            LAYOUT.block_size,
        )
        .await
        .unwrap();
        f(&treestore, &blockstore).await;
        treestore.async_drop().await.unwrap();
        blockstore.async_drop().await.unwrap();
    }

    pub async fn create_empty_tree(
        treestore: &DataTreeStore<SharedBlockStore<TrackingBlockStore<InMemoryBlockStore>>>,
        blockstore: &SharedBlockStore<TrackingBlockStore<InMemoryBlockStore>>,
    ) -> BlockId {
        let tree = treestore.create_tree().await.unwrap();
        let id = *tree.root_node_id();
        std::mem::drop(tree);
        treestore.clear_cache_slow().await.unwrap();
        blockstore.get_and_reset_totals();
        id
    }

    pub async fn create_nonempty_tree(
        treestore: &DataTreeStore<SharedBlockStore<TrackingBlockStore<InMemoryBlockStore>>>,
        blockstore: &SharedBlockStore<TrackingBlockStore<InMemoryBlockStore>>,
    ) -> BlockId {
        let mut tree = treestore.create_tree().await.unwrap();
        tree.resize_num_bytes(NUM_LEAVES * LAYOUT.max_bytes_per_leaf() as u64)
            .await
            .unwrap();
        let id = *tree.root_node_id();
        std::mem::drop(tree);
        treestore.clear_cache_slow().await.unwrap();
        blockstore.get_and_reset_totals();
        id
    }

    pub fn num_inner_nodes_above_num_consecutive_leaves(first_leaf: u64, num_leaves: u64) -> u64 {
        num_existing_inner_nodes_above_num_consecutive_leaves(
            first_leaf,
            num_leaves,
            first_leaf + num_leaves,
        )
        .get_assert_same()
    }

    #[derive(Clone, Copy, Debug)]
    pub struct NodeCount {
        existing: u64,
        total: u64,
    }
    impl NodeCount {
        fn get_assert_same(&self) -> u64 {
            assert_eq!(self.existing, self.total);
            self.total
        }
    }

    pub fn num_existing_inner_nodes_above_num_consecutive_leaves(
        first_leaf: u64,
        num_leaves: u64,
        existing_num_leaves: u64,
    ) -> NodeCount {
        let mut num_inner_nodes = NodeCount {
            existing: 0,
            total: 0,
        };
        let mut num_leaves_per_node_on_current_level = 1;
        let mut num_nodes_current_level = num_leaves;
        for _ in 0..DEPTH {
            num_leaves_per_node_on_current_level *= LAYOUT.max_children_per_inner_node() as u64;
            num_nodes_current_level = DivCeil::div_ceil(
                num_nodes_current_level,
                LAYOUT.max_children_per_inner_node() as u64,
            );
            let first_node_current_level = first_leaf / num_leaves_per_node_on_current_level;
            let last_node_current_level =
                (first_leaf + num_leaves - 1) / num_leaves_per_node_on_current_level;
            let last_existing_node_current_level =
                (existing_num_leaves - 1) / num_leaves_per_node_on_current_level;
            num_inner_nodes.existing +=
                last_existing_node_current_level.saturating_sub(first_node_current_level) + 1;
            num_inner_nodes.total +=
                last_node_current_level.saturating_sub(first_node_current_level) + 1;
        }
        assert_eq!(
            1, num_nodes_current_level,
            "Root level should only have one node"
        );
        num_inner_nodes
    }

    pub fn num_nodes_written_when_growing_tree(old_num_leaves: u64, new_num_leaves: u64) -> u64 {
        assert!(new_num_leaves >= old_num_leaves);
        let mut num_nodes_written = 0;
        let mut old_num_nodes_current_level = old_num_leaves;
        let mut new_num_nodes_current_level = new_num_leaves;
        for _ in 0..DEPTH {
            assert!(new_num_nodes_current_level >= old_num_nodes_current_level);
            num_nodes_written += new_num_nodes_current_level - old_num_nodes_current_level;
            old_num_nodes_current_level = DivCeil::div_ceil(
                old_num_nodes_current_level,
                LAYOUT.max_children_per_inner_node() as u64,
            );
            new_num_nodes_current_level = DivCeil::div_ceil(
                new_num_nodes_current_level,
                LAYOUT.max_children_per_inner_node() as u64,
            );
        }
        assert_eq!(
            1, old_num_nodes_current_level,
            "Root level should only have one node"
        );
        while new_num_nodes_current_level > 1 {
            // We need to grow the tree size by one level.
            // Precondition: we already created one node on the current level for the old tree 9or there was one preexisting in the old tree)
            let old_num_nodes_current_level = 1;
            num_nodes_written += new_num_nodes_current_level - old_num_nodes_current_level;
            // We also need to add a node one level above ourselves to create the new root
            // and because the next loop iteration assumes that the "old" tree already has a node there
            // Because the blob it has to remain the same, this new root actually is written to the
            // current root, and a new node is created with the previous data of the current root.
            // This means we have to write 2 additional nodes.
            num_nodes_written += 2;
            new_num_nodes_current_level = DivCeil::div_ceil(
                new_num_nodes_current_level,
                LAYOUT.max_children_per_inner_node() as u64,
            );
            // Postcondition: All nodes on the current level were created plus a new root node on the level one above
        }
        assert_eq!(
            1, new_num_nodes_current_level,
            "Root level should only have one node"
        );
        num_nodes_written
    }
}

mod num_nodes {
    use super::testutils::*;
    use super::*;

    #[tokio::test]
    async fn empty_tree() {
        with_treestore_and_tracking_blockstore(|treestore, blockstore| {
            Box::pin(async move {
                let block_id = create_empty_tree(treestore, blockstore).await;
                let mut tree = treestore.load_tree(block_id).await.unwrap().unwrap();

                assert_eq!(
                    ActionCounts {
                        loaded: 1,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );
                assert_eq!(1, tree.num_nodes().await.unwrap());
                assert_eq!(
                    ActionCounts {
                        loaded: 0,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );
            })
        })
        .await
    }

    #[tokio::test]
    async fn nonempty_tree() {
        with_treestore_and_tracking_blockstore(|treestore, blockstore| {
            Box::pin(async move {
                let block_id = create_nonempty_tree(treestore, blockstore).await;
                let mut tree = treestore.load_tree(block_id).await.unwrap().unwrap();
                assert_eq!(
                    ActionCounts {
                        loaded: 1,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );

                // The tree has `DEPTH+1` right border nodes. A call to num_nodes() should look up `DEPTH-1` nodes,
                // because the root node is already loaded and, as opposed to num_bytes, the leaf doesn't need to be loaded.
                assert_eq!(NUM_NODES, tree.num_nodes().await.unwrap());
                assert_eq!(
                    ActionCounts {
                        loaded: (DEPTH - 1) as u32,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );

                // Calling num_nodes() again shouldn't load any more nodes, it's now cached.
                assert_eq!(NUM_NODES, tree.num_nodes().await.unwrap());
                assert_eq!(
                    ActionCounts {
                        loaded: 0,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );
            })
        })
        .await
    }
}

mod num_bytes {
    use super::testutils::*;
    use super::*;

    #[tokio::test]
    async fn empty_tree() {
        with_treestore_and_tracking_blockstore(|treestore, blockstore| {
            Box::pin(async move {
                let block_id = create_empty_tree(treestore, blockstore).await;
                let mut tree = treestore.load_tree(block_id).await.unwrap().unwrap();
                assert_eq!(
                    ActionCounts {
                        loaded: 1,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );

                assert_eq!(0, tree.num_bytes().await.unwrap());
                assert_eq!(
                    ActionCounts {
                        loaded: 0,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );
            })
        })
        .await
    }

    #[tokio::test]
    async fn nonempty_tree() {
        with_treestore_and_tracking_blockstore(|treestore, blockstore| {
            Box::pin(async move {
                let block_id = create_nonempty_tree(treestore, blockstore).await;
                let mut tree = treestore.load_tree(block_id).await.unwrap().unwrap();
                assert_eq!(
                    ActionCounts {
                        loaded: 1,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );

                // The tree has `DEPTH+1` right border nodes. A call to num_bytes() should look up `DEPTH` nodes,
                // because the root node is already loaded and, as opposed to num_leaves, the leaf needs to be loaded.
                assert_eq!(NUM_BYTES, tree.num_bytes().await.unwrap());
                assert_eq!(
                    ActionCounts {
                        loaded: DEPTH as u32,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );

                // Calling num_bytes() again shouldn't load any more nodes, it's now cached.
                assert_eq!(NUM_BYTES, tree.num_bytes().await.unwrap());
                assert_eq!(
                    ActionCounts {
                        loaded: 0,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );
            })
        })
        .await
    }

    #[tokio::test]
    async fn calling_num_bytes_after_num_nodes_known() {
        with_treestore_and_tracking_blockstore(|treestore, blockstore| {
            Box::pin(async move {
                let block_id = create_nonempty_tree(treestore, blockstore).await;
                let mut tree = treestore.load_tree(block_id).await.unwrap().unwrap();
                assert_eq!(
                    ActionCounts {
                        loaded: 1,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );

                // The tree has `DEPTH+1` right border nodes. A call to num_nodes() should look up `DEPTH-1` nodes,
                // because the root node is already loaded and, as opposed to num_bytes, the leaf doesn't need to be loaded.
                assert_eq!(NUM_NODES, tree.num_nodes().await.unwrap());
                assert_eq!(
                    ActionCounts {
                        loaded: (DEPTH - 1) as u32,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );

                // Calling num_bytes() should now only have to load one node (i.e. the leaf)
                assert_eq!(NUM_BYTES, tree.num_bytes().await.unwrap());
                assert_eq!(
                    ActionCounts {
                        loaded: 1,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );

                // Calling num_bytes() again shouldn't load any more nodes, it's now cached.
                assert_eq!(NUM_BYTES, tree.num_bytes().await.unwrap());
                assert_eq!(
                    ActionCounts {
                        loaded: 0,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );
            })
        })
        .await
    }
}

mod create_tree {
    use super::testutils::*;
    use super::*;

    #[tokio::test]
    async fn only_creates_one_leaf() {
        with_treestore_and_tracking_blockstore(|treestore, blockstore| {
            Box::pin(async move {
                treestore.create_tree().await.unwrap();
                treestore.clear_cache_slow().await.unwrap();
                assert_eq!(
                    ActionCounts {
                        exists: 1,
                        stored: 1,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );
            })
        })
        .await
    }
}

mod try_create_tree {
    use super::testutils::*;
    use super::*;

    #[tokio::test]
    async fn nonexisting_tree() {
        with_treestore_and_tracking_blockstore(|treestore, blockstore| {
            Box::pin(async move {
                let block_id = BlockId::from_hex("1bacce38f52f578d4196331b8deadbe9").unwrap();
                treestore.try_create_tree(block_id).await.unwrap().unwrap();
                treestore.clear_cache_slow().await.unwrap();
                assert_eq!(
                    ActionCounts {
                        exists: 1,
                        stored: 1,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );
            })
        })
        .await
    }

    #[tokio::test]
    async fn existing_tree() {
        with_treestore_and_tracking_blockstore(|treestore, blockstore| {
            Box::pin(async move {
                let block_id = BlockId::from_hex("1bacce38f52f578d4196331b8deadbe9").unwrap();

                // First make sure that the tree already exists
                treestore.try_create_tree(block_id).await.unwrap().unwrap();
                treestore.clear_cache_slow().await.unwrap();
                blockstore.get_and_reset_totals();

                // And then run our creation op
                assert!(treestore.try_create_tree(block_id).await.unwrap().is_none());
                treestore.clear_cache_slow().await.unwrap();
                assert_eq!(
                    ActionCounts {
                        exists: 1,
                        stored: 0,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );
            })
        })
        .await
    }
}

macro_rules! instantiate_read_tests {
    ($read_fn:ident) => {
        #[tokio::test]
        async fn givenNumBytesAlreadyLoaded_readOneLeaf() {
            with_treestore_and_tracking_blockstore(|treestore, blockstore| {
                Box::pin(async move {
                    let block_id = create_nonempty_tree(treestore, blockstore).await;
                    let mut tree = treestore.load_tree(block_id).await.unwrap().unwrap();
                    // Load num_bytes so that the size cache is already loaded
                    tree.num_bytes().await.unwrap();
                    treestore.clear_unloaded_blocks_from_cache().await.unwrap();
                    blockstore.get_and_reset_totals();

                    let read_offset = (40.5 * LAYOUT.max_bytes_per_leaf() as f32) as u64;

                    ($read_fn(&mut tree, read_offset, 1)).await.unwrap();

                    // The tree has `DEPTH+1` nodes on the path from the root to the leaf. The root shouldn't get loaded because
                    // it is already loaded inside of the `tree` instance. That means reading the leaf should load `DEPTH` nodes.
                    assert_eq!(
                        ActionCounts {
                            loaded: DEPTH as u32,
                            ..Default::default()
                        },
                        blockstore.get_and_reset_totals(),
                    );
                })
            })
            .await
        }

        #[tokio::test]
        async fn givenNumBytesNotLoadedYet_readOneLeaf() {
            with_treestore_and_tracking_blockstore(|treestore, blockstore| {
                Box::pin(async move {
                    let block_id = create_nonempty_tree(treestore, blockstore).await;
                    let mut tree = treestore.load_tree(block_id).await.unwrap().unwrap();
                    blockstore.get_and_reset_totals();

                    let read_offset = (40.5 * LAYOUT.max_bytes_per_leaf() as f32) as u64;

                    $read_fn(&mut tree, read_offset, 1).await.unwrap();

                    // The tree has `DEPTH+1` nodes on the path from the root to the leaf. The root shouldn't get loaded because
                    // it is already loaded inside of the `tree` instance. That means reading the leaf should load `DEPTH` nodes.
                    // TODO This should be `let expected_loaded =  DEPTH as u32,`
                    //      but the current implementation is inefficent because it first needs to load the num_bytes in to the cache
                    let expected_loaded = 14;
                    assert_eq!(
                        ActionCounts {
                            loaded: expected_loaded,
                            ..Default::default()
                        },
                        blockstore.get_and_reset_totals(),
                    );
                })
            })
            .await
        }

        #[tokio::test]
        async fn givenNumBytesAlreadyLoaded_readMultipleLeaves() {
            with_treestore_and_tracking_blockstore(|treestore, blockstore| {
                Box::pin(async move {
                    let block_id = create_nonempty_tree(treestore, blockstore).await;
                    let mut tree = treestore.load_tree(block_id).await.unwrap().unwrap();
                    // Load num_bytes so that the size cache is already loaded
                    tree.num_bytes().await.unwrap();
                    treestore.clear_unloaded_blocks_from_cache().await.unwrap();
                    blockstore.get_and_reset_totals();

                    const FIRST_ACCESSED_LEAF: u64 = 40;
                    const NUM_ACCESSED_LEAVES: u64 = 11;

                    let read_offset = ((FIRST_ACCESSED_LEAF as f32 + 0.5)
                        * LAYOUT.max_bytes_per_leaf() as f32) as u64;
                    let read_len =
                        (NUM_ACCESSED_LEAVES as usize - 1) * LAYOUT.max_bytes_per_leaf() as usize;

                    $read_fn(&mut tree, read_offset, read_len).await.unwrap();

                    let expected_num_loaded_inner_nodes =
                        num_inner_nodes_above_num_consecutive_leaves(
                            FIRST_ACCESSED_LEAF,
                            NUM_ACCESSED_LEAVES,
                        );
                    let expected_num_loaded_inner_nodes_without_root =
                        expected_num_loaded_inner_nodes - 1;
                    let expected_num_loaded_nodes =
                        expected_num_loaded_inner_nodes_without_root + NUM_ACCESSED_LEAVES;

                    assert_eq!(
                        ActionCounts {
                            loaded: expected_num_loaded_nodes as u32,
                            ..Default::default()
                        },
                        blockstore.get_and_reset_totals(),
                    );
                })
            })
            .await
        }

        #[tokio::test]
        async fn givenNumBytesNotLoadedYet_readMultipleLeaves() {
            with_treestore_and_tracking_blockstore(|treestore, blockstore| {
                Box::pin(async move {
                    let block_id = create_nonempty_tree(treestore, blockstore).await;
                    let mut tree = treestore.load_tree(block_id).await.unwrap().unwrap();
                    blockstore.get_and_reset_totals();

                    const FIRST_ACCESSED_LEAF: u64 = 40;
                    const NUM_ACCESSED_LEAVES: u64 = 11;

                    let read_offset = ((FIRST_ACCESSED_LEAF as f32 + 0.5)
                        * LAYOUT.max_bytes_per_leaf() as f32) as u64;
                    let read_len =
                        (NUM_ACCESSED_LEAVES as usize - 1) * LAYOUT.max_bytes_per_leaf() as usize;

                    $read_fn(&mut tree, read_offset, read_len).await.unwrap();

                    let expected_num_loaded_inner_nodes =
                        num_inner_nodes_above_num_consecutive_leaves(
                            FIRST_ACCESSED_LEAF,
                            NUM_ACCESSED_LEAVES,
                        );
                    let _expected_num_loaded_inner_nodes_without_root =
                        expected_num_loaded_inner_nodes - 1;
                    // TODO This should just be `let expected_num_loaded_nodes = expected_num_loaded_inner_nodes_without_root + NUM_ACCESSED_LEAVES`
                    //      but the current implementation is inefficent because it first needs to load the num_bytes in to the cache
                    let expected_num_loaded_nodes = 33;

                    assert_eq!(
                        ActionCounts {
                            loaded: expected_num_loaded_nodes as u32,
                            ..Default::default()
                        },
                        blockstore.get_and_reset_totals(),
                    );
                })
            })
            .await
        }
    };
}

#[allow(non_snake_case)]
mod read_bytes {
    use super::testutils::*;
    use super::*;

    async fn read_fn<B: BlockStore + Send + Sync>(
        tree: &mut DataTree<'_, B>,
        offset: u64,
        len: usize,
    ) -> Result<()> {
        let mut target = vec![0; len];
        tree.read_bytes(offset, &mut target).await?;
        Ok(())
    }

    instantiate_read_tests!(read_fn);
}

#[allow(non_snake_case)]
mod try_read_bytes {
    use super::testutils::*;
    use super::*;

    async fn read_fn<B: BlockStore + Send + Sync>(
        tree: &mut DataTree<'_, B>,
        offset: u64,
        len: usize,
    ) -> Result<()> {
        let mut target = vec![0; len];
        tree.try_read_bytes(offset, &mut target).await?;
        Ok(())
    }

    instantiate_read_tests!(read_fn);
}

#[allow(non_snake_case)]
mod read_all {
    use super::testutils::*;
    use super::*;

    #[tokio::test]
    async fn givenNumBytesAlreadyLoaded_readAll() {
        with_treestore_and_tracking_blockstore(|treestore, blockstore| {
            Box::pin(async move {
                let block_id = create_nonempty_tree(treestore, blockstore).await;
                let mut tree = treestore.load_tree(block_id).await.unwrap().unwrap();
                // Load num_bytes so that the size cache is already loaded
                tree.num_bytes().await.unwrap();
                treestore.clear_unloaded_blocks_from_cache().await.unwrap();
                blockstore.get_and_reset_totals();

                tree.read_all().await.unwrap();

                // We need to load the full tree except for the root node, which is already loaded in the `tree` instance.
                assert_eq!(
                    ActionCounts {
                        loaded: NUM_NODES as u32 - 1,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );
            })
        })
        .await
    }

    #[tokio::test]
    async fn givenNumBytesNotLoadedYet_readAll() {
        with_treestore_and_tracking_blockstore(|treestore, blockstore| {
            Box::pin(async move {
                let block_id = create_nonempty_tree(treestore, blockstore).await;
                let mut tree = treestore.load_tree(block_id).await.unwrap().unwrap();
                blockstore.get_and_reset_totals();

                tree.read_all().await.unwrap();

                // We need to load the full tree except for the root node, which is already loaded in the `tree` instance.
                assert_eq!(
                    ActionCounts {
                        loaded: NUM_NODES as u32 - 1,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );
            })
        })
        .await
    }
}

#[allow(non_snake_case)]
mod write_bytes {
    use super::testutils::*;
    use super::*;

    #[tokio::test]
    async fn givenNumBytesAlreadyLoaded_whenWritingDoesntGrowTree_writePartOfOneLeaf() {
        with_treestore_and_tracking_blockstore(|treestore, blockstore| {
            Box::pin(async move {
                let block_id = create_nonempty_tree(treestore, blockstore).await;
                let mut tree = treestore.load_tree(block_id).await.unwrap().unwrap();
                // Load num_bytes so that the size cache is already loaded
                tree.num_bytes().await.unwrap();
                treestore.clear_unloaded_blocks_from_cache().await.unwrap();
                blockstore.get_and_reset_totals();

                let write_offset = (40.5 * LAYOUT.max_bytes_per_leaf() as f32) as u64;

                tree.write_bytes(&[0; 1], write_offset).await.unwrap();

                // Even before writing, we need to load the inner nodes on the path to the leaf.
                // The tree has `DEPTH+1` nodes on the path from the root to the leaf. The root shouldn't get loaded because
                // it is already loaded inside of the `tree` instance. We also have to load the leaf itself, because we only write part of it.
                // That means reading the leaf should load `DEPTH` nodes.
                assert_eq!(
                    ActionCounts {
                        loaded: DEPTH as u32,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );

                // After flushing, the new content should have been written
                std::mem::drop(tree);
                treestore.clear_cache_slow().await.unwrap();
                assert_eq!(
                    ActionCounts {
                        stored: 1,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );
            })
        })
        .await
    }

    #[tokio::test]
    async fn givenNumBytesNotLoadedYet_whenWritingDoesntGrowTree_writePartOfOneLeaf() {
        with_treestore_and_tracking_blockstore(|treestore, blockstore| {
            Box::pin(async move {
                let block_id = create_nonempty_tree(treestore, blockstore).await;
                let mut tree = treestore.load_tree(block_id).await.unwrap().unwrap();
                blockstore.get_and_reset_totals();

                let write_offset = (40.5 * LAYOUT.max_bytes_per_leaf() as f32) as u64;

                tree.write_bytes(&[0; 1], write_offset).await.unwrap();

                // Even before writing, we need to load the inner nodes on the path to the leaf.
                // The tree has `DEPTH+1` nodes on the path from the root to the leaf. The root shouldn't get loaded because
                // it is already loaded inside of the `tree` instance. We also have to load the leaf itself, because we only write part of it.
                // That means reading the leaf should load `DEPTH` nodes.
                let expected_loaded = DEPTH as u32;
                assert_eq!(
                    ActionCounts {
                        loaded: expected_loaded,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );

                // After flushing, the new content should have been written
                std::mem::drop(tree);
                treestore.clear_cache_slow().await.unwrap();
                assert_eq!(
                    ActionCounts {
                        stored: 1,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );
            })
        })
        .await
    }

    #[tokio::test]
    async fn givenNumBytesAlreadyLoaded_whenWritingDoesntGrowTree_writeAFullLeaf() {
        with_treestore_and_tracking_blockstore(|treestore, blockstore| {
            Box::pin(async move {
                let block_id = create_nonempty_tree(treestore, blockstore).await;
                let mut tree = treestore.load_tree(block_id).await.unwrap().unwrap();
                // Load num_bytes so that the size cache is already loaded
                tree.num_bytes().await.unwrap();
                treestore.clear_unloaded_blocks_from_cache().await.unwrap();
                blockstore.get_and_reset_totals();

                let write_offset = 40 * LAYOUT.max_bytes_per_leaf();

                tree.write_bytes(
                    &[0; LAYOUT.max_bytes_per_leaf() as usize],
                    write_offset as u64,
                )
                .await
                .unwrap();

                // Even before writing, we need to load the inner nodes on the path to the leaf.
                // The tree has `DEPTH+1` nodes on the path from the root to the leaf. The root shouldn't get loaded because
                // it is already loaded inside of the `tree` instance. We don't have to load the leaf itself, because we fully overwrite it.
                // That means reading the leaf should load `DEPTH - 1` nodes.
                assert_eq!(
                    ActionCounts {
                        exists: 1, // TODO Why do we need exists here?
                        loaded: DEPTH as u32 - 1,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );

                // After flushing, the new content should have been written
                std::mem::drop(tree);
                treestore.clear_cache_slow().await.unwrap();
                assert_eq!(
                    ActionCounts {
                        stored: 1,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );
            })
        })
        .await
    }

    #[tokio::test]
    async fn givenNumBytesNotLoadedYet_whenWritingDoesntGrowTree_writeAFullLeaf() {
        with_treestore_and_tracking_blockstore(|treestore, blockstore| {
            Box::pin(async move {
                let block_id = create_nonempty_tree(treestore, blockstore).await;
                let mut tree = treestore.load_tree(block_id).await.unwrap().unwrap();
                blockstore.get_and_reset_totals();

                let write_offset = 40 * LAYOUT.max_bytes_per_leaf();

                tree.write_bytes(
                    &[0; LAYOUT.max_bytes_per_leaf() as usize],
                    write_offset as u64,
                )
                .await
                .unwrap();

                // Even before writing, we need to load the inner nodes on the path to the leaf.
                // The tree has `DEPTH+1` nodes on the path from the root to the leaf. The root shouldn't get loaded because
                // it is already loaded inside of the `tree` instance. We don't have to load the leaf itself, because we fully overwrite it.
                // That means reading the leaf should load `DEPTH - 1` nodes.
                let expected_loaded = DEPTH as u32 - 1;
                assert_eq!(
                    ActionCounts {
                        exists: 1, // TODO Why do we need exists here?
                        loaded: expected_loaded,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );

                // After flushing, the new content should have been written
                std::mem::drop(tree);
                treestore.clear_cache_slow().await.unwrap();
                assert_eq!(
                    ActionCounts {
                        stored: 1,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );
            })
        })
        .await
    }

    #[tokio::test]
    async fn givenNumBytesAlreadyLoaded_whenWritingDoesntGrowTree_writeMultipleLeaves() {
        with_treestore_and_tracking_blockstore(|treestore, blockstore| {
            Box::pin(async move {
                let block_id = create_nonempty_tree(treestore, blockstore).await;
                let mut tree = treestore.load_tree(block_id).await.unwrap().unwrap();
                // Load num_bytes so that the size cache is already loaded
                tree.num_bytes().await.unwrap();
                treestore.clear_unloaded_blocks_from_cache().await.unwrap();
                blockstore.get_and_reset_totals();

                const FIRST_ACCESSED_LEAF: u64 = 40;
                const NUM_ACCESSED_LEAVES: u64 = 11;

                let write_offset = ((FIRST_ACCESSED_LEAF as f32 + 0.5)
                    * LAYOUT.max_bytes_per_leaf() as f32) as u64;
                const WRITE_LEN: usize =
                    (NUM_ACCESSED_LEAVES as usize - 1) * LAYOUT.max_bytes_per_leaf() as usize;

                tree.write_bytes(&[0; WRITE_LEN], write_offset)
                    .await
                    .unwrap();

                // Even before writing, we need to load the inner nodes on the path to the leaf.
                let expected_num_loaded_inner_nodes = num_inner_nodes_above_num_consecutive_leaves(
                    FIRST_ACCESSED_LEAF,
                    NUM_ACCESSED_LEAVES,
                );
                let expected_num_loaded_inner_nodes_without_root =
                    expected_num_loaded_inner_nodes - 1;
                // We also have to load the first and the last leaf of the writing region because we're
                // only partially overwriting those
                let expected_num_loaded_nodes = expected_num_loaded_inner_nodes_without_root + 2;
                assert_eq!(
                    ActionCounts {
                        exists: NUM_ACCESSED_LEAVES as u32 - 2, // TODO Why do we need exists here?
                        loaded: expected_num_loaded_nodes as u32,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );

                // After flushing, the new content should have been written
                std::mem::drop(tree);
                treestore.clear_cache_slow().await.unwrap();
                assert_eq!(
                    ActionCounts {
                        stored: NUM_ACCESSED_LEAVES as u32,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );
            })
        })
        .await
    }

    #[tokio::test]
    async fn givenNumBytesNotLoadedYet_whenWritingDoesntGrowTree_writeMultipleLeaves() {
        with_treestore_and_tracking_blockstore(|treestore, blockstore| {
            Box::pin(async move {
                let block_id = create_nonempty_tree(treestore, blockstore).await;
                let mut tree = treestore.load_tree(block_id).await.unwrap().unwrap();
                blockstore.get_and_reset_totals();

                const FIRST_ACCESSED_LEAF: u64 = 40;
                const NUM_ACCESSED_LEAVES: u64 = 11;

                let write_offset = ((FIRST_ACCESSED_LEAF as f32 + 0.5)
                    * LAYOUT.max_bytes_per_leaf() as f32) as u64;
                const WRITE_LEN: usize =
                    (NUM_ACCESSED_LEAVES as usize - 1) * LAYOUT.max_bytes_per_leaf() as usize;

                tree.write_bytes(&[0; WRITE_LEN], write_offset)
                    .await
                    .unwrap();

                // Even before writing, we need to load the inner nodes on the path to the leaf.
                let expected_num_loaded_inner_nodes = num_inner_nodes_above_num_consecutive_leaves(
                    FIRST_ACCESSED_LEAF,
                    NUM_ACCESSED_LEAVES,
                );
                let expected_num_loaded_inner_nodes_without_root =
                    expected_num_loaded_inner_nodes - 1;
                // We also have to load the first and the last leaf of the writing region because we're
                // only partially overwriting those
                let expected_num_loaded_nodes = expected_num_loaded_inner_nodes_without_root + 2;

                assert_eq!(
                    ActionCounts {
                        exists: NUM_ACCESSED_LEAVES as u32 - 2, // TODO Why do we need exists here?
                        loaded: expected_num_loaded_nodes as u32,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );

                // After flushing, the new content should have been written
                std::mem::drop(tree);
                treestore.clear_cache_slow().await.unwrap();
                assert_eq!(
                    ActionCounts {
                        stored: NUM_ACCESSED_LEAVES as u32,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );
            })
        })
        .await
    }

    #[tokio::test]
    async fn givenNumBytesAlreadyLoaded_whenWritingGrowsTree_writePartOfOneLeaf() {
        with_treestore_and_tracking_blockstore(|treestore, blockstore| {
            Box::pin(async move {
                let block_id = create_nonempty_tree(treestore, blockstore).await;
                let mut tree = treestore.load_tree(block_id).await.unwrap().unwrap();
                // Load num_bytes so that the size cache is already loaded
                tree.num_bytes().await.unwrap();
                treestore.clear_unloaded_blocks_from_cache().await.unwrap();
                blockstore.get_and_reset_totals();

                const WRITTEN_LEAF_INDEX: u32 = 140;

                let write_offset =
                    ((WRITTEN_LEAF_INDEX as f32 + 0.5) * LAYOUT.max_bytes_per_leaf() as f32) as u64;

                tree.write_bytes(&[0; 1], write_offset).await.unwrap();

                // Even before writing, we need to load the inner nodes on the path to the leaf.
                // The tree has `DEPTH+1` nodes on the path from the root to the leaf. The root shouldn't get loaded because
                // it is already loaded inside of the `tree` instance. We also have to load the leaf itself, because we only write part of it.
                // That means reading the leaf should load `DEPTH` nodes.
                assert_eq!(
                    ActionCounts {
                        exists: 84, // TODO Why do we need these exist and can we calculate this number based on the tree structure?
                        loaded: DEPTH as u32,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );

                // After flushing, the new content should have been written
                std::mem::drop(tree);
                treestore.clear_cache_slow().await.unwrap();
                let expected_stored =
                    num_nodes_written_when_growing_tree(NUM_LEAVES, WRITTEN_LEAF_INDEX as u64 + 1)
                        as u32;
                assert_eq!(
                    ActionCounts {
                        stored: expected_stored + 3, // TODO Why + 3 ?
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );
            })
        })
        .await
    }

    #[tokio::test]
    async fn givenNumBytesNotLoadedYet_whenWritingGrowsTree_writePartOfOneLeaf() {
        with_treestore_and_tracking_blockstore(|treestore, blockstore| {
            Box::pin(async move {
                let block_id = create_nonempty_tree(treestore, blockstore).await;
                let mut tree = treestore.load_tree(block_id).await.unwrap().unwrap();
                blockstore.get_and_reset_totals();

                const WRITTEN_LEAF_INDEX: u32 = 140;
                let write_offset =
                    ((WRITTEN_LEAF_INDEX as f32 + 0.5) * LAYOUT.max_bytes_per_leaf() as f32) as u64;

                tree.write_bytes(&[0; 1], write_offset).await.unwrap();

                // Even before writing, we need to load the inner nodes on the path to the leaf.
                // The tree has `DEPTH+1` nodes on the path from the root to the leaf. The root shouldn't get loaded because
                // it is already loaded inside of the `tree` instance. We also have to load the leaf itself, because we only write part of it.
                // That means reading the leaf should load `DEPTH` nodes.
                let expected_loaded = DEPTH as u32;
                assert_eq!(
                    ActionCounts {
                        exists: 84, // TODO Why do we need these exist and can we calculate this number based on the tree structure?
                        loaded: expected_loaded,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );

                // After flushing, the new content should have been written
                std::mem::drop(tree);
                treestore.clear_cache_slow().await.unwrap();
                let expected_stored =
                    num_nodes_written_when_growing_tree(NUM_LEAVES, WRITTEN_LEAF_INDEX as u64 + 1)
                        as u32;
                assert_eq!(
                    ActionCounts {
                        stored: expected_stored + 3, // TODO Why + 3 ?
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );
            })
        })
        .await
    }

    #[tokio::test]
    async fn givenNumBytesAlreadyLoaded_whenWritingGrowsTree_writeAFullLeaf() {
        with_treestore_and_tracking_blockstore(|treestore, blockstore| {
            Box::pin(async move {
                let block_id = create_nonempty_tree(treestore, blockstore).await;
                let mut tree = treestore.load_tree(block_id).await.unwrap().unwrap();
                // Load num_bytes so that the size cache is already loaded
                tree.num_bytes().await.unwrap();
                treestore.clear_unloaded_blocks_from_cache().await.unwrap();
                blockstore.get_and_reset_totals();

                const WRITTEN_LEAF_INDEX: u32 = 140;
                let write_offset = WRITTEN_LEAF_INDEX * LAYOUT.max_bytes_per_leaf();

                tree.write_bytes(
                    &[0; LAYOUT.max_bytes_per_leaf() as usize],
                    write_offset as u64,
                )
                .await
                .unwrap();

                // Even before writing, we need to load the inner nodes on the path to the leaf.
                // The tree has `DEPTH+1` nodes on the path from the root to the leaf. The root shouldn't get loaded because
                // it is already loaded inside of the `tree` instance. We don't have to load the leaf itself, because we fully overwrite it.
                // That means reading the leaf should load `DEPTH - 1` nodes.
                // TODO For some reason, we actually need to load `DEPTH` nodes. Maybe we do load the leaf. Fix this.
                assert_eq!(
                    ActionCounts {
                        exists: 84, // TODO Why do we need these exist and can we calculate this number based on the tree structure?
                        loaded: DEPTH as u32,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );

                // After flushing, the new content should have been written
                std::mem::drop(tree);
                treestore.clear_cache_slow().await.unwrap();
                let expected_stored =
                    num_nodes_written_when_growing_tree(NUM_LEAVES, WRITTEN_LEAF_INDEX as u64 + 1)
                        as u32;
                assert_eq!(
                    ActionCounts {
                        stored: expected_stored + 3, // TODO Why + 3 ?
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );
            })
        })
        .await
    }

    #[tokio::test]
    async fn givenNumBytesNotLoadedYet_whenWritingGrowsTree_writeAFullLeaf() {
        with_treestore_and_tracking_blockstore(|treestore, blockstore| {
            Box::pin(async move {
                let block_id = create_nonempty_tree(treestore, blockstore).await;
                let mut tree = treestore.load_tree(block_id).await.unwrap().unwrap();
                blockstore.get_and_reset_totals();

                const WRITTEN_LEAF_INDEX: u32 = 140;
                let write_offset = WRITTEN_LEAF_INDEX * LAYOUT.max_bytes_per_leaf();

                tree.write_bytes(
                    &[0; LAYOUT.max_bytes_per_leaf() as usize],
                    write_offset as u64,
                )
                .await
                .unwrap();

                // Even before writing, we need to load the inner nodes on the path to the leaf.
                // The tree has `DEPTH+1` nodes on the path from the root to the leaf. The root shouldn't get loaded because
                // it is already loaded inside of the `tree` instance. We don't have to load the leaf itself, because we fully overwrite it.
                // That means reading the leaf should load `DEPTH - 1` nodes.
                // TODO For some reason, we actually need to load `DEPTH` nodes. Maybe we do load the leaf. Fix this.
                let expected_loaded = DEPTH as u32;
                assert_eq!(
                    ActionCounts {
                        exists: 84, // TODO Why do we need these exist and can we calculate this number based on the tree structure?
                        loaded: expected_loaded,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );

                // After flushing, the new content should have been written
                std::mem::drop(tree);
                treestore.clear_cache_slow().await.unwrap();
                let expected_stored =
                    num_nodes_written_when_growing_tree(NUM_LEAVES, WRITTEN_LEAF_INDEX as u64 + 1)
                        as u32;
                assert_eq!(
                    ActionCounts {
                        stored: expected_stored + 3, // TODO Why + 3 ?
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );
            })
        })
        .await
    }

    #[tokio::test]
    async fn givenNumBytesAlreadyLoaded_whenWritingGrowsTree_writeMultipleLeaves() {
        with_treestore_and_tracking_blockstore(|treestore, blockstore| {
            Box::pin(async move {
                let block_id = create_nonempty_tree(treestore, blockstore).await;
                let mut tree = treestore.load_tree(block_id).await.unwrap().unwrap();
                // Load num_bytes so that the size cache is already loaded
                tree.num_bytes().await.unwrap();
                treestore.clear_unloaded_blocks_from_cache().await.unwrap();
                blockstore.get_and_reset_totals();

                const FIRST_ACCESSED_LEAF: u64 = 140;
                const NUM_ACCESSED_LEAVES: u64 = 11;

                let write_offset = ((FIRST_ACCESSED_LEAF as f32 + 0.5)
                    * LAYOUT.max_bytes_per_leaf() as f32) as u64;
                const WRITE_LEN: usize =
                    (NUM_ACCESSED_LEAVES as usize - 1) * LAYOUT.max_bytes_per_leaf() as usize;

                tree.write_bytes(&[0; WRITE_LEN], write_offset)
                    .await
                    .unwrap();

                // Even before writing, we need to load the inner nodes on the path to the rightmost pre-existing leaf.
                // The tree has `DEPTH+1` nodes on the path from the root to the leaf. The root shouldn't get loaded because
                // it is already loaded inside of the `tree` instance. We don't have to load the leaf itself, because we fully overwrite it.
                // That means reading the leaf should load `DEPTH - 1` nodes.
                // TODO For some reason, we actually need to load `DEPTH` nodes. Maybe we do load the leaf. Fix this.
                let expected_loaded = DEPTH as u32;
                assert_eq!(
                    ActionCounts {
                        exists: 103, // TODO Why do we need these exist and can we calculate this number based on the tree structure?
                        loaded: expected_loaded,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );

                // After flushing, the new content should have been written
                std::mem::drop(tree);
                treestore.clear_cache_slow().await.unwrap();
                let expected_stored = num_nodes_written_when_growing_tree(
                    NUM_LEAVES,
                    FIRST_ACCESSED_LEAF + NUM_ACCESSED_LEAVES,
                ) as u32;
                assert_eq!(
                    ActionCounts {
                        stored: expected_stored + 3, // TODO Why +3?
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );
            })
        })
        .await
    }

    #[tokio::test]
    async fn givenNumBytesNotLoadedYet_whenWritingGrowsTree_writeMultipleLeaves() {
        with_treestore_and_tracking_blockstore(|treestore, blockstore| {
            Box::pin(async move {
                let block_id = create_nonempty_tree(treestore, blockstore).await;
                let mut tree = treestore.load_tree(block_id).await.unwrap().unwrap();
                blockstore.get_and_reset_totals();

                const FIRST_ACCESSED_LEAF: u64 = 140;
                const NUM_ACCESSED_LEAVES: u64 = 11;

                let write_offset = ((FIRST_ACCESSED_LEAF as f32 + 0.5)
                    * LAYOUT.max_bytes_per_leaf() as f32) as u64;
                const WRITE_LEN: usize =
                    (NUM_ACCESSED_LEAVES as usize - 1) * LAYOUT.max_bytes_per_leaf() as usize;

                tree.write_bytes(&[0; WRITE_LEN], write_offset)
                    .await
                    .unwrap();

                // Even before writing, we need to load the inner nodes on the path to the rightmost pre-existing leaf.
                // The tree has `DEPTH+1` nodes on the path from the root to the leaf. The root shouldn't get loaded because
                // it is already loaded inside of the `tree` instance. We don't have to load the leaf itself, because we fully overwrite it.
                // That means reading the leaf should load `DEPTH - 1` nodes.
                // TODO For some reason, we actually need to load `DEPTH` nodes. Maybe we do load the leaf. Fix this.
                let expected_loaded = DEPTH as u32;
                assert_eq!(
                    ActionCounts {
                        exists: 103, // TODO Why do we need these exist and can we calculate this number based on the tree structure?
                        loaded: expected_loaded,
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );

                // After flushing, the new content should have been written
                std::mem::drop(tree);
                treestore.clear_cache_slow().await.unwrap();
                let expected_stored = num_nodes_written_when_growing_tree(
                    NUM_LEAVES,
                    FIRST_ACCESSED_LEAF + NUM_ACCESSED_LEAVES,
                ) as u32;
                assert_eq!(
                    ActionCounts {
                        stored: expected_stored + 3, // TODO Why +3?
                        ..Default::default()
                    },
                    blockstore.get_and_reset_totals(),
                );
            })
        })
        .await
    }
}

// TODO Test resize_num_bytes, remove, all_blocks
