use std::fmt::Debug;
use std::hash::Hash;

use futures::future::{BoxFuture, Shared};

use cryfs_utils::{async_drop::AsyncDrop, event::Event};

use crate::entry::{
    intent::{Intent, ReloadInfo, RequestImmediateDropResponse},
    waiter::EntryLoadingWaiter,
};

#[derive(Debug)]
pub struct EntryStateLoading<V, E>
where
    V: AsyncDrop + Debug + Send + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    /// loading_result is a future that will hold the result of the loading operation once it is complete.
    /// See [LoadingResult] for an explanation of the possible results.
    loading_result: Shared<BoxFuture<'static, LoadingResult<E>>>,

    /// Number of tasks currently waiting for this entry to be loaded.
    /// This is only ever incremented. Even if a waiter completes, it won't be decremented.
    num_waiters: usize,

    /// Intent to drop the value after loading completes, with optional reload.
    /// If Some, when loading completes the value will be dropped using the intent's drop_fn.
    intent: Option<Intent<V, E>>,
}

pub enum LoadingResult<E> {
    /// The entry was successfully loaded. This loading result means the entry state was already
    /// changed to [super::EntryState::Loaded] and can be accessed immediately.
    Loaded,

    /// The entry was not found. The entry was removed from the map.
    NotFound,

    /// An error occurred while loading the entry. The entry state was removed from the map.
    Error(E),
}

impl<E> Clone for LoadingResult<E>
where
    E: Clone + Debug + Send + Sync,
{
    fn clone(&self) -> Self {
        match self {
            LoadingResult::Loaded => LoadingResult::Loaded,
            LoadingResult::NotFound => LoadingResult::NotFound,
            LoadingResult::Error(err) => LoadingResult::Error(err.clone()),
        }
    }
}

impl<V, E> EntryStateLoading<V, E>
where
    V: AsyncDrop + Debug + Send + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    pub fn new(loading_result: BoxFuture<'static, LoadingResult<E>>) -> Self {
        use futures::FutureExt as _;
        EntryStateLoading {
            loading_result: loading_result.shared(),
            num_waiters: 0,
            intent: None,
        }
    }

    /// Create a new Loading state from reload info (after a drop completes).
    pub fn new_from_reload(reload: ReloadInfo<V, E>) -> Self {
        let (reload_future, num_waiters, new_intent) = reload.into_parts();
        EntryStateLoading {
            loading_result: reload_future,
            num_waiters,
            intent: new_intent.map(|b| *b),
        }
    }

    pub fn new_dummy() -> Self {
        use futures::FutureExt as _;
        EntryStateLoading {
            loading_result: futures::future::pending().boxed().shared(),
            num_waiters: 0,
            intent: None,
        }
    }

    pub fn add_waiter<K>(&mut self, key: K) -> EntryLoadingWaiter<K, E>
    where
        K: Hash + Eq + Clone + Debug + Send + Sync,
    {
        self.num_waiters += 1;
        EntryLoadingWaiter::new(key, self.loading_result.clone())
    }

    pub fn num_waiters(&self) -> usize {
        self.num_waiters
    }

    /// Check if an intent (drop request) exists.
    pub fn has_intent(&self) -> bool {
        self.intent.is_some()
    }

    /// Get a mutable reference to the intent, if any.
    pub fn intent_mut(&mut self) -> Option<&mut Intent<V, E>> {
        self.intent.as_mut()
    }

    /// Set an intent (drop request) for this loading entry.
    /// Returns the on_dropped event that will be triggered when the drop completes.
    pub fn set_intent<F>(
        &mut self,
        drop_fn: impl FnOnce(Option<cryfs_utils::async_drop::AsyncDropGuard<V>>) -> F
        + Send
        + Sync
        + 'static,
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
        drop_fn: impl FnOnce(Option<cryfs_utils::async_drop::AsyncDropGuard<V>>) -> F
        + Send
        + Sync
        + 'static,
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
        drop_fn: impl FnOnce(Option<cryfs_utils::async_drop::AsyncDropGuard<V>>) -> F
        + Send
        + Sync
        + 'static,
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

    /// Consume this state and return the intent (if any).
    pub fn into_intent(self) -> Option<Intent<V, E>> {
        self.intent
    }
}
