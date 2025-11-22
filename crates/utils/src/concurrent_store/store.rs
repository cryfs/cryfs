use anyhow::Error;
use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::FutureExt as _;
use futures::future::{BoxFuture, Shared};
use std::collections::hash_map::Entry;
use std::marker::PhantomData;
use std::{
    collections::HashMap,
    fmt::Debug,
    hash::Hash,
    sync::{Arc, Mutex},
};

use crate::concurrent_store::entry::{
    EntryLoadingWaiter, EntryStateDropping, EntryStateLoaded, EntryStateLoading,
    ImmediateDropRequest, ImmediateDropRequestResponse, LoadingResult,
};
use crate::concurrent_store::guard::LoadedEntryGuard;
use crate::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard},
    concurrent_store::entry::EntryState,
};
use crate::{mr_oneshot_channel, with_async_drop_2};

// TODO This is currently not cancellation safe. If a task waiting for a blob to load is cancelled, the num_waiters and num_unfulfilled_waiters counts will be wrong.

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
pub struct ConcurrentStore<K, V, D>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    V: AsyncDrop + Debug + Send + Sync + 'static,
    D: Clone + Debug + Send + Sync + 'static,
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
    pub(super) entries: Arc<Mutex<HashMap<K, EntryState<V, D>>>>,

    _d: PhantomData<D>,
}
impl<K, V, D> ConcurrentStore<K, V, D>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    V: AsyncDrop + Debug + Send + Sync + 'static,
    D: Clone + Debug + Send + Sync + 'static,
{
    /// Create a new empty [ConcurrentStore].
    pub fn new() -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(ConcurrentStore {
            entries: Arc::new(Mutex::new(HashMap::new())),
            _d: PhantomData,
        })
    }

    /// Try to insert a new entry by loading it using the provided loading function.
    /// If an entry with the same key is already loaded, an error is returned.
    /// The loading function is only called if no other loading or loaded entry with the same key exists.
    pub async fn try_insert_with_key<F>(
        this: &AsyncDropGuard<AsyncDropArc<Self>>,
        key: K,
        loading_fn: impl FnOnce() -> F + Send + 'static,
    ) -> Result<(), anyhow::Error>
    where
        F: Future<Output = Result<AsyncDropGuard<V>>> + Send,
    {
        let loading_future = {
            let mut entries = this.entries.lock().unwrap();
            match entries.entry(key.clone()) {
                Entry::Occupied(entry) => Err(anyhow::anyhow!(
                    "Key {key:?} is already loaded",
                    key = entry.key()
                )),
                Entry::Vacant(entry) => {
                    let loading_future = async move {
                        let loaded_entry = loading_fn().await?;
                        Ok(Some(loaded_entry))
                    };
                    let mut loading_future = this.make_loading_future(key.clone(), loading_future);
                    let loading_result = loading_future.add_waiter();
                    entry.insert(EntryState::Loading(loading_future));
                    Ok(loading_result)
                }
            }
        }?;

        // Now the lock on `entries` is released, so we can await the loading future without blocking other operations.
        let mut loaded = loading_future
            .wait_until_loaded(this, key)
            .await?
            .expect("This shouldn't happen, our loading_future always returns Some");

        loaded.async_drop().await?;
        Ok(())
    }

    /// Insert a new entry that was just created and has a new key assigned.
    /// This must not be an existing key or it can cause race conditions or panics.
    /// This key also must not be used in any other calls before this completes.
    /// Only after this function call returns are we set up to deal with concurrent accesses.
    pub fn insert_with_new_key(
        this: &AsyncDropGuard<AsyncDropArc<Self>>,
        key: K,
        value: AsyncDropGuard<V>,
    ) -> AsyncDropGuard<LoadedEntryGuard<K, V, D>> {
        // We now have the newly assigned key and are fully loaded. Insert it into the map.
        let mut entries = this.entries.lock().unwrap();
        // No unfulfilled waiters, we just created it
        let loaded = EntryStateLoaded::new_without_unfulfilled_waiters(value);
        let loaded_entry = loaded.get_entry();
        match entries.entry(key.clone()) {
            Entry::Occupied(_) => {
                panic!("Entry with key {key:?} is already loaded even though we just created it",);
            }
            Entry::Vacant(entry) => {
                entry.insert(EntryState::Loaded(loaded));
            }
        }

        LoadedEntryGuard::new(AsyncDropArc::clone(this), key, loaded_entry)
    }

    /// Load an entry if it is not already loaded, or return the existing loaded entry.
    pub async fn get_loaded_or_insert_loading<'a, F, I>(
        this: &AsyncDropGuard<AsyncDropArc<Self>>,
        key: K,
        loading_fn_input: AsyncDropGuard<AsyncDropArc<I>>,
        mut loading_fn: impl FnOnce(AsyncDropGuard<AsyncDropArc<I>>) -> F + Send,
    ) -> Result<Option<AsyncDropGuard<LoadedEntryGuard<K, V, D>>>, anyhow::Error>
    where
        F: Future<Output = Result<Option<AsyncDropGuard<V>>>> + Send + 'static,
        I: AsyncDrop + Debug + Send,
        <I as AsyncDrop>::Error: std::error::Error + Send + Sync + 'static,
    {
        with_async_drop_2!(loading_fn_input, {
            loop {
                let entry_state =
                    this._clone_or_create_entry_state(key.clone(), &loading_fn_input, loading_fn);
                // Now the lock on `this.entries` is released, so we can await the loading future without blocking other operations.

                match entry_state {
                    CloneOrCreateEntryStateResult::Loaded { entry } => {
                        // Oh, the entry is already loaded! We can just return it
                        // Returning means we shift the responsibility to call async_drop on the [AsyncDropArc] to our caller.
                        return Ok(Some(LoadedEntryGuard::new(
                            AsyncDropArc::clone(this),
                            key,
                            entry,
                        )));
                    }
                    CloneOrCreateEntryStateResult::Loading { loading_result } => {
                        return loading_result
                            .wait_until_loaded(this, key.clone())
                            .await
                            .with_context(|| {
                                format!("Error while try_insert'ing entry with key {key:?}")
                            });
                    }
                    CloneOrCreateEntryStateResult::Dropping {
                        future,
                        loading_fn: returned_loading_fn,
                        _r: _,
                        _i: _,
                    } => {
                        future.await;
                        // Reset the `loading_fn` [FnOnce]. Since `_clone_or_create_entry_state` didn't use it, it returned it and we can use it in the next iteration.
                        loading_fn = returned_loading_fn;
                        // After the drop is complete, we can try to load the entry again.
                        continue;
                    }
                    CloneOrCreateEntryStateResult::ImmediateDropRequested {
                        on_dropped,
                        loading_fn: returned_loading_fn,
                    } => {
                        // Entry is either loading or loaded, but an immediate drop was requested. Let's wait until current processing is complete, dropping was processed, and then retry.
                        let _ = on_dropped.recv().await;
                        // Reset the `loading_fn` [FnOnce]. Since `_clone_or_create_entry_state` didn't use it, it returned it and we can use it in the next iteration.
                        loading_fn = returned_loading_fn;
                        continue;
                    }
                }
            }
        })
    }

    /// Check if an entry is either loading or loaded, and if yes return it.
    /// If the entry is not loading or loaded, return None.
    pub async fn get_if_loading_or_loaded(
        this: &AsyncDropGuard<AsyncDropArc<Self>>,
        key: &K,
    ) -> Result<Option<AsyncDropGuard<LoadedEntryGuard<K, V, D>>>, anyhow::Error> {
        let waiter = {
            let mut entries = this.entries.lock().unwrap();
            match entries.get_mut(key) {
                Some(EntryState::Loaded(loaded)) => {
                    return Ok(Some(LoadedEntryGuard::new(
                        AsyncDropArc::clone(this),
                        key.clone(), // TODO Avoid this clone by making the `key` parameter owned?
                        loaded.get_entry(),
                    )));
                }
                Some(EntryState::Loading(loading)) => loading.add_waiter(),
                None | Some(EntryState::Dropping { .. }) => return Ok(None),
            }
        };

        // Now entries are unlocked and we can wait for loading to complete
        waiter.wait_until_loaded(this, key.clone()).await
    }

    /// Note: This function clones a [EntryState], which may clone the [AsyncDropGuard] contained if it is [EntryState::Loaded]. It is the callers responsibility to async_drop that.
    fn _clone_or_create_entry_state<'s, F, R, I>(
        &'s self,
        key: K,
        loading_fn_input: &AsyncDropGuard<AsyncDropArc<I>>,
        loading_fn: F,
    ) -> CloneOrCreateEntryStateResult<V, F, R, I, D>
    where
        F: FnOnce(AsyncDropGuard<AsyncDropArc<I>>) -> R + Send,
        R: Future<Output = Result<Option<AsyncDropGuard<V>>>> + Send + 'static,
        I: AsyncDrop + Debug + Send,
        <I as AsyncDrop>::Error: std::error::Error + Send + Sync + 's,
    {
        let mut entries = self.entries.lock().unwrap();
        match entries.entry(key.clone()) {
            Entry::Occupied(mut entry) => match entry.get_mut() {
                EntryState::Loaded(loaded) => match loaded.immediate_drop_requested() {
                    Some(on_dropped) => CloneOrCreateEntryStateResult::ImmediateDropRequested {
                        on_dropped,
                        loading_fn,
                    },
                    None => CloneOrCreateEntryStateResult::Loaded {
                        entry: loaded.get_entry(),
                    },
                },
                EntryState::Loading(loading) => match loading.immediate_drop_requested() {
                    Some(on_dropped) => CloneOrCreateEntryStateResult::ImmediateDropRequested {
                        on_dropped,
                        loading_fn,
                    },
                    None => CloneOrCreateEntryStateResult::Loading {
                        // The caller is responsible for decreasing num_unfulfilled_waiters when it gets the entry.
                        loading_result: loading.add_waiter(),
                    },
                },
                EntryState::Dropping(state) => CloneOrCreateEntryStateResult::Dropping {
                    future: state.future().clone(),
                    _r: PhantomData,
                    _i: PhantomData,
                    loading_fn,
                },
            },
            Entry::Vacant(entry) => {
                // No loading operation is in progress, so we start a new one.
                let mut loading_future = self
                    .make_loading_future(key, loading_fn(AsyncDropArc::clone(loading_fn_input)));
                let loading_result = loading_future.add_waiter();
                entry.insert(EntryState::Loading(loading_future));
                CloneOrCreateEntryStateResult::Loading { loading_result }
            }
        }
    }

    /// Create a loading future that will load the entry using the provided loading function, and update the entry state upon completion.
    fn make_loading_future(
        &self,
        key: K,
        loading_fn: impl Future<Output = Result<Option<AsyncDropGuard<V>>>> + Send + 'static,
    ) -> EntryStateLoading<V, D> {
        let entries = Arc::clone(&self.entries);
        let loading_task = async move {
            // Run loading_fn concurrently, without a lock on `entries`.
            let result = loading_fn.await;

            match result {
                Ok(Some(entry)) => {
                    // Now that we're loaded, change the entry state in the map to loaded so that any waiters (current or future) can access it.
                    let mut entries = entries.lock().unwrap();
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
                    *state = EntryState::Loaded(EntryStateLoaded::new_from_just_finished_loading(
                        entry, loading,
                    ));

                    LoadingResult::Loaded
                }
                Ok(None) => {
                    // We're loaded but the entry wasn't found. Remove the entry from the map and return the error to any waiters.
                    let mut entries = entries.lock().unwrap();
                    let Some(_) = entries.remove(&key) else {
                        panic!("Entry with key {:?} was not found in the map", key);
                    };
                    LoadingResult::NotFound
                }
                Err(err) => {
                    // An error occurred while loading the entry. Remove the entry from the map and return the error to any waiters.
                    let mut entries = entries.lock().unwrap();
                    let Some(_) = entries.remove(&key) else {
                        panic!("Entry with key {:?} was not found in the map", key);
                    };
                    LoadingResult::Error(Arc::new(err))
                }
            }
        };
        EntryStateLoading::new(loading_task.boxed())
    }

    /// Request immediate drop of the entry with the given key.
    /// * If the entry is loading or loaded, all further tasks wanting to load this key will be blocked from loading,
    ///   and once all current tasks with access are complete, the given drop_fn will be executed with exclusive access to the entry,
    ///   and the entry will be unloaded from the map. After drop_fn completes, other tasks can load it again.
    /// * If the entry is not loaded, all tasks wanting to load this key will be blocked from loading,
    ///   and the given drop_fn will be executed with a `None` value. After drop_fn completes, other tasks can load it again.
    /// * If the entry is already dropping, the existing dropping future will be returned.
    pub fn request_immediate_drop<F>(
        &self,
        key: K,
        drop_fn: impl FnOnce(Option<AsyncDropGuard<V>>) -> F + Send + Sync + 'static,
    ) -> RequestImmediateDropResult<D>
    where
        F: Future<Output = D> + Send + 'static,
    {
        let mut entries = self.entries.lock().unwrap();
        match entries.entry(key) {
            Entry::Occupied(mut entry) => match entry.get_mut() {
                EntryState::Loaded(loaded) => {
                    match loaded.request_immediate_drop_if_not_yet_requested(drop_fn) {
                        ImmediateDropRequestResponse::Requested { on_dropped } => {
                            RequestImmediateDropResult::ImmediateDropRequested { on_dropped }
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
                            ImmediateDropRequestResponse::Requested { on_dropped } => {
                            RequestImmediateDropResult::ImmediateDropRequested { on_dropped }
                        }
                        ImmediateDropRequestResponse::NotRequestedBecauseItWasAlreadyRequestedEarlier {
                            on_earlier_request_complete,
                        } => {
                            RequestImmediateDropResult::AlreadyDropping { future: on_earlier_request_complete }
                        },
                        }
                }
                EntryState::Dropping(dropping) => RequestImmediateDropResult::AlreadyDropping {
                    future: dropping.future().clone(),
                },
            },
            Entry::Vacant(entry) => {
                // The entry is not loaded or loading. Let's add a dummy entry to block other tasks from loading it while we execute drop_fn.
                let (completion_sender, entry_receiver) = mr_oneshot_channel::channel();
                let drop_future = self.make_drop_future_for_unloaded_entry(
                    entry.key().clone(),
                    drop_fn,
                    completion_sender,
                );
                entry.insert(EntryState::Dropping(EntryStateDropping::new(
                    drop_future.clone(),
                )));

                std::mem::drop(entries);

                // In the other cases, we register the drop future to the loaded entry and it will be executed by that entry's unload logic.
                // However, if no entry was loaded, then there is no unload to execute the drop future. We have to execute it ourselves.
                tokio::task::spawn(drop_future);

                RequestImmediateDropResult::ImmediateDropRequested {
                    on_dropped: entry_receiver,
                }
            }
        }
    }

    /// Called by [LoadedEntryGuard] when it is dropped.
    pub(super) async fn unload(
        &self,
        key: K,
        mut entry: AsyncDropGuard<AsyncDropArc<V>>,
    ) -> Result<()> {
        // First drop the entry to decrement the reference count
        entry.async_drop().await.unwrap(); // TODO No unwrap? But what to do if it fails?
        std::mem::drop(entry);

        // Now check if we're the last reference. If yes, remove the entry from our map.
        self._drop_if_no_references(key).await;
        Ok(())
    }

    /// Check if there are no more references to the entry with the given key, and if yes, async drop it.
    async fn _drop_if_no_references(&self, key: K) {
        let drop_future = {
            let mut entries = self.entries.lock().unwrap();
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
                    // In that case, we just ignore the request.
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
                        let drop_future =
                            self.make_drop_future_for_loaded_entry(entry.key().clone(), loaded);
                        *entry.get_mut() =
                            EntryState::Dropping(EntryStateDropping::new(drop_future.clone()));
                        Some(drop_future)
                    } else {
                        // There are still references to the entry, so we just return
                        None
                    }
                }
                EntryState::Dropping { .. } => {
                    // Because of the way unload releases the references (and reduces the reference count) without a lock before
                    // calling into this function, there is a race condition and it is possible that multiple tasks unloading the
                    // same entry both first decrement the refcount, which then reaches zero, and then both call into here.
                    // The first one will change the state to Dropping, the second one will find it already in Dropping state.
                    // We can just ignore that second call since the first call will take care of dropping the entry.
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
        &self,
        key: K,
        loaded: EntryStateLoaded<V, D>,
    ) -> Shared<BoxFuture<'static, ()>> {
        let (immediate_drop_request, mut entry) = loaded.into_inner();
        let entries = Arc::clone(&self.entries);
        async move {
            // This will be awaited after the lock on entries is released, so we can concurrently drop
            // the entry without blocking other operations.
            match immediate_drop_request {
                ImmediateDropRequest::Requested {
                    drop_fn,
                    completion_sender,
                } => {
                    // An immediate drop was requested. Execute the user-provided drop function with exclusive access to the entry.
                    Self::_execute_immediate_drop(
                        entries,
                        key,
                        Some(entry),
                        drop_fn,
                        completion_sender,
                    )
                    .await;
                }
                ImmediateDropRequest::NotRequested => {
                    // otherwise just drop the entry
                    entry.async_drop().await.unwrap(); // TODO No unwrap
                    Self::_remove_dropping_entry(entries, &key).await; // always remove the entry from the map, even if drop_fn failed
                }
            }
        }
        .boxed()
        .shared()
    }

    fn make_drop_future_for_unloaded_entry<F>(
        &self,
        key: K,
        drop_fn: impl FnOnce(Option<AsyncDropGuard<V>>) -> F + Send + 'static,
        completion_sender: mr_oneshot_channel::Sender<D>,
    ) -> Shared<BoxFuture<'static, ()>>
    where
        F: Future<Output = D> + Send + 'static,
    {
        let entries = Arc::clone(&self.entries);
        Self::_execute_immediate_drop(entries, key, None, drop_fn, completion_sender)
            .boxed()
            .shared()
    }

    async fn _execute_immediate_drop<F>(
        entries: Arc<Mutex<HashMap<K, EntryState<V, D>>>>,
        key: K,
        entry: Option<AsyncDropGuard<V>>,
        drop_fn: impl FnOnce(Option<AsyncDropGuard<V>>) -> F,
        completion_sender: mr_oneshot_channel::Sender<D>,
    ) where
        F: Future<Output = D>,
    {
        // Execute drop_fn without holding the lock on entries
        let drop_result = drop_fn(entry).await;
        // Only now that drop_fn is complete, we remove the entry from the map and notify any tasks waiting for loading this entry.
        // This guarantees that drop_fn has exclusive access to entry.
        Self::_remove_dropping_entry(entries, &key).await; // always remove the entry from the map, even if drop_fn failed
        completion_sender.send(drop_result);
    }

    async fn _remove_dropping_entry(entries: Arc<Mutex<HashMap<K, EntryState<V, D>>>>, key: &K) {
        let mut entries = entries.lock().unwrap();
        let Some(entry) = entries.remove(key) else {
            panic!("Entry with key {:?} was not found in the map", key);
        };
        let EntryState::Dropping { .. } = entry else {
            panic!("Entry with key {:?} is not in dropping state", key);
        };
    }
}

impl<K, V, D> Debug for ConcurrentStore<K, V, D>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    V: AsyncDrop + Debug + Send + Sync + 'static,
    D: Clone + Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConcurrentStore").finish()
    }
}

#[async_trait]
impl<K, V, D> AsyncDrop for ConcurrentStore<K, V, D>
where
    K: Hash + Eq + Clone + Debug + Send + Sync + 'static,
    V: AsyncDrop + Debug + Send + Sync + 'static,
    D: Clone + Debug + Send + Sync + 'static,
{
    type Error = Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        assert!(
            self.entries.lock().unwrap().is_empty(),
            "There are still loading tasks running or loaded entries being used by some tasks. Please wait for them to complete before dropping the ConcurrentStore."
        );
        Ok(())
    }
}

/// Basically the same as [EntryState], but the [Self::Dropping] state carries an additional member `loading_fn` so the `FnOnce` can be returned from
/// the method if not used.
enum CloneOrCreateEntryStateResult<V, F, R, I, D>
where
    V: AsyncDrop + Debug + Send + Sync + 'static,
    F: FnOnce(AsyncDropGuard<AsyncDropArc<I>>) -> R + Send,
    R: Future<Output = Result<Option<AsyncDropGuard<V>>>> + Send + 'static,
    I: AsyncDrop + Debug + Send,
    D: Clone + Debug + Send + Sync + 'static,
{
    Loading {
        loading_result: EntryLoadingWaiter,
    },
    Loaded {
        entry: AsyncDropGuard<AsyncDropArc<V>>,
    },
    Dropping {
        future: Shared<BoxFuture<'static, ()>>,
        _r: PhantomData<R>,
        _i: PhantomData<I>,
        loading_fn: F,
    },
    /// EntryState is either Loading or Loaded, but an immediate drop was requested. We need to block further accesses until dropping is complete.
    ImmediateDropRequested {
        on_dropped: mr_oneshot_channel::Receiver<D>,
        loading_fn: F,
    },
}

/// Result of requesting immediate drop of an entry.
pub enum RequestImmediateDropResult<D>
where
    D: Clone + Debug + Send + Sync + 'static,
{
    /// Immediate drop request accepted. The entry was either loading or loaded and the specified drop function will be executed with the entry,
    /// or the entry was not loaded and the specified drop function will be executed with None.
    ImmediateDropRequested {
        /// on_dropped will be completed once the entry has been fully dropped
        on_dropped: mr_oneshot_channel::Receiver<D>,
    },
    /// Immediate drop request failed because the entry is already in dropping state.
    /// This could be from the last task giving up its guard, or by an earlier immediate drop request.
    AlreadyDropping {
        future: Shared<BoxFuture<'static, ()>>,
    },
}
