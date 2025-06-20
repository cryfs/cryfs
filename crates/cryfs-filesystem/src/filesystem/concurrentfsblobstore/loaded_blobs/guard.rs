use async_trait::async_trait;
use cryfs_rustfs::{FsError, FsResult};
use std::fmt::Debug;

use super::LoadedBlobs;
use crate::filesystem::{
    concurrentfsblobstore::loaded_blobs::RequestRemovalResult, fsblobstore::FsBlob,
};
use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard, AsyncDropTokioMutex};

#[derive(Debug)]
pub struct LoadedBlobGuard<'s, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    loaded_blobs: &'s LoadedBlobs<B>,
    blob_id: BlobId,
    blob: AsyncDropGuard<AsyncDropArc<AsyncDropTokioMutex<FsBlob<B>>>>,
}

impl<'s, B> LoadedBlobGuard<'s, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    pub(super) fn new(
        loaded_blobs: &'s LoadedBlobs<B>,
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
                on_removed.wait().await;
                Ok(())
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
impl<'s, B> AsyncDrop for LoadedBlobGuard<'s, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        let blob = std::mem::replace(&mut self.blob, AsyncDropGuard::new_invalid());
        self.loaded_blobs.unload(self.blob_id, blob).await?;
        Ok(())
    }
}
