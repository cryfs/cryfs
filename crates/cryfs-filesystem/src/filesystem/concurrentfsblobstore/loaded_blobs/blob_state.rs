use std::{fmt::Debug, sync::Arc};

use cryfs_blobstore::BlobStore;
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard, AsyncDropTokioMutex},
    event::Event,
};
use futures::{
    FutureExt as _,
    future::{BoxFuture, Shared},
};

use crate::filesystem::fsblobstore::FsBlob;

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
                removal_requested,
            }) => write!(
                f,
                "Loaded({:?}, {}, {})",
                blob,
                num_unfulfilled_waiters,
                if removal_requested.is_some() {
                    "removal requested"
                } else {
                    "no removal requested"
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
    /// If Some: While we're loading, another thread triggered a remove for this blob. Don't allow further loaders, and when this is unloaded, remove the blob. The event will be triggered once removal is complete.
    /// If None: No removal was requested.
    removal_requested: Option<Event>,
}

impl BlobStateLoading {
    pub fn new(loading_result: BoxFuture<'static, LoadingResult>) -> Self {
        BlobStateLoading {
            loading_result: loading_result.shared(),
            num_waiters: 0,
            removal_requested: None,
        }
    }

    pub fn add_waiter(&mut self) -> Shared<BoxFuture<'static, LoadingResult>> {
        self.num_waiters += 1;
        self.loading_result.clone()
    }

    pub fn num_waiters(&self) -> usize {
        self.num_waiters
    }

    pub fn request_removal(&mut self) -> Event {
        if let Some(event) = &self.removal_requested {
            event.clone()
        } else {
            let event = Event::new();
            self.removal_requested = Some(event.clone());
            event
        }
    }

    pub fn removal_requested(&self) -> Option<&Event> {
        self.removal_requested.as_ref()
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
    /// If Some: While we're loading, another thread triggered a remove for this blob. Don't allow further loaders, and when this is unloaded, remove the blob. The event will be triggered once removal is complete.
    /// If None: No removal was requested.
    removal_requested: Option<Event>,
}

impl<B> BlobStateLoaded<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    pub fn new(
        blob: AsyncDropGuard<FsBlob<B>>,
        num_unfulfilled_waiters: usize,
        removal_requested: Option<Event>,
    ) -> Self {
        BlobStateLoaded {
            blob: AsyncDropArc::new(AsyncDropTokioMutex::new(blob)),
            num_unfulfilled_waiters,
            removal_requested,
        }
    }

    pub fn get_blob_without_decreasing_num_unfulfilled_waiters(
        &self,
    ) -> AsyncDropGuard<AsyncDropArc<AsyncDropTokioMutex<FsBlob<B>>>> {
        AsyncDropArc::clone(&self.blob)
    }

    pub fn get_blob_and_decrease_num_unfulfilled_waiters(
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

    pub fn into_inner(self) -> AsyncDropGuard<FsBlob<B>> {
        assert!(
            self.num_unfulfilled_waiters == 0,
            "Cannot consume BlobStateLoaded while there are unfulfilled waiters"
        );
        AsyncDropTokioMutex::into_inner(AsyncDropArc::try_unwrap(self.blob).unwrap())
    }

    pub fn request_removal(&mut self) -> Event {
        if let Some(event) = &self.removal_requested {
            event.clone()
        } else {
            let event = Event::new();
            self.removal_requested = Some(event.clone());
            event
        }
    }

    pub fn removal_requested(&self) -> Option<&Event> {
        self.removal_requested.as_ref()
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
