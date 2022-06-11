use anyhow::{ensure, Result};
use async_trait::async_trait;
use clru::CLruCache;
use futures::future::join_all;
use std::fmt::Debug;
use std::future::Future;
use std::sync::Mutex;
use std::hash::Hash;
use std::num::NonZeroUsize;
use log::warn;

// TODO This isn't complete yet. Look at Cache.h and add features (e.g. making space before each push, periodic evicting, ...)

#[async_trait]
pub trait Cache<Key, Value> {
    type OnEvictFn;

    fn new(capacity: NonZeroUsize, on_evict_fn: Self::OnEvictFn) -> Self;

    fn num_filled_entries(&self) -> usize;
    fn push(&mut self, key: Key, value: Value) -> Result<()>;
    fn pop(&mut self, key: &Key) -> Option<Value>;
}

#[async_trait]
pub trait EvictionCallback<Key, Value> {
    // TODO Can we avoid the Box around the future?
    async fn on_evict(&self, k: Key, v: Value) -> Result<()>;
}

pub struct LRUCache<Key, Value, OnEvictFn>
where
    OnEvictFn: EvictionCallback<Key, Value>,
    Key: Debug + Hash + Eq,
{
    cache: CLruCache<Key, Value>,
    on_evict_fn: OnEvictFn,
}

#[async_trait]
impl<Key, Value, OnEvictFn> Cache<Key, Value> for LRUCache<Key, Value, OnEvictFn>
where
    OnEvictFn: EvictionCallback<Key, Value>,
    Key: Debug + Hash + Eq,
{
    type OnEvictFn = OnEvictFn;

    fn new(capacity: NonZeroUsize, on_evict_fn: OnEvictFn) -> Self {
        Self {
            cache: CLruCache::new(capacity),
            on_evict_fn,
        }
    }

    fn num_filled_entries(&self) -> usize {
        self.cache.len()
    }

    fn push(&mut self, key: Key, value: Value) -> Result<()> {
        ensure!(!self.cache.contains(&key), "Tried to insert key {:?} into cache but it already exists. This can only be a race condition between different actions trying to modify the same block.", key);
        let insert_result = self.cache.put(key, value);
        assert!(
            insert_result.is_none(),
            "This can't happen because we just checked above that it doesn't exist"
        );
        Ok(())
    }

    fn pop(&mut self, key: &Key) -> Option<Value> {
        self.cache.pop(key)
    }
}

impl<Key, Value, OnEvictFn> LRUCache<Key, Value, OnEvictFn>
where
    OnEvictFn: EvictionCallback<Key, Value>,
    Key: Debug + Hash + Eq,
{
    async fn evict_all(&mut self) -> Result<()> {
        let cache = self._take_cache();
        // Use join_all and not try_join_all, because we don't want to cancel
        // other futures if one fails.
        let results: Vec<Result<()>> = join_all(
            cache
                .into_iter()
                .map(|(key, value)| self.on_evict_fn.on_evict(key, value)),
        )
        .await;
        let results: Result<Vec<()>> = results.into_iter().collect();
        results?;
        Ok(())
    }

    fn _take_cache(&mut self) -> CLruCache<Key, Value> {
        let mut cache = CLruCache::new(NonZeroUsize::new(self.cache.capacity()).expect(
            "This can't happen because the previous cache was initialized with NonZeroUsize",
        ));
        std::mem::swap(&mut self.cache, &mut cache);
        cache
    }
}

impl<Key, Value, OnEvictFn> Drop for LRUCache<Key, Value, OnEvictFn>
where
    Key: Debug + Hash + Eq,
    OnEvictFn: EvictionCallback<Key, Value>,
{
    fn drop(&mut self) {
        match tokio::runtime::Handle::try_current() {
            Ok(runtime) => {
                // Handle::block_on can't drive io or timers. Only Runtime::block_on can drive them, see https://docs.rs/tokio/1.5.0/tokio/runtime/struct.Handle.html#method.block_on
                // This might deadlock if there isn't another thread doing Runtime::block_on().
                runtime
                    .block_on(self.evict_all())
                    .unwrap();
            }
            Err(err) => {
                warn!("Called Cache::drop from outside of a tokio runtime. Starting our own runtime.");
                let runtime = tokio::runtime::Runtime::new().unwrap();
                runtime
                    .block_on(self.evict_all())
                    .unwrap();
            }
        };
    }
}
