use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use cryfs_blobstore::{BlobId, BlobStore, RemoveResult};
use cryfs_concurrent_store::{LoadedEntryGuard, RequestImmediateDropResult};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard, AsyncDropTokioMutex},
    with_async_drop_2,
};
use lockable::InfallibleUnwrap as _;

use crate::fsblobstore::FsBlob;

#[derive(Debug)]
pub struct LoadedBlobGuard<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    loaded_blob: AsyncDropGuard<
        LoadedEntryGuard<BlobId, AsyncDropTokioMutex<FsBlob<B>>, Arc<anyhow::Error>>,
    >,
}

impl<B> LoadedBlobGuard<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    pub(super) fn new(
        loaded_blob: AsyncDropGuard<
            LoadedEntryGuard<BlobId, AsyncDropTokioMutex<FsBlob<B>>, Arc<anyhow::Error>>,
        >,
    ) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self { loaded_blob })
    }

    pub fn blob_id(&self) -> BlobId {
        *self.loaded_blob.key()
    }

    pub async fn with_lock<R, F>(&self, f: F) -> R
    where
        F: AsyncFnOnce(&mut FsBlob<B>) -> R,
    {
        let mut guard = self.loaded_blob.value().lock().await;
        f(&mut *guard).await
    }

    /// Check if a removal is in progress for this blob.
    pub fn is_removal_requested(&self) -> bool {
        // The only scenario where we request an immediate drop is when a removal is in progress
        self.loaded_blob.is_immediate_drop_requested()
    }

    /// Request removal of the blob.
    /// Immediately marks the blob as being removed so no new users can acquire it.
    /// Returns a future that completes when the removal is done.
    pub async fn request_removal(
        this: AsyncDropGuard<Self>,
    ) -> Result<impl Future<Output = Result<RemoveResult, Arc<anyhow::Error>>>, anyhow::Error> {
        with_async_drop_2!(this, {
            loop {
                match this.loaded_blob.request_immediate_drop(
                |blob| async move {
                    let Some(blob) = blob else {
                        panic!("The blob wasn't loaded. This can't happen because we hold the LoadedBlobGuard");
                    };
                    let blob = AsyncDropTokioMutex::into_inner(blob);
                    FsBlob::remove(blob).await?;
                    Ok(RemoveResult::SuccessfullyRemoved)
                },
            ) {
                RequestImmediateDropResult::ImmediateDropRequested { drop_result } => {
                    return Ok(drop_result);
                }
                RequestImmediateDropResult::AlreadyDropping { future } => {
                    // Blob is currently dropping, let's wait until that is done and then retry
                    future.await;
                    continue;
                }
            }
            }
        })
    }
}

#[async_trait]
impl<B> AsyncDrop for LoadedBlobGuard<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.loaded_blob.async_drop().await.infallible_unwrap();
        Ok(())
    }
}
