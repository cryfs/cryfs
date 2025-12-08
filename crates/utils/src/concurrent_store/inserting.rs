use anyhow::Result;
use std::{fmt::Debug, hash::Hash};

use crate::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard},
    concurrent_store::{LoadedEntryGuard, entry::EntryLoadingWaiter, store::ConcurrentStoreInner},
    safe_panic, with_async_drop_2_infallible,
};

/// Represents a newly inserted entry that is currently in the process of being inserted.
#[must_use]
pub struct Inserting<K, V, E>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    V: AsyncDrop + Debug + Send + Sync + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    // Always Some, except for during destruction
    inner: Option<InsertingInner<K, V, E>>,
}

struct InsertingInner<K, V, E>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    V: AsyncDrop + Debug + Send + Sync + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    store: AsyncDropGuard<AsyncDropArc<ConcurrentStoreInner<K, V, E>>>,

    // Invariant: The entry is marked as "loading", which is why we use [EntryLoadingWaiter],
    // but our loading function never returns a None. It always returns a Some. So we don't need to deal with the
    // None case of EntryLoadingWaiter.
    waiter: EntryLoadingWaiter<K, E>,
}

impl<K, V, E> Inserting<K, V, E>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    V: AsyncDrop + Debug + Send + Sync + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    pub(super) fn new(
        store: AsyncDropGuard<AsyncDropArc<ConcurrentStoreInner<K, V, E>>>,
        waiter: EntryLoadingWaiter<K, E>,
    ) -> Self {
        Inserting {
            inner: Some(InsertingInner { store, waiter }),
        }
    }

    /// Wait until the entry is loaded, and return a guard for the loaded entry.
    /// If the entry was not found, return None.
    /// If an error occurred while loading, return the error.
    pub async fn wait_until_inserted(
        mut self,
    ) -> Result<AsyncDropGuard<LoadedEntryGuard<K, V, E>>, E>
    where
        K: Hash + Eq + Clone + Debug + Send + Sync,
        V: AsyncDrop + Debug + Send + Sync,
    {
        let InsertingInner { waiter, store } = self.inner.take().expect("Already destructed");
        with_async_drop_2_infallible!(store, {
            Ok(waiter.wait_until_loaded(&store).await?.expect(
                "Invariant violated: Inserting should never return None from the loading function",
            ))
        })
    }
}
impl<K, V, E> Drop for Inserting<K, V, E>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    V: AsyncDrop + Debug + Send + Sync + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    fn drop(&mut self) {
        if self.inner.is_some() {
            safe_panic!("Inserting was dropped without a call to wait_until_inserted.");
        }
    }
}
