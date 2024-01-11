use anyhow::{ensure, Result};
use std::collections::hash_map::{Entry, HashMap, OccupiedEntry};
use std::collections::hash_set::HashSet;
use std::error::Error;
use std::fmt::{self, Debug};
use std::hash::Hash;

pub trait HashMapExt<K, V> {
    // TODO Remove this once HashMap::try_insert is stable in std
    fn try_insert(&mut self, key: K, value: V) -> Result<&mut V, OccupiedError<'_, K, V>>;
}

impl<K: Debug + PartialEq + Eq + Hash, V> HashMapExt<K, V> for HashMap<K, V> {
    fn try_insert(&mut self, key: K, value: V) -> Result<&mut V, OccupiedError<'_, K, V>> {
        match self.entry(key) {
            Entry::Occupied(entry) => Err(OccupiedError { entry, value }),
            Entry::Vacant(entry) => Ok(entry.insert(value)),
        }
    }
}

/// The error returned by [`try_insert`](HashMap::try_insert) when the key already exists.
///
/// Contains the occupied entry, and the value that was not inserted.
pub struct OccupiedError<'a, K: 'a, V: 'a> {
    /// The entry in the map that was already occupied.
    pub entry: OccupiedEntry<'a, K, V>,
    /// The value which was not inserted, because the entry was already occupied.
    pub value: V,
}

impl<K: Debug, V: Debug> Debug for OccupiedError<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OccupiedError")
            .field("key", self.entry.key())
            .field("old_value", self.entry.get())
            .field("new_value", &self.value)
            .finish_non_exhaustive()
    }
}

impl<'a, K: Debug, V: Debug> fmt::Display for OccupiedError<'a, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to insert {:?}, key {:?} already exists with value {:?}",
            self.value,
            self.entry.key(),
            self.entry.get(),
        )
    }
}

impl<'a, K: fmt::Debug, V: fmt::Debug> Error for OccupiedError<'a, K, V> {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        "key already exists"
    }
}

pub trait HashSetExt<K> {
    fn try_insert(&mut self, item: K) -> Result<()>;
}

impl<K: Debug + PartialEq + Eq + Hash> HashSetExt<K> for HashSet<K> {
    fn try_insert(&mut self, item: K) -> Result<()> {
        ensure!(
            !self.contains(&item),
            "HashSet.try_insert: item {:?} already exists",
            item,
        );
        let insert_result = self.insert(item);
        assert!(
            insert_result,
            "Can't fail because we just checked this above",
        );
        Ok(())
    }
}

// TODO Tests
