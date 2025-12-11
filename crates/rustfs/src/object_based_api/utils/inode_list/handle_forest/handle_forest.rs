use std::borrow::Borrow;
use std::fmt::Debug;
use std::hash::Hash;

use async_trait::async_trait;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard, AsyncDropHashMap};
use cryfs_utils::containers::OccupiedError;
use derive_more::{Display, Error};

use crate::common::{HandlePool, HandleTrait, HandleWithGeneration};
use crate::object_based_api::utils::inode_list::handle_forest::DelayedHandleRelease;
use crate::object_based_api::utils::inode_list::handle_forest::node::{
    Node, RemoveResult, TryRemoveChildByHandleError,
};

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
    // * C: Each Node.children[_].parent entry points back to the original node, with the correct EdgeKey stored in the parent pointer.
    //   * Note: The reverse isn't necessarily true. An orphaned node might still point to its parent even though the parent doesn't have it in its chldren array.
    // * Note: Not every node may have a parent. We're a forest, there can be orphaned nodes that no parent points to.
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
        let node: &mut AsyncDropGuard<Node<Handle, EdgeKey, NodeValue>> =
            self.nodes.get_mut(index)?;
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
    pub async fn try_insert<I>(
        &mut self,
        parent_handle: Handle,
        edge: EdgeKey,
        mut value_fn_input: AsyncDropGuard<I>,
        value_fn: impl AsyncFnOnce(
            AsyncDropGuard<I>,
            &HandleWithGeneration<Handle>,
        ) -> AsyncDropGuard<NodeValue>,
    ) -> Result<
        (
            HandleWithGeneration<Handle>,
            &Node<Handle, EdgeKey, NodeValue>,
        ),
        TryInsertError,
    >
    where
        I: AsyncDrop + Debug,
    {
        let Some(parent) = self.nodes.get_mut(&parent_handle) else {
            value_fn_input.async_drop().await.unwrap(); // TODO No unwrap
            return Err(TryInsertError::ParentNotFound);
        };

        let new_handle = self.handles.acquire();

        let Ok(_) = parent.try_insert_child(edge.clone(), new_handle.handle.clone()) else {
            value_fn_input.async_drop().await.unwrap(); // TODO No unwrap
            self.handles.undo_acquire(new_handle.handle);
            return Err(TryInsertError::AlreadyExists);
        };

        let value = value_fn(value_fn_input, &new_handle).await;

        match self.nodes.try_insert(
            new_handle.handle.clone(),
            Node::new(parent_handle, edge, value),
        ) {
            Ok(node) => Ok((new_handle, node)),
            Err(OccupiedError {
                entry: _,
                mut value,
            }) => {
                value.async_drop().await.unwrap();
                panic!("Invariant A violated");
            }
        }
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

        let result = if let Some((parent_handle, edge_from_parent)) = node.parent() {
            let Some(parent_node) = self.nodes.get_mut(parent_handle) else {
                panic!("Invariant B1 violated");
            };
            match parent_node.try_remove_child_by_handle(&handle, edge_from_parent) {
                Ok(RemoveResult::NoChildrenLeft) => TryRemoveResult::JustRemovedLastChildOfParent {
                    parent_handle: parent_handle.clone(),
                },
                Ok(RemoveResult::StillHasChildren) => TryRemoveResult::ParentStillHasChildren {
                    parent_handle: parent_handle.clone(),
                },
                Err(TryRemoveChildByHandleError::EdgeNotFound)
                | Err(TryRemoveChildByHandleError::EdgeLeadsToDifferentNode) => {
                    TryRemoveResult::ParentDidntHaveRemovedNodeAsChild {
                        parent_handle: parent_handle.clone(),
                    }
                }
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

    pub fn make_node_into_orphan<K>(
        &mut self,
        parent_handle: &Handle,
        edge: &K,
    ) -> Result<(), MakeOrphanError>
    where
        K: ?Sized + Hash + Eq,
        EdgeKey: Borrow<K>,
    {
        let parent_node = self
            .nodes
            .get_mut(parent_handle)
            .ok_or(MakeOrphanError::ParentNotFound)?;

        let (_removed_handle, remove_result) = parent_node
            .try_remove_child(edge)
            .ok_or(MakeOrphanError::ChildNotFound)?;
        match remove_result {
            RemoveResult::NoChildrenLeft | RemoveResult::StillHasChildren => { /* ok */ }
        }

        // We're leaving the child orphaned, that's ok for invariant C.
        Ok(())
    }

    pub fn move_node<E>(
        &mut self,
        old_parent_handle: Handle,
        old_edge: &E,
        new_parent_handle: Handle,
        new_edge: EdgeKey,
    ) -> Result<MoveInodeSuccess, MoveInodeError>
    where
        EdgeKey: Borrow<E>,
        E: ?Sized + Hash + Eq + ToOwned<Owned = EdgeKey>,
    {
        // Remove from old parent
        let old_parent_node = self
            .nodes
            .get_mut(&old_parent_handle)
            .ok_or(MoveInodeError::OldParentNotFound)?;
        let (child_handle, _remove_result) = old_parent_node
            .try_remove_child(old_edge)
            .ok_or(MoveInodeError::ChildNotFound)?;

        let reinsert_to_old_parent_for_exception_safety = |nodes: &mut AsyncDropGuard<
            AsyncDropHashMap<Handle, Node<Handle, EdgeKey, NodeValue>>,
        >| {
            let old_parent_node = nodes
                .get_mut(&old_parent_handle)
                .expect("We just had it above");
            old_parent_node
                .try_insert_child(old_edge.to_owned(), child_handle.clone())
                .expect("We just removed the child above");
        };

        let Some(new_parent_node) = self.nodes.get_mut(&new_parent_handle) else {
            reinsert_to_old_parent_for_exception_safety(&mut self.nodes);
            return Err(MoveInodeError::NewParentNotFound);
        };

        // Insert into new parent
        let overwritten_child =
            new_parent_node.insert_child(new_edge.clone(), child_handle.clone());

        // Update child's parent pointer
        let child_node = self
            .nodes
            .get_mut(&child_handle)
            .expect("Invariant B2 violated");
        child_node.set_parent(Some((new_parent_handle, new_edge)));

        if overwritten_child.is_some() {
            Ok(MoveInodeSuccess::OrphanedExistingChildInNewParent)
        } else {
            Ok(MoveInodeSuccess::AddedAsNewChildToNewParent)
        }
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
pub enum MakeOrphanError {
    ParentNotFound,
    ChildNotFound,
}

#[must_use]
pub enum MoveInodeSuccess {
    AddedAsNewChildToNewParent,
    OrphanedExistingChildInNewParent,
}

#[derive(Error, Debug, Display)]
pub enum MoveInodeError {
    OldParentNotFound,
    NewParentNotFound,
    ChildNotFound,
}

#[derive(Error, Debug, Display)]
pub enum TryRemoveResult<Handle>
where
    Handle: HandleTrait + Send,
{
    NoParent,

    /// Our parent node doesn't have us as a child. We're an orphaned node.
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
pub enum TryInsertError {
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
