use super::super::super::data_node_store::NodeLayout;
#[cfg(any(feature = "slow-tests-1", feature = "slow-tests-2",))]
use super::super::super::data_tree_store::DataTree;
use super::super::testutils::*;
use cryfs_blockstore::BlockId;
#[cfg(feature = "slow-tests-any")]
use cryfs_blockstore::BlockStore;
#[cfg(feature = "slow-tests-any")]
use cryfs_utils::testutils::data_fixture::DataFixture;
#[cfg(any(
    feature = "slow-tests-1",
    feature = "slow-tests-4",
    feature = "slow-tests-5",
    feature = "slow-tests-6",
))]
use divrem::DivCeil;
#[cfg(feature = "slow-tests-any")]
use rstest::rstest;
#[cfg(feature = "slow-tests-any")]
use rstest_reuse::{apply, template};

#[cfg(feature = "slow-tests-any")]
mod testutils {
    #[cfg(any(feature = "slow-tests-4", feature = "slow-tests-5"))]
    use super::super::super::super::data_node_store::DataNode;
    use super::super::super::super::data_node_store::DataNodeStore;
    #[cfg(any(feature = "slow-tests-4", feature = "slow-tests-5"))]
    use super::super::super::{DataTree, DataTreeStore};
    use super::*;

    #[cfg(any(feature = "slow-tests-4", feature = "slow-tests-5"))]
    use futures::future;

    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum ParamNum {
        Val(u64),
        MaxBytesPerLeafMinus(u64),
        HalfMaxBytesPerLeaf,
        MaxChildrenPerInnerNodeMinus(u64),
        NumFullLeavesForThreeLevelTreeWithLastInnerHasOneChild,
        NumFullLeavesForThreeLevelTreeWithLastInnerHasHalfNumChildren,
        NumFullLeavesForThreeLevelTree,
        NumFullLeavesForFourLevelMinDataTree,
    }
    impl ParamNum {
        pub fn eval(&self, layout: NodeLayout) -> u64 {
            match self {
                Self::Val(val) => *val,
                Self::MaxBytesPerLeafMinus(val) => layout.max_bytes_per_leaf() as u64 - val,
                Self::HalfMaxBytesPerLeaf => layout.max_bytes_per_leaf() as u64 / 2,
                Self::MaxChildrenPerInnerNodeMinus(val) => {
                    layout.max_children_per_inner_node() as u64 - val
                }
                Self::NumFullLeavesForThreeLevelTreeWithLastInnerHasOneChild => {
                    layout.max_children_per_inner_node() as u64
                        * (layout.max_children_per_inner_node() as u64 - 1)
                        + 1
                        - 1
                }
                Self::NumFullLeavesForThreeLevelTreeWithLastInnerHasHalfNumChildren => {
                    layout.max_children_per_inner_node() as u64
                        * (layout.max_children_per_inner_node() as u64 - 1)
                        + layout.max_children_per_inner_node() as u64 / 2
                        - 1
                }
                Self::NumFullLeavesForThreeLevelTree => {
                    layout.max_children_per_inner_node() as u64
                        * layout.max_children_per_inner_node() as u64
                        - 1
                }
                Self::NumFullLeavesForFourLevelMinDataTree => {
                    (layout.max_children_per_inner_node() as u64 - 1)
                        * layout.max_children_per_inner_node() as u64
                        * layout.max_children_per_inner_node() as u64
                }
            }
        }
    }

    /// Parameter for creating a tree.
    /// This can be used for parameterized tests
    #[derive(Clone, Copy)]
    pub struct Parameter {
        /// Number of full leaves in the tree (the tree will have a total of [num_full_leaves] + 1 leaves)
        pub num_full_leaves: ParamNum,

        /// Number of bytes in the last leaf
        pub last_leaf_num_bytes: ParamNum,
    }
    impl Parameter {
        #[cfg(any(
            feature = "slow-tests-1",
            feature = "slow-tests-4",
            feature = "slow-tests-5",
            feature = "slow-tests-6",
        ))]
        pub fn expected_num_nodes(&self, layout: NodeLayout) -> u64 {
            let num_leaves = 1 + self.num_full_leaves.eval(layout);
            expected_num_nodes_for_num_leaves(num_leaves, layout)
        }

        #[cfg(any(
            feature = "slow-tests-1",
            feature = "slow-tests-2",
            feature = "slow-tests-4",
        ))]
        pub fn expected_num_leaves(&self, layout: NodeLayout) -> u64 {
            self.num_full_leaves.eval(layout) + 1
        }

        #[cfg(any(
            feature = "slow-tests-1",
            feature = "slow-tests-2",
            feature = "slow-tests-3",
            feature = "slow-tests-4",
            feature = "slow-tests-5",
        ))]
        pub fn expected_num_bytes(&self, layout: NodeLayout) -> u64 {
            self.num_full_leaves.eval(layout) * layout.max_bytes_per_leaf() as u64
                + self.last_leaf_num_bytes.eval(layout)
        }

        #[cfg(any(feature = "slow-tests-4", feature = "slow-tests-5"))]
        pub fn expected_depth(&self, layout: NodeLayout) -> u8 {
            let num_leaves = 1 + self.num_full_leaves.eval(layout);
            expected_depth_for_num_leaves(num_leaves, layout)
        }

        #[cfg(feature = "slow-tests-1")]
        pub async fn create_tree<B: BlockStore + Send + Sync>(
            &self,
            nodestore: &DataNodeStore<B>,
        ) -> BlockId {
            let id = manually_create_tree(
                nodestore,
                self.num_full_leaves.eval(*nodestore.layout()),
                self.last_leaf_num_bytes.eval(*nodestore.layout()),
                |_offset, num_bytes| vec![0; num_bytes].into(),
            )
            .await;
            nodestore.clear_cache_slow().await.unwrap();
            id
        }

        pub async fn create_tree_with_data<B: BlockStore + Send + Sync>(
            &self,
            nodestore: &DataNodeStore<B>,
            data: &DataFixture,
        ) -> BlockId {
            let generate_leaf_data = |offset: u64, num_bytes: usize| {
                let mut result = vec![0; num_bytes];
                data.generate(offset, &mut result);
                result.into()
            };
            let id = manually_create_tree(
                nodestore,
                self.num_full_leaves.eval(*nodestore.layout()),
                self.last_leaf_num_bytes.eval(*nodestore.layout()),
                generate_leaf_data,
            )
            .await;
            nodestore.clear_cache_slow().await.unwrap();
            id
        }
    }

    pub const TREE_ONE_LEAF: ParamNum = ParamNum::Val(0);
    pub const TREE_TWO_LEAVES: ParamNum = ParamNum::Val(1);
    pub const TREE_TWO_LEVEL_ALMOST_FULL: ParamNum = ParamNum::MaxChildrenPerInnerNodeMinus(2);
    pub const TREE_TWO_LEVEL_FULL: ParamNum = ParamNum::MaxChildrenPerInnerNodeMinus(1);
    pub const TREE_THREE_LEVEL_WITH_LAST_INNER_HAS_ONE_CHILD: ParamNum =
        ParamNum::NumFullLeavesForThreeLevelTreeWithLastInnerHasOneChild;
    pub const TREE_THREE_LEVEL_WITH_LAST_INNER_HAS_HALF_NUM_CHILDREN: ParamNum =
        ParamNum::NumFullLeavesForThreeLevelTreeWithLastInnerHasHalfNumChildren;
    pub const TREE_THREE_LEVEL_FULL: ParamNum = ParamNum::NumFullLeavesForThreeLevelTree;
    pub const TREE_FOUR_LEVEL_MIN_DATA: ParamNum = ParamNum::NumFullLeavesForFourLevelMinDataTree;

    #[template]
    #[rstest]
    fn tree_parameters(
        #[values(
            TREE_ONE_LEAF,
            TREE_TWO_LEAVES,
            TREE_TWO_LEVEL_ALMOST_FULL,
            TREE_TWO_LEVEL_FULL,
            TREE_THREE_LEVEL_WITH_LAST_INNER_HAS_ONE_CHILD,
            TREE_THREE_LEVEL_WITH_LAST_INNER_HAS_HALF_NUM_CHILDREN,
            TREE_THREE_LEVEL_FULL,
            TREE_FOUR_LEVEL_MIN_DATA
        )]
        param_num_full_leaves: ParamNum,
        #[values(
            ParamNum::Val(0),
            ParamNum::Val(1),
            ParamNum::HalfMaxBytesPerLeaf,
            ParamNum::MaxBytesPerLeafMinus(0)
        )]
        param_last_leaf_num_bytes: ParamNum,
    ) {
    }

    #[cfg(any(
        feature = "slow-tests-1",
        feature = "slow-tests-2",
        feature = "slow-tests-4",
    ))]
    #[derive(Clone, Copy, Debug)]
    pub enum LeafIndex {
        FromStart(i64),
        FromMid(i64),
        FromEnd(i64),
    }
    #[cfg(any(
        feature = "slow-tests-1",
        feature = "slow-tests-2",
        feature = "slow-tests-4",
    ))]
    impl LeafIndex {
        pub fn get(&self, num_leaves: u64) -> u64 {
            match self {
                LeafIndex::FromStart(i) => *i as u64,
                LeafIndex::FromMid(i) => (num_leaves as i64 / 2 + i).max(0) as u64,
                LeafIndex::FromEnd(i) => (num_leaves as i64 + i).max(0) as u64,
            }
        }
    }

    #[cfg(any(feature = "slow-tests-4", feature = "slow-tests-5"))]
    pub async fn assert_is_max_data_tree<B: BlockStore + Send + Sync>(
        root_id: BlockId,
        expected_depth: u8,
        nodestore: &DataNodeStore<B>,
    ) {
        let root = nodestore
            .load(root_id)
            .await
            .unwrap()
            .expect("Node not found");
        let root_depth = root.depth();
        assert_eq!(
            root_depth, expected_depth,
            "Expected a node at depth {expected_depth} but found one at depth {root_depth}"
        );
        match root {
            DataNode::Leaf(leaf) => {
                assert_eq!(nodestore.layout().max_bytes_per_leaf(), leaf.num_bytes());
            }
            DataNode::Inner(inner) => {
                let next_expected_depth = expected_depth.checked_sub(1).unwrap();
                let children = inner.children();
                assert_eq!(
                    nodestore.layout().max_children_per_inner_node() as usize,
                    children.len()
                );
                future::join_all(children.map(|child_id| async move {
                    Box::pin(assert_is_max_data_tree(
                        child_id,
                        next_expected_depth,
                        nodestore,
                    ))
                    .await;
                }))
                .await;
            }
        }
    }

    #[cfg(any(feature = "slow-tests-4", feature = "slow-tests-5"))]
    pub async fn assert_is_left_max_data_tree<B: BlockStore + Send + Sync>(
        root_id: BlockId,
        expected_depth: u8,
        nodestore: &DataNodeStore<B>,
    ) {
        let root = nodestore
            .load(root_id)
            .await
            .unwrap()
            .expect("Node not found");
        let root_depth = root.depth();
        assert_eq!(
            expected_depth, root_depth,
            "Expected a node at depth {expected_depth} but found one at depth {root_depth}"
        );
        match root {
            DataNode::Leaf(_) => {}
            DataNode::Inner(inner) => {
                let next_expected_depth = expected_depth.checked_sub(1).unwrap();
                let children = inner.children();
                let children_len = children.len();
                assert!(children_len >= 1);
                future::join_all(
                    children
                        .enumerate()
                        .map(|(child_index, child_id)| async move {
                            if child_index == children_len - 1 {
                                Box::pin(assert_is_left_max_data_tree(
                                    child_id,
                                    next_expected_depth,
                                    nodestore,
                                ))
                                .await;
                            } else {
                                // Children not on the right boundary need to be full
                                assert_is_max_data_tree(child_id, next_expected_depth, nodestore)
                                    .await;
                            }
                        }),
                )
                .await;
            }
        }
    }

    #[cfg(any(feature = "slow-tests-4", feature = "slow-tests-5"))]
    pub async fn flush_caches<'a, B: BlockStore + Send + Sync>(
        tree: DataTree<'a, B>,
        nodestore: &DataNodeStore<B>,
        treestore: &DataTreeStore<B>,
    ) -> BlockId {
        let root_id = *tree.root_node_id();
        // Flush tree
        std::mem::drop(tree);

        // Flush tree store cache
        treestore.clear_cache_slow().await.unwrap();
        nodestore.clear_cache_slow().await.unwrap();

        root_id
    }

    #[cfg(any(feature = "slow-tests-4", feature = "slow-tests-5"))]
    pub async fn assert_tree_structure<B: BlockStore + Send + Sync>(
        root_id: BlockId,
        expected_depth: u8,
        nodestore: &DataNodeStore<B>,
    ) {
        // The root node must be a leaf or have more than one child, otherwise it would be a degenerate tree
        {
            let root = nodestore
                .load(root_id)
                .await
                .unwrap()
                .expect("Root node not found");
            match root {
                DataNode::Leaf(_) => {}
                DataNode::Inner(inner) => {
                    assert!(inner.children().len() >= 2);
                }
            }
        }

        assert_is_left_max_data_tree(root_id, expected_depth, nodestore).await;
    }

    #[cfg(any(feature = "slow-tests-4", feature = "slow-tests-5"))]
    pub async fn assert_leaf_data_is_correct<B: BlockStore + Send + Sync>(
        root_id: BlockId,
        expected_data: &[u8],
        nodestore: &DataNodeStore<B>,
    ) {
        for_each_leaf(root_id, 0, nodestore, &|leaf_index, leaf_data| {
            let start_byte_index =
                leaf_index as usize * nodestore.layout().max_bytes_per_leaf() as usize;
            let end_byte_index = (start_byte_index
                + nodestore.layout().max_bytes_per_leaf() as usize)
                .min(expected_data.len());
            let expected_leaf_data = &expected_data[start_byte_index..end_byte_index];
            assert_eq!(expected_leaf_data, leaf_data);
        })
        .await;
    }

    #[cfg(any(feature = "slow-tests-4", feature = "slow-tests-5"))]
    pub async fn for_each_leaf<B: BlockStore + Send + Sync>(
        root_id: BlockId,
        first_leaf_index: u64,
        nodestore: &DataNodeStore<B>,
        leaf_callback: &(impl Fn(u64, &[u8]) + Sync),
    ) {
        let root = nodestore
            .load(root_id)
            .await
            .unwrap()
            .expect("Node not found");
        match root {
            DataNode::Leaf(leaf) => {
                let leaf_bytes = leaf.data();
                leaf_callback(first_leaf_index, leaf_bytes);
            }
            DataNode::Inner(inner) => {
                let num_leaves_per_child = (nodestore.layout().max_children_per_inner_node()
                    as u64)
                    .pow(inner.depth().get() as u32 - 1);
                let children = inner.children();
                future::join_all(children.enumerate().map(|(child_index, child_id)| {
                    Box::pin(for_each_leaf(
                        child_id,
                        first_leaf_index + child_index as u64 * num_leaves_per_child,
                        nodestore,
                        leaf_callback,
                    ))
                }))
                .await;
            }
        }
    }
}

// TODO Remove this macro and go back to using #[tokio::test] once https://github.com/la10736/rstest/issues/184 is resolved
macro_rules! run_tokio_test {
    ($code:block) => {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move { $code });
    };
}

mod num_bytes_and_num_nodes {
    #[cfg(feature = "slow-tests-1")]
    use super::testutils::*;
    use super::*;

    #[test]
    fn new_tree() {
        run_tokio_test!({
            with_treestore(|store| {
                Box::pin(async move {
                    let mut tree = store.create_tree().await.unwrap();
                    assert_eq!(0, tree.num_bytes().await.unwrap());
                    assert_eq!(1, tree.num_nodes().await.unwrap());
                })
            })
            .await;
        });
    }

    #[test]
    fn check_test_setup() {
        run_tokio_test!({
            with_treestore(|store| {
                Box::pin(async move {
                    let layout = NodeLayout {
                        block_size: PHYSICAL_BLOCK_SIZE,
                    };
                    // Just make sure our calculation of LAYOUT is correct
                    assert_eq!(
                        layout.max_bytes_per_leaf() as u64,
                        store.virtual_block_size_bytes().as_u64(),
                    );
                })
            })
            .await;
        });
    }

    #[cfg(feature = "slow-tests-1")]
    #[apply(super::testutils::tree_parameters)]
    #[test]
    fn build_tree_via_resize_and_check_num_bytes_and_num_nodes(
        #[values(40, 64, 512)] block_size_bytes: u32,
        param_num_full_leaves: ParamNum,
        param_last_leaf_num_bytes: ParamNum,
    ) {
        let param = Parameter {
            num_full_leaves: param_num_full_leaves,
            last_leaf_num_bytes: param_last_leaf_num_bytes,
        };
        run_tokio_test!({
            let layout = NodeLayout { block_size_bytes };
            if param.num_full_leaves.eval(layout) > 0 && param.last_leaf_num_bytes.eval(layout) == 0
            {
                // This is a special case where we can't build the tree via a call to [resize_num_bytes]
                // because that would never leave the last leaf empty
                return;
            }
            with_treestore_with_blocksize(block_size_bytes, move |treestore| {
                Box::pin(async move {
                    let mut tree = treestore.create_tree().await.unwrap();
                    let num_bytes = layout.max_bytes_per_leaf() as u64
                        * param.num_full_leaves.eval(layout)
                        + param.last_leaf_num_bytes.eval(layout);
                    tree.resize_num_bytes(num_bytes).await.unwrap();
                    assert_eq!(num_bytes, tree.num_bytes().await.unwrap());
                    assert_eq!(
                        param.expected_num_nodes(layout),
                        tree.num_nodes().await.unwrap()
                    );

                    // Check the values are still the same when queried again
                    // (they should now be returned from the cache instead of calculated)
                    assert_eq!(num_bytes, tree.num_bytes().await.unwrap());
                    assert_eq!(
                        param.expected_num_nodes(layout),
                        tree.num_nodes().await.unwrap()
                    );
                })
            })
            .await;
        });
    }

    #[cfg(feature = "slow-tests-1")]
    #[apply(super::testutils::tree_parameters)]
    #[test]
    fn build_tree_manually_and_check_num_bytes_and_num_nodes(
        #[values(40, 64, 512)] block_size_bytes: u32,
        param_num_full_leaves: ParamNum,
        param_last_leaf_num_bytes: ParamNum,
    ) {
        let param = Parameter {
            num_full_leaves: param_num_full_leaves,
            last_leaf_num_bytes: param_last_leaf_num_bytes,
        };
        run_tokio_test!({
            let layout = NodeLayout { block_size_bytes };
            with_treestore_and_nodestore_with_blocksize(
                block_size_bytes,
                |treestore, nodestore| {
                    Box::pin(async move {
                        let root_id = param.create_tree(nodestore).await;

                        let mut tree = treestore.load_tree(root_id).await.unwrap().unwrap();
                        assert_eq!(
                            param.expected_num_bytes(layout),
                            tree.num_bytes().await.unwrap()
                        );
                        assert_eq!(
                            param.expected_num_nodes(layout),
                            tree.num_nodes().await.unwrap()
                        );

                        // Check the values are still the same when queried again
                        // (they should now be returned from the cache instead of calculated)
                        assert_eq!(
                            param.expected_num_bytes(layout),
                            tree.num_bytes().await.unwrap()
                        );
                        assert_eq!(
                            param.expected_num_nodes(layout),
                            tree.num_nodes().await.unwrap()
                        );
                    })
                },
            )
            .await;
        });
    }
}

mod root_node_id {
    use super::*;

    #[test]
    fn after_creating_one_node_tree() {
        run_tokio_test!({
            with_treestore(|store| {
                Box::pin(async move {
                    let root_id = BlockId::from_hex("18834bc490faaab6bfdc6a53864cd0a8").unwrap();
                    let tree = store.try_create_tree(root_id).await.unwrap().unwrap();
                    assert_eq!(root_id, *tree.root_node_id());
                })
            })
            .await
        });
    }

    #[test]
    fn after_loading_one_node_tree() {
        run_tokio_test!({
            with_treestore(|store| {
                Box::pin(async move {
                    let root_id = BlockId::from_hex("18834bc490faaab6bfdc6a53864cd0a8").unwrap();
                    store.try_create_tree(root_id).await.unwrap().unwrap();
                    let tree = store.load_tree(root_id).await.unwrap().unwrap();
                    assert_eq!(root_id, *tree.root_node_id());
                })
            })
            .await
        });
    }

    #[test]
    fn after_creating_multi_node_tree() {
        run_tokio_test!({
            with_treestore(|store| {
                Box::pin(async move {
                    let root_id = BlockId::from_hex("18834bc490faaab6bfdc6a53864cd0a8").unwrap();
                    let mut tree = store.try_create_tree(root_id).await.unwrap().unwrap();
                    tree.resize_num_bytes(store.virtual_block_size_bytes().as_u64() * 100)
                        .await
                        .unwrap();
                    assert_eq!(root_id, *tree.root_node_id());
                })
            })
            .await
        });
    }

    #[test]
    fn after_loading_multi_node_tree() {
        run_tokio_test!({
            with_treestore(|store| {
                Box::pin(async move {
                    let root_id = BlockId::from_hex("18834bc490faaab6bfdc6a53864cd0a8").unwrap();
                    let mut tree = store.try_create_tree(root_id).await.unwrap().unwrap();
                    tree.resize_num_bytes(store.virtual_block_size_bytes().as_u64() * 100)
                        .await
                        .unwrap();
                    std::mem::drop(tree);

                    let tree = store.load_tree(root_id).await.unwrap().unwrap();
                    assert_eq!(root_id, *tree.root_node_id());
                })
            })
            .await
        });
    }
}

#[cfg(any(
    feature = "slow-tests-1",
    feature = "slow-tests-2",
    feature = "slow-tests-4",
))]
macro_rules! instantiate_read_write_tests {
        ($test_fn:ident) => {
            #[apply(super::testutils::tree_parameters)]
            #[test]
            fn whole_tree(
                #[values(40, 64, 512)] block_size_bytes: u32,
                param_num_full_leaves: ParamNum,
                param_last_leaf_num_bytes: ParamNum,
            ) {
                let param = Parameter {
                    num_full_leaves: param_num_full_leaves,
                    last_leaf_num_bytes: param_last_leaf_num_bytes,
                };
                run_tokio_test!({
                    let layout = NodeLayout { block_size_bytes };
                    let expected_num_bytes = param.expected_num_bytes(layout);
                    $test_fn(block_size_bytes, param, 0, expected_num_bytes as usize).await;
                });
            }

            #[apply(super::testutils::tree_parameters)]
            #[test]
            fn single_byte(
                #[values(40, 64, 512)] block_size_bytes: u32,
                param_num_full_leaves: ParamNum,
                param_last_leaf_num_bytes: ParamNum,
                #[values(LeafIndex::FromStart(0), LeafIndex::FromStart(1), LeafIndex::FromMid(0), LeafIndex::FromEnd(-1), LeafIndex::FromEnd(0), LeafIndex::FromEnd(1))]
                leaf_index: LeafIndex,
                #[values(
                    ParamNum::Val(0),
                    ParamNum::Val(1),
                    ParamNum::HalfMaxBytesPerLeaf,
                    ParamNum::MaxBytesPerLeafMinus(2),
                    ParamNum::MaxBytesPerLeafMinus(1)
                )]
                byte_index_in_leaf: ParamNum,
            ) {
                let param = Parameter {
                    num_full_leaves: param_num_full_leaves,
                    last_leaf_num_bytes: param_last_leaf_num_bytes,
                };
                run_tokio_test!({
                    let layout = NodeLayout { block_size_bytes };
                    let leaf_index = leaf_index.get(param.expected_num_leaves(layout));
                    let byte_index = leaf_index * layout.max_bytes_per_leaf() as u64
                        + byte_index_in_leaf.eval(layout);
                    $test_fn(block_size_bytes, param, byte_index, 1).await;
                });
            }

            #[apply(super::testutils::tree_parameters)]
            #[test]
            fn two_bytes(
                #[values(40, 64, 512)] block_size_bytes: u32,
                param_num_full_leaves: ParamNum,
                param_last_leaf_num_bytes: ParamNum,
                #[values(LeafIndex::FromStart(0), LeafIndex::FromStart(1), LeafIndex::FromMid(0), LeafIndex::FromEnd(-1), LeafIndex::FromEnd(0), LeafIndex::FromEnd(1))]
                leaf_index: LeafIndex,
                // The last value of `LAYOUT.max_bytes_per_leaf() as u64 - 1` means we read across the leaf boundary
                #[values(
                    ParamNum::Val(0),
                    ParamNum::Val(1),
                    ParamNum::HalfMaxBytesPerLeaf,
                    ParamNum::MaxBytesPerLeafMinus(2),
                    ParamNum::MaxBytesPerLeafMinus(1)
                )]
                first_byte_index_in_leaf: ParamNum,
            ) {
                let param = Parameter {
                    num_full_leaves: param_num_full_leaves,
                    last_leaf_num_bytes: param_last_leaf_num_bytes,
                };
                run_tokio_test!({
                    let layout = NodeLayout { block_size_bytes };
                    let leaf_index = leaf_index.get(param.expected_num_leaves(layout));
                    let first_byte_index = leaf_index * layout.max_bytes_per_leaf() as u64
                        + first_byte_index_in_leaf.eval(layout);
                    $test_fn(block_size_bytes, param, first_byte_index, 2).await;
                });
            }

            #[apply(super::testutils::tree_parameters)]
            #[test]
            fn single_leaf(
                #[values(40, 64, 512)] block_size_bytes: u32,
                param_num_full_leaves: ParamNum,
                param_last_leaf_num_bytes: ParamNum,
                #[values(
                    LeafIndex::FromStart(0),
                    LeafIndex::FromMid(0),
                    LeafIndex::FromEnd(0),
                    LeafIndex::FromEnd(1)
                )]
                leaf_index: LeafIndex,
                #[values(
                    // Ranges starting at the beginning of the leaf
                    (ParamNum::Val(0), ParamNum::Val(0)), (ParamNum::Val(0), ParamNum::Val(1)), (ParamNum::Val(0), ParamNum::HalfMaxBytesPerLeaf), (ParamNum::Val(0), ParamNum::MaxBytesPerLeafMinus(2)), (ParamNum::Val(0), ParamNum::MaxBytesPerLeafMinus(1)), (ParamNum::Val(0), ParamNum::MaxBytesPerLeafMinus(0)),
                    // Ranges in the middle
                    (ParamNum::Val(1), ParamNum::Val(2)), (ParamNum::Val(2), ParamNum::Val(2)), (ParamNum::HalfMaxBytesPerLeaf, ParamNum::MaxBytesPerLeafMinus(1)),
                    // Ranges going until the end of the leaf
                    (ParamNum::Val(1), ParamNum::MaxBytesPerLeafMinus(0)), (ParamNum::HalfMaxBytesPerLeaf, ParamNum::MaxBytesPerLeafMinus(0)), (ParamNum::MaxBytesPerLeafMinus(1), ParamNum::MaxBytesPerLeafMinus(0)), (ParamNum::MaxBytesPerLeafMinus(0), ParamNum::MaxBytesPerLeafMinus(0))
                )]
                byte_indices: (ParamNum, ParamNum),
            ) {
                let param = Parameter {
                    num_full_leaves: param_num_full_leaves,
                    last_leaf_num_bytes: param_last_leaf_num_bytes,
                };
                run_tokio_test!({
                    let layout = NodeLayout { block_size_bytes };
                    let (begin_byte_index_in_leaf, end_byte_index_in_leaf) = byte_indices;
                    let first_leaf_byte = leaf_index.get(param.expected_num_leaves(layout))
                        * layout.max_bytes_per_leaf() as u64;
                    let begin_byte_index = first_leaf_byte + begin_byte_index_in_leaf.eval(layout);
                    let end_byte_index = first_leaf_byte + end_byte_index_in_leaf.eval(layout);
                    $test_fn(
                        block_size_bytes,
                        param,
                        begin_byte_index,
                        (end_byte_index - begin_byte_index) as usize,
                    )
                    .await;
                });
            }

            #[apply(super::testutils::tree_parameters)]
            #[test]
            fn across_leaves(
                #[values(40, 64, 512)] block_size_bytes: u32,
                param_num_full_leaves: ParamNum,
                param_last_leaf_num_bytes: ParamNum,
                #[values(
                    (LeafIndex::FromStart(0), LeafIndex::FromStart(1)),
                    (LeafIndex::FromStart(0), LeafIndex::FromMid(0)),
                    (LeafIndex::FromStart(0), LeafIndex::FromEnd(0)),
                    (LeafIndex::FromStart(0), LeafIndex::FromEnd(10)),
                    (LeafIndex::FromStart(10), LeafIndex::FromEnd(-10)),
                    (LeafIndex::FromMid(0), LeafIndex::FromMid(1)),
                    (LeafIndex::FromMid(0), LeafIndex::FromEnd(0)),
                    (LeafIndex::FromMid(0), LeafIndex::FromEnd(10)),
                    (LeafIndex::FromEnd(0), LeafIndex::FromEnd(5)),
                    (LeafIndex::FromEnd(1), LeafIndex::FromEnd(5)),
                    (LeafIndex::FromEnd(20), LeafIndex::FromEnd(50)),
                )]
                leaf_indices: (LeafIndex, LeafIndex),
                #[values(
                    ParamNum::Val(0),
                    ParamNum::HalfMaxBytesPerLeaf,
                    ParamNum::MaxBytesPerLeafMinus(1)
                )]
                begin_byte_index_in_leaf: ParamNum,
                #[values(
                    ParamNum::Val(1),
                    ParamNum::HalfMaxBytesPerLeaf,
                    ParamNum::MaxBytesPerLeafMinus(0)
                )]
                end_byte_index_in_leaf: ParamNum,
            ) {
                let param = Parameter {
                    num_full_leaves: param_num_full_leaves,
                    last_leaf_num_bytes: param_last_leaf_num_bytes,
                };
                run_tokio_test!({
                    let layout = NodeLayout { block_size_bytes };
                    let (begin_leaf_index, last_leaf_index) = leaf_indices;
                    let begin_byte_index = {
                        let first_leaf_byte = begin_leaf_index.get(param.expected_num_leaves(layout))
                            * layout.max_bytes_per_leaf() as u64;
                        first_leaf_byte + begin_byte_index_in_leaf.eval(layout)
                    };
                    let end_byte_index = {
                        let first_leaf_byte = last_leaf_index.get(param.expected_num_leaves(layout))
                            * layout.max_bytes_per_leaf() as u64;
                        first_leaf_byte + end_byte_index_in_leaf.eval(layout)
                    };
                    if end_byte_index < begin_byte_index {
                        return;
                    }
                    $test_fn(
                        block_size_bytes,
                        param,
                        begin_byte_index,
                        (end_byte_index - begin_byte_index) as usize,
                    )
                    .await;
                });
            }
        };
    }

#[cfg(feature = "slow-tests-1")]
mod read_bytes {
    use super::testutils::*;
    use super::*;

    async fn assert_reads_correct_data<'a, B: BlockStore + Send + Sync>(
        tree: &mut DataTree<'a, B>,
        data: &DataFixture,
        offset: u64,
        num_bytes: usize,
    ) {
        let mut read_data = vec![0; num_bytes];
        tree.read_bytes(offset, &mut read_data).await.unwrap();
        let mut expected_data = vec![0; num_bytes];
        data.generate(offset, &mut expected_data);
        assert_eq!(expected_data, read_data);
    }

    async fn assert_reading_is_out_of_range<'a, B: BlockStore + Send + Sync>(
        tree: &mut DataTree<'a, B>,
        layout: NodeLayout,
        params: &Parameter,
        offset: u64,
        num_bytes: usize,
    ) {
        assert_eq!(
                tree.read_bytes(offset, &mut vec![0; num_bytes])
                    .await
                    .unwrap_err()
                    .to_string(),
                format!("DataTree::read_bytes() tried to read range {}..{} but only has {} bytes stored. Use try_read_bytes() if this should be allowed.", offset, offset + num_bytes as u64, params.expected_num_bytes(layout)),
            );
    }

    async fn test_read_bytes(
        block_size_bytes: u32,
        param: Parameter,
        offset: u64,
        num_bytes: usize,
    ) {
        let layout = NodeLayout { block_size_bytes };
        let expected_num_bytes = param.expected_num_bytes(layout);
        with_treestore_and_nodestore_with_blocksize(block_size_bytes, |treestore, nodestore| {
            Box::pin(async move {
                let data = DataFixture::new(0);
                let tree_id = param.create_tree_with_data(nodestore, &data).await;
                let mut tree = treestore.load_tree(tree_id).await.unwrap().unwrap();

                if offset + num_bytes as u64 > expected_num_bytes {
                    assert_reading_is_out_of_range(&mut tree, layout, &param, offset, num_bytes)
                        .await;
                } else {
                    assert_reads_correct_data(&mut tree, &data, offset, num_bytes).await;
                }
            })
        })
        .await;
    }

    instantiate_read_write_tests!(test_read_bytes);

    // TODO Test read_bytes, try_read_bytes and read_all don't change any nodes (i.e. didn't change the tree and didn't add any new nodes)
}

#[cfg(feature = "slow-tests-2")]
mod try_read_bytes {
    use super::testutils::*;
    use super::*;

    async fn assert_reads_correct_data<'a, B: BlockStore + Send + Sync>(
        tree: &mut DataTree<'a, B>,
        data: &DataFixture,
        offset: u64,
        num_bytes: usize,
    ) {
        let mut read_data = vec![0; num_bytes];
        let num_read_bytes = tree.try_read_bytes(offset, &mut read_data).await.unwrap();
        assert_eq!(num_bytes, num_read_bytes);
        let mut expected_data = vec![0; num_bytes];
        data.generate(offset, &mut expected_data);
        assert_eq!(expected_data, read_data);
    }

    async fn assert_reading_is_out_of_range<'a, B: BlockStore + Send + Sync>(
        tree: &mut DataTree<'a, B>,
        layout: NodeLayout,
        data: &DataFixture,
        param: Parameter,
        offset: u64,
        num_bytes: usize,
    ) {
        let mut read_data = vec![0; num_bytes];
        let num_read_bytes = tree.try_read_bytes(offset, &mut read_data).await.unwrap();
        let expected_num_read_bytes = param
            .expected_num_bytes(layout)
            .saturating_sub(offset)
            .min(num_bytes as u64) as usize;
        assert_eq!(expected_num_read_bytes, num_read_bytes);
        let mut expected_data = vec![0; expected_num_read_bytes];
        data.generate(offset, &mut expected_data);
        assert_eq!(expected_data, &read_data[..expected_num_read_bytes]);
        assert_eq!(
            vec![0; num_bytes - expected_num_read_bytes],
            &read_data[expected_num_read_bytes..]
        );
    }

    async fn test_try_read_bytes(
        block_size_bytes: u32,
        param: Parameter,
        offset: u64,
        num_bytes: usize,
    ) {
        let layout = NodeLayout { block_size_bytes };
        with_treestore_and_nodestore_with_blocksize(block_size_bytes, |treestore, nodestore| {
            Box::pin(async move {
                let data = DataFixture::new(0);
                let tree_id = param.create_tree_with_data(nodestore, &data).await;
                let mut tree = treestore.load_tree(tree_id).await.unwrap().unwrap();
                if offset + num_bytes as u64 > param.expected_num_bytes(layout) {
                    assert_reading_is_out_of_range(
                        &mut tree, layout, &data, param, offset, num_bytes,
                    )
                    .await;
                } else {
                    assert_reads_correct_data(&mut tree, &data, offset, num_bytes).await;
                }
            })
        })
        .await;
    }

    instantiate_read_write_tests!(test_try_read_bytes);
}

#[cfg(feature = "slow-tests-3")]
mod read_all {
    use super::testutils::*;
    use super::*;
    use cryfs_utils::data::Data;

    #[apply(super::testutils::tree_parameters)]
    #[test]
    fn read_whole_tree(
        #[values(40, 64, 512)] block_size_bytes: u32,
        param_num_full_leaves: ParamNum,
        param_last_leaf_num_bytes: ParamNum,
    ) {
        let param = Parameter {
            num_full_leaves: param_num_full_leaves,
            last_leaf_num_bytes: param_last_leaf_num_bytes,
        };
        run_tokio_test!({
            let layout = NodeLayout { block_size_bytes };
            with_treestore_and_nodestore_with_blocksize(
                block_size_bytes,
                |treestore, nodestore| {
                    Box::pin(async move {
                        let data = DataFixture::new(0);
                        let tree_id = param.create_tree_with_data(nodestore, &data).await;
                        let mut tree = treestore.load_tree(tree_id).await.unwrap().unwrap();

                        let read_data = tree.read_all().await.unwrap();
                        assert_eq!(param.expected_num_bytes(layout) as usize, read_data.len());
                        let expected_data: Data =
                            data.get(param.expected_num_bytes(layout) as usize).into();
                        assert_eq!(expected_data, read_data);
                    })
                },
            )
            .await;
        });
    }
}

#[cfg(feature = "slow-tests-4")]
mod write_bytes {
    use super::testutils::*;
    use super::*;
    use cryfs_utils::data::Data;

    async fn test_write_bytes(
        block_size_bytes: u32,
        params: Parameter,
        offset: u64,
        num_bytes: usize,
    ) {
        let layout = NodeLayout { block_size_bytes };
        with_treestore_and_nodestore_with_blocksize(block_size_bytes, |treestore, nodestore| {
            Box::pin(async move {
                let base_data = DataFixture::new(0);
                let write_data = DataFixture::new(1);

                // Create tree with `base_data`
                let tree_id = params.create_tree_with_data(nodestore, &base_data).await;
                let tree = treestore.load_tree(tree_id).await.unwrap().unwrap();

                // Check the test case set it up correctly
                flush_caches(tree, nodestore, treestore).await;
                assert_tree_structure(tree_id, params.expected_depth(layout), nodestore).await;
                assert_eq!(
                    params.expected_num_nodes(layout),
                    nodestore.num_nodes().await.unwrap()
                );
                let mut tree = treestore.load_tree(tree_id).await.unwrap().unwrap();

                // Fill size cache (so we can check if it gets correctly updated)
                assert_eq!(
                    params.expected_num_bytes(layout),
                    tree.num_bytes().await.unwrap(),
                );

                // Write subregion with `write_data`
                let source = write_data.get(num_bytes);
                tree.write_bytes(&source, offset).await.unwrap();

                // Calculate what we expect the tree data to be now
                let expected_new_data: Data = {
                    let mut data: Vec<u8> =
                        base_data.get(params.expected_num_bytes(layout) as usize);
                    if num_bytes == 0 {
                        // Writing doesn't grow the tree if num_bytes == 0, even if
                        // offset is beyond the current tree data size.
                        // TODO Is this actually the behavior we want? Or do we want write_bytes to grow the tree here?
                    } else {
                        if offset as usize + num_bytes > data.len() {
                            data.resize(offset as usize + num_bytes, 0);
                        }
                        data[offset as usize..offset as usize + num_bytes].copy_from_slice(&source);
                    }
                    data.into()
                };

                // Check new tree size (as read from size cache)
                assert_eq!(
                    expected_new_data.len() as u64,
                    tree.num_bytes().await.unwrap()
                );

                // Check new tree size (as read after clearing size cache)
                std::mem::drop(tree);
                let mut tree = treestore.load_tree(tree_id).await.unwrap().unwrap();
                assert_eq!(
                    expected_new_data.len() as u64,
                    tree.num_bytes().await.unwrap()
                );

                // Check tree data using `read_all()`.
                let read_data = tree.read_all().await.unwrap();
                assert_eq!(expected_new_data, read_data);

                // Check the new tree structure is valid
                let writing_grew_data =
                    expected_new_data.len() as u64 > params.expected_num_bytes(layout);
                let expected_depth = if writing_grew_data {
                    expected_depth_for_num_bytes(expected_new_data.len() as u64, layout)
                } else {
                    // We don't use `expected_depth_for_num_bytes` here because it would be inaccurate
                    // for the corner case where we created a tree with last_leaf_size == 0.
                    params.expected_depth(layout)
                };
                flush_caches(tree, nodestore, treestore).await;
                assert_tree_structure(tree_id, expected_depth, nodestore).await;

                // Check it hasn't created any orphan nodes
                let expected_new_num_leaves = if writing_grew_data {
                    DivCeil::div_ceil(
                        expected_new_data.len() as u64,
                        layout.max_bytes_per_leaf() as u64,
                    )
                } else {
                    // We don't use `div_ceil` here because it would be inaccurate
                    // for the corner case where we created a tree with last_leaf_size == 0.
                    params.num_full_leaves.eval(layout) + 1
                };
                let expected_num_nodes =
                    expected_num_nodes_for_num_leaves(expected_new_num_leaves, layout);
                assert_eq!(expected_num_nodes, nodestore.num_nodes().await.unwrap());

                // Read whole tree back and check its leaves have the correct data
                assert_leaf_data_is_correct(tree_id, &expected_new_data, nodestore).await;
            })
        })
        .await;
    }

    instantiate_read_write_tests!(test_write_bytes);
}

// TODO Test flush

#[cfg(feature = "slow-tests-5")]
mod resize_num_bytes {
    use super::testutils::*;
    use super::*;
    use cryfs_utils::data::Data;

    fn test_resize(
        block_size_bytes: u32,
        param_before_resize: Parameter,
        param_after_resize: Parameter,
    ) {
        run_tokio_test!({
            let layout = NodeLayout { block_size_bytes };
            with_treestore_and_nodestore_with_blocksize(
                block_size_bytes,
                |treestore, nodestore| {
                    Box::pin(async move {
                        let data = DataFixture::new(0);

                        // Create tree with `data`
                        let tree_id = param_before_resize
                            .create_tree_with_data(nodestore, &data)
                            .await;
                        let tree = treestore.load_tree(tree_id).await.unwrap().unwrap();

                        // Check the test case set it up correctly
                        flush_caches(tree, nodestore, treestore).await;
                        assert_tree_structure(
                            tree_id,
                            param_before_resize.expected_depth(layout),
                            nodestore,
                        )
                        .await;
                        assert_eq!(
                            param_before_resize.expected_num_nodes(layout),
                            nodestore.num_nodes().await.unwrap()
                        );
                        let mut tree = treestore.load_tree(tree_id).await.unwrap().unwrap();

                        // Fill size cache (so we can check if it gets correctly updated)
                        assert_eq!(
                            param_before_resize.expected_num_bytes(layout),
                            tree.num_bytes().await.unwrap()
                        );
                        assert_eq!(
                            param_before_resize.expected_num_nodes(layout),
                            tree.num_nodes().await.unwrap()
                        );

                        // Resize tree with `resize_num_bytes`
                        let new_num_bytes = param_after_resize.expected_num_bytes(layout);
                        tree.resize_num_bytes(new_num_bytes).await.unwrap();

                        // Check key didn't change
                        assert_eq!(tree_id, *tree.root_node_id());

                        // Check tree has correct size (looked up from the size cache of the `tree` instance)
                        assert_eq!(new_num_bytes, tree.num_bytes().await.unwrap());
                        assert_eq!(
                            param_after_resize.expected_num_nodes(layout),
                            tree.num_nodes().await.unwrap()
                        );

                        // Check tree has correct size (looked up after clearing the size cache of the `tree` instance)
                        std::mem::drop(tree);
                        let mut tree = treestore.load_tree(tree_id).await.unwrap().unwrap();
                        assert_eq!(new_num_bytes, tree.num_bytes().await.unwrap());
                        assert_eq!(
                            param_after_resize.expected_num_nodes(layout),
                            tree.num_nodes().await.unwrap()
                        );

                        // Check tree data using `read_all`
                        let read_data = tree.read_all().await.unwrap();
                        let old_num_bytes = param_before_resize.expected_num_bytes(layout);
                        let expected_new_data: Data = {
                            let mut expected_new_data = vec![0; new_num_bytes as usize];
                            expected_new_data[..old_num_bytes.min(new_num_bytes) as usize]
                                .copy_from_slice(
                                    &data.get(old_num_bytes.min(new_num_bytes) as usize),
                                );
                            expected_new_data.into()
                        };
                        assert_eq!(new_num_bytes, read_data.len() as u64);
                        assert_eq!(expected_new_data, read_data);

                        // Check the new tree structure is valid
                        flush_caches(tree, nodestore, treestore).await;
                        assert_tree_structure(
                            tree_id,
                            param_after_resize.expected_depth(layout),
                            nodestore,
                        )
                        .await;

                        // Check the data in the leaves is correct
                        assert_leaf_data_is_correct(tree_id, &expected_new_data, nodestore).await;

                        // Check there weren't too many nodes created or left behind
                        assert_eq!(
                            param_after_resize.expected_num_nodes(layout),
                            nodestore.num_nodes().await.unwrap()
                        );
                    })
                },
            )
            .await;
        });
    }

    #[apply(super::testutils::tree_parameters)]
    #[test]
    fn test_resize_basic(
        #[values(40, 64, 512)] block_size_bytes: u32,
        param_num_full_leaves: ParamNum,
        param_last_leaf_num_bytes: ParamNum,
        // param2_num_full_leaves and param2_last_leaf_num_bytes are set up the same way
        // as param_num_full_leaves and param_last_leaf_num_bytes are set up using `#[apply(super::testutils::tree_parameters)]`.
        // TODO Probably better to use 2 separate `#[apply(...)]` attributes here but seems `rstest` doesn't support that yet.
        #[values(
            TREE_ONE_LEAF,
            TREE_TWO_LEAVES,
            TREE_TWO_LEVEL_ALMOST_FULL,
            TREE_TWO_LEVEL_FULL,
            TREE_THREE_LEVEL_WITH_LAST_INNER_HAS_ONE_CHILD,
            TREE_THREE_LEVEL_WITH_LAST_INNER_HAS_HALF_NUM_CHILDREN,
            TREE_THREE_LEVEL_FULL,
            TREE_FOUR_LEVEL_MIN_DATA
        )]
        param2_num_full_leaves: ParamNum,
        #[values(
            ParamNum::Val(1),
            ParamNum::HalfMaxBytesPerLeaf,
            ParamNum::MaxBytesPerLeafMinus(1),
            ParamNum::MaxBytesPerLeafMinus(0)
        )]
        param2_last_leaf_num_bytes: ParamNum,
    ) {
        let param_before_resize = Parameter {
            num_full_leaves: param_num_full_leaves,
            last_leaf_num_bytes: param_last_leaf_num_bytes,
        };
        let param_after_resize = Parameter {
            num_full_leaves: param2_num_full_leaves,
            last_leaf_num_bytes: param2_last_leaf_num_bytes,
        };
        test_resize(block_size_bytes, param_before_resize, param_after_resize);
    }

    #[apply(super::testutils::tree_parameters)]
    #[test]
    fn test_resize_to_zero(
        #[values(40, 64, 512)] block_size_bytes: u32,
        param_num_full_leaves: ParamNum,
        param_last_leaf_num_bytes: ParamNum,
    ) {
        let param_before_resize = Parameter {
            num_full_leaves: param_num_full_leaves,
            last_leaf_num_bytes: param_last_leaf_num_bytes,
        };
        let param_after_resize = Parameter {
            num_full_leaves: ParamNum::Val(0),
            last_leaf_num_bytes: ParamNum::Val(0),
        };
        test_resize(block_size_bytes, param_before_resize, param_after_resize);
    }
}

#[cfg(feature = "slow-tests-6")]
mod remove {
    use super::testutils::*;
    use super::*;

    #[apply(super::testutils::tree_parameters)]
    #[test]
    fn test_remove(
        #[values(40, 64, 512)] block_size_bytes: u32,
        param_num_full_leaves: ParamNum,
        param_last_leaf_num_bytes: ParamNum,
    ) {
        let param = Parameter {
            num_full_leaves: param_num_full_leaves,
            last_leaf_num_bytes: param_last_leaf_num_bytes,
        };
        run_tokio_test!({
            with_treestore_and_nodestore_with_blocksize(
                block_size_bytes,
                |treestore, nodestore| {
                    Box::pin(async move {
                        let data = DataFixture::new(0);

                        // Create tree with `data`
                        let tree_id = param.create_tree_with_data(nodestore, &data).await;
                        let tree = treestore.load_tree(tree_id).await.unwrap().unwrap();

                        // Remove tree
                        tree.remove().await.unwrap();

                        // Check tree is gone
                        assert!(treestore.load_tree(tree_id).await.unwrap().is_none());

                        // Check nodes are deleted
                        treestore.clear_cache_slow().await.unwrap();
                        nodestore.clear_cache_slow().await.unwrap();
                        assert_eq!(0, nodestore.num_nodes().await.unwrap());
                    })
                },
            )
            .await;
        });
    }
}

#[cfg(feature = "slow-tests-6")]
mod all_blocks {
    use super::testutils::*;
    use super::*;
    use futures::stream::TryStreamExt;
    use std::collections::HashSet;

    #[apply(super::testutils::tree_parameters)]
    #[test]
    fn test_all_blocks(
        #[values(40, 64, 512)] block_size_bytes: u32,
        param_num_full_leaves: ParamNum,
        param_last_leaf_num_bytes: ParamNum,
    ) {
        let param = Parameter {
            num_full_leaves: param_num_full_leaves,
            last_leaf_num_bytes: param_last_leaf_num_bytes,
        };
        let layout = NodeLayout { block_size_bytes };
        run_tokio_test!({
            with_treestore_and_nodestore_with_blocksize(
                block_size_bytes,
                |treestore, nodestore| {
                    Box::pin(async move {
                        let data = DataFixture::new(0);

                        // Create a tree
                        let tree1_id = param.create_tree_with_data(nodestore, &data).await;
                        let expected_tree1_blocks: HashSet<BlockId> = {
                            treestore.clear_cache_slow().await.unwrap();
                            nodestore.clear_cache_slow().await.unwrap();
                            let all_blocks: Result<HashSet<BlockId>, _> =
                                nodestore.all_nodes().await.unwrap().try_collect().await;
                            all_blocks.unwrap()
                        };
                        assert_eq!(
                            param.expected_num_nodes(layout),
                            expected_tree1_blocks.len() as u64,
                        );

                        // Create another tree
                        let tree2_id = param.create_tree_with_data(nodestore, &data).await;
                        let expected_tree2_blocks: HashSet<BlockId> = {
                            treestore.clear_cache_slow().await.unwrap();
                            nodestore.clear_cache_slow().await.unwrap();
                            let all_blocks: Result<HashSet<BlockId>, _> =
                                nodestore.all_nodes().await.unwrap().try_collect().await;
                            let mut expected_blocks = all_blocks.unwrap();
                            for block in &expected_tree1_blocks {
                                expected_blocks.remove(block);
                            }
                            expected_blocks
                        };
                        assert_eq!(
                            param.expected_num_nodes(layout),
                            expected_tree2_blocks.len() as u64,
                        );

                        let tree1 = treestore.load_tree(tree1_id).await.unwrap().unwrap();
                        let tree2 = treestore.load_tree(tree2_id).await.unwrap().unwrap();

                        let tree1_blocks: Result<HashSet<BlockId>, _> =
                            tree1.all_blocks().unwrap().try_collect().await;
                        let tree1_blocks = tree1_blocks.unwrap();
                        assert_eq!(expected_tree1_blocks, tree1_blocks);

                        let tree2_blocks: Result<HashSet<BlockId>, _> =
                            tree2.all_blocks().unwrap().try_collect().await;
                        let tree2_blocks = tree2_blocks.unwrap();
                        assert_eq!(expected_tree2_blocks, tree2_blocks);
                    })
                },
            )
            .await;
        });
    }
}
