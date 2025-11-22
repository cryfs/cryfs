use anyhow::Result;
use async_trait::async_trait;
use cryfs_utils::concurrent_store::{ConcurrentStore, RequestImmediateDropResult};
use cryfs_utils::mr_oneshot_channel;
use futures::future::{BoxFuture, Shared};
use std::fmt::Debug;
use std::sync::Arc;

use crate::filesystem::concurrentfsblobstore::loaded_blobs::guard::LoadedBlobGuard;
use crate::filesystem::fsblobstore::{FsBlob, FsBlobStore};
use cryfs_blobstore::{BlobId, BlobStore, RemoveResult};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard, AsyncDropTokioMutex};

#[derive(Debug)]
pub struct LoadedBlobs<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    // TODO Here (and in other places using BlockId or BlobId as hash map/set key), use a faster hash function, e.g. just take the first 8 bytes of the id. Ids are already random.
    loaded_blobs: AsyncDropGuard<
        AsyncDropArc<
            ConcurrentStore<
                BlobId,
                AsyncDropTokioMutex<FsBlob<B>>,
                Result<RemoveResult, Arc<anyhow::Error>>,
            >,
        >,
    >,
}

impl<B> LoadedBlobs<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    pub fn new() -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            loaded_blobs: AsyncDropArc::new(ConcurrentStore::new()),
        })
    }

    pub async fn try_insert_with_id<F>(
        &self,
        blob_id: BlobId,
        loading_fn: impl FnOnce() -> F + Send + 'static,
    ) -> Result<(), anyhow::Error>
    where
        F: Future<Output = Result<AsyncDropGuard<FsBlob<B>>>> + Send,
    {
        let loading_fn = move || async move { loading_fn().await.map(AsyncDropTokioMutex::new) };
        ConcurrentStore::try_insert_with_key(&self.loaded_blobs, blob_id, loading_fn).await
    }

    /// Insert a new blob that was just created and has a new blob id assigned.
    /// This must not be an existing blob id or it can cause race conditions or panics.
    /// This id also must not be used in any other calls before this completes.
    /// Only after this function call returns are we set up to deal with concurrent accesses.
    pub fn insert_with_new_id(
        &self,
        blob: AsyncDropGuard<FsBlob<B>>,
    ) -> AsyncDropGuard<LoadedBlobGuard<B>> {
        LoadedBlobGuard::new(ConcurrentStore::insert_with_new_key(
            &self.loaded_blobs,
            blob.blob_id(),
            AsyncDropTokioMutex::new(blob),
        ))
    }

    pub async fn get_loaded_or_insert_loading<F>(
        &self,
        blob_id: BlobId,
        blobstore: AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
        loading_fn: impl FnOnce(AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>) -> F + Send + 'static,
    ) -> Result<Option<AsyncDropGuard<LoadedBlobGuard<B>>>, anyhow::Error>
    where
        F: Future<Output = Result<Option<AsyncDropGuard<FsBlob<B>>>>> + Send + 'static,
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
    {
        let loading_fn =
            move |i| async move { loading_fn(i).await.map(|v| v.map(AsyncDropTokioMutex::new)) };
        ConcurrentStore::get_loaded_or_insert_loading(
            &self.loaded_blobs,
            blob_id,
            blobstore,
            loading_fn,
        )
        .await
        .map(|v| v.map(LoadedBlobGuard::new))
    }

    pub async fn get_if_loading_or_loaded(
        &self,
        blob_id: &BlobId,
    ) -> Result<Option<AsyncDropGuard<LoadedBlobGuard<B>>>, anyhow::Error> {
        ConcurrentStore::get_if_loading_or_loaded(&self.loaded_blobs, blob_id)
            .await
            .map(|v| v.map(LoadedBlobGuard::new))
    }

    pub fn request_removal(
        &self,
        blob_id: BlobId,
        blobstore: &AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
    ) -> RequestRemovalResult {
        let mut blobstore = AsyncDropArc::clone(blobstore);
        let request = self
            .loaded_blobs
            .request_immediate_drop(blob_id, move |blob| async move {
                if let Some(blob) = blob {
                    let blob = AsyncDropTokioMutex::into_inner(blob);
                    let remove_result = FsBlob::remove(blob).await.map_err(Arc::new);
                    blobstore
                        .async_drop()
                        .await
                        .map_err(|err| Arc::new(err.into()))?;
                    remove_result?;
                    Ok(RemoveResult::SuccessfullyRemoved)
                } else {
                    // The blob wasn't loaded, we can just remove it from the base store
                    // We're doing this within the drop handler of `request_immediate_drop()`, because that gives us
                    // exclusive access and blocks other tasks from loading this blob.
                    let result = blobstore.remove_by_id(&blob_id).await.map_err(Arc::new);
                    blobstore
                        .async_drop()
                        .await
                        .map_err(|err| Arc::new(err.into()))?;
                    result
                }
            });
        match request {
            RequestImmediateDropResult::ImmediateDropRequested { on_dropped }
            // An earlier immediate drop result can only be a remove request, because that's the only scenario in which we request immediate drops.
            // So we're fine and that earlier request will remove it for us.
            | RequestImmediateDropResult::AlreadyDroppingFromEarlierImmediateDrop { on_dropped } => {
                RequestRemovalResult::RemovalRequested {
                    on_removed: on_dropped,
                }
            }
            RequestImmediateDropResult::AlreadyDroppingWithoutImmediateDrop { future } => {
                RequestRemovalResult::AlreadyDropping { future }
            }
        }
    }
}

#[async_trait]
impl<B> AsyncDrop for LoadedBlobs<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.loaded_blobs.async_drop().await?;
        Ok(())
    }
}

pub enum RequestRemovalResult {
    /// Removal request accepted
    RemovalRequested {
        /// on_dropped will be completed once the entry has been fully dropped
        on_removed: mr_oneshot_channel::Receiver<Result<RemoveResult, Arc<anyhow::Error>>>,
    },
    /// Removal failed because the entry is already in dropping state.
    AlreadyDropping {
        future: Shared<BoxFuture<'static, ()>>,
    },
}
