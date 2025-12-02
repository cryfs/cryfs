use anyhow::Result;
use std::{fmt::Debug, hash::Hash, sync::Arc};

use crate::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard},
    concurrent_store::{ConcurrentStore, LoadedEntryGuard, entry::EntryLoadingWaiter},
    safe_panic, with_async_drop_2_infallible,
};

/// Represents the result of trying to get an entry from the store,
/// which may be in the process of loading, already loaded, or not found.
#[must_use]
pub struct LoadingOrLoaded<K, V, E>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    V: AsyncDrop + Debug + Send + Sync + 'static,
    E: Debug + Send + Sync + 'static,
{
    // Always Some except when being dropped
    inner: Option<LoadingOrLoadedInner<K, V, E>>,
}

enum LoadingOrLoadedInner<K, V, E>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    V: AsyncDrop + Debug + Send + Sync + 'static,
    E: Debug + Send + Sync + 'static,
{
    NotFound,
    Loading {
        waiter: EntryLoadingWaiter<K, E>,
        store: AsyncDropGuard<AsyncDropArc<ConcurrentStore<K, V, E>>>,
    },
    Loaded(AsyncDropGuard<LoadedEntryGuard<K, V, E>>),
}

impl<K, V, E> LoadingOrLoaded<K, V, E>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    V: AsyncDrop + Debug + Send + Sync + 'static,
    E: Debug + Send + Sync + 'static,
{
    pub(super) fn new_not_found() -> Self {
        Self {
            inner: Some(LoadingOrLoadedInner::NotFound),
        }
    }

    pub(super) fn new_loading(
        store: AsyncDropGuard<AsyncDropArc<ConcurrentStore<K, V, E>>>,
        waiter: EntryLoadingWaiter<K, E>,
    ) -> Self {
        Self {
            inner: Some(LoadingOrLoadedInner::Loading { store, waiter }),
        }
    }

    pub(super) fn new_loaded(loaded: AsyncDropGuard<LoadedEntryGuard<K, V, E>>) -> Self {
        Self {
            inner: Some(LoadingOrLoadedInner::Loaded(loaded)),
        }
    }

    pub async fn wait_until_loaded(
        mut self,
    ) -> Result<Option<AsyncDropGuard<LoadedEntryGuard<K, V, E>>>, Arc<E>> {
        match self.inner.take().expect("Already destructed") {
            LoadingOrLoadedInner::NotFound => Ok(None),
            LoadingOrLoadedInner::Loaded(loaded) => Ok(Some(loaded)),
            LoadingOrLoadedInner::Loading { store, waiter } => {
                with_async_drop_2_infallible!(store, { waiter.wait_until_loaded(&store).await })
            }
        }
    }
}

impl<K, V, E> Drop for LoadingOrLoaded<K, V, E>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    V: AsyncDrop + Debug + Send + Sync + 'static,
    E: Debug + Send + Sync + 'static,
{
    fn drop(&mut self) {
        if self.inner.is_some() {
            safe_panic!("LoadingOrLoaded was dropped without a call to wait_for_loaded.");
        }
    }
}
