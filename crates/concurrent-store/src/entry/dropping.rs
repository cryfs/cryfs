use std::fmt::Debug;

use futures::{
    FutureExt as _,
    future::{BoxFuture, Shared},
};

use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    event::Event,
};

use crate::entry::intent::{Intent, ReloadInfo, RequestImmediateDropResponse};

/// Represents an entry that is currently being dropped (async drop in progress).
pub struct EntryStateDropping<V, E>
where
    V: AsyncDrop + Debug + Send + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    /// The shared future representing the ongoing drop operation.
    future: Shared<BoxFuture<'static, ()>>,

    /// Event that is triggered when the drop completes.
    /// This is used for AlreadyDropping responses.
    on_dropped: Event,

    /// Optional reload operation to perform after the drop completes.
    /// If Some, the entry will be reloaded after the drop.
    /// If None, the entry will be removed from the store.
    reload: Option<ReloadInfo<V, E>>,
}

impl<V, E> EntryStateDropping<V, E>
where
    V: AsyncDrop + Debug + Send + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    /// Create a new Dropping state with a drop future.
    /// The on_dropped event is created internally and triggered when the future completes.
    pub fn new(drop_future: BoxFuture<'static, ()>) -> Self {
        let on_dropped = Event::new();
        let on_dropped_clone = on_dropped.clone();
        let future = async move {
            drop_future.await;
            on_dropped_clone.trigger();
        }
        .boxed()
        .shared();

        Self {
            future,
            on_dropped,
            reload: None,
        }
    }

    pub fn new_dummy() -> Self {
        Self::new(futures::future::ready(()).boxed())
    }

    /// Get the drop future.
    pub fn future(&self) -> &Shared<BoxFuture<'static, ()>> {
        &self.future
    }

    /// Consume this state and return the future.
    pub fn into_future(self) -> Shared<BoxFuture<'static, ()>> {
        self.future
    }

    /// Get a reference to the on_dropped event.
    pub fn on_dropped(&self) -> &Event {
        &self.on_dropped
    }

    /// Check if a reload is pending.
    pub fn has_reload(&self) -> bool {
        self.reload.is_some()
    }

    /// Get a reference to the reload info, if any.
    pub fn reload(&self) -> Option<&ReloadInfo<V, E>> {
        self.reload.as_ref()
    }

    /// Get a mutable reference to the reload info, if any.
    pub fn reload_mut(&mut self) -> Option<&mut ReloadInfo<V, E>> {
        self.reload.as_mut()
    }

    /// Set the reload info for this dropping state.
    pub fn set_reload(&mut self, reload: ReloadInfo<V, E>) {
        assert!(self.reload.is_none(), "Reload already set");
        self.reload = Some(reload);
    }

    /// Take the reload info, if any.
    pub fn take_reload(&mut self) -> Option<ReloadInfo<V, E>> {
        self.reload.take()
    }

    /// Request immediate drop. Since the entry is already being dropped,
    /// this walks the reload chain to find where to attach a new intent.
    pub fn request_immediate_drop<F>(
        &mut self,
        drop_fn: impl FnOnce(Option<AsyncDropGuard<V>>) -> F + Send + Sync + 'static,
    ) -> RequestImmediateDropResponse
    where
        F: Future<Output = ()> + Send + 'static,
    {
        match &mut self.reload {
            None => {
                // No reload pending - can't attach, drop already in progress
                let on_dropped = self.on_dropped.clone();
                RequestImmediateDropResponse::AlreadyDropping {
                    on_current_drop_complete: async move { on_dropped.wait().await }
                        .boxed()
                        .shared(),
                }
            }
            Some(reload) => {
                // Has reload - walk the chain
                Self::walk_reload_chain_for_drop(reload, drop_fn)
            }
        }
    }

    /// Walk the reload chain to find where to set a new intent.
    fn walk_reload_chain_for_drop<F>(
        reload: &mut ReloadInfo<V, E>,
        drop_fn: impl FnOnce(Option<AsyncDropGuard<V>>) -> F + Send + Sync + 'static,
    ) -> RequestImmediateDropResponse
    where
        F: Future<Output = ()> + Send + 'static,
    {
        match reload.new_intent_mut() {
            None => {
                // No new intent - set it here
                let (new_intent, on_dropped) = Intent::new(drop_fn);
                reload.set_new_intent(new_intent);
                RequestImmediateDropResponse::Requested { on_dropped }
            }
            Some(new_intent) => {
                // Has new intent - check if it has a reload
                match new_intent.reload_mut() {
                    None => {
                        // No reload in new_intent - can't attach, drop already pending
                        let on_dropped = new_intent.on_dropped().clone();
                        RequestImmediateDropResponse::AlreadyDropping {
                            on_current_drop_complete: async move { on_dropped.wait().await }
                                .boxed()
                                .shared(),
                        }
                    }
                    Some(nested_reload) => {
                        // Has nested reload - recurse
                        Self::walk_reload_chain_for_drop(nested_reload, drop_fn)
                    }
                }
            }
        }
    }

    /// Check if immediate drop was requested for this entry (via reload chain).
    pub fn immediate_drop_requested(&self) -> Option<&Event> {
        // Check if there's any intent in the reload chain
        self.reload
            .as_ref()
            .and_then(|r| r.new_intent())
            .map(|i| i.on_dropped())
    }
}

impl<V, E> Debug for EntryStateDropping<V, E>
where
    V: AsyncDrop + Debug + Send + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntryStateDropping")
            .field("reload", &self.reload)
            .finish_non_exhaustive()
    }
}
