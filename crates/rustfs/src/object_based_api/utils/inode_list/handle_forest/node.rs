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
    parent: Option<Handle>,
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

    pub fn new(parent: Handle, value: AsyncDropGuard<NodeValue>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            parent: Some(parent),
            children: HashMap::new(),
            value,
        })
    }

    pub fn parent_handle(&self) -> Option<&Handle> {
        self.parent.as_ref()
    }

    pub(super) fn set_parent(&mut self, parent: Option<Handle>) {
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
        self.children.try_insert(edge, value)?;
        Ok(())
    }

    // Returns old handle if it was overwritten
    pub(super) fn insert_child(&mut self, edge: EdgeKey, value: Handle) -> Option<Handle> {
        self.children.insert(edge, value)
    }

    pub(super) fn try_remove_child_by_handle(
        &mut self,
        child_ino: &Handle,
    ) -> Option<RemoveResult> {
        let mut remove_key: Option<EdgeKey> = None;
        // TODO Might be better to keep a reverse map from child_ino to name to avoid this linear scan
        //      Or maybe check call sites for whether they alrady have the parent_ino and name available, that would make this cheaper here.
        for (edge_key, entry_child_ino) in &self.children {
            if entry_child_ino == child_ino {
                remove_key = Some(edge_key.clone());
                break;
            }
        }
        if let Some(edge_key) = remove_key {
            let (removed_handle, remove_result) = self
                .try_remove_child(&edge_key)
                .expect("This should never happen because we just found the child by inode number");
            assert_eq!(*child_ino, removed_handle);
            Some(remove_result)
        } else {
            None
        }
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

    pub fn has_child<K>(&self, edge: &K) -> bool
    where
        K: ?Sized + Hash + Eq,
        EdgeKey: Borrow<K>,
    {
        self.children.contains_key(edge)
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
