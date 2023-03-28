//! This module contains tests that ensure that tree operations only access the minimal required number of nodes.

use futures::future::BoxFuture;

use cryfs_blockstore::{
    ActionCounts, BlockId, InMemoryBlockStore, LockingBlockStore, SharedBlockStore,
    TrackingBlockStore,
};

use super::super::testutils::*;
use crate::on_blocks::{data_node_store::NodeLayout, data_tree_store::DataTreeStore};

const LAYOUT: NodeLayout = NodeLayout {
    block_size_bytes: 40,
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
            LAYOUT.block_size_bytes,
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

// TODO Test read_bytes, try_read_bytes, read_all, write_bytes, flush, resize_num_bytes, remove, all_blocks,
