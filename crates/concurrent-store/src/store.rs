use anyhow::Result;
use async_trait::async_trait;
use futures::FutureExt as _;
use futures::future::{BoxFuture, Shared};
use lockable::{InfallibleUnwrap as _, Never};
use std::collections::hash_map::Entry;
use std::{collections::HashMap, fmt::Debug, hash::Hash, sync::Mutex};

use crate::Inserting;
use crate::LoadingOrLoaded;
use crate::entry::EntryState;
use crate::entry::{
    EntryLoadingWaiter, EntryStateDropping, EntryStateLoaded, EntryStateLoading, Intent,
    LoadingResult, ReloadInfo, RequestImmediateDropResponse,
};
use crate::guard::LoadedEntryGuard;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};
use cryfs_utils::event::Event;
use cryfs_utils::stream::for_each_unordered;
use cryfs_utils::with_async_drop_2;

// TODO This is currently not cancellation safe. If a task waiting for an entry to load is cancelled, the num_waiters and num_unfulfilled_waiters counts will be wrong.

/// A concurrent store allows loading objects from an underlying store, while handling concurrency.
///
/// If the same entry is loaded multiple times, only the first loading operation is executed, and any further tasks will just receive a reference to the same loaded object.
/// Once the last task releases its reference, the object is asynchronously dropped.
///
/// If any tasks want to load it while it is being dropped, the state machine will queue a reload
/// that executes after the drop completes, allowing the caller to get a waiter immediately.
///
/// # Atomicity Guarantee
///
/// All public methods on this store are **synchronous** and update state atomically under a mutex.
/// This means that if you call one operation and then immediately call another operation on the
/// same thread (without awaiting the returned future), the second operation will immediately see
/// the state changes made by the first operation.
///
/// For example:
/// - After calling `get_loaded_or_insert_loading()`, a subsequent call to `get_if_loading_or_loaded()`
///   will see the entry in Loading state, even before the loading future is awaited.
/// - request_immediate_drop immediately changes the entry into a "drop requested" state and later operations
///   will treat the entry as if it is already dropped (e.g. it get_if_loading_or_loaded will not return it,
///   get_loaded_or_insert_loading will schedule a new loading operation).
/// - Exception: is_fully_absent() will only return true if the item is not currently being dropped.
/// - After executing async_drop().await on the last LoadedEntryGuard for an entry, dropping has completed
///   and the entry is now fully absent (i.e. `is_fully_absent()` returns true).
///
/// This guarantee is essential for correct concurrent behavior and is enforced by the mutex
/// protecting the internal state.
///
/// # Parameters
///
/// * K: Key type for the entries in the store.
/// * V: Value type for the entries in the store.
/// * E: Error type for loading operations.
pub struct ConcurrentStore<K, V, E>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    V: AsyncDrop + Debug + Send + Sync + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    inner: AsyncDropGuard<AsyncDropArc<ConcurrentStoreInner<K, V, E>>>,
}

pub(super) struct ConcurrentStoreInner<K, V, E>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    V: AsyncDrop + Debug + Send + Sync + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    /// The state machine for entries has 3 physical states:
    /// - Loading: Entry is being loaded
    /// - Loaded: Entry is loaded and available
    /// - Dropping: Entry is being dropped (async drop in progress)
    ///
    /// Each state can have an optional `intent` (or `reload` for Dropping) that indicates
    /// future operations to perform. See [Intent] and [ReloadInfo] for details.
    entries: Mutex<HashMap<K, EntryState<V, E>>>,
}

impl<K, V, E> ConcurrentStore<K, V, E>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    V: AsyncDrop + Debug + Send + Sync + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    /// Create a new empty [ConcurrentStore].
    pub fn new() -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(ConcurrentStore {
            inner: AsyncDropArc::new(AsyncDropGuard::new(ConcurrentStoreInner {
                entries: Mutex::new(HashMap::new()),
            })),
        })
    }

    pub fn clone_ref(&self) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(ConcurrentStore {
            inner: AsyncDropArc::clone(&self.inner),
        })
    }

    /// Try to insert a new entry by loading it using the provided loading function.
    /// If an entry with the same key is already loaded or in any state, an error is returned.
    /// The loading function is only called if no other entry with the same key exists.
    ///
    /// This is a sync function that returns immediately. The actual loading happens
    /// when the caller awaits the returned [Inserting].
    ///
    /// The Loading state is immediately visible to subsequent calls (e.g., `get_if_loading_or_loaded`
    /// will see it), even before the returned future is awaited.
    ///
    /// Warning: This function has an exception to the atomicity guarantee. If an entry has been loaded before
    /// and is currently dropping, i.e. dropping hasn't completed yet, it will not succeed the insert operation but will fail.
    pub fn try_insert_loading<F>(
        &self,
        key: K,
        loading_fn: impl FnOnce() -> F + Send + 'static,
    ) -> Result<Inserting<K, V, E>>
    where
        F: Future<Output = Result<AsyncDropGuard<V>, E>> + Send,
    {
        let mut entries = self.inner.entries.lock().unwrap();
        match entries.entry(key) {
            Entry::Occupied(entry) => match entry.get() {
                EntryState::Loading(_) | EntryState::Loaded(_) => Err(anyhow::anyhow!(
                    "Key {key:?} is already loading or loaded",
                    key = entry.key()
                )),
                EntryState::Dropping(_) => {
                    // Per design decision: error immediately when encountering Dropping - matches "try" semantics
                    Err(anyhow::anyhow!(
                        "Key {key:?} is currently dropping",
                        key = entry.key()
                    ))
                }
            },
            Entry::Vacant(entry) => {
                let loading_future = async move {
                    let loaded_entry = loading_fn().await?;
                    Ok(Some(loaded_entry))
                };
                let mut loading_future =
                    self.make_loading_future(entry.key().clone(), loading_future);
                let loading_result = loading_future.add_waiter(entry.key().clone());
                entry.insert(EntryState::Loading(loading_future));
                Ok(Inserting::new(
                    AsyncDropArc::clone(&self.inner),
                    loading_result,
                ))
            }
        }
    }

    /// Insert a new entry that was just created and has a new key assigned.
    /// This will return an Error if the key already exists in any state.
    ///
    /// On error, the value is returned to the caller, who is responsible for async_drop.
    ///
    /// The Loaded state is immediately visible to subsequent calls (e.g., `get_if_loading_or_loaded`
    /// will see it).
    ///
    /// Warning: This function has an exception to the atomicity guarantee. If an entry has been loaded before
    /// and is currently dropping, i.e. dropping hasn't completed yet, it will not succeed the insert operation but will fail.
    pub fn try_insert_loaded(
        &self,
        key: K,
        value: AsyncDropGuard<V>,
    ) -> Result<AsyncDropGuard<LoadedEntryGuard<K, V, E>>, AsyncDropGuard<V>> {
        let mut entries = self.inner.entries.lock().unwrap();
        match entries.entry(key.clone()) {
            Entry::Occupied(_) => {
                // Entry exists in some state - return value to caller
                Err(value)
            }
            Entry::Vacant(entry) => {
                let loaded = EntryStateLoaded::new_without_unfulfilled_waiters(value);
                // No unfulfilled waiters, we just created it
                let loaded_entry = loaded.get_entry();

                let key = entry.key().clone();
                entry.insert(EntryState::Loaded(loaded));
                Ok(LoadedEntryGuard::new(
                    AsyncDropArc::clone(&self.inner),
                    key,
                    loaded_entry,
                ))
            }
        }
    }

    /// Load an entry if it is not already loaded, or return the existing loaded entry.
    /// This function is synchronous and returns immediately - the actual loading happens
    /// when the caller awaits the returned [LoadingOrLoaded].
    ///
    /// The state change (to Loading if starting a new load, or staying at Loaded/Loading if
    /// joining an existing one) is immediately visible to subsequent calls, even before the
    /// returned future is awaited.
    pub fn get_loaded_or_insert_loading<'a, F, I>(
        &self,
        key: K,
        loading_fn_input: &AsyncDropGuard<AsyncDropArc<I>>,
        loading_fn: impl FnOnce(AsyncDropGuard<AsyncDropArc<I>>) -> F + Send + 'static,
    ) -> LoadingOrLoaded<K, V, E>
    where
        F: Future<Output = Result<Option<AsyncDropGuard<V>>, E>> + Send + 'static,
        I: AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    {
        self._clone_or_create_entry_state(key.clone(), loading_fn_input, loading_fn)
    }

    /// Check if an entry is either loading or loaded, and if yes return it.
    /// If the entry is not loading or loaded, return None.
    ///
    /// This will see Loading/Loaded states set by prior calls (e.g., `get_loaded_or_insert_loading`
    /// or `try_insert_loading`), even if those calls' futures haven't been awaited yet.
    ///
    /// If the entry has a drop intent but also has a reload scheduled (via `get_loaded_or_insert_loading`),
    /// this method will return a waiter for that reload rather than None.
    pub fn get_if_loading_or_loaded(&self, key: K) -> LoadingOrLoaded<K, V, E> {
        let mut entries = self.inner.entries.lock().unwrap();
        match entries.get_mut(&key) {
            Some(EntryState::Loaded(loaded)) => {
                match loaded.intent_mut() {
                    None => {
                        // No intent - return the loaded entry
                        LoadingOrLoaded::new_loaded(LoadedEntryGuard::new(
                            AsyncDropArc::clone(&self.inner),
                            key,
                            loaded.get_entry(),
                        ))
                    }
                    Some(intent) => {
                        // Has intent - walk the chain to find any existing reload
                        match Self::walk_chain_for_existing_reload(key.clone(), intent) {
                            Some(waiter) => LoadingOrLoaded::new_loading(
                                AsyncDropArc::clone(&self.inner),
                                waiter,
                            ),
                            None => LoadingOrLoaded::new_not_found(),
                        }
                    }
                }
            }
            Some(EntryState::Loading(loading)) => {
                match loading.intent_mut() {
                    None => {
                        // No intent - add waiter to this loading
                        LoadingOrLoaded::new_loading(
                            AsyncDropArc::clone(&self.inner),
                            loading.add_waiter(key),
                        )
                    }
                    Some(intent) => {
                        // Has intent - walk the chain to find any existing reload
                        match Self::walk_chain_for_existing_reload(key.clone(), intent) {
                            Some(waiter) => LoadingOrLoaded::new_loading(
                                AsyncDropArc::clone(&self.inner),
                                waiter,
                            ),
                            None => LoadingOrLoaded::new_not_found(),
                        }
                    }
                }
            }
            Some(EntryState::Dropping(dropping)) => {
                // Check if there's a reload scheduled
                match dropping.reload_mut() {
                    Some(reload) => {
                        // Has reload - walk the chain to find where to add a waiter
                        let waiter =
                            Self::walk_reload_chain_for_existing_waiter(key.clone(), reload);
                        LoadingOrLoaded::new_loading(AsyncDropArc::clone(&self.inner), waiter)
                    }
                    None => LoadingOrLoaded::new_not_found(),
                }
            }
            None => LoadingOrLoaded::new_not_found(),
        }
    }

    /// Return all entries that are loading or loaded.
    ///
    /// If an entry has a drop intent but also has a reload scheduled (via `get_loaded_or_insert_loading`),
    /// this method will include a waiter for that reload rather than excluding the entry.
    pub fn all_loading_or_loaded(&self) -> Vec<LoadingOrLoaded<K, V, E>> {
        let mut entries = self.inner.entries.lock().unwrap();
        let mut result = Vec::with_capacity(entries.len());
        for (key, entry_state) in entries.iter_mut() {
            match entry_state {
                EntryState::Loaded(loaded) => {
                    match loaded.intent_mut() {
                        None => {
                            // No intent - return the loaded entry
                            result.push(LoadingOrLoaded::new_loaded(LoadedEntryGuard::new(
                                AsyncDropArc::clone(&self.inner),
                                key.clone(),
                                loaded.get_entry(),
                            )));
                        }
                        Some(intent) => {
                            // Has intent - walk the chain to find any existing reload
                            if let Some(waiter) =
                                Self::walk_chain_for_existing_reload(key.clone(), intent)
                            {
                                result.push(LoadingOrLoaded::new_loading(
                                    AsyncDropArc::clone(&self.inner),
                                    waiter,
                                ));
                            }
                            // If no reload found, skip this entry
                        }
                    }
                }
                EntryState::Loading(loading) => {
                    match loading.intent_mut() {
                        None => {
                            // No intent - add waiter to this loading
                            result.push(LoadingOrLoaded::new_loading(
                                AsyncDropArc::clone(&self.inner),
                                loading.add_waiter(key.clone()),
                            ));
                        }
                        Some(intent) => {
                            // Has intent - walk the chain to find any existing reload
                            if let Some(waiter) =
                                Self::walk_chain_for_existing_reload(key.clone(), intent)
                            {
                                result.push(LoadingOrLoaded::new_loading(
                                    AsyncDropArc::clone(&self.inner),
                                    waiter,
                                ));
                            }
                            // If no reload found, skip this entry
                        }
                    }
                }
                EntryState::Dropping(dropping) => {
                    // Check if there's a reload scheduled
                    if let Some(reload) = dropping.reload_mut() {
                        // Has reload - walk the chain to find where to add a waiter
                        let waiter =
                            Self::walk_reload_chain_for_existing_waiter(key.clone(), reload);
                        result.push(LoadingOrLoaded::new_loading(
                            AsyncDropArc::clone(&self.inner),
                            waiter,
                        ));
                    }
                    // If no reload, skip this entry
                }
            }
        }
        result
    }

    /// Returns `true` if the entry with the given key is completely absent from the store.
    ///
    /// Returns `false` if the entry is in any state (Loading, Loaded, or Dropping).
    /// This reflects the current state, including pending operations from prior calls
    /// whose futures haven't been awaited yet.
    pub fn is_fully_absent(&self, key: &K) -> bool {
        let entries = self.inner.entries.lock().unwrap();
        !entries.contains_key(key)
    }

    #[cfg(any(test, feature = "testutils"))]
    pub fn is_empty(&self) -> bool {
        let entries = self.inner.entries.lock().unwrap();
        entries.is_empty()
    }

    /// Get or create an entry state. This function implements the chain walking algorithm
    /// for get_loaded_or_insert_loading - it walks to the deepest level of the intent/reload
    /// chain and either adds a waiter or sets a new reload.
    fn _clone_or_create_entry_state<'s, F, R, I>(
        &'s self,
        key: K,
        loading_fn_input: &AsyncDropGuard<AsyncDropArc<I>>,
        loading_fn: F,
    ) -> LoadingOrLoaded<K, V, E>
    where
        F: FnOnce(AsyncDropGuard<AsyncDropArc<I>>) -> R + Send + 'static,
        R: Future<Output = Result<Option<AsyncDropGuard<V>>, E>> + Send + 'static,
        I: AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    {
        let mut entries = self.inner.entries.lock().unwrap();
        match entries.entry(key.clone()) {
            Entry::Occupied(mut entry) => match entry.get_mut() {
                EntryState::Loaded(loaded) => {
                    match loaded.intent_mut() {
                        None => {
                            // No intent - just return the entry
                            LoadingOrLoaded::new_loaded(LoadedEntryGuard::new(
                                AsyncDropArc::clone(&self.inner),
                                key,
                                loaded.get_entry(),
                            ))
                        }
                        Some(intent) => {
                            // Has intent - walk the chain
                            let waiter = self.walk_chain_for_reload(
                                key.clone(),
                                intent,
                                loading_fn_input,
                                loading_fn,
                            );
                            LoadingOrLoaded::new_loading(AsyncDropArc::clone(&self.inner), waiter)
                        }
                    }
                }
                EntryState::Loading(loading) => {
                    match loading.intent_mut() {
                        None => {
                            // No intent - just add a waiter
                            LoadingOrLoaded::new_loading(
                                AsyncDropArc::clone(&self.inner),
                                loading.add_waiter(key),
                            )
                        }
                        Some(intent) => {
                            // Has intent - walk the chain
                            let waiter = self.walk_chain_for_reload(
                                key.clone(),
                                intent,
                                loading_fn_input,
                                loading_fn,
                            );
                            LoadingOrLoaded::new_loading(AsyncDropArc::clone(&self.inner), waiter)
                        }
                    }
                }
                EntryState::Dropping(dropping) => {
                    match dropping.reload_mut() {
                        None => {
                            // No reload - set one
                            let reload_future = self.make_reload_future(
                                key.clone(),
                                dropping.on_dropped().clone(),
                                loading_fn_input,
                                loading_fn,
                            );
                            let reload = ReloadInfo::new(reload_future.clone());
                            let waiter = EntryLoadingWaiter::new(
                                key.clone(),
                                reload.reload_future().clone(),
                            );
                            dropping.set_reload(reload);
                            LoadingOrLoaded::new_loading(AsyncDropArc::clone(&self.inner), waiter)
                        }
                        Some(reload) => {
                            // Has reload - walk the chain
                            let waiter = self.walk_reload_chain_for_reload(
                                key.clone(),
                                reload,
                                loading_fn_input,
                                loading_fn,
                            );
                            LoadingOrLoaded::new_loading(AsyncDropArc::clone(&self.inner), waiter)
                        }
                    }
                }
            },
            Entry::Vacant(entry) => {
                // No loading operation is in progress, so we start a new one.
                let mut loading_future = self.make_loading_future(
                    key.clone(),
                    loading_fn(AsyncDropArc::clone(loading_fn_input)),
                );
                let loading_result = loading_future.add_waiter(key);
                entry.insert(EntryState::Loading(loading_future));
                LoadingOrLoaded::new_loading(AsyncDropArc::clone(&self.inner), loading_result)
            }
        }
    }

    /// Walk the intent chain to find where to set a reload or add a waiter.
    fn walk_chain_for_reload<F, R, I>(
        &self,
        key: K,
        intent: &mut Intent<V, E>,
        loading_fn_input: &AsyncDropGuard<AsyncDropArc<I>>,
        loading_fn: F,
    ) -> EntryLoadingWaiter<K, E>
    where
        F: FnOnce(AsyncDropGuard<AsyncDropArc<I>>) -> R + Send + 'static,
        R: Future<Output = Result<Option<AsyncDropGuard<V>>, E>> + Send + 'static,
        I: AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    {
        match intent.reload_mut() {
            None => {
                // No reload - set one
                let reload_future = self.make_reload_future(
                    key.clone(),
                    intent.on_dropped().clone(),
                    loading_fn_input,
                    loading_fn,
                );
                let reload = ReloadInfo::new(reload_future.clone());
                let waiter = EntryLoadingWaiter::new(key, reload.reload_future().clone());
                intent.set_reload(reload);
                waiter
            }
            Some(reload) => {
                // Has reload - continue walking
                self.walk_reload_chain_for_reload(key, reload, loading_fn_input, loading_fn)
            }
        }
    }

    /// Walk the reload chain to find where to set a reload or add a waiter.
    /// Iteratively walks through reload→intent→reload→... to find the deepest level.
    fn walk_reload_chain_for_reload<F, R, I>(
        &self,
        key: K,
        mut reload: &mut ReloadInfo<V, E>,
        loading_fn_input: &AsyncDropGuard<AsyncDropArc<I>>,
        loading_fn: F,
    ) -> EntryLoadingWaiter<K, E>
    where
        F: FnOnce(AsyncDropGuard<AsyncDropArc<I>>) -> R + Send + 'static,
        R: Future<Output = Result<Option<AsyncDropGuard<V>>, E>> + Send + 'static,
        I: AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    {
        // Walk to the deepest reload in the chain
        while reload.has_deeper_reload() {
            reload = reload
                .new_intent_mut()
                .expect("has_deeper_reload returned true")
                .reload_mut()
                .expect("has_deeper_reload returned true");
        }
        // Now we're at the deepest reload. Check if it has an intent without reload.
        if reload.has_new_intent() {
            // Intent exists but has no reload - create one there
            let next_intent = reload
                .new_intent_mut()
                .expect("has_new_intent returned true");
            let reload_future = self.make_reload_future(
                key.clone(),
                next_intent.on_dropped().clone(),
                loading_fn_input,
                loading_fn,
            );
            let new_reload = ReloadInfo::new(reload_future.clone());
            let waiter = EntryLoadingWaiter::new(key, new_reload.reload_future().clone());
            next_intent.set_reload(new_reload);
            waiter
        } else {
            // No new intent - add waiter to this reload
            let future = reload.add_waiter();
            EntryLoadingWaiter::new(key, future)
        }
    }

    /// Walk the intent chain to find an existing reload (without creating a new one).
    /// Used by `get_if_loading_or_loaded` to find scheduled reloads.
    /// Returns None if no reload exists in the chain.
    fn walk_chain_for_existing_reload(
        key: K,
        intent: &mut Intent<V, E>,
    ) -> Option<EntryLoadingWaiter<K, E>> {
        // First check if the intent has a reload
        let reload = intent.reload_mut()?;
        // If yes, walk the reload chain iteratively
        Some(Self::walk_reload_chain_for_existing_waiter(key, reload))
    }

    /// Walk the reload chain to find where to add a waiter for an existing reload.
    /// Used by `get_if_loading_or_loaded` when a reload is already scheduled.
    /// Iteratively walks through reload→intent→reload→... to find the deepest reload.
    fn walk_reload_chain_for_existing_waiter(
        key: K,
        mut reload: &mut ReloadInfo<V, E>,
    ) -> EntryLoadingWaiter<K, E> {
        // Walk to the deepest reload in the chain
        while reload.has_deeper_reload() {
            reload = reload
                .new_intent_mut()
                .expect("has_deeper_reload returned true")
                .reload_mut()
                .expect("has_deeper_reload returned true");
        }
        // Add waiter to the deepest reload
        let future = reload.add_waiter();
        EntryLoadingWaiter::new(key, future)
    }

    /// Create a reload future that waits for on_dropped, then loads the entry.
    fn make_reload_future<F, R, I>(
        &self,
        key: K,
        on_dropped: Event,
        loading_fn_input: &AsyncDropGuard<AsyncDropArc<I>>,
        loading_fn: F,
    ) -> Shared<BoxFuture<'static, LoadingResult<E>>>
    where
        F: FnOnce(AsyncDropGuard<AsyncDropArc<I>>) -> R + Send + 'static,
        R: Future<Output = Result<Option<AsyncDropGuard<V>>, E>> + Send + 'static,
        I: AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    {
        let inner = AsyncDropArc::clone(&self.inner);
        let loading_fn_input = AsyncDropArc::clone(loading_fn_input);
        let reload_task = async move {
            // Wait for the drop to complete
            on_dropped.wait().await;

            // Now load the entry
            with_async_drop_2!(inner, {
                let result = loading_fn(loading_fn_input).await;

                match result {
                    Ok(Some(entry)) => {
                        // Loading succeeded. Transition to Loaded.
                        let mut entries = inner.entries.lock().unwrap();
                        let Some(state) = entries.get_mut(&key) else {
                            // Entry was removed - this can happen if reload was cancelled
                            return Ok(LoadingResult::NotFound);
                        };

                        // The entry could be in various states depending on what happened
                        // while we were loading. Find the reload we belong to and consume it.
                        match state {
                            EntryState::Dropping(dropping) => {
                                // We're the first reload after the drop
                                let reload = dropping.take_reload().expect(
                                    "Expected reload in Dropping state after reload completes",
                                );
                                let (_, num_waiters, new_intent) = reload.into_parts();
                                *state = EntryState::Loaded(EntryStateLoaded::new_from_reload(
                                    entry,
                                    num_waiters,
                                    new_intent.map(|b| *b),
                                ));
                            }
                            EntryState::Loading(loading) => {
                                // This shouldn't happen in normal flow, but handle gracefully
                                let loading =
                                    std::mem::replace(loading, EntryStateLoading::new_dummy());
                                *state = EntryState::Loaded(
                                    EntryStateLoaded::new_from_just_finished_loading(
                                        entry, loading,
                                    ),
                                );
                            }
                            _ => {
                                panic!(
                                    "Unexpected state {:?} after reload completes for key {:?}",
                                    state, key
                                );
                            }
                        }

                        Ok(LoadingResult::Loaded)
                    }
                    Ok(None) => {
                        // Loading found nothing. Remove the entry.
                        let mut entries = inner.entries.lock().unwrap();
                        entries.remove(&key);
                        Ok(LoadingResult::NotFound)
                    }
                    Err(err) => {
                        // Loading failed. Remove the entry.
                        let mut entries = inner.entries.lock().unwrap();
                        entries.remove(&key);
                        Ok(LoadingResult::Error(err))
                    }
                }
            })
            .infallible_unwrap()
        };
        reload_task.boxed().shared()
    }

    /// Create a loading future that will load the entry using the provided loading function, and update the entry state upon completion.
    fn make_loading_future(
        &self,
        key: K,
        loading_fn: impl Future<Output = Result<Option<AsyncDropGuard<V>>, E>> + Send + 'static,
    ) -> EntryStateLoading<V, E> {
        let inner = AsyncDropArc::clone(&self.inner);
        let loading_task = async move {
            with_async_drop_2!(inner, {
                // Run loading_fn concurrently, without a lock on `entries`.
                let result = loading_fn.await;

                match result {
                    Ok(Some(entry)) => {
                        // Now that we're loaded, change the entry state in the map
                        let mut entries = inner.entries.lock().unwrap();
                        let Some(state) = entries.get_mut(&key) else {
                            panic!("Entry with key {:?} was not found in the map", key);
                        };
                        let EntryState::Loading(loading) = state else {
                            panic!("Entry with key {:?} is not in loading state", key);
                        };
                        let loading = std::mem::replace(loading, EntryStateLoading::new_dummy());
                        *state = EntryState::Loaded(
                            EntryStateLoaded::new_from_just_finished_loading(entry, loading),
                        );

                        Ok(LoadingResult::Loaded)
                    }
                    Ok(None) => {
                        // We're loaded but the entry wasn't found. Remove the entry from the map
                        let mut entries = inner.entries.lock().unwrap();
                        let Some(_) = entries.remove(&key) else {
                            panic!("Entry with key {:?} was not found in the map", key);
                        };
                        Ok(LoadingResult::NotFound)
                    }
                    Err(err) => {
                        // An error occurred while loading the entry
                        let mut entries = inner.entries.lock().unwrap();
                        let Some(_) = entries.remove(&key) else {
                            panic!("Entry with key {:?} was not found in the map", key);
                        };
                        Ok(LoadingResult::Error(err))
                    }
                }
            })
            .infallible_unwrap()
        };
        EntryStateLoading::new(loading_task.boxed())
    }

    /// Request immediate drop of the entry with the given key.
    ///
    /// This is a synchronous function that returns immediately. The entry transitions to
    /// Dropping state (or sets an intent to drop if still Loading/Loaded with references).
    /// The actual drop happens asynchronously when the returned future is awaited.
    ///
    /// The state change is immediately visible to subsequent calls. For example,
    /// `is_fully_absent()` called immediately after will return `false` because the entry
    /// is in Dropping state, even before the drop completes.
    pub fn request_immediate_drop<D, F>(
        &self,
        key: K,
        drop_fn: impl FnOnce(Option<AsyncDropGuard<V>>) -> F + Send + Sync + 'static,
    ) -> RequestImmediateDropResult<D>
    where
        D: Debug + Send + 'static,
        F: Future<Output = D> + Send + 'static,
    {
        ConcurrentStoreInner::request_immediate_drop(&self.inner, key, drop_fn)
    }

    /// Helper method that retries request_immediate_drop if the entry is already dropping.
    /// Returns once the immediate drop completes.
    pub async fn request_immediate_drop_and_wait<D, G, F>(
        &self,
        key: K,
        make_drop_fn: impl Fn() -> G + Send,
    ) -> D
    where
        D: Debug + Send + 'static,
        G: FnOnce(Option<AsyncDropGuard<V>>) -> F + Send + Sync + 'static,
        F: Future<Output = D> + Send + 'static,
    {
        loop {
            match self.request_immediate_drop(key.clone(), make_drop_fn()) {
                RequestImmediateDropResult::ImmediateDropRequested { drop_result } => {
                    return drop_result.await;
                }
                RequestImmediateDropResult::AlreadyDropping { future } => {
                    future.await;
                    continue;
                }
            }
        }
    }
}

impl<K, V, E> ConcurrentStoreInner<K, V, E>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    V: AsyncDrop + Debug + Send + Sync + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    /// Check if immediate drop was requested for the entry with the given key.
    pub(crate) fn is_immediate_drop_requested(
        this: &AsyncDropGuard<AsyncDropArc<ConcurrentStoreInner<K, V, E>>>,
        key: &K,
    ) -> bool {
        let entries = this.entries.lock().unwrap();
        match entries.get(key) {
            Some(EntryState::Loaded(loaded)) => loaded.immediate_drop_requested().is_some(),
            Some(EntryState::Loading(loading)) => loading.immediate_drop_requested().is_some(),
            Some(EntryState::Dropping(dropping)) => dropping.immediate_drop_requested().is_some(),
            None => false,
        }
    }

    /// Request immediate drop of the entry with the given key.
    pub fn request_immediate_drop<D, F>(
        this: &AsyncDropGuard<AsyncDropArc<ConcurrentStoreInner<K, V, E>>>,
        key: K,
        drop_fn: impl FnOnce(Option<AsyncDropGuard<V>>) -> F + Send + Sync + 'static,
    ) -> RequestImmediateDropResult<D>
    where
        D: Debug + Send + 'static,
        F: Future<Output = D> + Send + 'static,
    {
        let (drop_result_sender, drop_result_receiver) = tokio::sync::oneshot::channel();
        let key_clone = key.clone();
        let drop_fn = move |value| async move {
            let drop_fn_result = drop_fn(value).await;
            if let Err(err) = drop_result_sender.send(drop_fn_result) {
                log::warn!(
                    "Failed to send immediate drop result for entry with key {:?}: {:?}",
                    key_clone,
                    err
                );
            }
        };

        let mut entries = this.entries.lock().unwrap();
        match entries.entry(key) {
            Entry::Occupied(mut entry) => match entry.get_mut() {
                EntryState::Loaded(loaded) => {
                    match loaded.request_immediate_drop(drop_fn) {
                        RequestImmediateDropResponse::Requested { on_dropped } => {
                            let _ = on_dropped; // We use the oneshot channel instead
                            RequestImmediateDropResult::ImmediateDropRequested {
                                drop_result: async move {
                                    drop_result_receiver
                                        .await
                                        .expect("The sender should not be dropped")
                                }
                                .boxed(),
                            }
                        }
                        RequestImmediateDropResponse::AlreadyDropping {
                            on_current_drop_complete,
                        } => RequestImmediateDropResult::AlreadyDropping {
                            future: on_current_drop_complete,
                        },
                    }
                }
                EntryState::Loading(loading) => {
                    match loading.request_immediate_drop(drop_fn) {
                        RequestImmediateDropResponse::Requested { on_dropped } => {
                            let _ = on_dropped; // We use the oneshot channel instead
                            RequestImmediateDropResult::ImmediateDropRequested {
                                drop_result: async move {
                                    drop_result_receiver
                                        .await
                                        .expect("The sender should not be dropped")
                                }
                                .boxed(),
                            }
                        }
                        RequestImmediateDropResponse::AlreadyDropping {
                            on_current_drop_complete,
                        } => RequestImmediateDropResult::AlreadyDropping {
                            future: on_current_drop_complete,
                        },
                    }
                }
                EntryState::Dropping(dropping) => {
                    match dropping.request_immediate_drop(drop_fn) {
                        RequestImmediateDropResponse::Requested { on_dropped } => {
                            let _ = on_dropped; // We use the oneshot channel instead
                            RequestImmediateDropResult::ImmediateDropRequested {
                                drop_result: async move {
                                    drop_result_receiver
                                        .await
                                        .expect("The sender should not be dropped")
                                }
                                .boxed(),
                            }
                        }
                        RequestImmediateDropResponse::AlreadyDropping {
                            on_current_drop_complete,
                        } => RequestImmediateDropResult::AlreadyDropping {
                            future: on_current_drop_complete,
                        },
                    }
                }
            },
            Entry::Vacant(entry) => {
                // The entry is not loaded or loading. Create a dummy entry to block other tasks
                let drop_future = ConcurrentStoreInner::make_drop_future_for_unloaded_entry(
                    this,
                    entry.key().clone(),
                    drop_fn,
                );
                let dropping_state = EntryStateDropping::new(drop_future);
                let shared_future = dropping_state.future().clone();
                entry.insert(EntryState::Dropping(dropping_state));

                std::mem::drop(entries);

                RequestImmediateDropResult::ImmediateDropRequested {
                    drop_result: async move {
                        shared_future.await;
                        drop_result_receiver
                            .await
                            .expect("The sender should not be dropped")
                    }
                    .boxed(),
                }
            }
        }
    }

    /// Called by [LoadedEntryGuard] when it is dropped.
    pub(super) async fn unload(
        this: &AsyncDropGuard<AsyncDropArc<ConcurrentStoreInner<K, V, E>>>,
        key: K,
        mut entry: AsyncDropGuard<AsyncDropArc<V>>,
    ) {
        // First drop the entry to decrement the reference count
        entry.async_drop().await.unwrap(); // TODO No unwrap?
        std::mem::drop(entry);

        // Now check if we're the last reference. If yes, remove the entry from our map.
        Self::_drop_if_no_references(this, key).await;
    }

    /// Check if there are no more references to the entry with the given key, and if yes, async drop it.
    async fn _drop_if_no_references(
        this: &AsyncDropGuard<AsyncDropArc<ConcurrentStoreInner<K, V, E>>>,
        key: K,
    ) {
        let drop_future = {
            let mut entries = this.entries.lock().unwrap();
            let Entry::Occupied(mut entry) = entries.entry(key) else {
                // Entry was already removed (race condition)
                return;
            };
            match entry.get() {
                EntryState::Loading(_) => {
                    // This can happen due to race conditions - ignore
                    return;
                }
                EntryState::Loaded(loaded) => {
                    if loaded.num_tasks_with_access() == 0 {
                        // The reference in the map is the last reference, so we can drop the entry
                        let entry_state =
                            entry.insert(EntryState::Dropping(EntryStateDropping::new_dummy()));
                        let EntryState::Loaded(loaded) = entry_state else {
                            unreachable!("We already checked the state above");
                        };
                        let drop_future = ConcurrentStoreInner::make_drop_future_for_loaded_entry(
                            this,
                            entry.key().clone(),
                            loaded,
                        );
                        let dropping_state = EntryStateDropping::new(drop_future);
                        let shared_future = dropping_state.future().clone();
                        *entry.get_mut() = EntryState::Dropping(dropping_state);
                        Some(shared_future)
                    } else {
                        // There are still references to the entry
                        None
                    }
                }
                EntryState::Dropping(_) => {
                    // Already dropping (race condition)
                    None
                }
            }
        };

        if let Some(drop_future) = drop_future {
            drop_future.await;
        }
    }

    /// Create a drop future that will drop the entry and remove it from the map.
    fn make_drop_future_for_loaded_entry(
        this: &AsyncDropGuard<AsyncDropArc<ConcurrentStoreInner<K, V, E>>>,
        key: K,
        loaded: EntryStateLoaded<V, E>,
    ) -> BoxFuture<'static, ()> {
        let (intent, mut entry) = loaded.into_inner();
        let this = AsyncDropArc::clone(this);
        async move {
            with_async_drop_2!(this, {
                match intent {
                    Some(intent) => {
                        // Execute the drop function and handle reload if any
                        let reload = intent.execute_drop(Some(entry)).await;

                        if let Some(reload) = reload {
                            // There's a reload pending - transition to Loading
                            let mut entries = this.entries.lock().unwrap();
                            if let Some(state) = entries.get_mut(&key) {
                                *state =
                                    EntryState::Loading(EntryStateLoading::new_from_reload(reload));
                            }
                            // Note: The reload future will handle the actual loading
                        } else {
                            // No reload - remove the entry
                            Self::_remove_dropping_entry(&this, &key).await;
                        }
                    }
                    None => {
                        // No intent - just async drop the entry
                        entry.async_drop().await.unwrap(); // TODO No unwrap
                        Self::_remove_dropping_entry(&this, &key).await;
                    }
                }
                Ok(())
            })
            .infallible_unwrap()
        }
        .boxed()
    }

    fn make_drop_future_for_unloaded_entry<F>(
        this: &AsyncDropGuard<AsyncDropArc<ConcurrentStoreInner<K, V, E>>>,
        key: K,
        drop_fn: impl FnOnce(Option<AsyncDropGuard<V>>) -> F + Send + 'static,
    ) -> BoxFuture<'static, ()>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let this = AsyncDropArc::clone(this);
        async move {
            with_async_drop_2!(this, {
                Self::_execute_immediate_drop(&this, key, None, drop_fn).await;
                Ok(())
            })
            .infallible_unwrap()
        }
        .boxed()
    }

    async fn _execute_immediate_drop<F>(
        this: &AsyncDropGuard<AsyncDropArc<ConcurrentStoreInner<K, V, E>>>,
        key: K,
        entry: Option<AsyncDropGuard<V>>,
        drop_fn: impl FnOnce(Option<AsyncDropGuard<V>>) -> F,
    ) where
        F: Future<Output = ()>,
    {
        // Execute drop_fn without holding the lock on entries
        drop_fn(entry).await;
        // Only now that drop_fn is complete, we remove the entry from the map
        Self::_remove_dropping_entry(this, &key).await;
    }

    /// Called when a drop future completes to remove the entry from the map.
    /// If the entry has a reload pending, we transition to Loading instead.
    async fn _remove_dropping_entry(
        this: &AsyncDropGuard<AsyncDropArc<ConcurrentStoreInner<K, V, E>>>,
        key: &K,
    ) {
        let mut entries = this.entries.lock().unwrap();
        let Some(entry) = entries.get_mut(key) else {
            // Entry was already removed
            return;
        };
        match entry {
            EntryState::Dropping(dropping) => {
                if let Some(reload) = dropping.take_reload() {
                    // There's a reload pending - transition to Loading
                    *entry = EntryState::Loading(EntryStateLoading::new_from_reload(reload));
                } else {
                    // No reload - remove the entry
                    entries.remove(key);
                }
            }
            _ => {
                panic!(
                    "Entry with key {:?} is in unexpected state: {:?}",
                    key, entry
                );
            }
        }
    }

    pub(super) fn _finalize_waiter(
        this: &AsyncDropGuard<AsyncDropArc<ConcurrentStoreInner<K, V, E>>>,
        key: K,
    ) -> AsyncDropGuard<LoadedEntryGuard<K, V, E>> {
        let mut entries = this.entries.lock().unwrap();
        let Some(state) = entries.get_mut(&key) else {
            panic!("Entry with key {:?} was not found in the map", key);
        };
        let EntryState::Loaded(loaded) = state else {
            panic!("Entry with key {:?} is not in loaded state", key);
        };
        LoadedEntryGuard::new(
            AsyncDropArc::clone(this),
            key,
            loaded.get_entry_and_decrease_num_unfulfilled_waiters(),
        )
    }
}

impl<K, V, E> Debug for ConcurrentStoreInner<K, V, E>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    V: AsyncDrop + Debug + Send + Sync + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConcurrentStoreInner").finish()
    }
}

#[async_trait]
impl<K, V, E> AsyncDrop for ConcurrentStoreInner<K, V, E>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    V: AsyncDrop + Debug + Send + Sync + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    type Error = Never;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        // Wait for any currently dropping entries to complete
        let mut entries = std::mem::take(&mut *self.entries.get_mut().unwrap());
        let dropping_futures: Vec<BoxFuture<'static, ()>> = entries
            .drain()
            .filter_map(|(_, entry_state)| match entry_state {
                EntryState::Dropping(dropping) => Some(dropping.into_future().boxed()),
                EntryState::Loading(loading) => {
                    panic!("There are still loading tasks running. Please async_drop all guards before dropping the ConcurrentStore. Witness: {loading:?}");
                }
                EntryState::Loaded(loaded) => {
                    panic!("There are still loaded entries. Please async_drop all guards before dropping the ConcurrentStore. Witness: {loaded:?}");
                }
            })
            .collect();
        for_each_unordered(dropping_futures.into_iter(), |future| async move {
            future.await;
            Ok::<(), Never>(())
        })
        .await
        .infallible_unwrap();
        Ok(())
    }
}

impl<K, V, E> Debug for ConcurrentStore<K, V, E>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    V: AsyncDrop + Debug + Send + Sync + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConcurrentStore").finish()
    }
}

#[async_trait]
impl<K, V, E> AsyncDrop for ConcurrentStore<K, V, E>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    V: AsyncDrop + Debug + Send + Sync + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    type Error = Never;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.inner.async_drop().await
    }
}

/// Result of requesting immediate drop of an entry.
pub enum RequestImmediateDropResult<D> {
    /// Immediate drop request accepted.
    ImmediateDropRequested {
        /// Future that completes with the result of the drop function.
        drop_result: BoxFuture<'static, D>,
    },
    /// Immediate drop request failed because a drop is already in progress
    /// and there's no reload to attach to.
    AlreadyDropping {
        future: Shared<BoxFuture<'static, ()>>,
    },
}
