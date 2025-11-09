use async_trait::async_trait;
use cryfs_rustfs::{FsError, FsResult};
use std::fmt::Debug;

use super::LoadedBlobs;
use crate::filesystem::{
    concurrentfsblobstore::loaded_blobs::RequestRemovalResult, fsblobstore::FsBlob,
};
use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard, AsyncDropTokioMutex},
    mr_oneshot_channel::RecvError,
};

#[derive(Debug)]
pub struct LoadedBlobGuard<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    loaded_blobs: AsyncDropGuard<AsyncDropArc<LoadedBlobs<B>>>,
    blob_id: BlobId,
    blob: AsyncDropGuard<AsyncDropArc<AsyncDropTokioMutex<FsBlob<B>>>>,
}

impl<B> LoadedBlobGuard<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    pub(super) fn new(
        loaded_blobs: AsyncDropGuard<AsyncDropArc<LoadedBlobs<B>>>,
        blob_id: BlobId,
        blob: AsyncDropGuard<AsyncDropArc<AsyncDropTokioMutex<FsBlob<B>>>>,
    ) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            loaded_blobs,
            blob_id,
            blob,
        })
    }

    pub fn blob_id(&self) -> BlobId {
        self.blob_id
    }

    pub async fn with_lock<R, F>(&self, f: F) -> R
    where
        F: AsyncFnOnce(&mut FsBlob<B>) -> R,
    {
        let mut guard = self.blob.lock().await;
        f(&mut *guard).await
    }

    pub async fn remove(mut this: AsyncDropGuard<Self>) -> FsResult<()> {
        match this.loaded_blobs.request_removal(this.blob_id) {
            RequestRemovalResult::RemovalRequested { on_removed } => {
                // Drop the blob so we don't hold a lock on it, which would prevent the removal. Removal waits until all readers relinquished their blob.
                this.async_drop().await?;
                std::mem::drop(this);
                // Wait until the blob is removed. If there are other readers, this will wait.
                on_removed
                    .recv()
                    .await
                    .map_err(|error: RecvError| FsError::InternalError {
                        error: error.into(),
                    })?
                    .map_err(|err| FsError::InternalError {
                        error: anyhow::anyhow!("Error during blob removal: {err}"),
                    })
            }
            RequestRemovalResult::NotLoaded => {
                panic!("This can't happen because we hold the LoadedBlobGuard");
            }
            RequestRemovalResult::Dropping { .. } => {
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
        let blob = std::mem::replace(&mut self.blob, AsyncDropGuard::new_invalid());
        self.loaded_blobs.unload(self.blob_id, blob).await?;
        self.loaded_blobs.async_drop().await.unwrap(); // TODO No unwrap
        Ok(())
    }
}
