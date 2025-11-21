use std::fmt::Debug;

use cryfs_blobstore::{BlobId, BlobStore, RemoveResult};
use cryfs_rustfs::{FsError, FsResult};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

use crate::filesystem::{
    concurrentfsblobstore::loaded_blobs::LoadedBlobGuard, fsblobstore::BlobType,
};

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
        F: AsyncFnOnce(&mut crate::filesystem::fsblobstore::FsBlob<B>) -> R,
    {
        self.blob.with_lock(f).await
    }

    /// Locks two blobs in a deadlock-free manner and returns guards for both.
    /// This is deadlock-free because it enforces an order of locking based on the blob id.
    pub async fn with_locks_2<R, F>(blob1: &Self, blob2: &Self, f: F) -> R
    where
        F: AsyncFnOnce(
            &mut crate::filesystem::fsblobstore::FsBlob<B>,
            &mut crate::filesystem::fsblobstore::FsBlob<B>,
        ) -> R,
    {
        assert!(
            blob1.blob_id() != blob2.blob_id(),
            "with_locks_2 must be called with two different blobs"
        );
        if blob1.blob_id() < blob2.blob_id() {
            blob1
                .with_lock(async |blob1| blob2.with_lock(async |blob2| f(blob1, blob2).await).await)
                .await
        } else {
            blob2
                .with_lock(async |blob2| blob1.with_lock(async |blob1| f(blob1, blob2).await).await)
                .await
        }
    }

    pub async fn remove(this: AsyncDropGuard<Self>) -> FsResult<RemoveResult> {
        LoadedBlobGuard::remove(this.unsafe_into_inner_dont_drop().blob).await
    }
}

#[async_trait::async_trait]
impl<B> AsyncDrop for ConcurrentFsBlob<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.blob.async_drop().await
    }
}
