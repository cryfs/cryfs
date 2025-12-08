use async_trait::async_trait;
use std::borrow::Borrow;
use std::hash::Hash;
use std::{collections::HashMap, fmt::Debug};

use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    containers::{HashMapExt, OccupiedError},
};

use crate::common::HandleTrait;

#[derive(Debug)]
pub struct Node<Handle, EdgeKey, NodeValue>
where
    Handle: HandleTrait + Send,
    EdgeKey: PartialEq + Eq + Hash + Clone + Debug + Send,
    NodeValue: AsyncDrop + Send + Debug,
    <NodeValue as AsyncDrop>::Error: Send,
{
    /// Parent inode, together with the EdgeKey used to get from the parent to this node
    parent: Option<(Handle, EdgeKey)>,
    children: HashMap<EdgeKey, Handle>,
    value: AsyncDropGuard<NodeValue>,
}

impl<Handle, EdgeKey, NodeValue> Node<Handle, EdgeKey, NodeValue>
where
    Handle: HandleTrait + Send,
    EdgeKey: PartialEq + Eq + Hash + Clone + Debug + Send,
    NodeValue: AsyncDrop + Send + Debug,
    <NodeValue as AsyncDrop>::Error: Send,
{
    pub fn new_root(value: AsyncDropGuard<NodeValue>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            parent: None,
            children: HashMap::new(),
            value,
        })
    }

    pub fn new(
        parent: Handle,
        edge: EdgeKey,
        value: AsyncDropGuard<NodeValue>,
    ) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            parent: Some((parent, edge)),
            children: HashMap::new(),
            value,
        })
    }

    pub fn parent_handle(&self) -> Option<&Handle> {
        self.parent.as_ref().map(|(handle, _)| handle)
    }

    pub fn parent(&self) -> Option<&(Handle, EdgeKey)> {
        self.parent.as_ref()
    }

    pub(super) fn set_parent(&mut self, parent: Option<(Handle, EdgeKey)>) {
        self.parent = parent;
    }

    pub fn get_child<E>(&self, edge: &E) -> Option<&Handle>
    where
        E: ?Sized + Hash + Eq,
        EdgeKey: Borrow<E>,
    {
        self.children.get(edge)
    }

    pub(super) fn try_insert_child(
        &mut self,
        edge: EdgeKey,
        value: Handle,
    ) -> Result<(), OccupiedError<'_, EdgeKey, Handle>> {
        HashMapExt::try_insert(&mut self.children, edge, value)?;
        Ok(())
    }

    // Returns old handle if it was overwritten
    pub(super) fn insert_child(&mut self, edge: EdgeKey, value: Handle) -> Option<Handle> {
        self.children.insert(edge, value)
    }

    /// Removes the child with the given edge/handle combination. If no such child exists, returns None.
    pub(super) fn try_remove_child_by_handle(
        &mut self,
        child_ino: &Handle,
        edge: &EdgeKey,
    ) -> Result<RemoveResult, TryRemoveChildByHandleError> {
        let Some((removed, remove_result)) = self.try_remove_child(edge) else {
            return Err(TryRemoveChildByHandleError::EdgeNotFound);
        };
        if removed != *child_ino {
            // Put it back
            self.children.insert(edge.clone(), removed);
            return Err(TryRemoveChildByHandleError::EdgeLeadsToDifferentNode);
        }
        Ok(remove_result)
    }

    pub(super) fn try_remove_child<K>(&mut self, edge: &K) -> Option<(Handle, RemoveResult)>
    where
        K: ?Sized + Hash + Eq,
        EdgeKey: Borrow<K>,
    {
        let removed = self.children.remove(edge)?;
        if self.children.is_empty() {
            Some((removed, RemoveResult::NoChildrenLeft))
        } else {
            Some((removed, RemoveResult::StillHasChildren))
        }
    }

    pub fn num_children(&self) -> usize {
        self.children.len()
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    pub fn into_value(this: AsyncDropGuard<Self>) -> AsyncDropGuard<NodeValue> {
        this.unsafe_into_inner_dont_drop().value
    }

    pub fn value(&self) -> &AsyncDropGuard<NodeValue> {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut AsyncDropGuard<NodeValue> {
        &mut self.value
    }
}

#[async_trait]
impl<Handle, EdgeKey, NodeValue> AsyncDrop for Node<Handle, EdgeKey, NodeValue>
where
    Handle: HandleTrait + Send,
    EdgeKey: PartialEq + Eq + Hash + Clone + Debug + Send,
    NodeValue: AsyncDrop + Send + Debug,
    <NodeValue as AsyncDrop>::Error: Send,
{
    type Error = <NodeValue as AsyncDrop>::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.value.async_drop().await
    }
}

#[must_use]
pub enum RemoveResult {
    StillHasChildren,
    NoChildrenLeft,
}

pub enum TryRemoveChildByHandleError {
    EdgeNotFound,
    EdgeLeadsToDifferentNode,
}
