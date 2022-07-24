use anyhow::{anyhow, bail, Result};
use async_recursion::async_recursion;

use crate::blobstore::on_blocks::data_node_store::{DataInnerNode, DataNode, DataNodeStore};
use crate::blockstore::{low_level::BlockStore, BlockId};

#[derive(Clone, Copy)]
pub enum SizeCache {
    SizeUnknown,
    RootIsInnerNodeAndNumLeavesIsKnown {
        num_leaves: u64,
        // It's important to remember whether root is an inner node because if it was a leaf, then it would be the rightmost_leaf_id, and trying
        // to load it to calculate the size would cause a deadlock.
        rightmost_leaf_id: BlockId,
    },
    NumBytesIsKnown {
        num_leaves: u64,
        rightmost_leaf_num_bytes: u32,
    },
}

impl SizeCache {
    pub fn new() -> Self {
        SizeCache::SizeUnknown
    }

    pub async fn get_or_calculate_num_leaves<B: BlockStore + Send + Sync>(
        &mut self,
        node_store: &DataNodeStore<B>,
        root_node: &DataNode<B>,
    ) -> Result<u64> {
        match (*self, root_node) {
            (Self::SizeUnknown, DataNode::Inner(root_node)) => {
                let (num_leaves, rightmost_leaf_id) =
                    Self::_calculate_num_leaves_and_rightmost_leaf_id(node_store, root_node)
                        .await?;
                *self = SizeCache::RootIsInnerNodeAndNumLeavesIsKnown {
                    num_leaves,
                    rightmost_leaf_id,
                };
                Ok(num_leaves)
            }
            (Self::SizeUnknown, DataNode::Leaf(root_node)) => {
                let num_leaves = 1;
                *self = SizeCache::NumBytesIsKnown {
                    num_leaves,
                    rightmost_leaf_num_bytes: root_node.num_bytes(),
                };
                Ok(num_leaves)
            }
            (Self::RootIsInnerNodeAndNumLeavesIsKnown { num_leaves, .. }, _) => Ok(num_leaves),
            (Self::NumBytesIsKnown { num_leaves, .. }, _) => Ok(num_leaves),
        }
    }

    pub async fn get_or_calculate_num_bytes<B: BlockStore + Send + Sync>(
        &mut self,
        node_store: &DataNodeStore<B>,
        root_node: &DataNode<B>,
    ) -> Result<u64> {
        let calculate_num_bytes = |num_leaves: u64, rightmost_leaf_num_bytes: u32| {
            assert!(num_leaves >= 1);
            Ok((num_leaves - 1)
                .checked_mul(u64::from(node_store.max_bytes_per_leaf()))
                .ok_or_else(|| {
                    anyhow!(
                        "Overflow in (num_leaves-1)*max_bytes_per_leaf: ({}-1)*{}",
                        num_leaves,
                        node_store.max_bytes_per_leaf()
                    )
                })?
                .checked_add(u64::from(rightmost_leaf_num_bytes))
                .ok_or_else(|| {
                    anyhow!(
                        "Overflow in (num_leaves-1)*max_bytes_per_leaf+rightmost_leaf_num_bytes: ({}-1)*{}+{}", num_leaves, node_store.max_bytes_per_leaf(), rightmost_leaf_num_bytes
                    )
                })?)
        };
        match (*self, root_node) {
            (Self::SizeUnknown, DataNode::Inner(root_node)) => {
                let (num_leaves, rightmost_leaf_id) =
                    Self::_calculate_num_leaves_and_rightmost_leaf_id(node_store, root_node)
                        .await?;
                let rightmost_leaf_num_bytes =
                    Self::_calculate_leaf_size(node_store, rightmost_leaf_id).await?;
                *self = Self::NumBytesIsKnown {
                    num_leaves,
                    rightmost_leaf_num_bytes,
                };
                calculate_num_bytes(num_leaves, rightmost_leaf_num_bytes)
            }
            (Self::SizeUnknown, DataNode::Leaf(root_node)) => {
                let num_leaves = 1;
                let rightmost_leaf_num_bytes = root_node.num_bytes();
                *self = Self::NumBytesIsKnown {
                    num_leaves,
                    rightmost_leaf_num_bytes,
                };
                calculate_num_bytes(num_leaves, rightmost_leaf_num_bytes)
            }
            (
                Self::RootIsInnerNodeAndNumLeavesIsKnown {
                    num_leaves,
                    rightmost_leaf_id,
                },
                _,
            ) => {
                let rightmost_leaf_num_bytes =
                    Self::_calculate_leaf_size(node_store, rightmost_leaf_id).await?;
                *self = Self::NumBytesIsKnown {
                    num_leaves,
                    rightmost_leaf_num_bytes,
                };
                calculate_num_bytes(num_leaves, rightmost_leaf_num_bytes)
            }
            (
                Self::NumBytesIsKnown {
                    num_leaves,
                    rightmost_leaf_num_bytes,
                },
                _,
            ) => calculate_num_bytes(num_leaves, rightmost_leaf_num_bytes),
        }
    }

    #[async_recursion]
    async fn _calculate_num_leaves_and_rightmost_leaf_id<B: BlockStore + Send + Sync>(
        node_store: &DataNodeStore<B>,
        root_node: &DataInnerNode<B>,
    ) -> Result<(u64, BlockId)> {
        let depth = root_node.depth();
        let children = root_node.children();
        if depth == 1 {
            Ok((
                u64::try_from(children.len()).unwrap(),
                children
                    .last()
                    .expect("Inner node must have at least one child"),
            ))
        } else {
            let num_leaves_per_full_child = u64::from(node_store.max_children_per_inner_node())
                .checked_pow(u32::from(depth - 1))
                .ok_or_else(|| {
                    anyhow!(
                        "Overflow in max_children_per_inner_node^(depth-1): {}^({}-1)",
                        node_store.max_children_per_inner_node(),
                        depth,
                    )
                })?;
            let num_leaves_in_left_children = u64::try_from(children.len() - 1)
                .unwrap()
                .checked_mul(num_leaves_per_full_child)
                .ok_or_else(|| {
                    anyhow!(
                        "Overflow in (num_children-1)*num_leaves_per_full_child: ({}-1)*{}",
                        children.len(),
                        num_leaves_per_full_child,
                    )
                })?;
            let last_child_id = children
                .last()
                .expect("Inner node must have at least one child, that's a class invariant of DataInnerNode");
            let last_child = node_store.load(last_child_id).await?.ok_or_else(|| {
                anyhow!(
                    "Tried to load {:?} as a child node but couldn't find it",
                    last_child_id
                )
            })?;
            let (num_leaves_in_right_child, rightmost_leaf_id) = match last_child {
                DataNode::Leaf(_last_child) => {
                    bail!(
                        "Loaded {:?} as a leaf node but the inner node above it has depth {}",
                        last_child_id,
                        depth,
                    );
                }
                DataNode::Inner(last_child) => {
                    Self::_calculate_num_leaves_and_rightmost_leaf_id(node_store, &last_child)
                        .await?
                }
            };
            let num_leaves = num_leaves_in_left_children
                .checked_add(num_leaves_in_right_child)
                .ok_or_else(|| {
                    anyhow!(
                        "Overflow in num_leaves_in_left_children+num_leaves_in_right_child: {}+{}",
                        num_leaves_in_left_children,
                        num_leaves_in_right_child,
                    )
                })?;
            Ok((num_leaves, rightmost_leaf_id))
        }
    }

    async fn _calculate_leaf_size<B: BlockStore + Send + Sync>(
        node_store: &DataNodeStore<B>,
        rightmost_leaf_id: BlockId,
    ) -> Result<u32> {
        let leaf = node_store.load(rightmost_leaf_id).await?;
        match leaf {
            None => bail!(
                "Tried to load rightmost leaf {:?} but didn't find it",
                rightmost_leaf_id,
            ),
            Some(DataNode::Inner(inner)) => bail!(
                "Tried to load rightmost leaf {:?} but it was an inner node with depth {}",
                rightmost_leaf_id,
                inner.depth(),
            ),
            Some(DataNode::Leaf(leaf)) => Ok(leaf.num_bytes()),
        }
    }
}
