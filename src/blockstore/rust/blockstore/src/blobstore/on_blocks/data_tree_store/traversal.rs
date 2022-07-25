use anyhow::{anyhow, bail, Error, Result};
use async_recursion::async_recursion;
use futures::stream::{FuturesUnordered, Stream, StreamExt};
use std::borrow::Cow;
use std::future::{self, Future};
use std::num::NonZeroU64;

use crate::blobstore::on_blocks::data_node_store::{
    DataInnerNode, DataLeafNode, DataNode, DataNodeStore,
};
use crate::blockstore::{low_level::BlockStore, BlockId};
use crate::utils::num::NonZeroU64Ext;
use crate::utils::stream::{for_each_unordered, run_to_completion};

#[async_recursion]
pub async fn all_leaves<B, F>(
    store: &DataNodeStore<B>,
    root: &mut DataNode<B>,
    on_leaf: &(impl Sync + Fn(&mut DataLeafNode<B>) -> F),
) -> Result<()>
where
    B: BlockStore + Send + Sync,
    F: Future<Output = Result<()>> + Send,
{
    match root {
        DataNode::Leaf(leaf) => {
            on_leaf(leaf).await?;
        }
        DataNode::Inner(inner) => {
            for_each_unordered(_load_children(store, inner)?, |child| async move {
                let mut child = child.await?;
                all_leaves(store, &mut child, on_leaf).await?;
                Ok(())
            })
            .await?;
        }
    }
    Ok(())
}

fn _load_children<'a, 'b, 'r, B: BlockStore + Send + Sync>(
    store: &'a DataNodeStore<B>,
    inner: &'b DataInnerNode<B>,
) -> Result<impl 'r + Iterator<Item = impl 'a + Future<Output = Result<DataNode<B>>>>>
where
    'a: 'r,
    'b: 'r,
{
    let futures = inner.children().map(move |child_id| async move {
        let loaded: Result<DataNode<B>> = Ok(store.load(child_id).await?.ok_or_else(|| {
            anyhow!(
                "Tried to load child node {:?} but couldn't find it",
                child_id,
            )
        })?);
        loaded
    });
    Ok(futures)
}

pub struct NumLeavesAndRightmostLeafId {
    pub num_leaves: NonZeroU64,
    pub rightmost_leaf_id: BlockId,
}

#[async_recursion]
pub async fn calculate_num_leaves_and_rightmost_leaf_id<B: BlockStore + Send + Sync>(
    node_store: &DataNodeStore<B>,
    root_node: &DataInnerNode<B>,
) -> Result<NumLeavesAndRightmostLeafId> {
    let depth = root_node.depth();
    let children = root_node.children();
    assert!(
        children.len() >= 1,
        "Inner node must have at least 1 child, that's a class invariant of DataInnerNode"
    );
    if depth.get() == 1 {
        Ok(NumLeavesAndRightmostLeafId {
            num_leaves: NonZeroU64::new(u64::try_from(children.len()).unwrap()).unwrap(),
            rightmost_leaf_id: children
                .last()
                .expect("Inner node must have at least one child"),
        })
    } else {
        let num_leaves_per_full_child = u64::from(node_store.max_children_per_inner_node())
            .checked_pow(u32::from(depth.get() - 1))
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
        let last_child_id = children.last().expect(
            "Inner node must have at least one child, that's a class invariant of DataInnerNode",
        );
        let last_child = node_store.load(last_child_id).await?.ok_or_else(|| {
            anyhow!(
                "Tried to load {:?} as a child node but couldn't find it",
                last_child_id
            )
        })?;
        let NumLeavesAndRightmostLeafId {
            num_leaves: num_leaves_in_right_child,
            rightmost_leaf_id,
        } = match last_child {
            DataNode::Leaf(_last_child) => {
                bail!(
                    "Loaded {:?} as a leaf node but the inner node above it has depth {}",
                    last_child_id,
                    depth,
                );
            }
            DataNode::Inner(last_child) => {
                calculate_num_leaves_and_rightmost_leaf_id(node_store, &last_child).await?
            }
        };
        let num_leaves = num_leaves_in_right_child
            .checked_add(num_leaves_in_left_children)
            .ok_or_else(|| {
                anyhow!(
                    "Overflow in num_leaves_in_right_child+num_leaves_in_left_children: {}+{}",
                    num_leaves_in_right_child,
                    num_leaves_in_left_children,
                )
            })?;
        Ok(NumLeavesAndRightmostLeafId {
            num_leaves,
            rightmost_leaf_id,
        })
    }
}
