use std::fmt::Debug;

use futures::future::{BoxFuture, Shared};

use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    event::Event,
};

use crate::entry::loading::LoadingResult;

/// Intent to drop the current value, with optional reload afterwards.
/// Having a DropIntent means drop WILL happen.
pub struct DropIntent<V, E>
where
    V: AsyncDrop + Debug + Send + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    /// The function to call when dropping the value.
    /// This function is expected to drop the value and perform any cleanup.
    drop_fn: Box<dyn FnOnce(Option<AsyncDropGuard<V>>) -> BoxFuture<'static, ()> + Send + Sync>,

    /// Event that is triggered when the drop is complete.
    /// Other tasks can wait on this to know when the drop has finished.
    on_dropped: Event,

    /// Optional reload operation to perform after the drop completes.
    /// If Some, the entry will be reloaded after the drop.
    /// If None, the entry will be removed from the store.
    reload: Option<ReloadInfo<V, E>>,
}

impl<V, E> DropIntent<V, E>
where
    V: AsyncDrop + Debug + Send + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    /// Create a new DropIntent with a drop function and no reload.
    pub fn new<F>(
        drop_fn: impl FnOnce(Option<AsyncDropGuard<V>>) -> F + Send + Sync + 'static,
    ) -> (Self, Event)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let on_dropped = Event::new();
        let on_dropped_clone = on_dropped.clone();
        let drop_intent = Self {
            drop_fn: Box::new(move |value| Box::pin(drop_fn(value))),
            on_dropped,
            reload: None,
        };
        (drop_intent, on_dropped_clone)
    }

    /// Get a reference to the on_dropped event.
    pub fn on_dropped(&self) -> &Event {
        &self.on_dropped
    }

    /// Get a mutable reference to the reload info, if any.
    pub fn reload_mut(&mut self) -> Option<&mut ReloadInfo<V, E>> {
        self.reload.as_mut()
    }

    /// Set the reload info for this drop intent.
    pub fn set_reload(&mut self, reload: ReloadInfo<V, E>) {
        assert!(self.reload.is_none(), "Reload already set");
        self.reload = Some(reload);
    }

    /// Execute the drop function with the given value and trigger the on_dropped event.
    pub async fn execute_drop(self, value: Option<AsyncDropGuard<V>>) -> Option<ReloadInfo<V, E>> {
        (self.drop_fn)(value).await;
        self.on_dropped.trigger();
        self.reload
    }
}

impl<V, E> Debug for DropIntent<V, E>
where
    V: AsyncDrop + Debug + Send + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DropIntent")
            .field("reload", &self.reload.as_ref().map(|_| "Some(...)"))
            .finish_non_exhaustive()
    }
}

/// Info about a pending reload operation.
/// Note: next_drop_intent can recursively contain another reload, allowing unbounded depth.
pub struct ReloadInfo<V, E>
where
    V: AsyncDrop + Debug + Send + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    /// The shared future that will load the new value.
    /// This future waits for the preceding on_dropped event before loading.
    reload_future: Shared<BoxFuture<'static, LoadingResult<E>>>,

    /// Number of tasks waiting for this reload to complete.
    num_waiters: usize,

    /// DropIntent for the reloaded value (recursive).
    /// This allows unbounded nesting of drop/reload cycles.
    next_drop_intent: Option<Box<DropIntent<V, E>>>,
}

impl<V, E> ReloadInfo<V, E>
where
    V: AsyncDrop + Debug + Send + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    /// Create a new ReloadInfo with a reload future and one waiter.
    pub fn new(reload_future: Shared<BoxFuture<'static, LoadingResult<E>>>) -> Self {
        Self {
            reload_future,
            num_waiters: 1,
            next_drop_intent: None,
        }
    }

    /// Get the reload future.
    pub fn reload_future(&self) -> &Shared<BoxFuture<'static, LoadingResult<E>>> {
        &self.reload_future
    }

    /// Add a waiter and return the reload future.
    pub fn add_waiter(&mut self) -> Shared<BoxFuture<'static, LoadingResult<E>>> {
        self.num_waiters += 1;
        self.reload_future.clone()
    }

    /// Get a reference to the next drop intent, if any.
    pub fn next_drop_intent(&self) -> Option<&DropIntent<V, E>> {
        self.next_drop_intent.as_ref().map(|b| b.as_ref())
    }

    /// Get a mutable reference to the next drop intent, if any.
    pub fn next_drop_intent_mut(&mut self) -> Option<&mut DropIntent<V, E>> {
        self.next_drop_intent.as_mut().map(|b| b.as_mut())
    }

    /// Check if there's a deeper reload in the chain (drop intent with reload).
    /// Used to enable iterative chain walking without borrow conflicts.
    pub fn has_deeper_reload(&self) -> bool {
        self.next_drop_intent
            .as_ref()
            .is_some_and(|drop_intent| drop_intent.reload.is_some())
    }

    /// Check if there's a next drop intent (regardless of whether it has a reload).
    /// Used to enable iterative chain walking without borrow conflicts.
    pub fn has_next_drop_intent(&self) -> bool {
        self.next_drop_intent.is_some()
    }

    /// Set the next drop intent for this reload.
    pub fn set_next_drop_intent(&mut self, drop_intent: DropIntent<V, E>) {
        assert!(self.next_drop_intent.is_none(), "Next drop intent already set");
        self.next_drop_intent = Some(Box::new(drop_intent));
    }

    /// Consume this reload info and return its components.
    pub fn into_parts(
        self,
    ) -> (
        Shared<BoxFuture<'static, LoadingResult<E>>>,
        usize,
        Option<Box<DropIntent<V, E>>>,
    ) {
        (self.reload_future, self.num_waiters, self.next_drop_intent)
    }
}

impl<V, E> Debug for ReloadInfo<V, E>
where
    V: AsyncDrop + Debug + Send + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReloadInfo")
            .field("num_waiters", &self.num_waiters)
            .field("next_drop_intent", &self.next_drop_intent.as_ref().map(|_| "Some(...)"))
            .finish_non_exhaustive()
    }
}

/// Response from request_immediate_drop indicating what action was taken.
pub enum RequestImmediateDropResponse {
    /// The drop was successfully requested. The caller can wait on the event
    /// to know when the drop completes.
    Requested { on_dropped: Event },

    /// A drop is already in progress and there's no reload to attach to.
    /// The caller can wait on this future to know when the current drop completes.
    AlreadyDropping {
        on_current_drop_complete: Shared<BoxFuture<'static, ()>>,
    },
}
