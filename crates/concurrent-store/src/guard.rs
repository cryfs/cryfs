use async_trait::async_trait;
use lockable::Never;
use std::fmt::Debug;
use std::hash::Hash;

use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};

use crate::{RequestImmediateDropResult, store::ConcurrentStoreInner};

/// Guard for a loaded entry in a ConcurrentStore.
/// This ensures that the entry remains loaded while the guard is held,
/// and unloads the entry when the last guard for a key is dropped.
#[derive(Debug)]
pub struct LoadedEntryGuard<K, V, E>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    V: AsyncDrop + Debug + Send + Sync + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    store: AsyncDropGuard<AsyncDropArc<ConcurrentStoreInner<K, V, E>>>,
    key: K,
    value: AsyncDropGuard<AsyncDropArc<V>>,
}

impl<K, V, E> LoadedEntryGuard<K, V, E>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    V: AsyncDrop + Debug + Send + Sync + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    pub(super) fn new(
        store: AsyncDropGuard<AsyncDropArc<ConcurrentStoreInner<K, V, E>>>,
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

    pub fn request_immediate_drop<D, F>(
        &self,
        drop_fn: impl FnOnce(Option<AsyncDropGuard<V>>) -> F + Send + Sync + 'static,
    ) -> RequestImmediateDropResult<D>
    where
        D: Debug + Send + 'static,
        F: Future<Output = D> + Send + 'static,
    {
        ConcurrentStoreInner::request_immediate_drop(&self.store, self.key.clone(), drop_fn)
    }
}

#[async_trait]
impl<K, V, E> AsyncDrop for LoadedEntryGuard<K, V, E>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    V: AsyncDrop + Debug + Send + Sync + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    type Error = Never;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        let value = std::mem::replace(&mut self.value, AsyncDropGuard::new_invalid());
        ConcurrentStoreInner::unload(&self.store, self.key.clone(), value).await;
        self.store.async_drop().await.unwrap(); // TODO No unwrap
        Ok(())
    }
}
