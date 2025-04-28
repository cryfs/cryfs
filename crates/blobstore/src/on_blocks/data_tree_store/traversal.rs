use anyhow::{Result, anyhow, bail, ensure};
use async_trait::async_trait;
use conv::{ConvUtil, DefaultApprox, RoundToNearest};
use divrem::DivCeil;
use futures::{
    future::{self, FutureExt},
    stream::{self, BoxStream, StreamExt},
};
use std::fmt::Debug;
use thiserror::Error;

use crate::on_blocks::data_node_store::{
    DataInnerNode, DataLeafNode, DataNode, DataNodeStore, NodeLayout,
};
use cryfs_blockstore::{BlockId, BlockStore};
use cryfs_utils::{async_drop::AsyncDrop, data::Data};

// TODO All following TODOs apply for here and for tree.rs
//  - Try to simplify the traversal logic and make it easier to understand.
//  - Remove parts of the logic and hope that a test case fails, or add test cases.
//  - Maybe also split this into separate files?
//  - Make traversals more concurrent, we can probably look at different child nodes concurrently.
//  - Look at direct operations vs .checked_XXX() and see which ones make sense
//  - Look at data types u32 vs u64 vs usize
//  - Look at assert vs ensure - when something can be caused by the data on disk instead of a programming bug, it must be ensure!
//  - Look at assertions and make sure they all show a good error message

pub enum LeafHandle<'a, B: BlockStore + AsyncDrop + Debug + Send> {
    Borrowed {
        leaf: &'a mut DataLeafNode<B>,
    },
    Owned {
        leaf: DataLeafNode<B>,
    },
    NotLoadedYet {
        store: &'a DataNodeStore<B>,
        leaf_block_id: BlockId,
    },
}

impl<'a, B: BlockStore + AsyncDrop + Debug + Send> LeafHandle<'a, B> {
    pub fn new_not_loaded_yet(store: &'a DataNodeStore<B>, leaf_block_id: BlockId) -> Self {
        Self::NotLoadedYet {
            store,
            leaf_block_id,
        }
    }

    pub fn new_borrowed(leaf: &'a mut DataLeafNode<B>) -> Self {
        Self::Borrowed { leaf }
    }

    pub async fn node(&mut self) -> Result<&mut DataLeafNode<B>> {
        match self {
            Self::Borrowed { leaf } => Ok(leaf),
            Self::Owned { leaf } => Ok(leaf),
            Self::NotLoadedYet {
                store,
                leaf_block_id,
            } => {
                let leaf = store.load(*leaf_block_id).await?.ok_or_else(|| {
                    anyhow!(
                        "Tried to load leaf node {:?} but couldn't find it",
                        leaf_block_id,
                    )
                })?;
                match leaf {
                    DataNode::Inner(inner) => bail!(
                        "Tried to load leaf node {:?} but was inner node with depth {}",
                        leaf_block_id,
                        inner.depth()
                    ),
                    DataNode::Leaf(leaf) => {
                        *self = Self::Owned { leaf };
                        if let Self::Owned { leaf } = self {
                            Ok(leaf)
                        } else {
                            panic!("We just set this to Self::Owned but now it's something else?");
                        }
                    }
                }
            }
        }
    }

    pub async fn overwrite_data(&mut self, source: &[u8]) -> Result<()> {
        match self {
            Self::Borrowed { leaf } => leaf.data_mut().copy_from_slice(source),
            Self::Owned { leaf } => leaf.data_mut().copy_from_slice(source),
            Self::NotLoadedYet {
                store,
                leaf_block_id,
            } => {
                store
                    .overwrite_with_leaf_node(leaf_block_id, source)
                    .await?;
            }
        }
        Ok(())
    }
}

#[async_trait]
pub trait TraversalCallbacks<B: BlockStore + AsyncDrop + Debug + Send> {
    async fn on_existing_leaf(
        &self,
        leaf_index: u64,
        is_right_border_leaf: bool,
        leaf: LeafHandle<'_, B>,
    ) -> Result<()>;
    fn on_create_leaf(&self, index: u64) -> Data;
    async fn on_backtrack_from_subtree(&self, node: &mut DataInnerNode<B>) -> Result<()>;
}

pub async fn traverse_and_return_new_root<
    B: BlockStore<Block: Send> + AsyncDrop + Debug + Send + Sync,
    C: TraversalCallbacks<B> + Sync,
    const ALLOW_WRITES: bool,
>(
    node_store: &DataNodeStore<B>,
    root: DataNode<B>,
    begin_index: u64,
    end_index: u64,
    callbacks: &C,
) -> Result<DataNode<B>> {
    _traverse_and_return_new_root::<B, C, ALLOW_WRITES>(
        node_store,
        root,
        begin_index,
        end_index,
        true,
        callbacks,
    )
    .await
}

async fn _traverse_and_return_new_root<
    B: BlockStore<Block: Send> + AsyncDrop + Debug + Send + Sync,
    C: TraversalCallbacks<B> + Sync,
    const ALLOW_WRITES: bool,
>(
    node_store: &DataNodeStore<B>,
    mut root: DataNode<B>,
    begin_index: u64,
    end_index: u64,
    is_left_border_of_traversal: bool,
    callbacks: &C,
) -> Result<DataNode<B>> {
    assert!(
        begin_index <= end_index,
        "Called _traverse_and_return_new_root with begin_index={} > end_index={}",
        begin_index,
        end_index
    );
    //TODO Test cases with numLeaves < / >= beginIndex, ideally test all configurations:
    //     beginIndex<endIndex<numLeaves, beginIndex=endIndex<numLeaves, beginIndex<endIndex=numLeaves, beginIndex=endIndex=numLeaves
    //     beginIndex<numLeaves<endIndex, beginIndex=numLeaves<endIndex,
    //     numLeaves<beginIndex<endIndex, numLeaves<beginIndex=endIndex

    let max_leaves_for_depth = node_store
        .layout()
        .num_leaves_per_full_subtree(root.depth())?;
    let should_increase_tree_depth = end_index > max_leaves_for_depth.get();
    ensure!(
        ALLOW_WRITES || !should_increase_tree_depth,
        "Tried to grow a tree on a read only traversal. Accessing end_index {} is out of bounds for tree with {} bytes",
        end_index,
        max_leaves_for_depth,
    );

    match &mut root {
        DataNode::Leaf(root) => {
            let max_bytes_per_leaf = node_store.layout().max_bytes_per_leaf();
            if ALLOW_WRITES && should_increase_tree_depth && root.num_bytes() != max_bytes_per_leaf
            {
                root.resize(max_bytes_per_leaf);
            }
            if begin_index == 0 && end_index >= 1 {
                let is_right_border_leaf: bool = end_index == 1;
                callbacks
                    .on_existing_leaf(0, is_right_border_leaf, LeafHandle::new_borrowed(root))
                    .await?;
            }
        }
        DataNode::Inner(root) => {
            _traverse_existing_subtree_of_inner_node::<B, C, ALLOW_WRITES>(
                node_store,
                root,
                begin_index.min(max_leaves_for_depth.get()),
                end_index.min(max_leaves_for_depth.get()),
                0,
                is_left_border_of_traversal,
                !should_increase_tree_depth,
                should_increase_tree_depth,
                callbacks,
            )
            .await?;
        }
    }

    // If the traversal goes too far right for a tree of this depth, increase tree depth by one and continue traversal.
    // This is recursive, i.e. will be repeated if the tree is still not deep enough.
    // We don't increase to the full needed tree depth in one step, because we want the traversal to go as far as possible
    // and only then increase the depth - this causes the tree to be in consistent shape (balanced) for longer.
    if ALLOW_WRITES && should_increase_tree_depth {
        // TODO Test cases that increase tree depth by 0, 1, 2, ... levels
        let root = _increase_tree_depth(node_store, root).await?;
        Box::pin(_traverse_and_return_new_root::<B, C, ALLOW_WRITES>(
            node_store,
            root,
            begin_index.max(max_leaves_for_depth.get()),
            end_index,
            false,
            callbacks,
        ))
        .await
    } else {
        // Once we're done growing the tree and done with the traversal, we might have to decrease tree depth,
        // because the callbacks could have deleted nodes (this happens for example when shrinking the tree using a traversal).
        _while_root_has_only_one_child_replace_root_with_its_child::<B, ALLOW_WRITES>(
            node_store, root,
        )
        .await
    }
}

// TODO leaf_offset u32 or u64?
async fn _traverse_existing_subtree<
    B: BlockStore<Block: Send> + AsyncDrop + Debug + Send + Sync,
    C: TraversalCallbacks<B> + Sync,
    const ALLOW_WRITES: bool,
>(
    node_store: &DataNodeStore<B>,
    block_id: BlockId,
    depth: u8,
    begin_index: u64,
    end_index: u64,
    leaf_offset: u64,
    is_left_border_of_traversal: bool,
    is_right_border_node: bool,
    grow_last_leaf: bool,
    callbacks: &C,
) -> Result<()> {
    if depth == 0 {
        assert!(
            begin_index <= 1 && end_index <= 1,
            "If root node is a leaf, the (sub)tree has only one leaf - access indices must be 0 or 1 but was begin_index={}, end_index={}",
            begin_index,
            end_index
        );
        let mut leaf_handle = LeafHandle::new_not_loaded_yet(node_store, block_id);
        if grow_last_leaf {
            let leaf_node = leaf_handle.node().await?;
            if leaf_node.num_bytes() != node_store.layout().max_bytes_per_leaf() {
                assert!(
                    ALLOW_WRITES,
                    "Can't grow the last leaf in a read-only traversal"
                );
                leaf_node.resize(node_store.layout().max_bytes_per_leaf());
            }
        }
        if begin_index == 0 && end_index == 1 {
            callbacks
                .on_existing_leaf(leaf_offset, is_right_border_node, leaf_handle)
                .await?;
        }
    } else {
        let node = node_store
            .load(block_id)
            .await?
            .ok_or_else(|| anyhow!("Couldn't find child node {:?}", block_id))?;
        match node {
            DataNode::Leaf(_) => bail!("Loaded a node at depth {} but it wasn't a leaf", depth),
            DataNode::Inner(mut node) => {
                ensure!(
                    node.depth().get() == depth,
                    "Expected to load an inner node with depth {} but node claims to be at depth {}",
                    depth,
                    node.depth()
                );
                _traverse_existing_subtree_of_inner_node::<B, C, ALLOW_WRITES>(
                    node_store,
                    &mut node,
                    begin_index,
                    end_index,
                    leaf_offset,
                    is_left_border_of_traversal,
                    is_right_border_node,
                    grow_last_leaf,
                    callbacks,
                )
                .await?;
            }
        }
    }
    Ok(())
}

// TODO leaf_offset u32 or u64?
async fn _traverse_existing_subtree_of_inner_node<
    B: BlockStore<Block: Send> + AsyncDrop + Debug + Send + Sync,
    C: TraversalCallbacks<B> + Sync,
    const ALLOW_WRITES: bool,
>(
    node_store: &DataNodeStore<B>,
    root: &mut DataInnerNode<B>,
    begin_index: u64,
    end_index: u64,
    leaf_offset: u64,
    is_left_border_of_traversal: bool,
    is_right_border_node: bool,
    grow_last_leaf: bool,
    callbacks: &C,
) -> Result<()> {
    assert!(begin_index <= end_index, "Invalid parameters");

    //TODO Call callbacks for different leaves in parallel?

    let leaves_per_child = node_store
        .layout()
        .num_leaves_per_full_subtree(root.depth().get() - 1)?
        .get();
    let begin_child = usize::try_from(begin_index / leaves_per_child).unwrap();
    let end_child = usize::try_from(DivCeil::div_ceil(end_index, leaves_per_child)).unwrap();

    assert!(
        end_child <= usize::try_from(node_store.layout().max_children_per_inner_node()).unwrap(),
        "Traversal region would need increasing the tree depth. This should have happened before calling this function."
    );
    let children = root.children();
    let num_children = children.len();
    assert!(
        !grow_last_leaf || end_child >= num_children,
        "Can only grow last leaf if it exists"
    );
    assert!(
        ALLOW_WRITES || end_child <= num_children,
        "Can only traverse out of bounds in a traversal that allows writes"
    );
    let should_grow_last_existing_leaf = grow_last_leaf || end_child > num_children;

    // If we traverse outside of the valid region (i.e. usually would only traverse to new leaves and not to the last leaf),
    // we still have to descend to the last old child to fill it with leaves and grow the last old leaf.
    if is_left_border_of_traversal && begin_child >= num_children {
        let child_block_id = root.children().last().expect("Node doesn't have children");
        let child_offset = u64::try_from(num_children - 1)
            .unwrap()
            .checked_mul(leaves_per_child)
            .ok_or_else(|| {
                anyhow!(
                    "Overflow in (num_children-1)*leaves_per_child: ({}-1)*{}",
                    num_children,
                    leaves_per_child
                )
            })?;
        struct PanicCallbacks;
        #[async_trait]
        impl<B: BlockStore<Block: Send> + AsyncDrop + Debug + Send + Sync> TraversalCallbacks<B>
            for PanicCallbacks
        {
            async fn on_existing_leaf(
                &self,
                _index: u64,
                _is_right_border_leaf: bool,
                _leaf: LeafHandle<'_, B>,
            ) -> Result<()> {
                panic!("We don't actually traverse any leaves");
            }
            fn on_create_leaf(&self, _index: u64) -> Data {
                panic!("We don't actually traverse any leaves");
            }
            async fn on_backtrack_from_subtree(&self, _node: &mut DataInnerNode<B>) -> Result<()> {
                panic!("We don't actually traverse any leaves");
            }
        }
        Box::pin(
            _traverse_existing_subtree::<B, PanicCallbacks, ALLOW_WRITES>(
                node_store,
                child_block_id,
                root.depth().get() - 1,
                leaves_per_child,
                leaves_per_child,
                child_offset,
                true,
                false,
                true,
                &PanicCallbacks,
            ),
        )
        .await?;
    }

    // Traverse existing children
    let existing_children = children
        .enumerate()
        .skip(begin_child)
        .take(end_child - begin_child);
    for (child_index, child_block_id) in existing_children {
        let child_offset = u64::try_from(child_index)
            .unwrap()
            .checked_mul(leaves_per_child)
            .ok_or_else(|| {
                anyhow!(
                    "Overflow in child_index*leaves_per_child: {}*{}",
                    child_index,
                    leaves_per_child
                )
            })?;
        let local_begin_index = begin_index.saturating_sub(child_offset);
        let local_end_index =
            leaves_per_child.min(end_index.checked_sub(child_offset).ok_or_else(|| {
                anyhow!(
                    "Overflow in end_index - child_offset: {}-{}",
                    end_index,
                    child_offset
                )
            })?);
        let is_first_child: bool = child_index == begin_child;
        let is_last_existing_child: bool = child_index == num_children - 1;
        let is_last_child = is_last_existing_child && num_children == end_child;
        assert!(
            local_end_index <= leaves_per_child,
            "We don't want the child to add a tree level because it doesn't have enough space for the traversal."
        );
        Box::pin(_traverse_existing_subtree::<B, C, ALLOW_WRITES>(
            node_store,
            child_block_id,
            root.depth().get() - 1,
            local_begin_index,
            local_end_index,
            leaf_offset.checked_add(child_offset).ok_or_else(|| {
                anyhow!(
                    "Overflow in leaf_offset+child_offset: {}+{}",
                    leaf_offset,
                    child_offset
                )
            })?,
            is_left_border_of_traversal && is_first_child,
            is_right_border_node && is_last_child,
            should_grow_last_existing_leaf && is_last_existing_child,
            callbacks,
        ))
        .await?;
    }

    // Traverse new children (including gap children, i.e. children that are created but not traversed because they're to the right of the current size, but to the left of the traversal region)
    for child_index in num_children..end_child {
        assert!(
            ALLOW_WRITES,
            "Can't create new children in a read-only traversal"
        );
        let child_offset = u64::try_from(child_index)
            .unwrap()
            .checked_mul(leaves_per_child)
            .ok_or_else(|| {
                anyhow!(
                    "Overflow in child_index*leaves_per_child: {}*{}",
                    child_index,
                    leaves_per_child
                )
            })?;
        let local_begin_index = leaves_per_child.min(begin_index.saturating_sub(child_offset));
        let local_end_index =
            leaves_per_child.min(end_index.checked_sub(child_offset).ok_or_else(|| {
                anyhow!(
                    "Overflow in end_index - child_offset: {}-{}",
                    end_index,
                    child_offset
                )
            })?);
        struct Callbacks<'a, C> {
            child_index: usize,
            begin_child: usize,
            layout: NodeLayout,
            callbacks: &'a C,
        }
        #[async_trait]
        impl<
            'a,
            B: BlockStore<Block: Send> + AsyncDrop + Debug + Send + Sync,
            C: TraversalCallbacks<B> + Sync,
        > CreateNewSubtreeCallbacks<B> for Callbacks<'a, C>
        {
            fn on_create_leaf(&self, index: u64) -> Data {
                if self.child_index >= self.begin_child {
                    self.callbacks.on_create_leaf(index)
                } else {
                    _create_max_size_leaf(&self.layout)
                }
            }
            async fn on_backtrack_from_subtree(&self, node: &mut DataInnerNode<B>) -> Result<()> {
                self.callbacks.on_backtrack_from_subtree(node).await
            }
        }
        let child = _create_new_subtree(
            node_store,
            local_begin_index,
            local_end_index,
            leaf_offset.checked_add(child_offset).ok_or_else(|| {
                anyhow!(
                    "Overflow in leaf_offset+child_offset: {}+{}",
                    leaf_offset,
                    child_offset
                )
            })?,
            root.depth().get() - 1,
            &Callbacks {
                child_index,
                begin_child,
                layout: *node_store.layout(),
                callbacks,
            },
        )
        .await?;
        root.add_child(&child)?;
    }

    // This is only a backtrack if we actually visited a leaf here
    if end_index > begin_index {
        callbacks.on_backtrack_from_subtree(root).await?;
    }

    Ok(())
}

fn _create_max_size_leaf(layout: &NodeLayout) -> Data {
    let max_bytes_per_leaf = usize::try_from(layout.max_bytes_per_leaf()).unwrap();
    Data::from(vec![0; max_bytes_per_leaf])
}

async fn _increase_tree_depth<B: BlockStore + AsyncDrop + Debug + Send>(
    node_store: &DataNodeStore<B>,
    root: DataNode<B>,
) -> Result<DataNode<B>> {
    let copy_of_old_root = node_store.create_new_node_as_copy_from(&root).await?;
    Ok(DataNode::Inner(root.convert_to_new_inner_node(
        copy_of_old_root,
        node_store.layout(),
    )))
}

#[async_trait]
trait CreateNewSubtreeCallbacks<B: BlockStore + AsyncDrop + Debug + Send + Sync> {
    fn on_create_leaf(&self, index: u64) -> Data;
    async fn on_backtrack_from_subtree(&self, node: &mut DataInnerNode<B>) -> Result<()>;
}

// TODO leaf_offset u32 or u64?
async fn _create_new_subtree<
    B: BlockStore + AsyncDrop + Debug + Send + Sync,
    C: CreateNewSubtreeCallbacks<B> + Sync,
>(
    node_store: &DataNodeStore<B>,
    begin_index: u64,
    end_index: u64,
    leaf_offset: u64,
    depth: u8,
    callbacks: &C,
) -> Result<DataNode<B>> {
    assert!(begin_index <= end_index, "Invalid parameters");

    if 0 == depth {
        assert!(
            begin_index <= 1 && end_index == 1,
            "With depth 0, we can only traverse one or zero leaves (i.e. traverse one leaf or traverse a gap leaf)."
        );
        let leaf_data = if begin_index == 0 {
            callbacks.on_create_leaf(leaf_offset)
        } else {
            _create_max_size_leaf(node_store.layout())
        };
        let leaf = node_store.create_new_leaf_node(&leaf_data).await?;
        Ok(DataNode::Leaf(leaf))
    } else {
        let max_children_per_inner_node = node_store.layout().max_children_per_inner_node();
        let min_needed_depth: u8 = (end_index
            .approx_as_by::<f64, DefaultApprox>()
            .unwrap()
            .log(f64::from(max_children_per_inner_node))
            .ceil())
        .approx_as_by::<u8, RoundToNearest>()
        .unwrap();
        assert!(
            depth >= min_needed_depth,
            "Given tree depth is {} but we need at least a depth of {} for end_index={} with max_children_per_inner_node={}",
            depth,
            min_needed_depth,
            end_index,
            max_children_per_inner_node
        );
        let leaves_per_child = node_store
            .layout()
            .num_leaves_per_full_subtree(depth - 1)?
            .get();
        let begin_child = begin_index / leaves_per_child;
        let end_child = DivCeil::div_ceil(end_index, leaves_per_child);

        let mut children = Vec::with_capacity(usize::try_from(end_child).unwrap());
        // TODO Remove redundancy of following two for loops by using min/max for calculating the parameters of the recursive call.
        // Create gap children (i.e. children before the traversal but after the current size)
        for child_index in 0..begin_child {
            let child_offset = child_index.checked_mul(leaves_per_child).ok_or_else(|| {
                anyhow!(
                    "Overflow in child_index*leaves_per_child: {}*{}",
                    child_index,
                    leaves_per_child
                )
            })?;
            struct Callbacks;
            #[async_trait]
            impl<B: BlockStore + AsyncDrop + Debug + Send + Sync> CreateNewSubtreeCallbacks<B> for Callbacks {
                fn on_create_leaf(&self, _index: u64) -> Data {
                    panic!("We're only creating gap leaves here, not traversing any");
                }
                async fn on_backtrack_from_subtree(
                    &self,
                    _node: &mut DataInnerNode<B>,
                ) -> Result<()> {
                    Ok(())
                }
            }
            let child = Box::pin(_create_new_subtree(
                node_store,
                leaves_per_child,
                leaves_per_child,
                leaf_offset + child_offset,
                depth - 1,
                &Callbacks,
            ))
            .await?;
            assert_eq!(
                child.depth(),
                depth - 1,
                "Created child node has wrong depth"
            );
            children.push(*child.block_id());
        }
        // Create new children that are traversed
        for child_index in begin_child..end_child {
            let child_offset = child_index * leaves_per_child;
            let local_begin_index = begin_index.saturating_sub(child_offset);
            let local_end_index = leaves_per_child.min(end_index - child_offset);
            let child = Box::pin(_create_new_subtree(
                node_store,
                local_begin_index,
                local_end_index,
                leaf_offset + child_offset,
                depth - 1,
                callbacks,
            ))
            .await?;
            assert_eq!(
                child.depth(),
                depth - 1,
                "Created child node has wrong depth"
            );
            children.push(*child.block_id());
        }

        assert!(!children.is_empty(), "No children created");
        let mut new_node = node_store.create_new_inner_node(depth, &children).await?;

        // This is only a backtrack if we actually created a leaf here
        if end_index > begin_index {
            callbacks.on_backtrack_from_subtree(&mut new_node).await?;
        }

        Ok(DataNode::Inner(new_node))
    }
}

async fn _while_root_has_only_one_child_replace_root_with_its_child<
    B: BlockStore + AsyncDrop + Debug + Send,
    const ALLOW_WRITES: bool,
>(
    node_store: &DataNodeStore<B>,
    root: DataNode<B>,
) -> Result<DataNode<B>> {
    match &root {
        DataNode::Leaf(_) => {
            // do nothing
            Ok(root)
        }
        DataNode::Inner(root_as_inner) => {
            if root_as_inner.num_children().get() == 1 {
                assert!(
                    ALLOW_WRITES,
                    "Can't decrease tree depth in a read-only traversal"
                );
                let new_root = _while_root_has_only_one_child_remove_root_return_child(
                    node_store,
                    &root_as_inner
                        .children()
                        .next()
                        .expect("Inner node must have at least one child"),
                )
                .await?;
                let overwritten_root = root.overwrite_node_with(&new_root, node_store.layout())?;
                node_store.remove(new_root).await?;
                Ok(overwritten_root)
            } else {
                Ok(root)
            }
        }
    }
}

// TODO Iterative instead of recursive implementation? We wouldn't need the Box::pin for the async recursion then.
async fn _while_root_has_only_one_child_remove_root_return_child<
    B: BlockStore + AsyncDrop + Debug + Send,
>(
    node_store: &DataNodeStore<B>,
    root_block_id: &BlockId,
) -> Result<DataNode<B>> {
    let current = node_store.load(*root_block_id).await?.ok_or_else(|| {
        anyhow!(
            "Tried to load {:?} to decrease tree depth but didn't find it",
            root_block_id,
        )
    })?;
    match &current {
        DataNode::Leaf(_leaf) => Ok(current),
        DataNode::Inner(inner) => {
            let num_children = inner.num_children().get();
            assert!(num_children >= 1);
            if num_children == 1 {
                let result = Box::pin(_while_root_has_only_one_child_remove_root_return_child(
                    node_store,
                    &inner
                        .children()
                        .next()
                        .expect("Inner node must have at least one child"),
                ))
                .await?;
                node_store.remove(current).await?;
                Ok(result)
            } else {
                Ok(current)
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum LoadNodeError {
    #[error("Node with id {node_id:?} not found")]
    NodeNotFound { node_id: BlockId },

    #[error("Error loading node {node_id:?}: {error:?}")]
    NodeLoadError {
        node_id: BlockId,
        error: anyhow::Error,
    },
}

// TODO Can we deduplicate `load_all_nodes_in_subtree{,_of_id}` with the functions above?  But make sure it doesn't get slower!
//      Otherwise, move them into a separate module.

pub async fn load_all_nodes_in_subtree_of_id<'a, B>(
    node_store: &'a DataNodeStore<B>,
    subtree_root_id: BlockId,
) -> BoxStream<'a, Result<DataNode<B>, LoadNodeError>>
where
    B: BlockStore<Block: Send> + AsyncDrop + Debug + Send + Sync,
{
    let child = node_store.load(subtree_root_id).await;
    match child {
        Ok(Some(child)) => load_all_nodes_in_subtree(node_store, child),
        Ok(None) => Box::pin(stream::once(future::ready(Err(
            LoadNodeError::NodeNotFound {
                node_id: subtree_root_id,
            },
        )))),
        Err(error) => Box::pin(stream::once(future::ready(Err(
            LoadNodeError::NodeLoadError {
                node_id: subtree_root_id,
                error,
            },
        )))),
    }
}

pub fn load_all_nodes_in_subtree<'a, B>(
    node_store: &'a DataNodeStore<B>,
    subtree_root: DataNode<B>,
) -> BoxStream<'a, Result<DataNode<B>, LoadNodeError>>
where
    B: BlockStore<Block: Send> + AsyncDrop + Debug + Send + Sync,
{
    match subtree_root {
        DataNode::Leaf(leaf) => stream::once(future::ready(Ok(DataNode::Leaf(leaf)))).boxed(),
        DataNode::Inner(inner) => {
            let block_ids_in_descendants = _load_all_nodes_descendants_of(node_store, &inner);
            stream::once(future::ready(Ok(DataNode::Inner(inner))))
                .chain(block_ids_in_descendants)
                .boxed()
        }
    }
}

fn _load_all_nodes_descendants_of<'a, B>(
    node_store: &'a DataNodeStore<B>,
    subtree_root: &DataInnerNode<B>,
) -> BoxStream<'a, Result<DataNode<B>, LoadNodeError>>
where
    B: BlockStore<Block: Send> + AsyncDrop + Debug + Send + Sync,
{
    // iter<stream<result<block_id>>>
    let subtree_stream = subtree_root.children().map(|child_id| {
        let child_stream = load_all_nodes_in_subtree_of_id(node_store, child_id);
        // Transform Future<Stream<Result<BlockId>>> into Stream<Result<BlockId>>
        child_stream.flatten_stream().boxed()
    });
    let subtree_stream = stream::select_all(subtree_stream).boxed();
    subtree_stream
}

// TODO Tests
