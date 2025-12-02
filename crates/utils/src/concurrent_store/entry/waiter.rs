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
pub struct EntryLoadingWaiter<K, E>
where
    K: Hash + Eq + Clone + Debug + Send + Sync,
    E: Clone + Debug + Send + Sync + 'static,
{
    // Alway Some unless destructed
    inner: Option<EntryLoadingWaiterInner<K, E>>,
}

struct EntryLoadingWaiterInner<K, E>
where
    K: Hash + Eq + Clone + Debug + Send + Sync,
    E: Clone + Debug + Send + Sync + 'static,
{
    key: K,
    loading_result: Shared<BoxFuture<'static, LoadingResult<E>>>,
}

impl<K, E> EntryLoadingWaiter<K, E>
where
    K: Hash + Eq + Clone + Debug + Send + Sync,
    E: Clone + Debug + Send + Sync + 'static,
{
    pub fn new(key: K, loading_result: Shared<BoxFuture<'static, LoadingResult<E>>>) -> Self {
        EntryLoadingWaiter {
            inner: Some(EntryLoadingWaiterInner {
                key,
                loading_result: loading_result,
            }),
        }
    }

    /// Wait until the entry is loaded, and return a guard for the loaded entry.
    /// If the entry was not found, return None.
    /// If an error occurred while loading, return the error.
    pub async fn wait_until_loaded<V>(
        mut self,
        store: &AsyncDropGuard<AsyncDropArc<ConcurrentStore<K, V, E>>>,
    ) -> Result<Option<AsyncDropGuard<LoadedEntryGuard<K, V, E>>>, E>
    where
        V: AsyncDrop + Debug + Send + Sync,
    {
        let inner = self.inner.take().expect("Already awaited");
        match inner.loading_result.await {
            LoadingResult::Loaded => {
                // _finalize_waiter will decrement the num_waiters refcount
                Ok(Some(Self::_finalize_waiter(store, inner.key)))
            }
            LoadingResult::NotFound => {
                // No need to decrement the num_waiters refcount here because the entry never made it to the Loaded state
                Ok(None)
            }
            LoadingResult::Error(err) => {
                // No need to decrement the num_waiters refcount here because the entry never made it to the Loaded state
                Err(err)
            }
        }
    }

    fn _finalize_waiter<V>(
        store: &AsyncDropGuard<AsyncDropArc<ConcurrentStore<K, V, E>>>,
        key: K,
    ) -> AsyncDropGuard<LoadedEntryGuard<K, V, E>>
    where
        V: AsyncDrop + Debug + Send + Sync,
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

impl<K, E> Drop for EntryLoadingWaiter<K, E>
where
    K: Hash + Eq + Clone + Debug + Send + Sync,
    E: Clone + Debug + Send + Sync + 'static,
{
    fn drop(&mut self) {
        if self.inner.is_some() {
            safe_panic!(
                "EntryLoadingWaiter was dropped without being awaited. This will lead to a memory leak because the number of waiters will not be decremented."
            );
        }
    }
}
