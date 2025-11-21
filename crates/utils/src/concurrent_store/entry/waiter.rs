use anyhow::Result;
use futures::future::{BoxFuture, Shared};
use std::fmt::Debug;
use std::hash::Hash;

use crate::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard},
    concurrent_store::{
        entry::{EntryState, loading::LoadingResult},
        guard::LoadedEntryGuard,
        store::ConcurrentStore,
    },
    safe_panic,
};

/// Handle for a task waiting for an entry to be loaded.
/// This can be redeemed against the entry once loading is completed.
/// It is an RAII type that ensures that the number of waiters is correctly tracked.
#[must_use]
pub struct EntryLoadingWaiter {
    // Alway Some unless destructed
    loading_result: Option<Shared<BoxFuture<'static, LoadingResult>>>,
}

impl EntryLoadingWaiter {
    pub fn new(loading_result: Shared<BoxFuture<'static, LoadingResult>>) -> Self {
        EntryLoadingWaiter {
            loading_result: Some(loading_result),
        }
    }

    /// Wait until the entry is loaded, and return a guard for the loaded entry.
    /// If the entry was not found, return None.
    /// If an error occurred while loading, return the error.
    pub async fn wait_until_loaded<K, V, D>(
        mut self,
        store: &AsyncDropGuard<AsyncDropArc<ConcurrentStore<K, V, D>>>,
        key: K,
    ) -> Result<Option<AsyncDropGuard<LoadedEntryGuard<K, V, D>>>>
    where
        K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
        V: AsyncDrop + Debug + Send + Sync + 'static,
        D: Clone + Debug + Send + Sync + 'static,
    {
        match self.loading_result.take().expect("Already dropped").await {
            LoadingResult::Loaded => {
                // _finalize_waiter will decrement the num_waiters refcount
                Ok(Some(Self::_finalize_waiter(store, key)))
            }
            LoadingResult::NotFound => {
                // No need to decrement the num_waiters refcount here because the entry never made it to the Loaded state
                Ok(None)
            }
            LoadingResult::Error(err) => {
                // No need to decrement the num_waiters refcount here because the entry never made it to the Loaded state
                Err(anyhow::anyhow!(
                    "Error while try_insert'ing entry with key {key:?}: {err}",
                ))
            }
        }
    }

    fn _finalize_waiter<K, V, D>(
        store: &AsyncDropGuard<AsyncDropArc<ConcurrentStore<K, V, D>>>,
        key: K,
    ) -> AsyncDropGuard<LoadedEntryGuard<K, V, D>>
    where
        K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
        V: AsyncDrop + Debug + Send + Sync + 'static,
        D: Clone + Debug + Send + Sync + 'static,
    {
        // This is not a race condition with dropping, i.e. the entry can't be in dropping state yet, because we are an "unfulfilled waiter",
        // i.e. the entry cannot be dropped until we decrease the count below.
        let mut entries = store.entries.lock().unwrap();
        let Some(state) = entries.get_mut(&key) else {
            panic!("Entry with key {:?} was not found in the map", key);
        };
        let EntryState::Loaded(loaded) = state else {
            panic!("Entry with key {:?} is not in loaded state", key);
        };
        LoadedEntryGuard::new(
            AsyncDropArc::clone(store),
            key,
            // [Self::_clone_or_create_entry_state] added a waiter, so we need to decrement num_unfulfilled_waiters.
            loaded.get_entry_and_decrease_num_unfulfilled_waiters(),
        )
    }
}

impl Drop for EntryLoadingWaiter {
    fn drop(&mut self) {
        if self.loading_result.is_some() {
            safe_panic!(
                "EntryLoadingWaiter was dropped without being awaited. This will lead to a memory leak because the number of waiters will not be decremented."
            );
        }
    }
}
