use std::fmt::Debug;
use std::hash::Hash;

use async_trait::async_trait;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard, AsyncDropHashMap};
use cryfs_utils::containers::OccupiedError;
use derive_more::{Display, Error};

use crate::common::{HandlePool, HandleTrait, HandleWithGeneration};
use crate::object_based_api::utils::inode_list::handle_forest::DelayedHandleRelease;
use crate::object_based_api::utils::inode_list::handle_forest::insert_transaction::InsertTransaction;
use crate::object_based_api::utils::inode_list::handle_forest::node::{Node, RemoveResult};

#[derive(Debug)]
pub struct HandleForest<Handle, EdgeKey, NodeValue>
where
    Handle: HandleTrait + Send,
    EdgeKey: PartialEq + Eq + Hash + Debug + Send + Clone,
    NodeValue: AsyncDrop + Send + Debug,
    <NodeValue as AsyncDrop>::Error: Send,
{
    // TODO Using a slab of inodes might be a more efficient way to assign inode numbers by index

    // Invariants:
    // * A: Each entry in `nodes` has its Handle key blocked in `handles`
    // * Note: There might be additional handles blocked in `handles` that don't have a correspondng entry in `nodes`.
    //   * Calls to [HandleForest::block_handle] will permanently block a handle without adding a node.
    //   * Instances of [InsertTransaction] or [DelayedHandleRelease] will temporarily block a handle that doesn't yet have or no longer has a node.
    handles: HandlePool<Handle>,

    // Invariants:
    // * B: Tree pointers are well formed, i.e.
    //   * B1: Each Node.parent pointer points to a valid entry in the `nodes` map (if Some. If None, the node is a root)
    //   * B2: Each Node.children[_] pointer points to a valid entry in the `nodes` map
    //   * TODO Should it be an invariant that Node.children[_].parent and/or Node.parent.children[_] point back to the original node?
    //          When a file gets deleted, its inode might point to a parent that doesn't point back to it because the file can't be looked up anymore.
    //          Or maybe we should then also remove the parnt pointer of the file inode? If we do that, we could have this invariant.
    //          See also TODO in TryRemoveResult::ParentDidntHaveRemovedNodeAsChild
    // * Note: Not every node may have a parent. We're a forest, there might be multiple roots.
    nodes: AsyncDropGuard<AsyncDropHashMap<Handle, Node<Handle, EdgeKey, NodeValue>>>,
}

impl<Handle, EdgeKey, NodeValue> HandleForest<Handle, EdgeKey, NodeValue>
where
    Handle: HandleTrait + Send,
    EdgeKey: PartialEq + Eq + Hash + Debug + Send + Clone,
    NodeValue: AsyncDrop + Send + Debug,
    <NodeValue as AsyncDrop>::Error: Send,
{
    pub fn new() -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            handles: HandlePool::new(),
            nodes: AsyncDropHashMap::new(),
        })
    }

    pub fn block_handle(&mut self, handle: Handle) {
        self.handles.acquire_specific(handle);
    }

    pub fn get(&self, index: &Handle) -> Option<&Node<Handle, EdgeKey, NodeValue>> {
        let node = self.nodes.get(index)?;
        Some(&**node)
    }

    pub fn get_mut(&mut self, index: &Handle) -> Option<&mut Node<Handle, EdgeKey, NodeValue>> {
        let node = self.nodes.get_mut(index)?;
        Some(&mut **node)
    }

    pub fn try_insert_root_with_specific_handle(
        &mut self,
        handle: Handle,
        value: AsyncDropGuard<NodeValue>,
    ) -> Result<(), AlreadyExistsError<NodeValue>> {
        match self.handles.try_acquire_specific(handle.clone()) {
            Some(_) => {
                // handle acquired, everything ok
            }
            None => {
                return Err(AlreadyExistsError { value });
            }
        }
        let node = Node::new_root(value);
        self.nodes
            .try_insert(handle, node)
            .expect("Invariant A violated");
        Ok(())
    }

    pub fn get_child_of_mut(
        &mut self,
        parent_handle: &Handle,
        edge: &EdgeKey,
    ) -> Result<
        (
            HandleWithGeneration<Handle>,
            &mut Node<Handle, EdgeKey, NodeValue>,
        ),
        GetChildOfError,
    > {
        let Some(parent_node) = self.nodes.get(parent_handle) else {
            return Err(GetChildOfError::ParentNotFound);
        };
        let Some(child_handle) = parent_node.get_child(edge) else {
            return Err(GetChildOfError::ChildNotFound);
        };
        let child_handle = child_handle.clone();
        let child_node = self
            .nodes
            .get_mut(&child_handle)
            .expect("Invariant B2 violated");
        let child_handle = self
            .handles
            .lookup(child_handle)
            .expect("Invariant A violated");
        Ok((child_handle, &mut *child_node))
    }

    // Invariant:
    // * value_fn is executed if and only if a new node is successfully inserted.
    //   * If `value_fn` is executed, a new node is successfully inserted (or we panic).
    //   * If `value_fn` was not executed, an error is returned.
    pub async fn try_insert(
        &mut self,
        parent_handle: Handle,
        edge: EdgeKey,
        value_fn: impl AsyncFnOnce(&HandleWithGeneration<Handle>) -> AsyncDropGuard<NodeValue>,
    ) -> Result<
        (
            HandleWithGeneration<Handle>,
            &Node<Handle, EdgeKey, NodeValue>,
        ),
        TryInsertError2,
    > {
        let Some(parent) = self.nodes.get_mut(&parent_handle) else {
            return Err(TryInsertError2::ParentNotFound);
        };

        let new_handle = self.handles.acquire();

        let Ok(_) = parent.try_insert_child(edge, new_handle.handle.clone()) else {
            self.handles.undo_acquire(new_handle.handle);
            return Err(TryInsertError2::AlreadyExists);
        };

        let value = value_fn(&new_handle).await;

        match self
            .nodes
            .try_insert(new_handle.handle.clone(), Node::new(parent_handle, value))
        {
            Ok(node) => Ok((new_handle, node)),
            Err(OccupiedError { entry: _, value: _ }) => {
                panic!("Invariant A violated");
            }
        }
    }

    pub fn start_insert_transaction(&mut self) -> InsertTransaction<Handle> {
        let reserved_handle = self.handles.acquire();
        InsertTransaction::new(reserved_handle)
    }

    pub(super) fn abort_insert_transaction(
        &mut self,
        reserved_handle: HandleWithGeneration<Handle>,
    ) {
        self.handles.undo_acquire(reserved_handle.handle);
    }

    pub(super) fn commit_insert_transaction(
        &mut self,
        parent_handle: Handle,
        edge: EdgeKey,
        value: AsyncDropGuard<NodeValue>,
        reserved_handle: HandleWithGeneration<Handle>,
    ) -> Result<(), TryInsertError<NodeValue>> {
        let Some(parent) = self.nodes.get_mut(&parent_handle) else {
            self.abort_insert_transaction(reserved_handle);
            return Err(TryInsertError::ParentNotFound { value });
        };

        let Ok(_) = parent.try_insert_child(edge, reserved_handle.handle.clone()) else {
            self.abort_insert_transaction(reserved_handle);
            return Err(TryInsertError::AlreadyExists { value });
        };

        match self
            .nodes
            .try_insert(reserved_handle.handle, Node::new(parent_handle, value))
        {
            Ok(_node) => (),
            Err(OccupiedError { entry: _, value: _ }) => {
                panic!("Invariant A violated");
            }
        }
        Ok(())
    }

    pub fn try_remove(
        &mut self,
        handle: Handle,
    ) -> Result<
        (
            AsyncDropGuard<NodeValue>,
            TryRemoveResult<Handle>,
            DelayedHandleRelease<Handle>,
        ),
        TryRemoveError,
    > {
        let node = self
            .nodes
            .get(&handle)
            .ok_or(TryRemoveError::NodeNotFound)?;

        if node.has_children() {
            return Err(TryRemoveError::NodeStillHasChildren);
        }
        // TODO Entry API might allow doing this without looking it up again
        let node = self
            .nodes
            .remove(&handle)
            .expect("We just checked above that it exists");

        let result = if let Some(parent_handle) = node.parent_handle() {
            let Some(parent_node) = self.nodes.get_mut(parent_handle) else {
                panic!("Invariant B1 violated");
            };
            match parent_node.try_remove_child_by_handle(&handle) {
                Some(RemoveResult::NoChildrenLeft) => {
                    TryRemoveResult::JustRemovedLastChildOfParent {
                        parent_handle: parent_handle.clone(),
                    }
                }
                Some(RemoveResult::StillHasChildren) => TryRemoveResult::ParentStillHasChildren {
                    parent_handle: parent_handle.clone(),
                },
                None => TryRemoveResult::ParentDidntHaveRemovedNodeAsChild {
                    parent_handle: parent_handle.clone(),
                },
            }
        } else {
            TryRemoveResult::NoParent
        };

        let node = Node::into_value(node);
        let delayed_handle_release = DelayedHandleRelease::new(handle);
        Ok((node, result, delayed_handle_release))
    }

    pub(super) fn release_removed_handle(&mut self, removed_handle: Handle) {
        self.handles.release(removed_handle);
    }

    #[cfg(feature = "testutils")]
    pub fn drain(
        &mut self,
    ) -> impl Iterator<Item = (Handle, AsyncDropGuard<Node<Handle, EdgeKey, NodeValue>>)> + '_ {
        self.handles = HandlePool::new();
        self.nodes.drain()
    }
}

#[derive(Error, Debug, Display)]
pub enum GetChildOfError {
    ParentNotFound,
    ChildNotFound,
}

#[derive(Error, Debug, Display)]
pub enum TryRemoveError {
    NodeNotFound,
    NodeStillHasChildren,
}

#[derive(Error, Debug, Display)]
pub enum TryRemoveResult<Handle>
where
    Handle: HandleTrait + Send,
{
    NoParent,

    /// Our parent node doesn't have us as a child.
    // TODO is this ok? See TODO above about whether we want that invariant.
    ParentDidntHaveRemovedNodeAsChild {
        parent_handle: Handle,
    },
    ParentStillHasChildren {
        parent_handle: Handle,
    },
    JustRemovedLastChildOfParent {
        parent_handle: Handle,
    },
}

#[must_use]
#[derive(Error, Debug, Display)]
pub enum TryInsertError<NodeValue>
where
    NodeValue: AsyncDrop + Send + Debug,
{
    ParentNotFound { value: AsyncDropGuard<NodeValue> },
    AlreadyExists { value: AsyncDropGuard<NodeValue> },
}

#[must_use]
#[derive(Error, Debug, Display)]
pub enum TryInsertError2 {
    ParentNotFound,
    AlreadyExists,
}

#[must_use]
#[derive(Error, Debug, Display)]
pub struct AlreadyExistsError<NodeValue>
where
    NodeValue: AsyncDrop + Send + Debug,
{
    value: AsyncDropGuard<NodeValue>,
}

#[async_trait]
impl<Handle, EdgeKey, NodeValue> AsyncDrop for HandleForest<Handle, EdgeKey, NodeValue>
where
    Handle: HandleTrait + Send,
    EdgeKey: PartialEq + Eq + Hash + Debug + Send + Clone,
    NodeValue: AsyncDrop + Send + Debug,
    <NodeValue as AsyncDrop>::Error: Send,
{
    type Error = <NodeValue as AsyncDrop>::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.nodes.async_drop().await
    }
}
