use std::fmt::Debug;

use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_rustfs::{FsError, FsResult};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

use crate::filesystem::{
    concurrentfsblobstore::loaded_blobs::LoadedBlobGuard, fsblobstore::BlobType,
};

#[derive(Debug)]
pub struct ConcurrentFsBlob<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    blob: AsyncDropGuard<LoadedBlobGuard<'a, B>>,
}

impl<'a, B> ConcurrentFsBlob<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    pub fn new(blob: AsyncDropGuard<LoadedBlobGuard<'a, B>>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self { blob })
    }

    pub fn blob_id(&self) -> BlobId {
        self.blob.blob_id()
    }

    pub async fn blob_type(&self) -> BlobType {
        // TODO Should we cache this instead of locking every time? Probably need to cache when calling Mutex::new() and pass through.
        self.blob.with_lock(async |b| b.blob_type()).await
    }

    // TODO A lot of call sites call with_lock on blobs they just created. Can we optimize this? Sounds silly to create it, add it to loaded blobs to allow other threads to access it, and then immediately lock it for initialization. Sounds even like there could be race conditions.
    pub async fn with_lock<R, F>(&self, f: F) -> R
    where
        F: AsyncFnOnce(&mut crate::filesystem::fsblobstore::FsBlob<B>) -> R,
    {
        self.blob.with_lock(f).await
    }

    pub async fn remove(this: AsyncDropGuard<Self>) -> FsResult<()> {
        LoadedBlobGuard::remove(this.unsafe_into_inner_dont_drop().blob).await
    }
}

#[async_trait::async_trait]
impl<'a, B> AsyncDrop for ConcurrentFsBlob<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.blob.async_drop().await
    }
}
