use anyhow::Result;
use async_trait::async_trait;
use futures::FutureExt as _;
use futures::future::{BoxFuture, Shared};
use lockable::{InfallibleUnwrap as _, Never};
use std::collections::hash_map::Entry;
use std::marker::PhantomData;
use std::{collections::HashMap, fmt::Debug, hash::Hash, sync::Mutex};

use crate::Inserting;
use crate::LoadingOrLoaded;
use crate::entry::EntryState;
use crate::entry::{
    EntryLoadingWaiter, EntryStateDropping, EntryStateDroppingThenLoading, EntryStateLoaded,
    EntryStateLoading, ImmediateDropRequest, ImmediateDropRequestResponse, LoadingResult,
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
/// If any tasks want to load it while it is being dropped, they'll wait until dropping is complete, and then start a new loading operation.
///
/// Parameters:
/// * K: Key type for the entries in the store.
/// * V: Value type for the entries in the store.
/// * R: Result type for immediate drop requests.
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
    /// [EntryState::Loading]:
    ///  * Loading of the entry is in progress, or has completed but we haven't updated the state yet.
    ///  * [EntryStateLoading::num_waiters] counts how many tasks are currently waiting for the loading to complete.
    ///  * Upon completion, the future itself will change the state to [EntryState::Loaded], or remove the entry from the map if loading failed or the entry wasn't found.
    /// [EntryState::Loaded]:
    ///  * The entry is loaded and ready to use.
    ///  * There can be multiple tasks with a reference to the [AsyncDropArc] in [EntryStateLoaded].
    ///  * There are still [EntryStateLoaded::num_unfulfilled_waiters] other tasks waiting for the entry to be loaded. Their future is ready, but they haven't polled it yet.
    ///  * When the last reference to the [AsyncDropArc] is dropped, the [LoadedEntryGuard] will ensure we call [Self::unload] to change to [EntryState::Dropping]
    /// [EntryState::Dropping]:
    ///  * The entry was loaded but is now in the middle of async_drop
    ///  * There are no tasks waiting for the entry to be loaded anymore (unless they were separately created after we started dropping, but those will first wait until dropping is complete and then start their loading attempt).
    ///  * There are no tasks with a reference to the [AsyncDropArc] anymore, the last task is just executing the async_drop.
    /// Immediate Dropping:
    ///   This is mostly used for when an entry should be not just dropped (i.e. flushed), but fully removed from the underlying store. Requesting immediate dropping will block any further readers from being added,
    ///   and as soon as all current readers are done, will call a user-provided function to remove the entry from the underlying store.
    ///   * In both, [EntryState::Loading] and [EntryState::Loaded], if an immediate drop is requested, the flag `immediate_drop_requested` is set to true.
    ///   * If the flag is true, once the last current reader is done, the entry will be removed from the map and the user-provided callback function will be called with exclusive access, to do the remove.
    ///   * Also, any further tasks trying to load the entry while `immediate_drop_requested=true` will wait until all tasks with current access are done, and the user-defined callback is complete, so it'll be exclusive access, and only then execute.
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
                EntryState::Loading(_) | EntryState::Loaded(_) => {
                    Err(anyhow::anyhow!(
                        "Key {key:?} is already loading",
                        key = entry.key()
                    ))
                }
                EntryState::Dropping(_) | EntryState::DroppingThenLoading(_) => {
                    // Per design decision: error immediately when encountering
                    // Dropping/DroppingThenLoading - matches "try" semantics
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
    pub async fn get_loaded_or_insert_loading<'a, F, I>(
        &self,
        key: K,
        loading_fn_input: &AsyncDropGuard<AsyncDropArc<I>>,
        mut loading_fn: impl FnOnce(AsyncDropGuard<AsyncDropArc<I>>) -> F + Send + 'static,
    ) -> LoadingOrLoaded<K, V, E>
    where
        F: Future<Output = Result<Option<AsyncDropGuard<V>>, E>> + Send + 'static,
        I: AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    {
        loop {
            let entry_state =
                self._clone_or_create_entry_state(key.clone(), loading_fn_input, loading_fn);
            // Now the lock on `this.entries` is released, so we can await the loading future without blocking other operations.

            match entry_state {
                CloneOrCreateEntryStateResult::Loaded { entry } => {
                    // Oh, the entry is already loaded! We can just return it
                    // Returning means we shift the responsibility to call async_drop on the [AsyncDropArc] to our caller.
                    return LoadingOrLoaded::new_loaded(LoadedEntryGuard::new(
                        AsyncDropArc::clone(&self.inner),
                        key,
                        entry,
                    ));
                }
                CloneOrCreateEntryStateResult::Loading { loading_result } => {
                    return LoadingOrLoaded::new_loading(
                        AsyncDropArc::clone(&self.inner),
                        loading_result,
                    );
                }
                CloneOrCreateEntryStateResult::ImmediateDropRequested {
                    on_dropped,
                    loading_fn: returned_loading_fn,
                    _r: _,
                    _i: _,
                } => {
                    // Entry is either loading or loaded, but an immediate drop was requested. Let's wait until current processing is complete, dropping was processed, and then retry.
                    let _ = on_dropped.wait().await;
                    // Reset the `loading_fn` [FnOnce]. Since `_clone_or_create_entry_state` didn't use it, it returned it and we can use it in the next iteration.
                    loading_fn = returned_loading_fn;
                    continue;
                }
            }
        }
    }

    /// Check if an entry is either loading or loaded, and if yes return it.
    /// If the entry is not loading or loaded, return None.
    ///
    /// The caller is expected to await the returned [EntryLoadingWaiter] (in the loading state of [LoadingOrLoaded])
    /// through [EntryLoadingWaiter::wait_until_loaded], and to drop the returned [AsyncDropGuard] in the loaded state of [LoadingOrLoaded].
    pub fn get_if_loading_or_loaded(&self, key: K) -> LoadingOrLoaded<K, V, E> {
        let mut entries = self.inner.entries.lock().unwrap();
        match entries.get_mut(&key) {
            Some(EntryState::Loaded(loaded)) => LoadingOrLoaded::new_loaded(LoadedEntryGuard::new(
                AsyncDropArc::clone(&self.inner),
                key,
                loaded.get_entry(),
            )),
            Some(EntryState::Loading(loading)) => LoadingOrLoaded::new_loading(
                AsyncDropArc::clone(&self.inner),
                loading.add_waiter(key),
            ),
            None | Some(EntryState::Dropping(_)) | Some(EntryState::DroppingThenLoading(_)) => {
                LoadingOrLoaded::new_not_found()
            }
        }
    }

    /// Return all entries that are loading, loaded.
    pub fn all_loading_or_loaded(&self) -> Vec<LoadingOrLoaded<K, V, E>> {
        let mut entries = self.inner.entries.lock().unwrap();
        let mut result = Vec::with_capacity(entries.len());
        for (key, entry_state) in entries.iter_mut() {
            match entry_state {
                EntryState::Loaded(loaded) => {
                    result.push(LoadingOrLoaded::new_loaded(LoadedEntryGuard::new(
                        AsyncDropArc::clone(&self.inner),
                        key.clone(),
                        loaded.get_entry(),
                    )));
                }
                EntryState::Loading(loading) => {
                    result.push(LoadingOrLoaded::new_loading(
                        AsyncDropArc::clone(&self.inner),
                        loading.add_waiter(key.clone()),
                    ));
                }
                EntryState::Dropping(_) | EntryState::DroppingThenLoading(_) => {
                    // Ignore dropping entries
                }
            }
        }
        result
    }

    pub fn is_fully_absent(&self, key: &K) -> bool {
        let entries = self.inner.entries.lock().unwrap();
        !entries.contains_key(key)
    }

    #[cfg(any(test, feature = "testutils"))]
    pub fn is_empty(&self) -> bool {
        let entries = self.inner.entries.lock().unwrap();
        entries.is_empty()
    }

    /// Note: This function clones a [EntryState], which may clone the [AsyncDropGuard] contained if it is [EntryState::Loaded]. It is the callers responsibility to async_drop that.
    fn _clone_or_create_entry_state<'s, F, R, I>(
        &'s self,
        key: K,
        loading_fn_input: &AsyncDropGuard<AsyncDropArc<I>>,
        loading_fn: F,
    ) -> CloneOrCreateEntryStateResult<K, V, E, F, R, I>
    where
        F: FnOnce(AsyncDropGuard<AsyncDropArc<I>>) -> R + Send + 'static,
        R: Future<Output = Result<Option<AsyncDropGuard<V>>, E>> + Send + 'static,
        I: AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    {
        let mut entries = self.inner.entries.lock().unwrap();
        match entries.entry(key.clone()) {
            Entry::Occupied(mut entry) => match entry.get_mut() {
                EntryState::Loaded(loaded) => match loaded.immediate_drop_requested() {
                    Some(on_dropped) => CloneOrCreateEntryStateResult::ImmediateDropRequested {
                        on_dropped: on_dropped.clone(),
                        loading_fn,
                        _r: PhantomData,
                        _i: PhantomData,
                    },
                    None => CloneOrCreateEntryStateResult::Loaded {
                        entry: loaded.get_entry(),
                    },
                },
                EntryState::Loading(loading) => match loading.immediate_drop_requested() {
                    Some(on_dropped) => CloneOrCreateEntryStateResult::ImmediateDropRequested {
                        on_dropped: on_dropped.clone(),
                        loading_fn,
                        _r: PhantomData,
                        _i: PhantomData,
                    },
                    None => CloneOrCreateEntryStateResult::Loading {
                        // The caller is responsible for decreasing num_unfulfilled_waiters when it gets the entry.
                        loading_result: loading.add_waiter(key),
                    },
                },
                EntryState::Dropping(state) => {
                    // Entry is being dropped. Transition to DroppingThenLoading so we can
                    // load immediately after the drop completes.
                    let drop_future = state.future().clone();
                    let pending_immediate_drop = state.take_immediate_drop_request();

                    // Create combined future: wait for drop, then load
                    let combined_future = self.make_dropping_then_loading_future(
                        key.clone(),
                        drop_future,
                        AsyncDropArc::clone(loading_fn_input),
                        loading_fn,
                    );

                    // Create DroppingThenLoading state with any pending immediate drop
                    let mut dtl = EntryStateDroppingThenLoading::new(combined_future);

                    // Propagate immediate drop request if any
                    if let ImmediateDropRequest::Requested {
                        drop_fn,
                        on_dropped,
                    } = pending_immediate_drop
                    {
                        // Re-request on the new DTL state
                        dtl.request_immediate_drop_if_not_yet_requested(|_| async move {
                            drop_fn(None).await;
                        });
                        // Note: on_dropped event is already signaled because we're taking
                        // the request from Dropping state. We don't need to propagate it.
                        let _ = on_dropped;
                    }

                    let waiter = dtl.add_waiter(key.clone());
                    *entry.get_mut() = EntryState::DroppingThenLoading(dtl);
                    CloneOrCreateEntryStateResult::Loading {
                        loading_result: waiter,
                    }
                }
                EntryState::DroppingThenLoading(dtl) => {
                    // Join the existing wait - add ourselves as a waiter
                    CloneOrCreateEntryStateResult::Loading {
                        loading_result: dtl.add_waiter(key),
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
                CloneOrCreateEntryStateResult::Loading { loading_result }
            }
        }
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
                        // Now that we're loaded, change the entry state in the map to loaded so that any waiters (current or future) can access it.
                        let mut entries = inner.entries.lock().unwrap();
                        let Some(state) = entries.get_mut(&key) else {
                            panic!("Entry with key {:?} was not found in the map", key);
                        };
                        let EntryState::Loading(loading) = state else {
                            panic!("Entry with key {:?} is not in loading state", key);
                        };
                        let loading = std::mem::replace(loading, EntryStateLoading::new_dummy());
                        // We are still in the middle of executing the future, so no waiter has completed yet.
                        // Also, we currently have a lock on `entry`, so no new waiters can be added.
                        // We're about to change the state to EntryState::Loaded, which will prevent further waiters to be added even after we release the lock.
                        *state = EntryState::Loaded(
                            EntryStateLoaded::new_from_just_finished_loading(entry, loading),
                        );

                        Ok(LoadingResult::Loaded)
                    }
                    Ok(None) => {
                        // We're loaded but the entry wasn't found. Remove the entry from the map and return the error to any waiters.
                        let mut entries = inner.entries.lock().unwrap();
                        let Some(_) = entries.remove(&key) else {
                            panic!("Entry with key {:?} was not found in the map", key);
                        };
                        Ok(LoadingResult::NotFound)
                    }
                    Err(err) => {
                        // An error occurred while loading the entry. Remove the entry from the map and return the error to any waiters.
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

    /// Create a combined future for the DroppingThenLoading state.
    /// This future waits for the drop to complete, then executes the loading function.
    fn make_dropping_then_loading_future<I, F, R>(
        &self,
        key: K,
        drop_future: Shared<BoxFuture<'static, ()>>,
        loading_fn_input: AsyncDropGuard<AsyncDropArc<I>>,
        loading_fn: F,
    ) -> Shared<BoxFuture<'static, LoadingResult<E>>>
    where
        F: FnOnce(AsyncDropGuard<AsyncDropArc<I>>) -> R + Send + 'static,
        R: Future<Output = Result<Option<AsyncDropGuard<V>>, E>> + Send + 'static,
        I: AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    {
        let inner = AsyncDropArc::clone(&self.inner);
        let combined_task = async move {
            // Phase 1: Wait for the current drop to complete
            drop_future.await;

            // Phase 2: Execute loading function
            with_async_drop_2!(inner, {
                let result = loading_fn(loading_fn_input).await;

                match result {
                    Ok(Some(entry)) => {
                        // Loading succeeded. Transition from DroppingThenLoading to Loaded.
                        let mut entries = inner.entries.lock().unwrap();
                        let Some(state) = entries.get_mut(&key) else {
                            panic!("Entry with key {:?} was not found in the map", key);
                        };
                        let EntryState::DroppingThenLoading(dtl) = state else {
                            panic!(
                                "Entry with key {:?} should be in DroppingThenLoading state but is {:?}",
                                key, state
                            );
                        };
                        // Take the DTL state to extract num_waiters and immediate_drop_request
                        let dtl = std::mem::replace(
                            dtl,
                            EntryStateDroppingThenLoading::new(
                                futures::future::ready(LoadingResult::NotFound).boxed().shared(),
                            ),
                        );
                        // Get num_waiters first before consuming dtl
                        let num_waiters = dtl.num_waiters();
                        let immediate_drop_request = dtl.into_immediate_drop_request();

                        // Create loaded state with propagated immediate_drop_request
                        *state = EntryState::Loaded(
                            EntryStateLoaded::new_from_dropping_then_loading(
                                entry,
                                num_waiters,
                                immediate_drop_request,
                            ),
                        );

                        Ok(LoadingResult::Loaded)
                    }
                    Ok(None) => {
                        // Loading found nothing. Remove the entry.
                        let mut entries = inner.entries.lock().unwrap();
                        let Some(_) = entries.remove(&key) else {
                            panic!("Entry with key {:?} was not found in the map", key);
                        };
                        Ok(LoadingResult::NotFound)
                    }
                    Err(err) => {
                        // Loading failed. Remove the entry.
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
        combined_task.boxed().shared()
    }

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
    ///
    /// This is useful when you need to ensure the drop happens, even if the entry
    /// is currently being dropped by another task.
    ///
    /// The `make_drop_fn` factory is called on each attempt because `FnOnce` closures
    /// are consumed when used.
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
            Some(EntryState::DroppingThenLoading(dtl)) => dtl.immediate_drop_requested().is_some(),
            None => false,
        }
    }

    /// Request immediate drop of the entry with the given key.
    /// * If the entry is loading or loaded, all further tasks wanting to load this key will be blocked from loading,
    ///   and once all current tasks with access are complete, the given drop_fn will be executed with exclusive access to the entry,
    ///   and the entry will be unloaded from the map. After drop_fn completes, other tasks can load it again.
    /// * If the entry is not loaded, all tasks wanting to load this key will be blocked from loading,
    ///   and the given drop_fn will be executed with a `None` value. After drop_fn completes, other tasks can load it again.
    /// * If the entry is already dropping, the existing dropping future will be returned.
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
                    match loaded.request_immediate_drop_if_not_yet_requested(drop_fn) {
                        ImmediateDropRequestResponse::Requested => {
                            RequestImmediateDropResult::ImmediateDropRequested { drop_result: async move {drop_result_receiver.await.expect("The sender should not be dropped")}.boxed() }
                        }
                        ImmediateDropRequestResponse::NotRequestedBecauseItWasAlreadyRequestedEarlier {
                            on_earlier_request_complete,
                        } => {
                            RequestImmediateDropResult::AlreadyDropping { future: on_earlier_request_complete }
                        },
                    }
                }
                EntryState::Loading(loading) => {
                    match loading.request_immediate_drop_if_not_yet_requested(drop_fn) {
                        ImmediateDropRequestResponse::Requested => {
                            RequestImmediateDropResult::ImmediateDropRequested { drop_result: async move {drop_result_receiver.await.expect("The sender should not be dropped")}.boxed() }
                        }
                        ImmediateDropRequestResponse::NotRequestedBecauseItWasAlreadyRequestedEarlier {
                            on_earlier_request_complete,
                        } => {
                            RequestImmediateDropResult::AlreadyDropping { future: on_earlier_request_complete }
                        },
                        }
                }
                EntryState::Dropping(dropping) => {
                    // Set the immediate drop request flag so it's immediately visible
                    match dropping.request_immediate_drop_if_not_yet_requested(drop_fn) {
                        ImmediateDropRequestResponse::Requested => {
                            // Flag set. Return AlreadyDropping but the flag IS visible.
                            // After current drop completes, the pending drop_fn will be executed.
                            RequestImmediateDropResult::AlreadyDropping {
                                future: dropping.future().clone(),
                            }
                        }
                        ImmediateDropRequestResponse::NotRequestedBecauseItWasAlreadyRequestedEarlier {
                            on_earlier_request_complete,
                        } => {
                            RequestImmediateDropResult::AlreadyDropping {
                                future: on_earlier_request_complete,
                            }
                        }
                    }
                }
                EntryState::DroppingThenLoading(dtl) => {
                    // Set the immediate drop request flag so it's immediately visible
                    match dtl.request_immediate_drop_if_not_yet_requested(drop_fn) {
                        ImmediateDropRequestResponse::Requested => {
                            // Flag set. Return AlreadyDropping but the flag IS visible.
                            // When loading completes, the immediate drop will be propagated.
                            RequestImmediateDropResult::AlreadyDropping {
                                future: dtl.combined_future().clone().map(|_| ()).boxed().shared(),
                            }
                        }
                        ImmediateDropRequestResponse::NotRequestedBecauseItWasAlreadyRequestedEarlier {
                            on_earlier_request_complete,
                        } => {
                            RequestImmediateDropResult::AlreadyDropping {
                                future: on_earlier_request_complete,
                            }
                        }
                    }
                }
            },
            Entry::Vacant(entry) => {
                // The entry is not loaded or loading. Let's add a dummy entry to block other tasks from loading it while we execute drop_fn.
                let drop_future = ConcurrentStoreInner::make_drop_future_for_unloaded_entry(
                    this,
                    entry.key().clone(),
                    drop_fn,
                );
                entry.insert(EntryState::Dropping(EntryStateDropping::new(
                    drop_future.clone(),
                )));

                std::mem::drop(entries);

                RequestImmediateDropResult::ImmediateDropRequested {
                    drop_result: async move {
                        // In the other cases, we register the drop future to the loaded entry and it will be executed by that entry's unload logic.
                        // However, if no entry was loaded, then there is no unload to execute the drop future. We have to execute it ourselves.
                        drop_future.await;

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
        entry.async_drop().await.unwrap(); // TODO No unwrap? But what to do if it fails? We need to guarantee that we still remove the entry since the guard is gone now.
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
                // This can happen due to the same race condition described below under `EntryState::Dropping`.
                // Another task already dropped the entry and completed dropping it before we got here.
                // This is benign.
                // Note that it is even possible that the entry was fully dropped and then re-loaded again
                // before we got here, but even that is ok because in that case the reference count would not be zero.
                return;
            };
            match entry.get() {
                EntryState::Loading(_) => {
                    // Unload only happens for loaded entries. There is no direct transition from Loading to call into here.
                    // However, because of the race condition mentioned in other comments in this function, it is possible
                    // that the entry was already fully unloaded, dropped, and then re-loaded before we call into here.
                    // In that case, we just ignore the request. The re-loaded guard will eventually call unload again when it is dropped.
                    return;
                }
                EntryState::Loaded(loaded) => {
                    if loaded.num_tasks_with_access() == 0 {
                        // The reference in the map is the last reference, so we can drop the entry
                        let entry_state =
                            // Dummy entry, will be replaced below
                            entry.insert(EntryState::Dropping(EntryStateDropping::new_dummy()));
                        let EntryState::Loaded(loaded) = entry_state else {
                            unreachable!("We already checked the state above, it should be Loaded");
                        };
                        let drop_future = ConcurrentStoreInner::make_drop_future_for_loaded_entry(
                            this,
                            entry.key().clone(),
                            loaded,
                        );
                        *entry.get_mut() =
                            EntryState::Dropping(EntryStateDropping::new(drop_future.clone()));
                        Some(drop_future)
                    } else {
                        // There are still references to the entry, so we just return
                        None
                    }
                }
                EntryState::Dropping(_) | EntryState::DroppingThenLoading(_) => {
                    // Because of the way unload releases the references (and reduces the reference count) without a lock before
                    // calling into this function, there is a race condition and it is possible that multiple tasks unloading the
                    // same entry both first decrement the refcount, which then reaches zero, and then both call into here.
                    // The first one will change the state to Dropping, the second one will find it already in Dropping state.
                    // We can just ignore that second call since the first call will take care of dropping the entry.
                    // Similarly for DroppingThenLoading - the entry is already in a dropping/loading cycle.
                    None
                }
            }
        };

        if let Some(drop_future) = drop_future {
            // Now the entry is marked as `Dropping` and the lock on `entries` is released. We can await the drop.
            drop_future.await;
        }
    }

    /// Create a drop future that will drop the entry and remove it from the map.
    /// This is called when the last reference to the entry is dropped.
    /// The future will update the entries map itself, no need for the caller to do that.
    fn make_drop_future_for_loaded_entry(
        this: &AsyncDropGuard<AsyncDropArc<ConcurrentStoreInner<K, V, E>>>,
        key: K,
        loaded: EntryStateLoaded<V>,
    ) -> Shared<BoxFuture<'static, ()>> {
        let (immediate_drop_request, mut entry) = loaded.into_inner();
        let this = AsyncDropArc::clone(this);
        async move {
            with_async_drop_2!(this, {
                // This will be awaited after the lock on entries is released, so we can concurrently drop
                // the entry without blocking other operations.
                match immediate_drop_request {
                    ImmediateDropRequest::Requested {
                        drop_fn,
                        on_dropped: _,
                    } => {
                        // An immediate drop was requested. Execute the user-provided drop function with exclusive access to the entry.
                        Self::_execute_immediate_drop(&this, key, Some(entry), drop_fn).await;
                    }
                    ImmediateDropRequest::NotRequested => {
                        // otherwise just drop the entry
                        entry.async_drop().await.unwrap(); // TODO No unwrap
                        Self::_remove_dropping_entry(&this, &key).await; // always remove the entry from the map, even if drop_fn failed
                    }
                }
                Ok(())
            })
            .infallible_unwrap()
        }
        .boxed()
        .shared()
    }

    fn make_drop_future_for_unloaded_entry<F>(
        this: &AsyncDropGuard<AsyncDropArc<ConcurrentStoreInner<K, V, E>>>,
        key: K,
        drop_fn: impl FnOnce(Option<AsyncDropGuard<V>>) -> F + Send + 'static,
    ) -> Shared<BoxFuture<'static, ()>>
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
        .shared()
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
        // Only now that drop_fn is complete, we remove the entry from the map and notify any tasks waiting for loading this entry.
        // This guarantees that drop_fn has exclusive access to entry.
        Self::_remove_dropping_entry(this, &key).await; // always remove the entry from the map, even if drop_fn failed
    }

    /// Called when a drop future completes to remove the entry from the map.
    /// If the entry has transitioned to DroppingThenLoading, we don't remove it -
    /// the combined future will handle the loading phase.
    async fn _remove_dropping_entry(
        this: &AsyncDropGuard<AsyncDropArc<ConcurrentStoreInner<K, V, E>>>,
        key: &K,
    ) {
        let pending_immediate_drop = {
            let mut entries = this.entries.lock().unwrap();
            let Some(entry) = entries.get_mut(key) else {
                panic!("Entry with key {:?} was not found in the map", key);
            };
            match entry {
                EntryState::Dropping(dropping) => {
                    // Take any pending immediate drop request before removing
                    let pending = dropping.take_immediate_drop_request();
                    entries.remove(key);
                    pending
                }
                EntryState::DroppingThenLoading(_) => {
                    // Entry transitioned to DroppingThenLoading while the drop was in progress.
                    // Don't remove it - the combined future will handle loading.
                    // The immediate_drop_request was already propagated to DroppingThenLoading
                    // when the transition happened.
                    return;
                }
                _ => {
                    panic!("Entry with key {:?} is in unexpected state: {:?}", key, entry);
                }
            }
        };

        // If there was a pending immediate drop request, execute it with None
        // (the entry is already gone)
        if let ImmediateDropRequest::Requested {
            drop_fn,
            on_dropped: _,
        } = pending_immediate_drop
        {
            drop_fn(None).await;
        }
    }

    pub(super) fn _finalize_waiter(
        this: &AsyncDropGuard<AsyncDropArc<ConcurrentStoreInner<K, V, E>>>,
        key: K,
    ) -> AsyncDropGuard<LoadedEntryGuard<K, V, E>> {
        // This is not a race condition with dropping, i.e. the entry can't be in dropping state yet, because we are an "unfulfilled waiter",
        // i.e. the entry cannot be dropped until we decrease the count below.
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
            // [Self::_clone_or_create_entry_state] added a waiter, so we need to decrement num_unfulfilled_waiters.
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
                    EntryState::DroppingThenLoading(dtl) => {
                        // Wait for the combined future (drop then load) to complete, then ignore the result
                        Some(dtl.combined_future().clone().map(|_| ()).boxed())
                    }
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

/// Basically the same as [EntryState], but the [Self::ImmediateDropRequested] state carries an additional member `loading_fn` so the `FnOnce` can be returned from
/// the method if not used.
enum CloneOrCreateEntryStateResult<K, V, E, F, R, I>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    V: AsyncDrop + Debug + Send + Sync + 'static,
    E: Clone + Debug + Send + Sync + 'static,
    F: FnOnce(AsyncDropGuard<AsyncDropArc<I>>) -> R + Send,
    R: Future<Output = Result<Option<AsyncDropGuard<V>>, E>> + Send + 'static,
    I: AsyncDrop + Debug + Send,
{
    Loading {
        loading_result: EntryLoadingWaiter<K, E>,
    },
    Loaded {
        entry: AsyncDropGuard<AsyncDropArc<V>>,
    },
    /// EntryState is either Loading or Loaded, but an immediate drop was requested. We need to block further accesses until dropping is complete.
    ImmediateDropRequested {
        on_dropped: Event,
        loading_fn: F,
        _r: PhantomData<R>,
        _i: PhantomData<I>,
    },
}

/// Result of requesting immediate drop of an entry.
pub enum RequestImmediateDropResult<D> {
    /// Immediate drop request accepted. The entry was either loading or loaded and the specified drop function will be executed with the entry,
    /// or the entry was not loaded and the specified drop function will be executed with None.
    ImmediateDropRequested {
        /// on_dropped will be completed once the entry has been fully dropped.
        /// The caller is expected to drive this future to completion,
        /// otherwise we may be stuck forever waiting for the drop to complete.
        drop_result: BoxFuture<'static, D>,
    },
    /// Immediate drop request failed because the entry is already in dropping state.
    /// This could be from the last task giving up its guard, or by an earlier immediate drop request.
    AlreadyDropping {
        future: Shared<BoxFuture<'static, ()>>,
    },
}
