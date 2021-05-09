use anyhow::Result;
use lru::LruCache;
use std::hash::Hash;

pub struct Cache<Key, Value, OnEvictFn>
where
    OnEvictFn: Fn(Key, Value),
    Key: Hash + Eq,
{
    cache: LruCache<Key, Value>,
    on_evict_fn: OnEvictFn,
}

impl<Key, Value, OnEvictFn> Cache<Key, Value, OnEvictFn>
where
    OnEvictFn: Fn(Key, Value),
    Key: Hash + Eq,
{
    pub fn new(capacity: usize, on_evict_fn: OnEvictFn) -> Self {
        Self {
            cache: LruCache::new(capacity),
            on_evict_fn,
        }
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn push(key: Key, value: Value) -> Result<()> {
        todo!()
    }

    pub fn pop(key: &Key) -> Option<Value> {
        todo!()
    }

    pub fn flush() {
        todo!()
    }
}
