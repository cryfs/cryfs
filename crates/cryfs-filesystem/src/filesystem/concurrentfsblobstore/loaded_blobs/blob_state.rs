use std::{fmt::Debug, sync::Arc};

use anyhow::Error;
use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard, AsyncDropTokioMutex},
    mr_oneshot_channel, safe_panic,
};
use futures::{
    FutureExt as _,
    future::{BoxFuture, Shared},
};

use crate::filesystem::{
    concurrentfsblobstore::loaded_blobs::{LoadedBlobGuard, LoadedBlobs},
    fsblobstore::FsBlob,
};

pub(super) enum BlobState<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    Loading(BlobStateLoading),
    Loaded(BlobStateLoaded<B>),
    Dropping(BlobStateDropping),
}

impl<B> Debug for BlobState<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlobState::Loading { .. } => write!(f, "Loading"),
            BlobState::Loaded(BlobStateLoaded {
                blob,
                num_unfulfilled_waiters,
                removal_request,
            }) => write!(
                f,
                "Loaded({:?}, {}, {})",
                blob,
                num_unfulfilled_waiters,
                match removal_request {
                    RemovalRequest::NotRequested => "removal not requested",
                    RemovalRequest::Requested { .. } => "removal requested",
                }
            ),
            BlobState::Dropping { .. } => write!(f, "Dropping"),
        }
    }
}

pub(super) struct BlobStateLoading {
    // TODO No BoxFuture
    /// loading_result is a future that will hold the result of the loading operation once it is complete.
    /// See [LoadingResult] for an explanation of the possible results.
    loading_result: Shared<BoxFuture<'static, LoadingResult>>,
    /// Number of tasks currently waiting for this blob to be loaded. This is only ever incremented. Even if a waiter completes, it won't be decremented.
    num_waiters: usize,
    /// If RemovalRequest::Requested: While we're loading, another thread triggered a remove for this blob. Don't allow further loaders, and when this is unloaded, remove the blob.
    removal_request: RemovalRequest,
}

impl BlobStateLoading {
    pub fn new(loading_result: BoxFuture<'static, LoadingResult>) -> Self {
        BlobStateLoading {
            loading_result: loading_result.shared(),
            num_waiters: 0,
            removal_request: RemovalRequest::NotRequested,
        }
    }

    pub fn new_dummy() -> Self {
        BlobStateLoading {
            loading_result: futures::future::pending().boxed().shared(),
            num_waiters: 0,
            removal_request: RemovalRequest::NotRequested,
        }
    }

    pub fn add_waiter(&mut self) -> BlobLoadingWaiter {
        self.num_waiters += 1;
        BlobLoadingWaiter::new(self.loading_result.clone())
    }

    pub fn num_waiters(&self) -> usize {
        self.num_waiters
    }

    pub fn request_removal(&mut self) -> mr_oneshot_channel::Receiver<Result<(), Arc<Error>>> {
        self.removal_request.request_removal()
    }

    pub fn removal_requested(
        &self,
    ) -> Option<mr_oneshot_channel::Receiver<Result<(), Arc<Error>>>> {
        self.removal_request.removal_requested()
    }
}

/// Handle for a task waiting for a blob to be loaded.
/// This can be redeemed against the blob once loading is completed.
/// It is an RAII type that ensures that the number of waiters is correctly tracked.
#[must_use]
pub(super) struct BlobLoadingWaiter {
    // Alway Some unless destructed
    loading_result: Option<Shared<BoxFuture<'static, LoadingResult>>>,
}

impl BlobLoadingWaiter {
    pub fn new(loading_result: Shared<BoxFuture<'static, LoadingResult>>) -> Self {
        BlobLoadingWaiter {
            loading_result: Some(loading_result),
        }
    }

    pub async fn wait_until_loaded<B>(
        mut self,
        loaded_blobs: &AsyncDropGuard<AsyncDropArc<LoadedBlobs<B>>>,
        blob_id: BlobId,
    ) -> anyhow::Result<Option<AsyncDropGuard<LoadedBlobGuard<B>>>>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
        <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
    {
        match self.loading_result.take().expect("Already dropped").await {
            LoadingResult::Loaded => {
                // _finalize_waiter will decrement the num_waiters refcount
                Ok(Some(Self::_finalize_waiter(loaded_blobs, blob_id)))
            }
            LoadingResult::NotFound => {
                // No need to decrement the num_waiters refcount here because the blob never made it to the Loaded state
                Ok(None)
            }
            LoadingResult::Error(err) => {
                // No need to decrement the num_waiters refcount here because the blob never made it to the Loaded state
                Err(anyhow::anyhow!(
                    "Error while try_insert'ing blob with id {}: {}",
                    blob_id,
                    err
                ))
            }
        }
    }

    fn _finalize_waiter<B>(
        loaded_blobs: &AsyncDropGuard<AsyncDropArc<LoadedBlobs<B>>>,
        blob_id: BlobId,
    ) -> AsyncDropGuard<LoadedBlobGuard<B>>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
        <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
    {
        // This is not a race condition with dropping, i.e. the blob can't be in dropping state yet, because we are an "unfulfilled waiter",
        // i.e. the blob cannot be dropped until we decrease the count below.
        let mut blobs = loaded_blobs.blobs.lock().unwrap();
        let Some(state) = blobs.get_mut(&blob_id) else {
            panic!("Blob with id {} was not found in the map", blob_id);
        };
        let BlobState::Loaded(loaded) = state else {
            panic!("Blob with id {} is not in loaded state", blob_id);
        };
        LoadedBlobGuard::new(
            AsyncDropArc::clone(loaded_blobs),
            blob_id,
            // [Self::_clone_or_create_blob_state] added a waiter, so we need to decrement num_unfulfilled_waiters.
            loaded.get_blob_and_decrease_num_unfulfilled_waiters(),
        )
    }
}

impl Drop for BlobLoadingWaiter {
    fn drop(&mut self) {
        if self.loading_result.is_some() {
            safe_panic!(
                "BlobLoadingWaiter was dropped without being awaited. This will lead to a memory leak because the number of waiters will not be decremented."
            );
        }
    }
}

pub(super) struct BlobStateLoaded<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    // TODO If we change FsBlob's API to not require `&mut`, then we can probably remove the AsyncDropTokioMutex here. Or should we use a RwLock?
    blob: AsyncDropGuard<AsyncDropArc<AsyncDropTokioMutex<FsBlob<B>>>>,
    /// Number of tasks that started waiting for this blob when it was in [BlobStateLoading],
    /// but haven't yet incremented the refcount of [Self::blob].
    /// This gets never increased, only initialized when the blob is loaded and decreased when a waiter gets its clone of the AsyncDropArc.
    /// If this is non-zero, then we shouldn't prune the blob yet even if the refcount is zero.
    num_unfulfilled_waiters: usize,
    /// If RemovalRequest::Requested: While we're loading, another thread triggered a remove for this blob. Don't allow further loaders, and when this is unloaded, remove the blob.
    removal_request: RemovalRequest,
}

impl<B> BlobStateLoaded<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    pub fn new_from_just_finished_loading(
        blob: AsyncDropGuard<FsBlob<B>>,
        loading: BlobStateLoading,
    ) -> Self {
        BlobStateLoaded {
            blob: AsyncDropArc::new(AsyncDropTokioMutex::new(blob)),
            num_unfulfilled_waiters: loading.num_waiters(),
            removal_request: loading.removal_request,
        }
    }

    pub fn new_without_unfulfilled_waiters(blob: AsyncDropGuard<FsBlob<B>>) -> Self {
        BlobStateLoaded {
            blob: AsyncDropArc::new(AsyncDropTokioMutex::new(blob)),
            num_unfulfilled_waiters: 0,
            removal_request: RemovalRequest::NotRequested,
        }
    }

    pub fn get_blob(&self) -> AsyncDropGuard<AsyncDropArc<AsyncDropTokioMutex<FsBlob<B>>>> {
        AsyncDropArc::clone(&self.blob)
    }

    fn get_blob_and_decrease_num_unfulfilled_waiters(
        &mut self,
    ) -> AsyncDropGuard<AsyncDropArc<AsyncDropTokioMutex<FsBlob<B>>>> {
        assert!(self.num_unfulfilled_waiters > 0);
        self.num_unfulfilled_waiters -= 1;
        AsyncDropArc::clone(&self.blob)
    }

    pub fn num_tasks_with_access(&self) -> usize {
        // num_unfulfilled_waiters are tasks that are waiting to get access to the blob, and will increment the refcount when they do.
        // We subtract 1 from the strong count because we don't want to count our self reference.
        self.num_unfulfilled_waiters + AsyncDropArc::strong_count(&self.blob) - 1
    }

    pub fn into_inner(self) -> (RemovalRequest, AsyncDropGuard<FsBlob<B>>) {
        assert!(
            self.num_unfulfilled_waiters == 0,
            "Cannot consume BlobStateLoaded while there are unfulfilled waiters"
        );
        let blob = AsyncDropTokioMutex::into_inner(AsyncDropArc::try_unwrap(self.blob).unwrap());
        (self.removal_request, blob)
    }

    pub fn request_removal(&mut self) -> mr_oneshot_channel::Receiver<Result<(), Arc<Error>>> {
        self.removal_request.request_removal()
    }

    pub fn removal_requested(
        &self,
    ) -> Option<mr_oneshot_channel::Receiver<Result<(), Arc<Error>>>> {
        self.removal_request.removal_requested()
    }
}

pub(super) struct BlobStateDropping {
    pub future: Shared<BoxFuture<'static, ()>>,
}

#[derive(Clone)]
pub(super) enum LoadingResult {
    /// The blob was successfully loaded. This loading result means the blob state was already changed to [BlobState::Loaded] and can be accessed immediately.
    Loaded,

    /// The blob was not found. The blob state was removed from the map.
    NotFound,

    /// An error occurred while loading the blob. The blob state was removed from the map.
    Error(Arc<anyhow::Error>),
}

pub(super) enum RemovalRequest {
    NotRequested,
    Requested {
        removal_result_sender: mr_oneshot_channel::Sender<Result<(), Arc<Error>>>,
    },
}

impl RemovalRequest {
    pub fn request_removal(&mut self) -> mr_oneshot_channel::Receiver<Result<(), Arc<Error>>> {
        match self {
            RemovalRequest::Requested {
                removal_result_sender,
                ..
            } => removal_result_sender.subscribe(),
            RemovalRequest::NotRequested => {
                let (removal_result_sender, removal_result_receiver) =
                    mr_oneshot_channel::channel();
                *self = RemovalRequest::Requested {
                    removal_result_sender,
                };
                removal_result_receiver
            }
        }
    }

    pub fn removal_requested(
        &self,
    ) -> Option<mr_oneshot_channel::Receiver<Result<(), Arc<Error>>>> {
        match self {
            RemovalRequest::Requested {
                removal_result_sender,
                ..
            } => Some(removal_result_sender.subscribe()),
            RemovalRequest::NotRequested => None,
        }
    }
}
