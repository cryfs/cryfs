//! HashMap wrapper for types that implement [`AsyncDrop`].
//!
//! This module provides [`AsyncDropHashMap`], which manages a collection of values
//! that require async cleanup. When the map is dropped, all contained values are
//! async-dropped concurrently.

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use super::{AsyncDrop, AsyncDropGuard};
use crate::containers::{HashMapExt, OccupiedError};
use crate::stream::for_each_unordered;

/// A HashMap that holds values with [`AsyncDrop`] semantics.
///
/// This container ensures that all values are properly async-dropped when the
/// map itself is dropped. Values are dropped concurrently.
///
/// # Type Parameters
///
/// * `K` - The key type
/// * `V` - The value type, which must implement [`AsyncDrop`]
#[derive(Debug)]
pub struct AsyncDropHashMap<K, V>
where
    K: PartialEq + Eq + Hash + Debug + Send,
    V: AsyncDrop + Send + Debug,
    <V as AsyncDrop>::Error: Send,
{
    map: HashMap<K, AsyncDropGuard<V>>,
}

impl<K, V> AsyncDropHashMap<K, V>
where
    K: PartialEq + Eq + Hash + Debug + Send,
    V: AsyncDrop + Send + Debug,
    <V as AsyncDrop>::Error: Send,
{
    /// Creates a new empty `AsyncDropHashMap`.
    pub fn new() -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            map: HashMap::new(),
        })
    }

    /// Attempts to insert a key-value pair into the map.
    ///
    /// Returns an error if the key already exists.
    pub fn try_insert(
        &mut self,
        key: K,
        value: AsyncDropGuard<V>,
    ) -> Result<&mut AsyncDropGuard<V>, OccupiedError<'_, K, AsyncDropGuard<V>>> {
        HashMapExt::try_insert(&mut self.map, key, value)
    }

    /// Removes a key from the map, returning the value if it existed.
    ///
    /// The caller is responsible for calling `async_drop` on the returned value.
    pub fn remove(&mut self, key: &K) -> Option<AsyncDropGuard<V>> {
        self.map.remove(key)
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, key: &K) -> Option<&AsyncDropGuard<V>> {
        self.map.get(key)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut AsyncDropGuard<V>> {
        self.map.get_mut(key)
    }

    /// Returns the number of elements in the map.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Drains the map, returning an iterator over all key-value pairs.
    ///
    /// The caller is responsible for calling `async_drop` on each returned value.
    #[cfg(feature = "testutils")]
    pub fn drain(&mut self) -> impl Iterator<Item = (K, AsyncDropGuard<V>)> {
        self.map.drain()
    }

    /// Returns an iterator over the map's key-value pairs.
    #[cfg(feature = "testutils")]
    pub fn iter(&self) -> impl Iterator<Item = (&K, &AsyncDropGuard<V>)> {
        self.map.iter()
    }
}

#[async_trait]
impl<K, V> AsyncDrop for AsyncDropHashMap<K, V>
where
    K: PartialEq + Eq + Hash + Debug + Send,
    V: AsyncDrop + Send + Debug,
    <V as AsyncDrop>::Error: Send,
{
    type Error = <V as AsyncDrop>::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        let values = self.map.drain().map(|(_key, value)| value);
        for_each_unordered(values, async move |mut value| {
            value.async_drop().await?;
            Ok(())
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Debug)]
    struct TestValue {
        id: i32,
        drop_counter: Arc<AtomicUsize>,
    }

    impl TestValue {
        fn new(id: i32, drop_counter: Arc<AtomicUsize>) -> AsyncDropGuard<Self> {
            AsyncDropGuard::new(Self { id, drop_counter })
        }
    }

    #[async_trait]
    impl AsyncDrop for TestValue {
        type Error = &'static str;

        async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
            self.drop_counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_new_creates_empty_map() {
        let mut map: AsyncDropGuard<AsyncDropHashMap<i32, TestValue>> = AsyncDropHashMap::new();
        assert_eq!(0, map.len());
        map.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_try_insert_and_get() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut map = AsyncDropHashMap::new();

        let value = TestValue::new(42, Arc::clone(&counter));
        map.try_insert(1, value).unwrap();

        assert_eq!(1, map.len());
        assert!(map.get(&1).is_some());
        assert_eq!(42, map.get(&1).unwrap().id);

        map.async_drop().await.unwrap();
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_try_insert_duplicate_fails() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut map = AsyncDropHashMap::new();

        let value1 = TestValue::new(1, Arc::clone(&counter));
        let value2 = TestValue::new(2, Arc::clone(&counter));

        map.try_insert(1, value1).unwrap();
        let result = map.try_insert(1, value2);
        assert!(result.is_err());

        // The rejected value needs to be cleaned up manually
        let mut rejected = result.unwrap_err().value;
        rejected.async_drop().await.unwrap();

        map.async_drop().await.unwrap();
        assert_eq!(2, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_remove() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut map = AsyncDropHashMap::new();

        let value = TestValue::new(42, Arc::clone(&counter));
        map.try_insert(1, value).unwrap();

        let mut removed = map.remove(&1).unwrap();
        assert_eq!(42, removed.id);
        assert_eq!(0, map.len());

        removed.async_drop().await.unwrap();
        map.async_drop().await.unwrap();
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_async_drop_drops_all_values() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut map = AsyncDropHashMap::new();

        for i in 0..5 {
            let value = TestValue::new(i, Arc::clone(&counter));
            map.try_insert(i, value).unwrap();
        }

        assert_eq!(5, map.len());
        assert_eq!(0, counter.load(Ordering::SeqCst));

        map.async_drop().await.unwrap();
        assert_eq!(5, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_get_mut() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut map = AsyncDropHashMap::new();

        let value = TestValue::new(42, Arc::clone(&counter));
        map.try_insert(1, value).unwrap();

        // Modify through get_mut
        map.get_mut(&1).unwrap().id = 100;
        assert_eq!(100, map.get(&1).unwrap().id);

        map.async_drop().await.unwrap();
    }
}
