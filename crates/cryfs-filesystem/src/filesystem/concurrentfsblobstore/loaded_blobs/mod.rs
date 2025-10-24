use anyhow::Result;
use async_trait::async_trait;
use cryfs_rustfs::FsResult;
use cryfs_utils::event::Event;
use cryfs_utils::with_async_drop_2;
use futures::FutureExt;
use futures::future::{BoxFuture, Shared};
use std::sync::{Arc, Mutex};
use std::{
    collections::{HashMap, hash_map::Entry},
    fmt::Debug,
};

use crate::filesystem::fsblobstore::{FsBlob, FsBlobStore};
use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard, AsyncDropTokioMutex};

mod blob_state;
use blob_state::{BlobState, BlobStateDropping, BlobStateLoaded, BlobStateLoading, LoadingResult};

mod guard;
pub use guard::LoadedBlobGuard;

// TODO Now that we have the new approach of Shared<LoadingResult>, can we actually apply this with a FsBlob<'a, B> instead of FsBlob<B>?

// TODO This is currently not cancellation safe. If a task waiting for a blob to load is cancelled, the num_waiters and num_unfulfilled_waiters counts will be wrong.

// TODO This is pretty similar to lockable::LockableHashMap, just that we're giving out handles to unlocked entries to multiple tasks and they can use those handles to lock entries. Can we maybe add that feature to lockable and use that instead?

#[derive(Debug)]
pub struct LoadedBlobs<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    /// [BlobState::Loading]:
    ///  * Loading of the blob is in progress, or has completed but we haven't updated the state yet.
    ///  * [BlobStateLoading::num_waiters] counts how many tasks are currently waiting for the loading to complete.
    ///  * Upon completion, the future itself will change the state to [BlobState::Loaded], or remove the entry from the map if loading failed or the blob wasn't found.
    /// [BlobState::Loaded]:
    ///  * The blob is loaded and ready to use.
    ///  * There can be multiple tasks with a reference to the [AsyncDropArc] in [BlobStateLoaded].
    ///  * There are still [BlobStateLoaded::num_unfulfilled_waiters] other tasks waiting for the blob to be loaded. Their future is ready, but they haven't polled it yet.
    ///  * When the last reference to the [AsyncDropArc] is dropped, the [LoadedBlobGuard] will ensure we call [Self::unload] to change to [BlobState::Dropping]
    /// [BlobState::Dropping]:
    ///  * The blob was loaded but is now in the middle of async_drop
    ///  * There are no tasks waiting for the blob to be loaded anymore (unless they were separately created after we started dropping, but those will first wait until dropping is complete and then start their loading attempt).
    ///  * There are no tasks with a reference to the [AsyncDropArc] anymore, the last task is just executing the async_drop.
    /// Blob Removal:
    ///   * In both, [BlobState::Loading] and [BlobState::Loaded], if a removal is requested, the flag `removal_requested` is set to true.
    ///   * If the flag is true, the blob will be automatically removed instead of unloaded when the last reference is dropped.
    ///   * Also, any further tasks trying to load the blob while `removal_requested=true` will wait until all tasks with current access are done, the removal is complete, and then execute.
    // TODO Here (and in other places using BlockId or BlobId as hash map/set key), use a faster hash function, e.g. just take the first 8 bytes of the id. Ids are already random.
    blobs: Arc<Mutex<HashMap<BlobId, BlobState<B>>>>,
}

impl<B> LoadedBlobs<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    pub fn new() -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            blobs: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn try_insert_with_id<F>(
        this: &AsyncDropGuard<AsyncDropArc<Self>>,
        blob_id: BlobId,
        loading_fn: impl FnOnce() -> F + Send + 'static,
    ) -> Result<(), anyhow::Error>
    where
        F: Future<Output = Result<AsyncDropGuard<FsBlob<B>>>> + Send,
    {
        let loading_future = {
            let mut blobs = this.blobs.lock().unwrap();
            match blobs.entry(blob_id) {
                Entry::Occupied(_) => Err(anyhow::anyhow!(
                    "Blob with id {} is already loaded",
                    blob_id
                )),
                Entry::Vacant(entry) => {
                    let loading_future = async move {
                        let loaded_blob = loading_fn().await?;
                        Ok(Some(loaded_blob))
                    };
                    let mut loading_future = this.make_loading_future(blob_id, loading_future);
                    let loading_result = loading_future.add_waiter();
                    entry.insert(BlobState::Loading(loading_future));
                    Ok(loading_result)
                }
            }
        }?;

        // Now the lock on `blobs` is released, so we can await the loading future without blocking other operations.
        let loading_result = loading_future.await;
        match loading_result {
            LoadingResult::Loaded => {
                // We added a waiter above, so we need to finalize it to decrement num_unfulfilled_waiters.
                // TODO This could be optimized, no need to increase and then decrease the refcount for the blob.
                //      BUT: We need to be careful. The current code assumes that any unfulfilled waiter will eventually
                //      (increment the refcount and then on decrement) call [Self::unload]. If we don't increment the refcount here,
                //      we need to ensure we still call [Self::unload] to check if we need to drop the blob.
                Self::_finalize_waiter(this, blob_id).async_drop().await?;
                Ok(())
            }
            LoadingResult::NotFound => {
                panic!("This shouldn't happen, our loading_future always returns Some");
            }
            LoadingResult::Error(err) => Err(anyhow::anyhow!(
                "Error while try_insert'ing blob with id {}: {}",
                blob_id,
                err
            )),
        }
    }

    /// Insert a new blob that was just created and has a new blob id assigned.
    /// This must not be an existing blob id or it can cause race conditions or panics.
    /// This id also must not be used in any other calls before this completes.
    /// Only after this function call returns are we set up to deal with concurrent accesses.
    pub fn insert_with_new_id(
        this: &AsyncDropGuard<AsyncDropArc<Self>>,
        blob: AsyncDropGuard<FsBlob<B>>,
    ) -> AsyncDropGuard<LoadedBlobGuard<B>> {
        let blob_id = blob.blob_id();

        // We now have the newly assigned blob id and are fully loaded. Insert it into the map.
        let mut blobs = this.blobs.lock().unwrap();
        let loaded = BlobStateLoaded::new(
            blob, 0, None, // No unfulfilled waiters, we just created it
        );
        let loaded_blob = loaded.get_blob_without_decreasing_num_unfulfilled_waiters();
        match blobs.entry(blob_id) {
            Entry::Occupied(_) => {
                panic!("Blob with id {blob_id} is already loaded even though we just created it",);
            }
            Entry::Vacant(entry) => {
                entry.insert(BlobState::Loaded(loaded));
            }
        }

        LoadedBlobGuard::new(AsyncDropArc::clone(this), blob_id, loaded_blob)
    }

    pub async fn get_loaded_or_insert_loading<F>(
        this: &AsyncDropGuard<AsyncDropArc<Self>>,
        blob_id: BlobId,
        blobstore: AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
        mut loading_fn: impl FnOnce(AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>) -> F + Send,
    ) -> Result<Option<AsyncDropGuard<LoadedBlobGuard<B>>>, anyhow::Error>
    where
        F: Future<Output = Result<Option<AsyncDropGuard<FsBlob<B>>>>> + Send + 'static,
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
    {
        with_async_drop_2!(blobstore, {
            loop {
                let blob_state = this._clone_or_create_blob_state(blob_id, &blobstore, loading_fn);
                // Now the lock on `this.blobs` is released, so we can await the loading future without blocking other operations.

                match blob_state {
                    CloneOrCreateBlobStateResult::Loaded { blob } => {
                        // Oh, the blob is already loaded! We can just return it
                        // Returning means we shift the responsibility to call async_drop on the [AsyncDropArc] to our caller.
                        return Ok(Some(LoadedBlobGuard::new(
                            AsyncDropArc::clone(this),
                            blob_id,
                            blob,
                        )));
                    }
                    CloneOrCreateBlobStateResult::Loading { loading_result } => {
                        let loading_result = loading_result.await;
                        return match loading_result {
                            LoadingResult::Loaded => {
                                // The blob was successfully loaded, we can get the result from the map
                                Ok(Some(Self::_finalize_waiter(this, blob_id)))
                            }
                            LoadingResult::NotFound => Ok(None),
                            LoadingResult::Error(err) => Err(anyhow::anyhow!(
                                "Error while try_insert'ing blob with id {}: {}",
                                blob_id,
                                err
                            )),
                        };
                    }
                    CloneOrCreateBlobStateResult::Dropping {
                        future,
                        loading_fn: returned_loading_fn,
                    } => {
                        future.await;
                        // Reset the `loading_fn` [FnOnce]. Since `_clone_or_create_blob_state` didn't use it, it returned it and we can use it in the next iteration.
                        loading_fn = returned_loading_fn;
                        // After the drop is complete, we can try to load the blob again.
                        continue;
                    }
                    CloneOrCreateBlobStateResult::RemovalRequested {
                        on_removed,
                        loading_fn: returned_loading_fn,
                    } => {
                        // Blob is either loading or loaded, but its removal was requested. Let's wait until current processing is complete, removal was processed, and then retry.
                        on_removed.wait().await;
                        // Reset the `loading_fn` [FnOnce]. Since `_clone_or_create_blob_state` didn't use it, it returned it and we can use it in the next iteration.
                        loading_fn = returned_loading_fn;
                        continue;
                    }
                }
            }
        })
    }

    fn _finalize_waiter(
        this: &AsyncDropGuard<AsyncDropArc<Self>>,
        blob_id: BlobId,
    ) -> AsyncDropGuard<LoadedBlobGuard<B>> {
        // This is not a race condition with dropping because we are an "unfulfilled waiter", i.e. the blob cannot be dropped until we decrease the count below.
        let mut blobs = this.blobs.lock().unwrap();
        let Some(state) = blobs.get_mut(&blob_id) else {
            panic!("Blob with id {} was not found in the map", blob_id);
        };
        let BlobState::Loaded(loaded) = state else {
            panic!("Blob with id {} is not in loaded state", blob_id);
        };
        LoadedBlobGuard::new(
            AsyncDropArc::clone(this),
            blob_id,
            // [Self::_clone_or_create_blob_state] added a waiter, so we need to decrement num_unfulfilled_waiters.
            loaded.get_blob_and_decrease_num_unfulfilled_waiters(),
        )
    }

    /// Note: This function clones a [BlobState], which may clone the [AsyncDropGuard] contained if it is [BlobState::Loaded]. It is the callers responsibility to async_drop that.
    fn _clone_or_create_blob_state<F, R>(
        &self,
        blob_id: BlobId,
        blobstore: &AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
        loading_fn: F,
    ) -> CloneOrCreateBlobStateResult<B, F, R>
    where
        F: FnOnce(AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>) -> R + Send,
        R: Future<Output = Result<Option<AsyncDropGuard<FsBlob<B>>>>> + Send + 'static,
        B: Sync,
    {
        let mut blobs = self.blobs.lock().unwrap();
        match blobs.entry(blob_id) {
            Entry::Occupied(mut entry) => match entry.get_mut() {
                BlobState::Loaded(loaded) => match loaded.removal_requested() {
                    Some(removal_requested) => CloneOrCreateBlobStateResult::RemovalRequested {
                        on_removed: removal_requested.clone(),
                        loading_fn,
                    },
                    None => CloneOrCreateBlobStateResult::Loaded {
                        // We didn't add a waiter, so we don't need to decrement num_unfulfilled_waiters.
                        blob: loaded.get_blob_without_decreasing_num_unfulfilled_waiters(),
                    },
                },
                BlobState::Loading(loading) => match loading.removal_requested() {
                    Some(removal_requested) => CloneOrCreateBlobStateResult::RemovalRequested {
                        on_removed: removal_requested.clone(),
                        loading_fn,
                    },
                    None => CloneOrCreateBlobStateResult::Loading {
                        // The caller is responsible for decreasing num_unfulfilled_waiters when it gets the blob.
                        // TODO Can we make this safer by encapsulating it into a `Waiter` struct that automatically decreases it when the Arc<FsBlob> is cloned?
                        loading_result: loading.add_waiter(),
                    },
                },
                BlobState::Dropping(state) => CloneOrCreateBlobStateResult::Dropping {
                    future: state.future.clone(),
                    loading_fn,
                },
            },
            Entry::Vacant(entry) => {
                // No loading operation is in progress, so we start a new one.
                let mut loading_future =
                    self.make_loading_future(blob_id, loading_fn(AsyncDropArc::clone(blobstore)));
                let loading_result = loading_future.add_waiter();
                entry.insert(BlobState::Loading(loading_future));
                CloneOrCreateBlobStateResult::Loading { loading_result }
            }
        }
    }

    fn make_loading_future(
        &self,
        blob_id: BlobId,
        loading_fn: impl Future<Output = Result<Option<AsyncDropGuard<FsBlob<B>>>>> + Send + 'static,
    ) -> BlobStateLoading {
        let blobs = Arc::clone(&self.blobs);
        let loading_task = async move {
            // Run loading_fn concurrently, without a lock on `blobs`.
            let result = loading_fn.await;

            match result {
                Ok(Some(blob)) => {
                    // Now that we're loaded, change the blob state in the map to loaded so that any waiters (current or future) can access it.
                    let mut blobs = blobs.lock().unwrap();
                    let Some(state) = blobs.get_mut(&blob_id) else {
                        panic!("Blob with id {} was not found in the map", blob_id);
                    };
                    let BlobState::Loading(loading) = state else {
                        panic!("Blob with id {} is not in loading state", blob_id);
                    };
                    // We are still in the middle of executing the future, so no waiter has completed yet.
                    // Also, we currently have a lock on `blob`, so no new waiters can be added.
                    // We're about to change the state to BlobState::Loaded, which will prevent further waiters to be added even after we release the lock.
                    let num_waiters = loading.num_waiters();
                    *state = BlobState::Loaded(BlobStateLoaded::new(
                        blob,
                        num_waiters,
                        loading.removal_requested().cloned(), // TODO No clone
                    ));

                    LoadingResult::Loaded
                }
                Ok(None) => {
                    // We're loaded but the blob wasn't found. Remove the entry from the map and return the error to any waiters.
                    let mut blobs = blobs.lock().unwrap();
                    let Some(_) = blobs.remove(&blob_id) else {
                        panic!("Blob with id {} was not found in the map", blob_id);
                    };
                    LoadingResult::NotFound
                }
                Err(err) => {
                    // An error occurred while loading the blob. Remove the entry from the map and return the error to any waiters.
                    let mut blobs = blobs.lock().unwrap();
                    let Some(_) = blobs.remove(&blob_id) else {
                        panic!("Blob with id {} was not found in the map", blob_id);
                    };
                    LoadingResult::Error(Arc::new(err))
                }
            }
        };
        BlobStateLoading::new(loading_task.boxed())
    }

    pub fn request_removal(&self, blob_id: BlobId) -> RequestRemovalResult {
        let mut blobs = self.blobs.lock().unwrap();
        match blobs.entry(blob_id) {
            Entry::Occupied(mut entry) => match entry.get_mut() {
                BlobState::Loaded(loaded) => RequestRemovalResult::RemovalRequested {
                    on_removed: loaded.request_removal(),
                },
                BlobState::Loading(loading) => RequestRemovalResult::RemovalRequested {
                    on_removed: loading.request_removal(),
                },
                BlobState::Dropping(BlobStateDropping { future, .. }) => {
                    RequestRemovalResult::Dropping {
                        future: future.clone(),
                    }
                }
            },
            Entry::Vacant(_) => RequestRemovalResult::NotLoaded,
        }
    }

    /// Called by [LoadedBlobGuard] when it is dropped.
    async fn unload(
        &self,
        blob_id: BlobId,
        mut blob: AsyncDropGuard<AsyncDropArc<AsyncDropTokioMutex<FsBlob<B>>>>,
    ) -> FsResult<()> {
        // First drop the blob to decrement the reference count
        blob.async_drop().await.unwrap(); // TODO No unwrap? But what to do if it fails?
        std::mem::drop(blob);

        // Now check if we're the last reference. If yes, remove the blob.
        self._drop_if_no_references(blob_id).await;
        Ok(())
    }

    async fn _drop_if_no_references(&self, blob_id: BlobId) {
        let drop_future = {
            let mut blobs = self.blobs.lock().unwrap();
            let Entry::Occupied(mut entry) = blobs.entry(blob_id) else {
                panic!("Blob with id {} was not found in the map", blob_id);
            };
            match entry.get() {
                BlobState::Loading(_) => {
                    panic!("unload called but the blob is still loading. This should not happen.");
                }
                BlobState::Loaded(loaded) => {
                    if loaded.num_tasks_with_access() == 0 {
                        // The reference in the map is the last reference, so we can drop the blob
                        let blob = entry.insert(BlobState::Dropping(BlobStateDropping {
                            // dummy future, will be replaced
                            future: futures::future::ready(()).boxed().shared(),
                        }));
                        let BlobState::Loaded(loaded) = blob else {
                            unreachable!("We already checked the state above, it should be Loaded");
                        };
                        let drop_future = self.make_drop_future(loaded);
                        *entry.get_mut() = BlobState::Dropping(BlobStateDropping {
                            future: drop_future.clone(),
                        });
                        Some(drop_future)
                    } else {
                        // There are still references to the blob, so we just return
                        None
                    }
                }
                BlobState::Dropping { .. } => {
                    // Because of the way unload releases the references (and reduces the reference count) without a lock before
                    // calling into this function, there is a race condition and it is possible that multiple tasks unloading the
                    // same blob both first decrement the refcount, which then reaches zero, and then both call into here.
                    // The first one will change the state to Dropping, the second one will find it already in Dropping state.
                    // We can just ignore that second call since the first call will take care of dropping the blob.
                    None
                }
            }
        };

        if let Some(drop_future) = drop_future {
            // Now the entry is marked as `Dropping` and the lock on `blobs` is released. We can await the drop.
            drop_future.await;
        }
    }

    fn make_drop_future(&self, loaded: BlobStateLoaded<B>) -> Shared<BoxFuture<'static, ()>> {
        let removal_requested = loaded.removal_requested().cloned(); // TODO No clone
        let mut blob = loaded.into_inner();
        let blob_id = blob.blob_id();
        let blobs = Arc::clone(&self.blobs);
        async move {
            let remove_entry_fn = || async {
                // Remove the entry from the map
                let mut blobs = blobs.lock().unwrap();
                let Some(entry) = blobs.remove(&blob_id) else {
                    panic!("Blob with id {} was not found in the map", blob_id);
                };
                let BlobState::Dropping { .. } = entry else {
                    panic!("Blob with id {} is not in dropping state", blob_id);
                };
            };
            // This will be awaited after the lock on blobs is released, so we can concurrently drop
            // the blob without blocking other operations.
            if let Some(removal_requested) = removal_requested {
                // If removal was requested, execute the removal
                FsBlob::remove(blob).await.unwrap(); // TODO No unwrap
                remove_entry_fn().await;
                removal_requested.trigger();
            } else {
                // otherwise just drop the blob
                blob.async_drop().await.unwrap(); // TODO No unwrap
                remove_entry_fn().await;
            }
        }
        .boxed()
        .shared()
    }
}

#[async_trait]
impl<B> AsyncDrop for LoadedBlobs<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        assert!(
            self.blobs.lock().unwrap().is_empty(),
            "There are still loading tasks running or loaded blobs being used by some tasks. Please wait for them to complete before dropping the LoadedBlobs."
        );
        Ok(())
    }
}

/// Basically the same as [BlobState], but the [Self::Dropping] state carries an additional member `loading_fn` so the `FnOnce` can be returned from
/// the method if not used.
enum CloneOrCreateBlobStateResult<B, F, R>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
    F: FnOnce(AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>) -> R + Send,
    R: Future<Output = Result<Option<AsyncDropGuard<FsBlob<B>>>>> + Send + 'static,
{
    Loading {
        loading_result: Shared<BoxFuture<'static, LoadingResult>>,
    },
    Loaded {
        blob: AsyncDropGuard<AsyncDropArc<AsyncDropTokioMutex<FsBlob<B>>>>,
    },
    Dropping {
        future: Shared<BoxFuture<'static, ()>>,
        loading_fn: F,
    },
    /// BlobState is either Loading or Loaded, but removal was requested. We need to block further accesses until removal is complete.
    RemovalRequested { on_removed: Event, loading_fn: F },
}

pub enum RequestRemovalResult {
    RemovalRequested {
        on_removed: Event,
    },
    NotLoaded,
    Dropping {
        future: Shared<BoxFuture<'static, ()>>,
    },
}
