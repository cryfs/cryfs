use anyhow::Result;
use futures::stream::Stream;
use futures::{future, stream::FuturesUnordered, StreamExt};
use lru::LruCache;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;
use std::sync::Arc;
use tokio::time::{Duration, Instant};

use super::error::TryLockError;
use super::guard::{Guard, GuardImpl, OwnedGuard};
use crate::utils::locked_mutex_guard::LockedMutexGuard;
use crate::utils::lru_into_iter::LruCacheIntoIter;

// TODO Fix code samples in documentation

pub(super) struct CacheEntry<V> {
    pub(super) value: Option<V>,

    // last_unlocked gets updated whenever we are finished with an item,
    // i.e. when it gets unlocked and returned to the cache.
    // Since getting it from the cache is the action that moves it to the top
    // of the LRU order, there can be a temporary mismatch between the
    // timestamp order and the LRU order, but only for as long as the
    // item is locked and we can't access the timestamp anyways while
    // it is locked.
    // TODO Test last_unlocked is correctly updated
    last_unlocked: Instant,
}

impl<V> Debug for CacheEntry<V> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("CacheEntry")
            .field("last_unlocked", &self.last_unlocked)
            .finish()
    }
}

/// A cache where individual keys can be locked/unlocked, even if they don't carry any data in the cache.
/// It initially considers all keys as "unlocked", but they can be locked
/// and if a second thread tries to acquire a lock for the same key, they will have to wait.
///
/// ```
/// use crate::blockstore::high_level::cache::LockableCache;
///
/// let pool: LockableCache<i64, String> = LockableCache::new();
/// # (|| -> Result<(), lockpool::PoisonError<_, _>> {
/// let guard1 = pool.lock(4)?;
/// let guard2 = pool.lock(5)?;
///
/// // This next line would cause a deadlock or panic because `4` is already locked on this thread
/// // let guard3 = pool.lock(4)?;
///
/// // After dropping the corresponding guard, we can lock it again
/// std::mem::drop(guard1);
/// let guard3 = pool.lock(4)?;
/// # Ok(())
/// # })().unwrap();
/// ```
///
/// You can use an arbitrary type to index cache entries by, as long as that type implements [PartialEq] + [Eq] + [Hash] + [Clone] + [Debug].
///
/// ```
/// use lockpool::{LockPool, SyncLockPool};
///
/// #[derive(PartialEq, Eq, Hash, Clone, Debug)]
/// struct CustomLockKey(u32);
///
/// let pool = SyncLockPool::new();
/// # (|| -> Result<(), lockpool::PoisonError<_, _>> {
/// let guard = pool.lock(CustomLockKey(4))?;
/// # Ok(())
/// # })().unwrap();
/// ```
///
/// Under the hood, a [LockableCache] is a [LruCache](lru::LruCache) of [Mutex](tokio::sync::Mutex)es, with some logic making sure there aren't any race conditions when adding or removing entries.
pub struct LockableCache<K, V>
where
    K: Eq + PartialEq + Hash + Clone + Debug,
    V: 'static,
{
    // We always use std::sync::Mutex for protecting the LruCache since its guards
    // never have to be kept across await boundaries, and std::sync::Mutex is faster
    // than tokio::sync::Mutex. But the inner per-key locks use tokio::sync::Mutex
    // because they need to be kep across await boundaries.
    // Invariants:
    // - Any entries not currently locked will never be None. None is only entered
    //   into the cache to denote values that are currrently locked but don't actually
    //   have data in the cache. This invariant is mostly meant to clean up space.
    // - The timestamps in CacheEntry will follow the same order as the LRU order of the cache,
    //   with an exception for currently locked entries that may be temporarily out of order
    //   while the entry is locked.
    // - We never hand the inner Arc around a cache entry out of the encapsulation of this class,
    //   except through non-cloneable Guard objects encapsulating those Arcs.
    //   This allows us to reason about which threads can or cannot increase the refcounts.
    // TODO Use the lockable crate instead
    cache_entries: std::sync::Mutex<LruCache<K, Arc<tokio::sync::Mutex<CacheEntry<V>>>>>,
}

impl<K, V> LockableCache<K, V>
where
    // TODO Can we remove the 'static bound from K and V?
    K: Eq + PartialEq + Hash + Clone + Debug + 'static,
    V: 'static,
{
    /// Create a new cache with no entries and no locked keys
    #[inline]
    pub fn new() -> Self {
        Self {
            cache_entries: std::sync::Mutex::new(LruCache::unbounded()),
        }
    }

    /// Return the number of cache entries.
    ///
    /// Corner case: Currently locked keys are counted even if they don't have any data in the cache.
    #[inline]
    pub fn num_entries_or_locked(&self) -> usize {
        self._cache_entries().len()
    }

    /// Lock a key and return a guard with any potential cache entry for that key.
    /// Any changes to that entry will be written back to the cache when the mutex leaves scope.
    /// Cache entries can be created by locking an entry and replacing the None with a Some and
    /// can be removed by replacing it with None.
    ///
    /// If the lock with this key is currently locked by a different thread, then the current thread blocks until it becomes available.
    /// Upon returning, the thread is the only thread with the lock held. A RAII guard is returned to allow scoped unlock
    /// of the lock. When the guard goes out of scope, the lock will be unlocked.
    ///
    /// This function can only be used from non-async contexts and will panic if used from async contexts.
    ///
    /// The exact behavior on locking a lock in the thread which already holds the lock is left unspecified.
    /// However, this function will not return on the second call (it might panic or deadlock, for example).
    ///
    /// Panics
    /// -----
    /// - This function might panic when called if the lock is already held by the current thread.
    /// - This function will also panic when called from an `async` context.
    ///   See documentation of [tokio::sync::Mutex] for details.
    ///
    /// Examples
    /// -----
    /// ```
    /// use lockpool::{LockPool, SyncLockPool};
    ///
    /// let pool = SyncLockPool::new();
    /// # (|| -> Result<(), lockpool::PoisonError<_, _>> {
    /// let guard1 = pool.lock(4)?;
    /// let guard2 = pool.lock(5)?;
    ///
    /// // This next line would cause a deadlock or panic because `4` is already locked on this thread
    /// // let guard3 = pool.lock(4)?;
    ///
    /// // After dropping the corresponding guard, we can lock it again
    /// std::mem::drop(guard1);
    /// let guard3 = pool.lock(4)?;
    /// # Ok(())
    /// # })().unwrap();
    /// ```
    pub fn blocking_lock(&self, key: K) -> Guard<'_, K, V> {
        Self::_blocking_lock(self, key)
    }

    /// Lock a lock by key and return a guard with any potential cache entry for that key.
    ///
    /// This is identical to [LockableCache::blocking_lock], but it works on an `Arc<LockableCache>` instead of a [LockableCache] and
    /// returns a [OwnedGuard] that binds its lifetime to the [LockableCache] in that [Arc]. Such an [OwnedGuard] can be more
    /// easily moved around or cloned.
    ///
    /// This function can be used from non-async contexts but will panic if used from async contexts.
    ///
    /// Panics
    /// -----
    /// - This function might panic when called if the lock is already held by the current thread.
    /// - This function will also panic when called from an `async` context.
    ///   See documentation of [tokio::sync::Mutex] for details.
    ///
    /// Examples
    /// -----
    /// ```
    /// use lockpool::{LockPool, SyncLockPool};
    /// use std::sync::Arc;
    ///
    /// let pool = Arc::new(SyncLockPool::new());
    /// # (|| -> Result<(), lockpool::PoisonError<_, _>> {
    /// let guard1 = pool.lock_owned(4)?;
    /// let guard2 = pool.lock_owned(5)?;
    ///
    /// // This next line would cause a deadlock or panic because `4` is already locked on this thread
    /// // let guard3 = pool.lock_owned(4)?;
    ///
    /// // After dropping the corresponding guard, we can lock it again
    /// std::mem::drop(guard1);
    /// let guard3 = pool.lock_owned(4)?;
    /// # Ok(())
    /// # })().unwrap();
    /// ```
    pub fn blocking_lock_owned(self: &Arc<Self>, key: K) -> OwnedGuard<K, V> {
        Self::_blocking_lock(Arc::clone(self), key)
    }

    /// Attempts to acquire the lock with the given key and if successful, returns a guard with any potential cache entry for that key.
    /// Any changes to that entry will be written back to the cache when the mutex leaves scope.
    /// Cache entries can be created by locking an entry and replacing the None with a Some and
    /// can be removed by replacing it with None.
    ///
    /// If the lock could not be acquired at this time, then [Err] is returned. Otherwise, a RAII guard is returned.
    /// The lock will be unlocked when the guard is dropped.
    ///
    /// This function does not block and can be used from both async and non-async contexts.
    ///
    /// Errors
    /// -----
    /// - If the lock could not be acquired because it is already locked, then this call will return [TryLockError::WouldBlock].
    ///
    /// Examples
    /// -----
    /// ```
    /// use lockpool::{TryLockError, LockPool, SyncLockPool};
    ///
    /// let pool = SyncLockPool::new();
    /// # (|| -> Result<(), lockpool::PoisonError<_, _>> {
    /// let guard1 = pool.lock(4)?;
    /// let guard2 = pool.lock(5)?;
    ///
    /// // This next line would cause a deadlock or panic because `4` is already locked on this thread
    /// let guard3 = pool.try_lock(4);
    /// assert!(matches!(guard3.unwrap_err(), TryLockError::WouldBlock));
    ///
    /// // After dropping the corresponding guard, we can lock it again
    /// std::mem::drop(guard1);
    /// let guard3 = pool.lock(4)?;
    /// # Ok(())
    /// # })().unwrap();
    /// ```
    pub fn try_lock(&self, key: K) -> Result<Guard<'_, K, V>, TryLockError> {
        Self::_try_lock(self, key)
    }

    /// Attempts to acquire the lock with the given key and if successful, returns a guard with any potential cache entry for that key.
    ///
    /// This is identical to [LockableCache::try_lock], but it works on an `Arc<LockableCache>` instead of a [LockableCache] and
    /// returns an [OwnedGuard] that binds its lifetime to the [LockableCache] in that [Arc]. Such an [OwnedGuard] can be more
    /// easily moved around or cloned.
    ///
    /// This function does not block and can be used in both async and non-async contexts.
    ///
    /// Errors
    /// -----
    /// - If the lock could not be acquired because it is already locked, then this call will return [TryLockError::WouldBlock].
    ///
    /// Examples
    /// -----
    /// ```
    /// use lockpool::{TryLockError, LockPool, SyncLockPool};
    /// use std::sync::Arc;
    ///
    /// let pool = Arc::new(SyncLockPool::new());
    /// # (|| -> Result<(), lockpool::PoisonError<_, _>> {
    /// let guard1 = pool.lock(4)?;
    /// let guard2 = pool.lock(5)?;
    ///
    /// // This next line would cause a deadlock or panic because `4` is already locked on this thread
    /// let guard3 = pool.try_lock_owned(4);
    /// assert!(matches!(guard3.unwrap_err(), TryLockError::WouldBlock));
    ///
    /// // After dropping the corresponding guard, we can lock it again
    /// std::mem::drop(guard1);
    /// let guard3 = pool.lock(4)?;
    /// # Ok(())
    /// # })().unwrap();
    /// ```
    pub fn try_lock_owned(self: &Arc<Self>, key: K) -> Result<OwnedGuard<K, V>, TryLockError> {
        Self::_try_lock(Arc::clone(self), key)
    }

    fn _cache_entries(
        &self,
    ) -> std::sync::MutexGuard<'_, LruCache<K, Arc<tokio::sync::Mutex<CacheEntry<V>>>>> {
        self.cache_entries
            .lock()
            .expect("The global mutex protecting the LockableCache is poisoned. This shouldn't happen since there shouldn't be any user code running while this lock is held so no thread should ever panic with it")
    }

    pub(super) fn _load_or_insert_mutex_for_key(
        &self,
        key: K,
    ) -> Arc<tokio::sync::Mutex<CacheEntry<V>>> {
        let mut cache_entries = self._cache_entries();
        let entry = cache_entries
            // TODO Remove clone()
            .get_or_insert(key.clone(), || {
                Arc::new(tokio::sync::Mutex::new(CacheEntry {
                    last_unlocked: Instant::now(),
                    value: None,
                }))
            })
            .expect(
                "Cache capacity is zero. This can't happen since we created an unbounded cache",
            );
        Arc::clone(entry)
    }

    fn _blocking_lock<S: Deref<Target = Self>>(this: S, key: K) -> GuardImpl<K, V, S> {
        let mutex = this._load_or_insert_mutex_for_key(key.clone());
        // Now we have an Arc::clone of the mutex for this key, and the global mutex is already unlocked so other threads can access the cache.
        // The following blocks until the mutex for this key is acquired.

        let guard = LockedMutexGuard::blocking_lock(mutex);
        GuardImpl::new(this, key, guard)
    }

    fn _try_lock<S: Deref<Target = Self>>(
        this: S,
        key: K,
    ) -> Result<GuardImpl<K, V, S>, TryLockError> {
        let mutex = this._load_or_insert_mutex_for_key(key.clone());
        // Now we have an Arc::clone of the mutex for this key, and the global mutex is already unlocked so other threads can access the cache.
        // The following tries to lock the mutex.

        let guard = match LockedMutexGuard::try_lock(mutex) {
            Ok(guard) => Ok(guard),
            Err(_) => Err(TryLockError::WouldBlock),
        }?;
        let guard = GuardImpl::new(this, key, guard);
        Ok(guard)
    }

    pub(super) fn _unlock(&self, key: &K, mut guard: LockedMutexGuard<CacheEntry<V>>) {
        let mut cache_entries = self._cache_entries();
        let mutex: &Arc<tokio::sync::Mutex<CacheEntry<V>>> = cache_entries
            .get(key)
            .expect("This entry must exist or the guard passed in as a parameter shouldn't exist");
        guard.last_unlocked = Instant::now();
        let entry_carries_a_value = guard.value.is_some();
        std::mem::drop(guard);

        // Now the guard is dropped and the lock for this key is unlocked.
        // If there are any other Self::lock() calls for this key already running and
        // waiting for the mutex, they will be unblocked now and their guard
        // will be created.
        // But since we still have the global mutex on self.cache_entries, currently no
        // thread can newly call Self::lock() and create a clone of our Arc. Similarly,
        // no other thread can enter Self::unlock() and reduce the strong_count of the Arc.
        // This means that if Arc::strong_count() == 1, we know that we can clean up
        // without race conditions.

        if Arc::strong_count(mutex) == 1 {
            // The guard we're about to drop is the last guard for this mutex,
            // the only other Arc pointing to it is the one in the hashmap.
            // If it carries a value, keep it, but it doesn't carry a value,
            // clean up to fulfill the invariant
            if !entry_carries_a_value {
                let remove_result = cache_entries.pop(key);
                assert!(
                    remove_result.is_some(),
                    "We just got this entry above from the hash map, it cannot have vanished since then"
                );
            }
        }
    }

    /// TODO Docs
    pub async fn async_lock(&self, key: K) -> Guard<'_, K, V> {
        Self::_async_lock(self, key).await
    }

    /// TODO Docs
    pub async fn async_lock_owned(self: &Arc<Self>, key: K) -> OwnedGuard<K, V> {
        Self::_async_lock(Arc::clone(self), key).await
    }

    async fn _async_lock<S: Deref<Target = Self>>(this: S, key: K) -> GuardImpl<K, V, S> {
        let mutex = this._load_or_insert_mutex_for_key(key.clone());
        // Now we have an Arc::clone of the mutex for this key, and the global mutex is already unlocked so other threads can access the cache.
        // The following blocks until the mutex for this key is acquired.

        let guard = LockedMutexGuard::async_lock(mutex).await;
        GuardImpl::new(this, key, guard)
    }

    /// TODO Docs
    /// TODO Test
    pub fn lock_entries_unlocked_for_longer_than(
        &self,
        duration: Duration,
    ) -> Vec<Guard<'_, K, V>> {
        let now = Instant::now();
        let mut result = vec![];
        let cache_entries = self._cache_entries();
        let mut current_entry_timestamp = None;
        // TODO Check that iter().rev() actually starts with the oldest ones and not with the newest once. Otherwise, remove .rev().
        for (key, entry) in cache_entries.iter().rev() {
            if Arc::strong_count(&entry) == 1 {
                // There is currently nobody who has access to this mutex and could lock it.
                // And since we're also blocking the global cache mutex, nobody can get it.
                // We must be able to lock this and we can safely prune it.
                let guard = LockedMutexGuard::try_lock(Arc::clone(&entry)).expect(
                    "We just checked that nobody can lock this. But for some reason it was locked.",
                );
                assert!(
                    guard.last_unlocked >= current_entry_timestamp.unwrap_or(guard.last_unlocked),
                    "Cache order broken - entries don't seem to be in LRU order"
                );
                current_entry_timestamp = Some(guard.last_unlocked);

                if now - guard.last_unlocked <= duration {
                    // The next entry is too new to be pruned
                    // TODO Assert that all remaining entries are too new to be pruned, i.e. continue walk through remaining entries and check order
                    return result;
                }

                result.push(Guard::new(self, key.clone(), guard));
            } else {
                // Somebody currently has access to this mutex and is likely going to lock it.
                // This means the entry shouldn't be pruned, it will soon get a new timestamp.
            }
        }

        // We ran out of entries to check, no entry is too new to be pruned.
        result
    }

    /// TODO Docs
    /// TODO Test
    pub fn into_entries_unordered(self) -> impl Stream<Item = (K, V)> {
        let entries: LruCache<_, _> = self.cache_entries.into_inner().expect("Lock poisoned");

        // We now have exclusive access to the LruCache object. No other thread or task can call lock() and increase
        // the refcount for one of the Arcs. They still can have (un-cloneable) Guard instances and those will eventually call
        // _unlock() on destruction. We just need to wait until the last thread gives up an Arc and then we can remove it from the mutex.

        let entries: FuturesUnordered<_> = entries
            .into_iter()
            .map(|(key, value)| future::ready((key, value)))
            .collect();
        entries.filter_map(|(key, value)| async {
            while Arc::strong_count(&value) > 1 {
                // TODO Is there a better alternative that doesn't involve busy waiting?
                tokio::task::yield_now().await;
            }
            // Now we're the last task having a reference to this arc.
            let value = Arc::try_unwrap(value)
                .expect("This can't fail since we are the only task having access");
            let value = value.into_inner();

            // Ignore None entries
            value.value.map(|value| (key, value))
        })
    }

    // TODO Docs
    // TODO Test
    pub fn keys(&self) -> Vec<K> {
        let cache_entries = self._cache_entries();
        cache_entries
            .iter()
            .map(|(key, _value)| key)
            .cloned()
            .collect()
    }
}

impl<K, V> Debug for LockableCache<K, V>
where
    K: Eq + PartialEq + Hash + Clone + Debug,
    V: 'static,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("LockableCache").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::super::error::TryLockError;
    use super::LockableCache;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread::{self, JoinHandle};
    use std::time::Duration;

    // TODO Add a test adding multiple entries and making sure all locking functions can read them
    // TODO Add tests checking that the async_lock, lock_owned, lock methods all block each other. For lock and lock_owned that can probably go into common tests.rs

    // Launch a thread that
    // 1. locks the given key
    // 2. once it has the lock, increments a counter
    // 3. then waits until a barrier is released before it releases the lock
    fn launch_thread_blocking_lock(
        pool: &Arc<LockableCache<isize, String>>,
        key: isize,
        counter: &Arc<AtomicU32>,
        barrier: Option<&Arc<Mutex<()>>>,
    ) -> JoinHandle<()> {
        let pool = Arc::clone(pool);
        let counter = Arc::clone(counter);
        let barrier = barrier.map(Arc::clone);
        thread::spawn(move || {
            let _guard = pool.blocking_lock(key);
            counter.fetch_add(1, Ordering::SeqCst);
            if let Some(barrier) = barrier {
                let _barrier = barrier.lock().unwrap();
            }
        })
    }

    fn launch_thread_blocking_lock_owned(
        pool: &Arc<LockableCache<isize, String>>,
        key: isize,
        counter: &Arc<AtomicU32>,
        barrier: Option<&Arc<Mutex<()>>>,
    ) -> JoinHandle<()> {
        let pool = Arc::clone(pool);
        let counter = Arc::clone(counter);
        let barrier = barrier.map(Arc::clone);
        thread::spawn(move || {
            let _guard = pool.blocking_lock_owned(key);
            counter.fetch_add(1, Ordering::SeqCst);
            if let Some(barrier) = barrier {
                let _barrier = barrier.lock().unwrap();
            }
        })
    }

    fn launch_thread_try_lock(
        pool: &Arc<LockableCache<isize, String>>,
        key: isize,
        counter: &Arc<AtomicU32>,
        barrier: Option<&Arc<Mutex<()>>>,
    ) -> JoinHandle<()> {
        let pool = Arc::clone(pool);
        let counter = Arc::clone(counter);
        let barrier = barrier.map(Arc::clone);
        thread::spawn(move || {
            let _guard = loop {
                match pool.try_lock(key) {
                    Err(_) =>
                    /* Continue loop */
                    {
                        ()
                    }
                    Ok(guard) => break guard,
                }
            };
            counter.fetch_add(1, Ordering::SeqCst);
            if let Some(barrier) = barrier {
                let _barrier = barrier.lock().unwrap();
            }
        })
    }

    fn launch_thread_try_lock_owned(
        pool: &Arc<LockableCache<isize, String>>,
        key: isize,
        counter: &Arc<AtomicU32>,
        barrier: Option<&Arc<Mutex<()>>>,
    ) -> JoinHandle<()> {
        let pool = Arc::clone(pool);
        let counter = Arc::clone(counter);
        let barrier = barrier.map(Arc::clone);
        thread::spawn(move || {
            let _guard = loop {
                match pool.try_lock_owned(key) {
                    Err(_) =>
                    /* Continue loop */
                    {
                        ()
                    }
                    Ok(guard) => break guard,
                }
            };
            counter.fetch_add(1, Ordering::SeqCst);
            if let Some(barrier) = barrier {
                let _barrier = barrier.lock().unwrap();
            }
        })
    }

    fn launch_thread_async_lock(
        pool: &Arc<LockableCache<isize, String>>,
        key: isize,
        counter: &Arc<AtomicU32>,
        barrier: Option<&Arc<Mutex<()>>>,
    ) -> JoinHandle<()> {
        let pool = Arc::clone(pool);
        let counter = Arc::clone(counter);
        let barrier = barrier.map(Arc::clone);
        thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let _guard = runtime.block_on(pool.async_lock(key));
            counter.fetch_add(1, Ordering::SeqCst);
            if let Some(barrier) = barrier {
                let _barrier = barrier.lock().unwrap();
            }
        })
    }

    fn launch_thread_async_lock_owned(
        pool: &Arc<LockableCache<isize, String>>,
        key: isize,
        counter: &Arc<AtomicU32>,
        barrier: Option<&Arc<Mutex<()>>>,
    ) -> JoinHandle<()> {
        let pool = Arc::clone(pool);
        let counter = Arc::clone(counter);
        let barrier = barrier.map(Arc::clone);
        thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let _guard = runtime.block_on(pool.async_lock_owned(key));
            counter.fetch_add(1, Ordering::SeqCst);
            if let Some(barrier) = barrier {
                let _barrier = barrier.lock().unwrap();
            }
        })
    }

    #[tokio::test]
    #[should_panic(
        expected = "Cannot start a runtime from within a runtime. This happens because a function (like `block_on`) attempted to block the current thread while the thread is being used to drive asynchronous tasks."
    )]
    async fn blocking_lock_from_async_context_with_sync_api() {
        let p = LockableCache::<isize, String>::new();
        let _ = p.blocking_lock(3);
    }

    #[tokio::test]
    #[should_panic(
        expected = "Cannot start a runtime from within a runtime. This happens because a function (like `block_on`) attempted to block the current thread while the thread is being used to drive asynchronous tasks."
    )]
    async fn blocking_lock_owned_from_async_context_with_sync_api() {
        let p = Arc::new(LockableCache::<isize, String>::new());
        let _ = p.blocking_lock_owned(3);
    }

    mod simple {
        use super::*;

        #[tokio::test]
        async fn async_lock() {
            let pool = LockableCache::<isize, String>::new();
            assert_eq!(0, pool.num_entries_or_locked());
            let guard = pool.async_lock(4).await;
            assert!(guard.is_none());
            assert_eq!(1, pool.num_entries_or_locked());
            std::mem::drop(guard);
            assert_eq!(0, pool.num_entries_or_locked());
        }

        #[tokio::test]
        async fn async_lock_owned() {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            assert_eq!(0, pool.num_entries_or_locked());
            let guard = pool.async_lock_owned(4).await;
            assert!(guard.is_none());
            assert_eq!(1, pool.num_entries_or_locked());
            std::mem::drop(guard);
            assert_eq!(0, pool.num_entries_or_locked());
        }

        #[test]
        fn blocking_lock() {
            let pool = LockableCache::<isize, String>::new();
            assert_eq!(0, pool.num_entries_or_locked());
            let guard = pool.blocking_lock(4);
            assert!(guard.is_none());
            assert_eq!(1, pool.num_entries_or_locked());
            std::mem::drop(guard);
            assert_eq!(0, pool.num_entries_or_locked());
        }

        #[test]
        fn blocking_lock_owned() {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            assert_eq!(0, pool.num_entries_or_locked());
            let guard = pool.blocking_lock_owned(4);
            assert!(guard.is_none());
            assert_eq!(1, pool.num_entries_or_locked());
            std::mem::drop(guard);
            assert_eq!(0, pool.num_entries_or_locked());
        }

        #[test]
        fn try_lock() {
            let pool = LockableCache::<isize, String>::new();
            assert_eq!(0, pool.num_entries_or_locked());
            let guard = pool.try_lock(4).unwrap();
            assert!(guard.is_none());
            assert_eq!(1, pool.num_entries_or_locked());
            std::mem::drop(guard);
            assert_eq!(0, pool.num_entries_or_locked());
        }

        #[test]
        fn try_lock_owned() {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            assert_eq!(0, pool.num_entries_or_locked());
            let guard = pool.try_lock_owned(4).unwrap();
            assert!(guard.is_none());
            assert_eq!(1, pool.num_entries_or_locked());
            std::mem::drop(guard);
            assert_eq!(0, pool.num_entries_or_locked());
        }
    }

    mod try_lock {
        use super::*;

        #[test]
        fn try_lock() {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            let guard = pool.blocking_lock(5);

            let error = pool.try_lock(5).unwrap_err();
            assert!(matches!(error, TryLockError::WouldBlock));

            // Check that we can stil lock other locks while the child is waiting
            {
                let _g = pool.try_lock(4).unwrap();
            }

            // Now free the lock so the we can get it again
            std::mem::drop(guard);

            // And check that we can get it again
            {
                let _g = pool.try_lock(5).unwrap();
            }

            assert_eq!(0, pool.num_entries_or_locked());
        }

        #[test]
        fn try_lock_owned() {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            let guard = pool.blocking_lock_owned(5);

            let error = pool.try_lock_owned(5).unwrap_err();
            assert!(matches!(error, TryLockError::WouldBlock));

            // Check that we can stil lock other locks while the child is waiting
            {
                let _g = pool.try_lock_owned(4).unwrap();
            }

            // Now free the lock so the we can get it again
            std::mem::drop(guard);

            // And check that we can get it again
            {
                let _g = pool.try_lock_owned(5).unwrap();
            }

            assert_eq!(0, pool.num_entries_or_locked());
        }
    }

    mod adding_cache_entries {
        use super::*;

        #[tokio::test]
        async fn async_lock() {
            let pool = LockableCache::<isize, String>::new();
            assert_eq!(0, pool.num_entries_or_locked());
            let mut guard = pool.async_lock(4).await;
            *guard = Some(String::from("Cache Entry Value"));
            assert_eq!(1, pool.num_entries_or_locked());
            std::mem::drop(guard);
            assert_eq!(1, pool.num_entries_or_locked());
            assert_eq!(
                *pool.async_lock(4).await,
                Some(String::from("Cache Entry Value"))
            );
        }

        #[tokio::test]
        async fn async_lock_owned() {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            assert_eq!(0, pool.num_entries_or_locked());
            let mut guard = pool.async_lock_owned(4).await;
            *guard = Some(String::from("Cache Entry Value"));
            assert_eq!(1, pool.num_entries_or_locked());
            std::mem::drop(guard);
            assert_eq!(1, pool.num_entries_or_locked());
            assert_eq!(
                *pool.async_lock_owned(4).await,
                Some(String::from("Cache Entry Value"))
            );
        }

        #[test]
        fn blocking_lock() {
            let pool = LockableCache::<isize, String>::new();
            assert_eq!(0, pool.num_entries_or_locked());
            let mut guard = pool.blocking_lock(4);
            *guard = Some(String::from("Cache Entry Value"));
            assert_eq!(1, pool.num_entries_or_locked());
            std::mem::drop(guard);
            assert_eq!(1, pool.num_entries_or_locked());
            assert_eq!(
                *pool.blocking_lock(4),
                Some(String::from("Cache Entry Value"))
            );
        }

        #[test]
        fn blocking_lock_owned() {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            assert_eq!(0, pool.num_entries_or_locked());
            let mut guard = pool.blocking_lock_owned(4);
            *guard = Some(String::from("Cache Entry Value"));
            assert_eq!(1, pool.num_entries_or_locked());
            std::mem::drop(guard);
            assert_eq!(1, pool.num_entries_or_locked());
            assert_eq!(
                *pool.blocking_lock_owned(4),
                Some(String::from("Cache Entry Value"))
            );
        }

        #[test]
        fn try_lock() {
            let pool = LockableCache::<isize, String>::new();
            assert_eq!(0, pool.num_entries_or_locked());
            let mut guard = pool.try_lock(4).unwrap();
            *guard = Some(String::from("Cache Entry Value"));
            assert_eq!(1, pool.num_entries_or_locked());
            std::mem::drop(guard);
            assert_eq!(1, pool.num_entries_or_locked());
            assert_eq!(
                *pool.try_lock(4).unwrap(),
                Some(String::from("Cache Entry Value"))
            );
        }

        #[test]
        fn try_lock_owned() {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            assert_eq!(0, pool.num_entries_or_locked());
            let mut guard = pool.try_lock_owned(4).unwrap();
            *guard = Some(String::from("Cache Entry Value"));
            assert_eq!(1, pool.num_entries_or_locked());
            std::mem::drop(guard);
            assert_eq!(1, pool.num_entries_or_locked());
            assert_eq!(
                *pool.try_lock_owned(4).unwrap(),
                Some(String::from("Cache Entry Value"))
            );
        }
    }

    mod removing_cache_entries {
        use super::*;

        #[tokio::test]
        async fn async_lock() {
            let pool = LockableCache::<isize, String>::new();
            *pool.async_lock(4).await = Some(String::from("Cache Entry Value"));

            assert_eq!(1, pool.num_entries_or_locked());
            let mut guard = pool.async_lock(4).await;
            *guard = None;
            std::mem::drop(guard);

            assert_eq!(0, pool.num_entries_or_locked());
            assert_eq!(*pool.async_lock(4).await, None);
        }

        #[tokio::test]
        async fn async_lock_owned() {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            *pool.async_lock_owned(4).await = Some(String::from("Cache Entry Value"));

            assert_eq!(1, pool.num_entries_or_locked());
            let mut guard = pool.async_lock_owned(4).await;
            *guard = None;
            std::mem::drop(guard);

            assert_eq!(0, pool.num_entries_or_locked());
            assert_eq!(*pool.async_lock_owned(4).await, None);
        }

        #[test]
        fn blocking_lock() {
            let pool = LockableCache::<isize, String>::new();
            *pool.blocking_lock(4) = Some(String::from("Cache Entry Value"));

            assert_eq!(1, pool.num_entries_or_locked());
            let mut guard = pool.blocking_lock(4);
            *guard = None;
            std::mem::drop(guard);

            assert_eq!(0, pool.num_entries_or_locked());
            assert_eq!(*pool.blocking_lock(4), None);
        }

        #[test]
        fn blocking_lock_owned() {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            *pool.blocking_lock_owned(4) = Some(String::from("Cache Entry Value"));

            assert_eq!(1, pool.num_entries_or_locked());
            let mut guard = pool.blocking_lock_owned(4);
            *guard = None;
            std::mem::drop(guard);

            assert_eq!(0, pool.num_entries_or_locked());
            assert_eq!(*pool.blocking_lock_owned(4), None);
        }

        #[test]
        fn try_lock() {
            let pool = LockableCache::<isize, String>::new();
            *pool.try_lock(4).unwrap() = Some(String::from("Cache Entry Value"));

            assert_eq!(1, pool.num_entries_or_locked());
            let mut guard = pool.try_lock(4).unwrap();
            *guard = None;
            std::mem::drop(guard);

            assert_eq!(0, pool.num_entries_or_locked());
            assert_eq!(*pool.try_lock(4).unwrap(), None);
        }

        #[test]
        fn try_lock_owned() {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            *pool.try_lock_owned(4).unwrap() = Some(String::from("Cache Entry Value"));

            assert_eq!(1, pool.num_entries_or_locked());
            let mut guard = pool.try_lock_owned(4).unwrap();
            *guard = None;
            std::mem::drop(guard);

            assert_eq!(0, pool.num_entries_or_locked());
            assert_eq!(*pool.try_lock_owned(4).unwrap(), None);
        }
    }

    mod multi {
        use super::*;

        #[tokio::test]
        async fn async_lock() {
            let pool = LockableCache::<isize, String>::new();
            assert_eq!(0, pool.num_entries_or_locked());
            let guard1 = pool.async_lock(1).await;
            assert!(guard1.is_none());
            assert_eq!(1, pool.num_entries_or_locked());
            let guard2 = pool.async_lock(2).await;
            assert!(guard2.is_none());
            assert_eq!(2, pool.num_entries_or_locked());
            let guard3 = pool.async_lock(3).await;
            assert!(guard3.is_none());
            assert_eq!(3, pool.num_entries_or_locked());

            std::mem::drop(guard2);
            assert_eq!(2, pool.num_entries_or_locked());
            std::mem::drop(guard1);
            assert_eq!(1, pool.num_entries_or_locked());
            std::mem::drop(guard3);
            assert_eq!(0, pool.num_entries_or_locked());
        }

        #[tokio::test]
        async fn async_lock_owned() {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            assert_eq!(0, pool.num_entries_or_locked());
            let guard1 = pool.async_lock_owned(1).await;
            assert!(guard1.is_none());
            assert_eq!(1, pool.num_entries_or_locked());
            let guard2 = pool.async_lock_owned(2).await;
            assert!(guard2.is_none());
            assert_eq!(2, pool.num_entries_or_locked());
            let guard3 = pool.async_lock_owned(3).await;
            assert!(guard3.is_none());
            assert_eq!(3, pool.num_entries_or_locked());

            std::mem::drop(guard2);
            assert_eq!(2, pool.num_entries_or_locked());
            std::mem::drop(guard1);
            assert_eq!(1, pool.num_entries_or_locked());
            std::mem::drop(guard3);
            assert_eq!(0, pool.num_entries_or_locked());
        }

        #[test]
        fn blocking_lock() {
            let pool = LockableCache::<isize, String>::new();
            assert_eq!(0, pool.num_entries_or_locked());
            let guard1 = pool.blocking_lock(1);
            assert!(guard1.is_none());
            assert_eq!(1, pool.num_entries_or_locked());
            let guard2 = pool.blocking_lock(2);
            assert!(guard2.is_none());
            assert_eq!(2, pool.num_entries_or_locked());
            let guard3 = pool.blocking_lock(3);
            assert!(guard3.is_none());
            assert_eq!(3, pool.num_entries_or_locked());

            std::mem::drop(guard2);
            assert_eq!(2, pool.num_entries_or_locked());
            std::mem::drop(guard1);
            assert_eq!(1, pool.num_entries_or_locked());
            std::mem::drop(guard3);
            assert_eq!(0, pool.num_entries_or_locked());
        }

        #[test]
        fn blocking_lock_owned() {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            assert_eq!(0, pool.num_entries_or_locked());
            let guard1 = pool.blocking_lock_owned(1);
            assert!(guard1.is_none());
            assert_eq!(1, pool.num_entries_or_locked());
            let guard2 = pool.blocking_lock_owned(2);
            assert!(guard2.is_none());
            assert_eq!(2, pool.num_entries_or_locked());
            let guard3 = pool.blocking_lock_owned(3);
            assert!(guard3.is_none());
            assert_eq!(3, pool.num_entries_or_locked());

            std::mem::drop(guard2);
            assert_eq!(2, pool.num_entries_or_locked());
            std::mem::drop(guard1);
            assert_eq!(1, pool.num_entries_or_locked());
            std::mem::drop(guard3);
            assert_eq!(0, pool.num_entries_or_locked());
        }

        #[test]
        fn try_lock() {
            let pool = LockableCache::<isize, String>::new();
            assert_eq!(0, pool.num_entries_or_locked());
            let guard1 = pool.try_lock(1).unwrap();
            assert!(guard1.is_none());
            assert_eq!(1, pool.num_entries_or_locked());
            let guard2 = pool.try_lock(2).unwrap();
            assert!(guard2.is_none());
            assert_eq!(2, pool.num_entries_or_locked());
            let guard3 = pool.try_lock(3).unwrap();
            assert!(guard3.is_none());
            assert_eq!(3, pool.num_entries_or_locked());

            std::mem::drop(guard2);
            assert_eq!(2, pool.num_entries_or_locked());
            std::mem::drop(guard1);
            assert_eq!(1, pool.num_entries_or_locked());
            std::mem::drop(guard3);
            assert_eq!(0, pool.num_entries_or_locked());
        }

        #[test]
        fn try_lock_owned() {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            assert_eq!(0, pool.num_entries_or_locked());
            let guard1 = pool.try_lock_owned(1).unwrap();
            assert!(guard1.is_none());
            assert_eq!(1, pool.num_entries_or_locked());
            let guard2 = pool.try_lock_owned(2).unwrap();
            assert!(guard2.is_none());
            assert_eq!(2, pool.num_entries_or_locked());
            let guard3 = pool.try_lock_owned(3).unwrap();
            assert!(guard3.is_none());
            assert_eq!(3, pool.num_entries_or_locked());

            std::mem::drop(guard2);
            assert_eq!(2, pool.num_entries_or_locked());
            std::mem::drop(guard1);
            assert_eq!(1, pool.num_entries_or_locked());
            std::mem::drop(guard3);
            assert_eq!(0, pool.num_entries_or_locked());
        }
    }

    mod concurrent {
        use super::*;

        #[tokio::test]
        async fn async_lock() {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            let guard = pool.async_lock(5).await;

            let counter = Arc::new(AtomicU32::new(0));

            let child = launch_thread_async_lock(&pool, 5, &counter, None);

            // Check that even if we wait, the child thread won't get the lock
            thread::sleep(Duration::from_millis(100));
            assert_eq!(0, counter.load(Ordering::SeqCst));

            // Check that we can still lock other locks while the child is waiting
            {
                let _g = pool.async_lock(4).await;
            }

            // Now free the lock so the child can get it
            std::mem::drop(guard);

            // And check that the child got it
            child.join().unwrap();
            assert_eq!(1, counter.load(Ordering::SeqCst));

            assert_eq!(0, pool.num_entries_or_locked());
        }

        #[tokio::test]
        async fn async_lock_owned() {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            let guard = pool.async_lock_owned(5).await;

            let counter = Arc::new(AtomicU32::new(0));

            let child = launch_thread_async_lock_owned(&pool, 5, &counter, None);

            // Check that even if we wait, the child thread won't get the lock
            thread::sleep(Duration::from_millis(100));
            assert_eq!(0, counter.load(Ordering::SeqCst));

            // Check that we can still lock other locks while the child is waiting
            {
                let _g = pool.async_lock_owned(4).await;
            }

            // Now free the lock so the child can get it
            std::mem::drop(guard);

            // And check that the child got it
            child.join().unwrap();
            assert_eq!(1, counter.load(Ordering::SeqCst));

            assert_eq!(0, pool.num_entries_or_locked());
        }

        #[test]
        fn blocking_lock() {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            let guard = pool.blocking_lock(5);

            let counter = Arc::new(AtomicU32::new(0));

            let child = launch_thread_blocking_lock(&pool, 5, &counter, None);

            // Check that even if we wait, the child thread won't get the lock
            thread::sleep(Duration::from_millis(100));
            assert_eq!(0, counter.load(Ordering::SeqCst));

            // Check that we can still lock other locks while the child is waiting
            {
                let _g = pool.blocking_lock(4);
            }

            // Now free the lock so the child can get it
            std::mem::drop(guard);

            // And check that the child got it
            child.join().unwrap();
            assert_eq!(1, counter.load(Ordering::SeqCst));

            assert_eq!(0, pool.num_entries_or_locked());
        }

        #[test]
        fn blocking_lock_owned() {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            let guard = pool.blocking_lock_owned(5);

            let counter = Arc::new(AtomicU32::new(0));

            let child = launch_thread_blocking_lock_owned(&pool, 5, &counter, None);

            // Check that even if we wait, the child thread won't get the lock
            thread::sleep(Duration::from_millis(100));
            assert_eq!(0, counter.load(Ordering::SeqCst));

            // Check that we can still lock other locks while the child is waiting
            {
                let _g = pool.blocking_lock_owned(4);
            }

            // Now free the lock so the child can get it
            std::mem::drop(guard);

            // And check that the child got it
            child.join().unwrap();
            assert_eq!(1, counter.load(Ordering::SeqCst));

            assert_eq!(0, pool.num_entries_or_locked());
        }

        #[test]
        fn try_lock() {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            let guard = pool.try_lock(5).unwrap();

            let counter = Arc::new(AtomicU32::new(0));

            let child = launch_thread_try_lock(&pool, 5, &counter, None);

            // Check that even if we wait, the child thread won't get the lock
            thread::sleep(Duration::from_millis(100));
            assert_eq!(0, counter.load(Ordering::SeqCst));

            // Check that we can still lock other locks while the child is waiting
            {
                let _g = pool.try_lock(4).unwrap();
            }

            // Now free the lock so the child can get it
            std::mem::drop(guard);

            // And check that the child got it
            child.join().unwrap();
            assert_eq!(1, counter.load(Ordering::SeqCst));

            assert_eq!(0, pool.num_entries_or_locked());
        }

        #[test]
        fn try_lock_owned() {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            let guard = pool.try_lock_owned(5).unwrap();

            let counter = Arc::new(AtomicU32::new(0));

            let child = launch_thread_try_lock_owned(&pool, 5, &counter, None);

            // Check that even if we wait, the child thread won't get the lock
            thread::sleep(Duration::from_millis(100));
            assert_eq!(0, counter.load(Ordering::SeqCst));

            // Check that we can still lock other locks while the child is waiting
            {
                let _g = pool.try_lock_owned(4).unwrap();
            }

            // Now free the lock so the child can get it
            std::mem::drop(guard);

            // And check that the child got it
            child.join().unwrap();
            assert_eq!(1, counter.load(Ordering::SeqCst));

            assert_eq!(0, pool.num_entries_or_locked());
        }
    }

    mod multi_concurrent {
        use super::*;

        #[tokio::test]
        async fn async_lock() {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            let guard = pool.async_lock(5).await;

            let counter = Arc::new(AtomicU32::new(0));
            let barrier = Arc::new(Mutex::new(()));
            let barrier_guard = barrier.lock().unwrap();

            let child1 = launch_thread_async_lock(&pool, 5, &counter, Some(&barrier));
            let child2 = launch_thread_async_lock(&pool, 5, &counter, Some(&barrier));

            // Check that even if we wait, the child thread won't get the lock
            thread::sleep(Duration::from_millis(100));
            assert_eq!(0, counter.load(Ordering::SeqCst));

            // Check that we can stil lock other locks while the children are waiting
            {
                let _g = pool.async_lock(4).await;
            }

            // Now free the lock so a child can get it
            std::mem::drop(guard);

            // Check that a child got it
            thread::sleep(Duration::from_millis(100));
            assert_eq!(1, counter.load(Ordering::SeqCst));

            // Allow the child to free the lock
            std::mem::drop(barrier_guard);

            // Check that the other child got it
            child1.join().unwrap();
            child2.join().unwrap();
            assert_eq!(2, counter.load(Ordering::SeqCst));

            assert_eq!(0, pool.num_entries_or_locked());
        }

        #[tokio::test]
        async fn async_lock_owned() {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            let guard = pool.async_lock_owned(5).await;

            let counter = Arc::new(AtomicU32::new(0));
            let barrier = Arc::new(Mutex::new(()));
            let barrier_guard = barrier.lock().unwrap();

            let child1 = launch_thread_async_lock_owned(&pool, 5, &counter, Some(&barrier));
            let child2 = launch_thread_async_lock_owned(&pool, 5, &counter, Some(&barrier));

            // Check that even if we wait, the child thread won't get the lock
            thread::sleep(Duration::from_millis(100));
            assert_eq!(0, counter.load(Ordering::SeqCst));

            // Check that we can stil lock other locks while the children are waiting
            {
                let _g = pool.async_lock_owned(4).await;
            }

            // Now free the lock so a child can get it
            std::mem::drop(guard);

            // Check that a child got it
            thread::sleep(Duration::from_millis(100));
            assert_eq!(1, counter.load(Ordering::SeqCst));

            // Allow the child to free the lock
            std::mem::drop(barrier_guard);

            // Check that the other child got it
            child1.join().unwrap();
            child2.join().unwrap();
            assert_eq!(2, counter.load(Ordering::SeqCst));

            assert_eq!(0, pool.num_entries_or_locked());
        }

        #[test]
        fn blocking_lock() {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            let guard = pool.blocking_lock(5);

            let counter = Arc::new(AtomicU32::new(0));
            let barrier = Arc::new(Mutex::new(()));
            let barrier_guard = barrier.lock().unwrap();

            let child1 = launch_thread_blocking_lock(&pool, 5, &counter, Some(&barrier));
            let child2 = launch_thread_blocking_lock(&pool, 5, &counter, Some(&barrier));

            // Check that even if we wait, the child thread won't get the lock
            thread::sleep(Duration::from_millis(100));
            assert_eq!(0, counter.load(Ordering::SeqCst));

            // Check that we can stil lock other locks while the children are waiting
            {
                let _g = pool.blocking_lock(4);
            }

            // Now free the lock so a child can get it
            std::mem::drop(guard);

            // Check that a child got it
            thread::sleep(Duration::from_millis(100));
            assert_eq!(1, counter.load(Ordering::SeqCst));

            // Allow the child to free the lock
            std::mem::drop(barrier_guard);

            // Check that the other child got it
            child1.join().unwrap();
            child2.join().unwrap();
            assert_eq!(2, counter.load(Ordering::SeqCst));

            assert_eq!(0, pool.num_entries_or_locked());
        }

        #[test]
        fn blocking_lock_owned() {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            let guard = pool.blocking_lock_owned(5);

            let counter = Arc::new(AtomicU32::new(0));
            let barrier = Arc::new(Mutex::new(()));
            let barrier_guard = barrier.lock().unwrap();

            let child1 = launch_thread_blocking_lock_owned(&pool, 5, &counter, Some(&barrier));
            let child2 = launch_thread_blocking_lock_owned(&pool, 5, &counter, Some(&barrier));

            // Check that even if we wait, the child thread won't get the lock
            thread::sleep(Duration::from_millis(100));
            assert_eq!(0, counter.load(Ordering::SeqCst));

            // Check that we can stil lock other locks while the children are waiting
            {
                let _g = pool.blocking_lock_owned(4);
            }

            // Now free the lock so a child can get it
            std::mem::drop(guard);

            // Check that a child got it
            thread::sleep(Duration::from_millis(100));
            assert_eq!(1, counter.load(Ordering::SeqCst));

            // Allow the child to free the lock
            std::mem::drop(barrier_guard);

            // Check that the other child got it
            child1.join().unwrap();
            child2.join().unwrap();
            assert_eq!(2, counter.load(Ordering::SeqCst));

            assert_eq!(0, pool.num_entries_or_locked());
        }

        #[test]
        fn try_lock() {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            let guard = pool.try_lock(5).unwrap();

            let counter = Arc::new(AtomicU32::new(0));
            let barrier = Arc::new(Mutex::new(()));
            let barrier_guard = barrier.lock().unwrap();

            let child1 = launch_thread_try_lock(&pool, 5, &counter, Some(&barrier));
            let child2 = launch_thread_try_lock(&pool, 5, &counter, Some(&barrier));

            // Check that even if we wait, the child thread won't get the lock
            thread::sleep(Duration::from_millis(100));
            assert_eq!(0, counter.load(Ordering::SeqCst));

            // Check that we can still lock other locks while the children are waiting
            {
                let _g = pool.try_lock(4).unwrap();
            }

            // Now free the lock so a child can get it
            std::mem::drop(guard);

            // Check that a child got it
            thread::sleep(Duration::from_millis(100));
            assert_eq!(1, counter.load(Ordering::SeqCst));

            // Allow the child to free the lock
            std::mem::drop(barrier_guard);

            // Check that the other child got it
            child1.join().unwrap();
            child2.join().unwrap();
            assert_eq!(2, counter.load(Ordering::SeqCst));

            assert_eq!(0, pool.num_entries_or_locked());
        }

        #[test]
        fn try_lock_owned() {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            let guard = pool.try_lock_owned(5).unwrap();

            let counter = Arc::new(AtomicU32::new(0));
            let barrier = Arc::new(Mutex::new(()));
            let barrier_guard = barrier.lock().unwrap();

            let child1 = launch_thread_try_lock_owned(&pool, 5, &counter, Some(&barrier));
            let child2 = launch_thread_try_lock_owned(&pool, 5, &counter, Some(&barrier));

            // Check that even if we wait, the child thread won't get the lock
            thread::sleep(Duration::from_millis(100));
            assert_eq!(0, counter.load(Ordering::SeqCst));

            // Check that we can stil lock other locks while the children are waiting
            {
                let _g = pool.try_lock_owned(4).unwrap();
            }

            // Now free the lock so a child can get it
            std::mem::drop(guard);

            // Check that a child got it
            thread::sleep(Duration::from_millis(100));
            assert_eq!(1, counter.load(Ordering::SeqCst));

            // Allow the child to free the lock
            std::mem::drop(barrier_guard);

            // Check that the other child got it
            child1.join().unwrap();
            child2.join().unwrap();
            assert_eq!(2, counter.load(Ordering::SeqCst));

            assert_eq!(0, pool.num_entries_or_locked());
        }
    }

    #[test]
    fn blocking_lock_owned_guards_can_be_passed_around() {
        let make_guard = || {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            pool.blocking_lock_owned(5)
        };
        let _guard = make_guard();
    }

    #[tokio::test]
    async fn async_lock_owned_guards_can_be_passed_around() {
        let make_guard = || async {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            pool.async_lock_owned(5).await
        };
        let _guard = make_guard().await;
    }

    #[test]
    fn test_try_lock_owned_guards_can_be_passed_around() {
        let make_guard = || {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            pool.try_lock_owned(5)
        };
        let guard = make_guard();
        assert!(guard.is_ok());
    }

    #[tokio::test]
    async fn async_lock_guards_can_be_held_across_await_points() {
        let task = async {
            let pool = LockableCache::<isize, String>::new();
            let guard = pool.async_lock(3).await;
            tokio::time::sleep(Duration::from_millis(10)).await;
            std::mem::drop(guard);
        };

        // We also need to move the task to a different thread because
        // only then the compiler checks whether the task is Send.
        thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(task);
        });
    }

    #[tokio::test]
    async fn async_lock_owned_guards_can_be_held_across_await_points() {
        let task = async {
            let pool = Arc::new(LockableCache::<isize, String>::new());
            let guard = pool.async_lock_owned(3).await;
            tokio::time::sleep(Duration::from_millis(10)).await;
            std::mem::drop(guard);
        };

        // We also need to move the task to a different thread because
        // only then the compiler checks whether the task is Send.
        thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(task);
        });
    }
}
