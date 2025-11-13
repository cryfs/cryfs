use std::fmt::Debug;
use std::hash::Hash;

use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    safe_panic,
};

use crate::{
    common::{HandleTrait, HandleWithGeneration},
    object_based_api::utils::inode_list::handle_forest::handle_forest::{
        HandleForest, TryInsertError,
    },
};

/// A transaction for inserting a new node into the HandleForest.
/// When the transaction is created, a handle is reserved for the new node.
/// The transaction must be either committed or aborted.
#[must_use]
pub struct InsertTransaction<Handle>
where
    Handle: HandleTrait + Send,
{
    // Always Some except after the transaction was committed or abortd
    reserved_handle: Option<HandleWithGeneration<Handle>>,
}

impl<Handle> InsertTransaction<Handle>
where
    Handle: HandleTrait + Send,
{
    pub fn new(reserved_handle: HandleWithGeneration<Handle>) -> Self {
        Self {
            reserved_handle: Some(reserved_handle),
        }
    }

    pub fn handle(&self) -> &HandleWithGeneration<Handle> {
        self.reserved_handle.as_ref().expect("Already destructed")
    }

    pub fn commit<EdgeKey, NodeValue>(
        mut self,
        forest: &mut HandleForest<Handle, EdgeKey, NodeValue>,
        parent_handle: Handle,
        edge: EdgeKey,
        value: AsyncDropGuard<NodeValue>,
    ) -> Result<(), TryInsertError<NodeValue>>
    where
        EdgeKey: PartialEq + Eq + Hash + Debug + Send + Clone,
        NodeValue: AsyncDrop + Send + Debug,
        <NodeValue as AsyncDrop>::Error: Send,
    {
        let reserved_handle = self.reserved_handle.take().expect("Already destructed");
        forest.commit_insert_transaction(parent_handle, edge, value, reserved_handle)?;
        Ok(())
    }

    pub fn abort<EdgeKey, NodeValue>(
        mut self,
        forest: &mut HandleForest<Handle, EdgeKey, NodeValue>,
    ) where
        EdgeKey: PartialEq + Eq + Hash + Debug + Send + Clone,
        NodeValue: AsyncDrop + Send + Debug,
        <NodeValue as AsyncDrop>::Error: Send,
    {
        let reserved_handle = self.reserved_handle.take().expect("Already destructed");
        forest.abort_insert_transaction(reserved_handle);
    }
}

impl<Handle> Drop for InsertTransaction<Handle>
where
    Handle: HandleTrait + Send,
{
    fn drop(&mut self) {
        if let Some(_) = self.reserved_handle.take() {
            safe_panic!("InsertTransaction dropped without calling commit() or abort()");
        }
    }
}
