use async_trait::async_trait;
use lockable::Never;
use std::fmt::Debug;
use std::hash::Hash;

use crate::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard},
    concurrent_store::store::ConcurrentStore,
};

/// Guard for a loaded entry in a ConcurrentStore.
/// This ensures that the entry remains loaded while the guard is held,
/// and unloads the entry when the last guard for a key is dropped.
#[derive(Debug)]
pub struct LoadedEntryGuard<K, V>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    V: AsyncDrop + Debug + Send + Sync + 'static,
{
    store: AsyncDropGuard<AsyncDropArc<ConcurrentStore<K, V>>>,
    key: K,
    value: AsyncDropGuard<AsyncDropArc<V>>,
}

impl<K, V> LoadedEntryGuard<K, V>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    V: AsyncDrop + Debug + Send + Sync + 'static,
{
    pub(super) fn new(
        store: AsyncDropGuard<AsyncDropArc<ConcurrentStore<K, V>>>,
        key: K,
        value: AsyncDropGuard<AsyncDropArc<V>>,
    ) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self { store, key, value })
    }

    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn value(&self) -> &AsyncDropGuard<AsyncDropArc<V>> {
        &self.value
    }

    pub fn store(&self) -> &AsyncDropGuard<AsyncDropArc<ConcurrentStore<K, V>>> {
        &self.store
    }
}

#[async_trait]
impl<K, V> AsyncDrop for LoadedEntryGuard<K, V>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    V: AsyncDrop + Debug + Send + Sync + 'static,
{
    type Error = Never;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        let value = std::mem::replace(&mut self.value, AsyncDropGuard::new_invalid());
        self.store.unload(self.key.clone(), value).await;
        self.store.async_drop().await.unwrap(); // TODO No unwrap
        Ok(())
    }
}
