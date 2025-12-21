use std::fmt::Debug;
use std::hash::Hash;

use futures::future::{BoxFuture, Shared};

use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    event::Event,
};

use crate::entry::{
    immediate_drop_request::{ImmediateDropRequest, ImmediateDropRequestResponse},
    loading::LoadingResult,
    waiter::EntryLoadingWaiter,
};

/// Represents an entry that is currently dropping but has pending load requests.
/// When the drop completes, this will transition to Loading state via the combined future.
///
/// This state allows `get_loaded_or_insert_loading` to return immediately (without awaiting)
/// even when an entry is being dropped. The returned waiter's future handles both:
/// 1. Waiting for the current drop to complete
/// 2. Loading the new value
#[derive(Debug)]
pub struct EntryStateDroppingThenLoading<V, E>
where
    V: AsyncDrop + Debug + Send + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    /// The shared future representing the combined operation:
    /// wait for drop to complete, then load the entry.
    combined_future: Shared<BoxFuture<'static, LoadingResult<E>>>,

    /// Number of tasks waiting for this entry to load.
    /// This is only ever incremented, similar to EntryStateLoading.
    num_waiters: usize,

    /// If an immediate drop was requested while in this state.
    /// This will be propagated to the new Loaded state when loading completes.
    immediate_drop_request: ImmediateDropRequest<V>,
}

impl<V, E> EntryStateDroppingThenLoading<V, E>
where
    V: AsyncDrop + Debug + Send + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    /// Create a new DroppingThenLoading state with a combined future.
    /// The first waiter is automatically counted.
    pub fn new(combined_future: Shared<BoxFuture<'static, LoadingResult<E>>>) -> Self {
        Self {
            combined_future,
            num_waiters: 0,
            immediate_drop_request: ImmediateDropRequest::NotRequested,
        }
    }

    /// Add a waiter for this entry. Returns an EntryLoadingWaiter that can be used
    /// to wait for the combined operation (drop + load) to complete.
    pub fn add_waiter<K>(&mut self, key: K) -> EntryLoadingWaiter<K, E>
    where
        K: Hash + Eq + Clone + Debug + Send + Sync,
    {
        self.num_waiters += 1;
        EntryLoadingWaiter::new(key, self.combined_future.clone())
    }

    /// Get the number of waiters for this entry.
    pub fn num_waiters(&self) -> usize {
        self.num_waiters
    }

    /// Request immediate drop for the entry once it's loaded.
    /// This will be propagated to the new Loaded state when loading completes.
    pub fn request_immediate_drop_if_not_yet_requested<F>(
        &mut self,
        drop_fn: impl FnOnce(Option<AsyncDropGuard<V>>) -> F + Send + Sync + 'static,
    ) -> ImmediateDropRequestResponse
    where
        F: Future<Output = ()> + Send,
    {
        self.immediate_drop_request
            .request_immediate_drop_if_not_yet_requested(drop_fn)
    }

    /// Check if immediate drop was requested for this entry.
    pub fn immediate_drop_requested(&self) -> Option<&Event> {
        self.immediate_drop_request.immediate_drop_requested()
    }

    /// Consume this state and return the immediate drop request.
    /// This is used when transitioning to Loading/Loaded state.
    pub fn into_immediate_drop_request(self) -> ImmediateDropRequest<V> {
        self.immediate_drop_request
    }

    /// Get a reference to the combined future.
    pub fn combined_future(&self) -> &Shared<BoxFuture<'static, LoadingResult<E>>> {
        &self.combined_future
    }
}
