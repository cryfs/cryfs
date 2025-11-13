use async_trait::async_trait;
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

    pub fn get_child(&self, edge: &EdgeKey) -> Option<&Handle> {
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
            Some(
                self.try_remove_child(&edge_key).expect(
                    "This should never happen because we just found the child by inode number",
                ),
            )
        } else {
            None
        }
    }

    pub(super) fn try_remove_child(&mut self, edge: &EdgeKey) -> Option<RemoveResult> {
        self.children.remove(edge)?;
        if self.children.is_empty() {
            Some(RemoveResult::NoChildrenLeft)
        } else {
            Some(RemoveResult::StillHasChildren)
        }
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
