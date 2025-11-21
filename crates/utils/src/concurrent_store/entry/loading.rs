use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Error;
use futures::{
    FutureExt as _,
    future::{BoxFuture, Shared},
};

use crate::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    concurrent_store::entry::{
        immediate_drop_request::{ImmediateDropRequest, ImmediateDropRequestResponse},
        waiter::EntryLoadingWaiter,
    },
    mr_oneshot_channel,
};

pub struct EntryStateLoading<V, D>
where
    V: AsyncDrop + Debug + Send + Sync + 'static,
    D: Clone + Debug + Send + Sync + 'static,
{
    /// loading_result is a future that will hold the result of the loading operation once it is complete.
    /// See [LoadingResult] for an explanation of the possible results.
    loading_result: Shared<BoxFuture<'static, LoadingResult>>, // TODO No BoxFuture but impl Future?
    /// Number of tasks currently waiting for this entry to be loaded. This is only ever incremented. Even if a waiter completes, it won't be decremented.
    num_waiters: usize,
    /// If ImmediateDropRequest::Requested: While we're loading, another thread triggered an immediate drop request for this entry. Don't allow further loaders, and when this is unloaded, call the triggering thread's callback with exclusive access.
    immediate_drop_request: ImmediateDropRequest<V, D>,
}

#[derive(Clone)]
pub enum LoadingResult {
    /// The entry was successfully loaded. This loading result means the entry state was already changed to [Entry::Loaded] and can be accessed immediately.
    Loaded,

    /// The entry was not found. The entry was removed from the map.
    NotFound,

    /// An error occurred while loading the entry. The entry state was removed from the map.
    Error(Arc<anyhow::Error>),
}

impl<V, D> EntryStateLoading<V, D>
where
    V: AsyncDrop + Debug + Send + Sync + 'static,
    D: Clone + Debug + Send + Sync + 'static,
{
    pub fn new(loading_result: BoxFuture<'static, LoadingResult>) -> Self {
        EntryStateLoading {
            loading_result: loading_result.shared(),
            num_waiters: 0,
            immediate_drop_request: ImmediateDropRequest::NotRequested,
        }
    }

    pub fn new_dummy() -> Self {
        EntryStateLoading {
            loading_result: futures::future::pending().boxed().shared(),
            num_waiters: 0,
            immediate_drop_request: ImmediateDropRequest::NotRequested,
        }
    }

    pub fn add_waiter(&mut self) -> EntryLoadingWaiter {
        self.num_waiters += 1;
        EntryLoadingWaiter::new(self.loading_result.clone())
    }

    pub fn num_waiters(&self) -> usize {
        self.num_waiters
    }

    pub fn request_immediate_drop_if_not_yet_requested<F>(
        &mut self,
        drop_fn: impl FnOnce(Option<AsyncDropGuard<V>>) -> F + Send + Sync + 'static,
    ) -> ImmediateDropRequestResponse<D>
    where
        F: Future<Output = Result<D, Arc<Error>>> + Send,
    {
        self.immediate_drop_request
            .request_immediate_drop_if_not_yet_requested(drop_fn)
    }

    pub fn immediate_drop_requested(
        &self,
    ) -> Option<mr_oneshot_channel::Receiver<Result<D, Arc<Error>>>> {
        self.immediate_drop_request.immediate_drop_requested()
    }

    pub fn into_immediate_drop_request(self) -> ImmediateDropRequest<V, D> {
        self.immediate_drop_request
    }
}
