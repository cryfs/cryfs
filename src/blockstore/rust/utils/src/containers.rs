use anyhow::{ensure, Result};
use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

pub trait HashMapExt<K, V> {
    // TODO Remove this once HashMap::try_insert is stable in std
    fn try_insert(&mut self, key: K, value: V) -> Result<()>;
}

impl<K: Debug + PartialEq + Eq + Hash, V> HashMapExt<K, V> for HashMap<K, V> {
    fn try_insert(&mut self, key: K, value: V) -> Result<()> {
        ensure!(
            !self.contains_key(&key),
            "HashMap.try_insert: key {:?} already exists",
            key,
        );
        let insert_result = self.insert(key, value);
        assert!(
            insert_result.is_none(),
            "Can't fail because we just checked this above",
        );
        Ok(())
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
