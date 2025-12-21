use anyhow::Result;
use async_trait::async_trait;
use cryfs_concurrent_store::{ConcurrentStore, RequestImmediateDropResult};
use futures::future::{BoxFuture, Shared};
use lockable::InfallibleUnwrap as _;
use std::fmt::Debug;
use std::sync::Arc;

use crate::concurrentfsblobstore::loaded_blobs::guard::LoadedBlobGuard;
use crate::fsblobstore::{FsBlob, FsBlobStore};
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
        // TODO Would ConcurrentStore<_, _, FsError> be better?
        ConcurrentStore<BlobId, AsyncDropTokioMutex<FsBlob<B>>, Arc<anyhow::Error>>,
    >,
}

impl<B> LoadedBlobs<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    pub fn new() -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            loaded_blobs: ConcurrentStore::new(),
        })
    }

    pub async fn try_insert_loading<F>(
        &self,
        blob_id: BlobId,
        loading_fn: impl FnOnce() -> F + Send + 'static,
    ) -> Result<(), Arc<anyhow::Error>>
    where
        F: Future<Output = Result<AsyncDropGuard<FsBlob<B>>, Arc<anyhow::Error>>> + Send,
    {
        let loading_fn = move || async move { loading_fn().await.map(AsyncDropTokioMutex::new) };
        let mut inserted = self
            .loaded_blobs
            .try_insert_loading(blob_id, loading_fn)?
            .wait_until_inserted()
            .await?;
        inserted.async_drop().await.infallible_unwrap();
        Ok(())
    }

    pub async fn try_insert_loaded(
        &self,
        blob: AsyncDropGuard<FsBlob<B>>,
    ) -> Result<AsyncDropGuard<LoadedBlobGuard<B>>> {
        let blob_id = blob.blob_id();
        match self
            .loaded_blobs
            .try_insert_loaded(blob_id, AsyncDropTokioMutex::new(blob))
        {
            Ok(guard) => Ok(LoadedBlobGuard::new(guard)),
            Err(mut value) => {
                // Entry already exists - async_drop the value and return error
                value.async_drop().await?;
                Err(anyhow::anyhow!(
                    "Blob with id {:?} already exists",
                    blob_id
                ))
            }
        }
    }

    pub async fn get_loaded_or_insert_loading<F>(
        &self,
        blob_id: BlobId,
        blobstore: &AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
        loading_fn: impl FnOnce(AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>) -> F + Send + 'static,
    ) -> Result<Option<AsyncDropGuard<LoadedBlobGuard<B>>>, Arc<anyhow::Error>>
    where
        F: Future<Output = Result<Option<AsyncDropGuard<FsBlob<B>>>, Arc<anyhow::Error>>>
            + Send
            + 'static,
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
    {
        let loading_fn =
            move |i| async move { loading_fn(i).await.map(|v| v.map(AsyncDropTokioMutex::new)) };
        Ok(self
            .loaded_blobs
            .get_loaded_or_insert_loading(blob_id, blobstore, loading_fn)
            .await
            .wait_until_loaded()
            .await?
            .map(LoadedBlobGuard::new))
    }

    pub async fn get_if_loading_or_loaded(
        &self,
        blob_id: BlobId,
    ) -> Result<Option<AsyncDropGuard<LoadedBlobGuard<B>>>, Arc<anyhow::Error>> {
        Ok(self
            .loaded_blobs
            .get_if_loading_or_loaded(blob_id)
            .wait_until_loaded()
            .await?
            .map(LoadedBlobGuard::new))
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
                    let remove_result = FsBlob::remove(blob).await;
                    blobstore.async_drop().await?;
                    remove_result?;
                    Ok(RemoveResult::SuccessfullyRemoved)
                } else {
                    // The blob wasn't loaded, we can just remove it from the base store
                    // We're doing this within the drop handler of `request_immediate_drop()`, because that gives us
                    // exclusive access and blocks other tasks from loading this blob.
                    let result = blobstore.remove_by_id(&blob_id).await;
                    blobstore.async_drop().await?;
                    result
                }
            });
        match request {
            RequestImmediateDropResult::ImmediateDropRequested { drop_result } => {
                RequestRemovalResult::RemovalRequested {
                    on_removed: drop_result,
                }
            }
            RequestImmediateDropResult::AlreadyDropping { future } => {
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
        /// on_removed will be completed once the entry has been fully dropped
        /// The caller is expected to drive this future to completion,
        /// otherwise we may be stuck forever waiting for the drop to complete.
        on_removed: BoxFuture<'static, Result<RemoveResult>>,
    },
    /// Removal failed because the entry is already in dropping state.
    AlreadyDropping {
        // TODO Is Event good enough here or do we benefit from the caller driving this?
        future: Shared<BoxFuture<'static, ()>>,
    },
}
