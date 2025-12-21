use std::fmt::Debug;

use futures::{
    FutureExt as _,
    future::{BoxFuture, Shared},
};

use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    event::Event,
};

use crate::entry::immediate_drop_request::{ImmediateDropRequest, ImmediateDropRequestResponse};

pub struct EntryStateDropping<V>
where
    V: AsyncDrop + Debug + Send + 'static,
{
    future: Shared<BoxFuture<'static, ()>>,
    /// If an immediate drop was requested while in this state.
    /// When the current drop completes, this drop_fn should be executed with None.
    immediate_drop_request: ImmediateDropRequest<V>,
}

impl<V> EntryStateDropping<V>
where
    V: AsyncDrop + Debug + Send + 'static,
{
    pub fn new(future: Shared<BoxFuture<'static, ()>>) -> Self {
        Self {
            future,
            immediate_drop_request: ImmediateDropRequest::NotRequested,
        }
    }

    pub fn new_dummy() -> Self {
        Self::new(futures::future::ready(()).boxed().shared())
    }

    pub fn future(&self) -> &Shared<BoxFuture<'static, ()>> {
        &self.future
    }

    pub fn into_future(self) -> Shared<BoxFuture<'static, ()>> {
        self.future
    }

    /// Request immediate drop for the entry. Since the entry is already being dropped,
    /// the drop_fn will be executed with None after the current drop completes.
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

    /// Take the immediate drop request, if any. This is used when the current drop completes
    /// to execute any pending immediate drop.
    pub fn take_immediate_drop_request(&mut self) -> ImmediateDropRequest<V> {
        std::mem::replace(&mut self.immediate_drop_request, ImmediateDropRequest::NotRequested)
    }
}

impl<V> Debug for EntryStateDropping<V>
where
    V: AsyncDrop + Debug + Send + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntryStateDropping")
            .field("immediate_drop_request", &self.immediate_drop_request)
            .finish_non_exhaustive()
    }
}
