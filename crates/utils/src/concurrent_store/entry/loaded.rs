use std::fmt::Debug;

use crate::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard},
    concurrent_store::entry::{
        immediate_drop_request::{ImmediateDropRequest, ImmediateDropRequestResponse},
        loading::EntryStateLoading,
    },
    mr_oneshot_channel,
};

pub struct EntryStateLoaded<V, D>
where
    V: AsyncDrop + Debug + Send + Sync + 'static,
    D: Clone + Debug + Send + Sync + 'static,
{
    entry: AsyncDropGuard<AsyncDropArc<V>>,
    /// Number of tasks that started waiting for this entry when it was in [Entry::Loading],
    /// but haven't yet incremented the refcount of [Self::entry].
    /// This gets never increased, only initialized when the entry is loaded and decreased when a waiter gets its clone of the AsyncDropArc.
    /// If this is non-zero, then we shouldn't prune the entry yet even if the refcount is zero.
    num_unfulfilled_waiters: usize,
    /// If ImmediateDropRequest::Requested: While we're loading, another thread triggered an immediate drop request for this entry. Don't allow further loaders, and when this is unloaded, call the triggering thread's callback with exclusive access.
    immediate_drop_request: ImmediateDropRequest<V, D>,
}

impl<V, D> EntryStateLoaded<V, D>
where
    V: AsyncDrop + Debug + Send + Sync + 'static,
    D: Clone + Debug + Send + Sync + 'static,
{
    pub fn new_from_just_finished_loading(
        entry: AsyncDropGuard<V>,
        loading: EntryStateLoading<V, D>,
    ) -> Self {
        EntryStateLoaded {
            entry: AsyncDropArc::new(entry),
            num_unfulfilled_waiters: loading.num_waiters(),
            immediate_drop_request: loading.into_immediate_drop_request(),
        }
    }

    pub fn new_without_unfulfilled_waiters(entry: AsyncDropGuard<V>) -> Self {
        EntryStateLoaded {
            entry: AsyncDropArc::new(entry),
            num_unfulfilled_waiters: 0,
            immediate_drop_request: ImmediateDropRequest::NotRequested,
        }
    }

    pub fn get_entry(&self) -> AsyncDropGuard<AsyncDropArc<V>> {
        AsyncDropArc::clone(&self.entry)
    }

    pub(super) fn get_entry_and_decrease_num_unfulfilled_waiters(
        &mut self,
    ) -> AsyncDropGuard<AsyncDropArc<V>> {
        assert!(self.num_unfulfilled_waiters > 0);
        self.num_unfulfilled_waiters -= 1;
        AsyncDropArc::clone(&self.entry)
    }

    pub fn num_tasks_with_access(&self) -> usize {
        // num_unfulfilled_waiters are tasks that are waiting to get access to the entry, and will increment the refcount when they do.
        // We subtract 1 from the strong count because we don't want to count our self reference.
        self.num_unfulfilled_waiters + AsyncDropArc::strong_count(&self.entry) - 1
    }

    pub fn into_inner(self) -> (ImmediateDropRequest<V, D>, AsyncDropGuard<V>) {
        assert!(
            self.num_unfulfilled_waiters == 0,
            "Cannot consume EntryStateLoaded while there are unfulfilled waiters"
        );
        let entry = AsyncDropArc::into_inner(self.entry).unwrap();
        (self.immediate_drop_request, entry)
    }

    pub fn request_immediate_drop_if_not_yet_requested<F>(
        &mut self,
        drop_fn: impl FnOnce(Option<AsyncDropGuard<V>>) -> F + Send + Sync + 'static,
    ) -> ImmediateDropRequestResponse<D>
    where
        F: Future<Output = D> + Send,
    {
        self.immediate_drop_request
            .request_immediate_drop_if_not_yet_requested(drop_fn)
    }

    pub fn immediate_drop_requested(&self) -> Option<mr_oneshot_channel::Receiver<D>> {
        self.immediate_drop_request.immediate_drop_requested()
    }
}
