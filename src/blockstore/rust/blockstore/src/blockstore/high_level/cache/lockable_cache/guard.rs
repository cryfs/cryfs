use std::fmt::{self, Debug};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use super::pool::{CacheEntry, LockableCache};
use crate::utils::mutex::LockedMutexGuard;

/// A RAII implementation of a scoped lock for locks from a [LockPool]. When this instance is dropped (falls out of scope), the lock will be unlocked.
#[must_use = "if unused the Mutex will immediately unlock"]
pub struct GuardImpl<K, V, P>
where
    K: Eq + PartialEq + Hash + Clone + Debug + 'static,
    V: 'static,
    P: Deref<Target = LockableCache<K, V>>,
{
    pool: P,
    key: K,
    // Invariant: Is always Some(LockedMutexGuard) unless in the middle of destruction
    guard: Option<LockedMutexGuard<CacheEntry<V>>>,
}

impl<'a, K, V, P> GuardImpl<K, V, P>
where
    K: Eq + PartialEq + Hash + Clone + Debug + 'static,
    V: 'static,
    P: Deref<Target = LockableCache<K, V>>,
{
    pub(super) fn new(pool: P, key: K, guard: LockedMutexGuard<CacheEntry<V>>) -> Self {
        Self {
            pool,
            key,
            guard: Some(guard),
        }
    }

    /// TODO Test
    #[inline]
    pub fn key(&self) -> &K {
        &self.key
    }
}

impl<K, V, P> Drop for GuardImpl<K, V, P>
where
    K: Eq + PartialEq + Hash + Clone + Debug + 'static,
    V: 'static,
    P: Deref<Target = LockableCache<K, V>>,
{
    fn drop(&mut self) {
        let guard = self
            .guard
            .take()
            .expect("The self.guard field must always be set unless this was already destructed");
        self.pool._unlock(&self.key, guard);
    }
}

impl<K, V, P> Debug for GuardImpl<K, V, P>
where
    K: Eq + PartialEq + Hash + Clone + Debug + 'static,
    V: 'static,
    P: Deref<Target = LockableCache<K, V>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GuardImpl({:?})", self.key)
    }
}

impl<K, V, P> Deref for GuardImpl<K, V, P>
where
    K: Eq + PartialEq + Hash + Clone + Debug + 'static,
    V: 'static,
    P: Deref<Target = LockableCache<K, V>>,
{
    type Target = Option<V>;
    fn deref(&self) -> &Option<V> {
        &self
            .guard
            .as_ref()
            .expect("The self.guard field must always be set unless this was already destructed")
            .value
    }
}

impl<K, V, P> DerefMut for GuardImpl<K, V, P>
where
    K: Eq + PartialEq + Hash + Clone + Debug + 'static,
    V: 'static,
    P: Deref<Target = LockableCache<K, V>>,
{
    fn deref_mut(&mut self) -> &mut Option<V> {
        &mut self
            .guard
            .as_mut()
            .expect("The self.guard field must always be set unless this was already destructed")
            .value
    }
}

pub type Guard<'a, K, V> = GuardImpl<K, V, &'a LockableCache<K, V>>;
pub type OwnedGuard<K, V> = GuardImpl<K, V, Arc<LockableCache<K, V>>>;
