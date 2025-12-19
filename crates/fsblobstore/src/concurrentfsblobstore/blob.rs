use std::{fmt::Debug, sync::Arc};

use cryfs_blobstore::{BlobId, BlobStore, RemoveResult};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

use crate::{concurrentfsblobstore::loaded_blobs::LoadedBlobGuard, fsblobstore::BlobType};

#[derive(Debug)]
pub struct ConcurrentFsBlob<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    blob: AsyncDropGuard<LoadedBlobGuard<B>>,
}

impl<B> ConcurrentFsBlob<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    pub fn new(blob: AsyncDropGuard<LoadedBlobGuard<B>>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self { blob })
    }

    pub fn blob_id(&self) -> BlobId {
        self.blob.blob_id()
    }

    pub async fn blob_type(&self) -> BlobType {
        // It's ok to lock here instead of caching it, because this is called very rarely
        self.blob.with_lock(async |b| b.blob_type()).await
    }

    pub async fn with_lock<R, F>(&self, f: F) -> R
    where
        F: AsyncFnOnce(&mut crate::fsblobstore::FsBlob<B>) -> R,
    {
        self.blob.with_lock(f).await
    }

    /// Check if a removal is in progress for this blob.
    pub fn is_removal_requested(&self) -> bool {
        self.blob.is_removal_requested()
    }

    /// Request removal of this blob.
    /// Immediately marks the blob as being removed so no new users can acquire it.
    /// Returns a future that completes when the removal is done.
    pub async fn request_removal(
        this: AsyncDropGuard<Self>,
    ) -> Result<impl Future<Output = Result<RemoveResult, Arc<anyhow::Error>>>, anyhow::Error> {
        LoadedBlobGuard::request_removal(this.unsafe_into_inner_dont_drop().blob).await
    }
}

#[async_trait::async_trait]
impl<B> AsyncDrop for ConcurrentFsBlob<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.blob.async_drop().await
    }
}
