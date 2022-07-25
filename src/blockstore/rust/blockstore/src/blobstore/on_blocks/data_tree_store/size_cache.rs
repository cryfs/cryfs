use anyhow::{anyhow, bail, Result};
use std::num::NonZeroU64;

use super::traversal;
use crate::blobstore::on_blocks::data_node_store::{DataNode, DataNodeStore, NodeLayout};
use crate::blockstore::{low_level::BlockStore, BlockId};

#[derive(Clone, Copy)]
pub enum SizeCache {
    SizeUnknown,
    RootIsInnerNodeAndNumLeavesIsKnown {
        num_leaves: NonZeroU64,
        // It's important to remember whether root is an inner node because if it was a leaf, then it would be the rightmost_leaf_id, and trying
        // to load it to calculate the size would cause a deadlock.
        rightmost_leaf_id: BlockId,
    },
    NumBytesIsKnown {
        num_leaves: NonZeroU64,
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
    ) -> Result<NonZeroU64> {
        match (*self, root_node) {
            (Self::SizeUnknown, DataNode::Inner(root_node)) => {
                let traversal::NumLeavesAndRightmostLeafId {
                    num_leaves,
                    rightmost_leaf_id,
                } = traversal::calculate_num_leaves_and_rightmost_leaf_id(node_store, root_node)
                    .await?;
                *self = SizeCache::RootIsInnerNodeAndNumLeavesIsKnown {
                    num_leaves,
                    rightmost_leaf_id,
                };
                Ok(num_leaves)
            }
            (Self::SizeUnknown, DataNode::Leaf(root_node)) => {
                let num_leaves = NonZeroU64::new(1).unwrap();
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
        let calculate_num_bytes = |num_leaves: NonZeroU64, rightmost_leaf_num_bytes: u32| {
            Ok((num_leaves.get() - 1)
                .checked_mul(u64::from(node_store.layout().max_bytes_per_leaf()))
                .ok_or_else(|| {
                    anyhow!(
                        "Overflow in (num_leaves-1)*max_bytes_per_leaf: ({}-1)*{}",
                        num_leaves,
                        node_store.layout().max_bytes_per_leaf(),
                    )
                })?
                .checked_add(u64::from(rightmost_leaf_num_bytes))
                .ok_or_else(|| {
                    anyhow!(
                        "Overflow in (num_leaves-1)*max_bytes_per_leaf+rightmost_leaf_num_bytes: ({}-1)*{}+{}", num_leaves, node_store.layout().max_bytes_per_leaf(), rightmost_leaf_num_bytes
                    )
                })?)
        };
        match (*self, root_node) {
            (Self::SizeUnknown, DataNode::Inner(root_node)) => {
                let traversal::NumLeavesAndRightmostLeafId {
                    num_leaves,
                    rightmost_leaf_id,
                } = traversal::calculate_num_leaves_and_rightmost_leaf_id(node_store, root_node)
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
                let num_leaves = NonZeroU64::new(1).unwrap();
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

    pub fn update(
        &mut self,
        layout: &NodeLayout,
        num_leaves: NonZeroU64,
        total_num_bytes: u64,
    ) -> Result<()> {
        let max_bytes_per_leaf = u64::from(layout.max_bytes_per_leaf());
        let num_bytes_in_left_leaves = (num_leaves.get() - 1) * max_bytes_per_leaf;
        let rightmost_leaf_num_bytes = u32::try_from(total_num_bytes.checked_sub(num_bytes_in_left_leaves).ok_or_else(||anyhow!("Tried to update cache to total_num_bytes={} but with max_bytes_per_leaf={} and num_leaves={}, we should have at least {}", total_num_bytes, max_bytes_per_leaf, num_leaves, num_bytes_in_left_leaves))?).unwrap();
        *self = Self::NumBytesIsKnown {
            num_leaves,
            rightmost_leaf_num_bytes,
        };
        Ok(())
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
