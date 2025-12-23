//! Extension traits for standard library collections.
//!
//! This module provides extension traits that add `try_insert` methods to
//! [`HashMap`] and [`HashSet`], allowing for fallible insertion that returns
//! an error if the key/item already exists.

use anyhow::{Result, ensure};
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

#[cfg(test)]
mod tests {
    use super::*;

    mod hashmap_ext {
        use super::*;

        #[test]
        fn test_try_insert_new_key_succeeds() {
            let mut map: HashMap<i32, &str> = HashMap::new();

            let result = map.try_insert(1, "one");

            assert!(result.is_ok());
            assert_eq!(Some(&"one"), map.get(&1));
        }

        #[test]
        fn test_try_insert_returns_mutable_reference() {
            let mut map: HashMap<i32, String> = HashMap::new();

            let value_ref = map.try_insert(1, String::from("one")).unwrap();
            value_ref.push_str(" modified");

            assert_eq!(Some(&String::from("one modified")), map.get(&1));
        }

        #[test]
        fn test_try_insert_existing_key_fails() {
            let mut map: HashMap<i32, &str> = HashMap::new();
            map.insert(1, "one");

            let result = map.try_insert(1, "uno");

            assert!(result.is_err());
            // Original value should be unchanged
            assert_eq!(Some(&"one"), map.get(&1));
        }

        #[test]
        fn test_occupied_error_contains_key_and_values() {
            let mut map: HashMap<i32, &str> = HashMap::new();
            map.insert(1, "one");

            let err = map.try_insert(1, "uno").unwrap_err();

            assert_eq!(&1, err.entry.key());
            assert_eq!(&"one", err.entry.get());
            assert_eq!("uno", err.value);
        }

        #[test]
        fn test_occupied_error_display() {
            let mut map: HashMap<i32, &str> = HashMap::new();
            map.insert(1, "one");

            let err = map.try_insert(1, "uno").unwrap_err();
            let display = format!("{}", err);

            assert!(display.contains("1"));
            assert!(display.contains("one"));
            assert!(display.contains("uno"));
        }
    }

    mod hashset_ext {
        use super::*;

        #[test]
        fn test_try_insert_new_item_succeeds() {
            let mut set: HashSet<i32> = HashSet::new();

            let result = set.try_insert(1);

            assert!(result.is_ok());
            assert!(set.contains(&1));
        }

        #[test]
        fn test_try_insert_existing_item_fails() {
            let mut set: HashSet<i32> = HashSet::new();
            set.insert(1);

            let result = set.try_insert(1);

            assert!(result.is_err());
            let err_msg = format!("{:?}", result.unwrap_err());
            assert!(err_msg.contains("already exists"));
        }

        #[test]
        fn test_try_insert_multiple_items() {
            let mut set: HashSet<i32> = HashSet::new();

            assert!(set.try_insert(1).is_ok());
            assert!(set.try_insert(2).is_ok());
            assert!(set.try_insert(3).is_ok());

            assert_eq!(3, set.len());
        }
    }
}
