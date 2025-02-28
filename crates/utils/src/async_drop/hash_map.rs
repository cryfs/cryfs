use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use super::{AsyncDrop, AsyncDropGuard};
use crate::containers::{HashMapExt, OccupiedError};
use crate::stream::for_each_unordered;

/// A HashMap that can hold values with [AsyncDrop] semantics.
/// It makes sure values are dropped correctly whenever necessary.
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
    pub fn new() -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            map: HashMap::new(),
        })
    }

    pub fn try_insert(
        &mut self,
        key: K,
        value: AsyncDropGuard<V>,
    ) -> Result<&mut AsyncDropGuard<V>, OccupiedError<'_, K, AsyncDropGuard<V>>> {
        HashMapExt::try_insert(&mut self.map, key, value)
    }

    pub fn remove(&mut self, key: &K) -> Option<AsyncDropGuard<V>> {
        self.map.remove(key)
    }

    pub fn get(&self, key: &K) -> Option<&AsyncDropGuard<V>> {
        self.map.get(key)
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
