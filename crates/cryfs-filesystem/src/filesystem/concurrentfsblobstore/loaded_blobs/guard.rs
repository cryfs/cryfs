use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use cryfs_blobstore::{BlobId, BlobStore, RemoveResult};
use cryfs_rustfs::{FsError, FsResult};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard, AsyncDropTokioMutex},
    concurrent_store::{LoadedEntryGuard, RequestImmediateDropResult},
    mr_oneshot_channel::RecvError,
};

use crate::filesystem::fsblobstore::FsBlob;

#[derive(Debug)]
pub struct LoadedBlobGuard<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    loaded_blob:
        AsyncDropGuard<LoadedEntryGuard<BlobId, AsyncDropTokioMutex<FsBlob<B>>, RemoveResult>>,
}

impl<B> LoadedBlobGuard<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    pub(super) fn new(
        loaded_blob: AsyncDropGuard<
            LoadedEntryGuard<BlobId, AsyncDropTokioMutex<FsBlob<B>>, RemoveResult>,
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

    pub async fn remove(mut this: AsyncDropGuard<Self>) -> FsResult<RemoveResult> {
        match this.loaded_blob.store().request_immediate_drop(
            *this.loaded_blob.key(),
            |blob| async move {
                let Some(blob) = blob else {
                    panic!("The blob wasn't loaded. This can't happen because we hold the LoadedBlobGuard");
                };
                let blob = AsyncDropTokioMutex::into_inner(blob);
                FsBlob::remove(blob).await.map_err(Arc::new)?;
                Ok(RemoveResult::SuccessfullyRemoved)
            },
        ) {
            RequestImmediateDropResult::ImmediateDropRequested { on_dropped }
            // An earlier immediate drop result can only be a remove request, because that's the only scenario in which we request immediate drops.
            // So we're fine and that earlier request will remove it for us.
            | RequestImmediateDropResult::AlreadyDroppingFromEarlierImmediateDrop { on_dropped } => {
                // Drop the blob so we don't hold a lock on it, which would prevent the removal. Removal waits until all readers relinquished their blob.
                this.async_drop().await?;
                std::mem::drop(this);
                // Wait until the blob is removed. If there are other readers, this will wait.
                on_dropped
                    .recv()
                    .await
                    .map_err(|error: RecvError| FsError::InternalError {
                        error: error.into(),
                    })?
                    .map_err(|err| FsError::InternalError {
                        error: anyhow::anyhow!("Error during blob removal: {err}"),
                    })
            }
            RequestImmediateDropResult::AlreadyDroppingWithoutImmediateDrop { .. } => {
                panic!("This can't happen because we hold the LoadedBlobGuard");
            }
        }
    }
}

#[async_trait]
impl<B> AsyncDrop for LoadedBlobGuard<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.loaded_blob.async_drop().await.map_err(|e| {
            log::error!("Error dropping LoadedBlobGuard: {:?}", e);
            FsError::UnknownError
        })?;
        Ok(())
    }
}
