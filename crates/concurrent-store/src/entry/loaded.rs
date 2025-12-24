use std::fmt::Debug;

use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard},
    event::Event,
};

use crate::entry::{
    intent::{Intent, RequestImmediateDropResponse},
    loading::EntryStateLoading,
};

pub struct EntryStateLoaded<V, E>
where
    V: AsyncDrop + Debug + Send + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    entry: AsyncDropGuard<AsyncDropArc<V>>,

    /// Number of tasks that started waiting for this entry when it was in [super::EntryState::Loading],
    /// but haven't yet incremented the refcount of [Self::entry].
    /// This gets never increased, only initialized when the entry is loaded and decreased when a waiter gets its clone of the AsyncDropArc.
    /// If this is non-zero, then we shouldn't prune the entry yet even if the refcount is zero.
    num_unfulfilled_waiters: usize,

    /// Intent to drop the value, with optional reload.
    /// If Some, when all guards are released the value will be dropped using the intent's drop_fn.
    intent: Option<Intent<V, E>>,
}

impl<V, E> EntryStateLoaded<V, E>
where
    V: AsyncDrop + Debug + Send + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    pub fn new_from_just_finished_loading(
        entry: AsyncDropGuard<V>,
        loading: EntryStateLoading<V, E>,
    ) -> Self {
        EntryStateLoaded {
            entry: AsyncDropArc::new(entry),
            num_unfulfilled_waiters: loading.num_waiters(),
            intent: loading.into_intent(),
        }
    }

    pub fn new_without_unfulfilled_waiters(entry: AsyncDropGuard<V>) -> Self {
        EntryStateLoaded {
            entry: AsyncDropArc::new(entry),
            num_unfulfilled_waiters: 0,
            intent: None,
        }
    }

    /// Create a new Loaded state from reload info (after a drop completes and reload finishes).
    pub fn new_from_reload(
        entry: AsyncDropGuard<V>,
        num_unfulfilled_waiters: usize,
        intent: Option<Intent<V, E>>,
    ) -> Self {
        EntryStateLoaded {
            entry: AsyncDropArc::new(entry),
            num_unfulfilled_waiters,
            intent,
        }
    }

    pub fn get_entry(&self) -> AsyncDropGuard<AsyncDropArc<V>> {
        AsyncDropArc::clone(&self.entry)
    }

    pub fn get_entry_and_decrease_num_unfulfilled_waiters(
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

    /// Consume this state and return the intent (if any) and the entry.
    pub fn into_inner(self) -> (Option<Intent<V, E>>, AsyncDropGuard<V>) {
        assert!(
            self.num_unfulfilled_waiters == 0,
            "Cannot consume EntryStateLoaded while there are unfulfilled waiters"
        );
        let entry = AsyncDropArc::into_inner(self.entry).unwrap();
        (self.intent, entry)
    }

    /// Get a mutable reference to the intent, if any.
    pub fn intent_mut(&mut self) -> Option<&mut Intent<V, E>> {
        self.intent.as_mut()
    }

    /// Set an intent (drop request) for this loaded entry.
    /// Returns the on_dropped event that will be triggered when the drop completes.
    pub fn set_intent<F>(
        &mut self,
        drop_fn: impl FnOnce(Option<AsyncDropGuard<V>>) -> F + Send + Sync + 'static,
    ) -> Event
    where
        F: Future<Output = ()> + Send + 'static,
    {
        assert!(self.intent.is_none(), "Intent already set");
        let (intent, on_dropped) = Intent::new(drop_fn);
        self.intent = Some(intent);
        on_dropped
    }

    /// Request immediate drop. Walks the intent/reload chain to find the deepest level
    /// and either sets a new intent or returns AlreadyDropping.
    pub fn request_immediate_drop<F>(
        &mut self,
        drop_fn: impl FnOnce(Option<AsyncDropGuard<V>>) -> F + Send + Sync + 'static,
    ) -> RequestImmediateDropResponse
    where
        F: Future<Output = ()> + Send + 'static,
    {
        match &mut self.intent {
            None => {
                // No intent - set one
                let on_dropped = self.set_intent(drop_fn);
                RequestImmediateDropResponse::Requested { on_dropped }
            }
            Some(intent) => {
                // Walk the chain to find where to attach
                Self::walk_intent_chain_for_drop(intent, drop_fn)
            }
        }
    }

    /// Walk the intent/reload chain to find where to set a new intent.
    fn walk_intent_chain_for_drop<F>(
        intent: &mut Intent<V, E>,
        drop_fn: impl FnOnce(Option<AsyncDropGuard<V>>) -> F + Send + Sync + 'static,
    ) -> RequestImmediateDropResponse
    where
        F: Future<Output = ()> + Send + 'static,
    {
        use futures::FutureExt as _;

        match intent.reload_mut() {
            None => {
                // No reload pending - can't attach, drop already pending
                let on_dropped = intent.on_dropped().clone();
                RequestImmediateDropResponse::AlreadyDropping {
                    on_current_drop_complete: async move { on_dropped.wait().await }
                        .boxed()
                        .shared(),
                }
            }
            Some(reload) => {
                // Has reload - check new_intent
                match reload.new_intent_mut() {
                    None => {
                        // No new intent - set it here
                        let (new_intent, on_dropped) = Intent::new(drop_fn);
                        reload.set_new_intent(new_intent);
                        RequestImmediateDropResponse::Requested { on_dropped }
                    }
                    Some(new_intent) => {
                        // Has new intent - recurse
                        Self::walk_intent_chain_for_drop(new_intent, drop_fn)
                    }
                }
            }
        }
    }

    /// Check if immediate drop was requested for this entry.
    pub fn immediate_drop_requested(&self) -> Option<&Event> {
        self.intent.as_ref().map(|i| i.on_dropped())
    }
}

impl<V, E> Debug for EntryStateLoaded<V, E>
where
    V: AsyncDrop + Debug + Send + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntryStateLoaded")
            .field("entry", &self.entry)
            .field("num_unfulfilled_waiters", &self.num_unfulfilled_waiters)
            .field("intent", &self.intent)
            .finish()
    }
}
