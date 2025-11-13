use std::fmt::Debug;
use std::hash::Hash;

use cryfs_utils::async_drop::AsyncDrop;
use cryfs_utils::safe_panic;

use crate::common::HandleTrait;
use crate::object_based_api::utils::inode_list::handle_forest::HandleForest;

/// A delayed release of a removed handle back to the HandleForest.
#[must_use]
pub struct DelayedHandleRelease<Handle>
where
    Handle: HandleTrait + Send,
{
    // Always Some except after destruction
    removed_handle: Option<Handle>,
}

impl<Handle> DelayedHandleRelease<Handle>
where
    Handle: HandleTrait + Send,
{
    pub fn new(removed_handle: Handle) -> Self {
        Self {
            removed_handle: Some(removed_handle),
        }
    }

    pub fn release<EdgeKey, NodeValue>(
        mut self,
        forest: &mut HandleForest<Handle, EdgeKey, NodeValue>,
    ) where
        EdgeKey: PartialEq + Eq + Hash + Debug + Send + Clone,
        NodeValue: AsyncDrop + Send + Debug,
        <NodeValue as AsyncDrop>::Error: Send,
    {
        let removed_handle = self.removed_handle.take().expect("Already destructed");
        forest.release_removed_handle(removed_handle);
    }
}

impl<Handle> Drop for DelayedHandleRelease<Handle>
where
    Handle: HandleTrait + Send,
{
    fn drop(&mut self) {
        if let Some(_) = self.removed_handle.take() {
            safe_panic!(
                "DelayedHandleRelease dropped without calling release(): releasing handle back to forest"
            );
        }
    }
}
