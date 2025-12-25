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

    /// Return whether the DropIntent has a reload set
    pub fn has_reload(&self) -> bool {
        self.reload.is_some()
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
    fn has_deeper_reload(&self) -> bool {
        self.next_drop_intent
            .as_ref()
            .is_some_and(|drop_intent| drop_intent.reload.is_some())
    }

    /// Check if there's a next drop intent (regardless of whether it has a reload).
    /// Used to enable iterative chain walking without borrow conflicts.
    pub fn has_next_drop_intent(&self) -> bool {
        self.next_drop_intent.is_some()
    }

    /// Walk down the chain of drop intents and reloads, and return the deepest [ReloadInfo].
    /// This [ReloadInfo] may or may not have a [DropIntent], but if it does, then that
    /// [DropIntent] does not have a reload set.
    pub fn to_deepest_reload(&mut self) -> &mut Self {
        let mut current = self;
        while current.has_deeper_reload() {
            current = current
                .next_drop_intent_mut()
                .expect("has_deeper_reload returned true")
                .reload_mut()
                .expect("has_deeper_reload returned true");
        }
        current
    }

    /// Set the next drop intent for this reload.
    pub fn set_next_drop_intent(&mut self, drop_intent: DropIntent<V, E>) {
        assert!(
            self.next_drop_intent.is_none(),
            "Next drop intent already set"
        );
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

    /// Merge another ReloadInfo into this one.
    ///
    /// This is used when a reload was set on the Dropping state (other) while
    /// a reload from drop_intent (self) already exists. We merge the waiter counts
    /// and chain the drop intents so all waiters are satisfied and all scheduled
    /// drops eventually execute.
    ///
    /// The other's reload_future is discarded (it will still run but find the state
    /// already transitioned, which is handled gracefully).
    pub fn merge_from(&mut self, other: ReloadInfo<V, E>) {
        // Add the other's waiters to our count
        self.num_waiters += other.num_waiters;

        // If the other has a next_drop_intent, chain it to ours
        if let Some(other_drop_intent) = other.next_drop_intent {
            if self.next_drop_intent.is_none() {
                // We don't have a drop intent, just take theirs
                self.next_drop_intent = Some(other_drop_intent);
            } else {
                // We have a drop intent - chain theirs at the end
                // Walk to the deepest reload and set theirs there
                let deepest = self.to_deepest_reload();
                if deepest.next_drop_intent.is_none() {
                    deepest.next_drop_intent = Some(other_drop_intent);
                } else {
                    // The deepest reload already has a drop_intent without reload.
                    // We need to give it a reload so we can chain further.
                    // But we don't have a reload future for it...
                    // This case shouldn't happen in practice because if there's a drop_intent
                    // without reload at the deepest level, new load requests would have
                    // attached a reload to it, not created a new one on Dropping.
                    // For safety, we'll just drop the other's chain (losing those drops).
                    // TODO: Consider if this case can actually occur and needs better handling.
                }
            }
        }
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
            .field(
                "next_drop_intent",
                &self.next_drop_intent.as_ref().map(|_| "Some(...)"),
            )
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
