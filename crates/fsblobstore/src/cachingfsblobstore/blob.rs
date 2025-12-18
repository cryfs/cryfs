use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use cryfs_blobstore::{BlobId, BlobStore, RemoveResult};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};

use super::cache::BlobCache;
use crate::concurrentfsblobstore::ConcurrentFsBlob;
use crate::fsblobstore::BlobType;

/// A wrapper around `ConcurrentFsBlob` that puts the blob back into the cache
/// when dropped, instead of immediately releasing it.
pub struct CachingFsBlob<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    /// The underlying blob. This is an Option so we can take it during async_drop.
    blob: Option<AsyncDropGuard<ConcurrentFsBlob<B>>>,
    /// Reference to the cache for putting the blob back when dropped.
    cache: AsyncDropGuard<AsyncDropArc<BlobCache<B>>>,
}

impl<B> CachingFsBlob<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    pub(super) fn new(
        blob: AsyncDropGuard<ConcurrentFsBlob<B>>,
        cache: AsyncDropGuard<AsyncDropArc<BlobCache<B>>>,
    ) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            blob: Some(blob),
            cache,
        })
    }

    /// Get the blob ID.
    pub fn blob_id(&self) -> BlobId {
        self.blob
            .as_ref()
            .expect("blob should be present")
            .blob_id()
    }

    /// Get the blob type.
    pub async fn blob_type(&self) -> BlobType {
        self.blob
            .as_ref()
            .expect("blob should be present")
            .blob_type()
            .await
    }

    /// Execute a function with exclusive access to the underlying blob.
    pub async fn with_lock<R, F>(&self, f: F) -> R
    where
        F: AsyncFnOnce(&mut crate::fsblobstore::FsBlob<B>) -> R,
    {
        self.blob
            .as_ref()
            .expect("blob should be present")
            .with_lock(f)
            .await
    }

    /// Remove this blob from the store.
    ///
    /// This bypasses the cache - the blob will be removed from the underlying store.
    pub async fn remove(this: AsyncDropGuard<Self>) -> Result<RemoveResult, Arc<anyhow::Error>> {
        let inner = this.unsafe_into_inner_dont_drop();
        let blob = inner.blob.expect("blob should be present");

        // It's possible that we have another instance in the cache, when another task loaded it.
        // Let's remove that instance too.
        // TODO This won't capture the case where a task is currently using it and will then return it to the cache
        //      after our remove operation. How should we handle that? Right now, it just goes back into the cache,
        //      unnecessarily delaying the actual removal.
        let from_cache = inner.cache.remove(&blob.blob_id());
        if let Some(mut cached_blob) = from_cache {
            cached_blob.async_drop().await?;
        }

        // Don't put back in cache - we're removing it
        // ConcurrentFsBlobStore will wait for all references to be dropped before removing
        ConcurrentFsBlob::remove(blob).await
    }
}

impl<B> Debug for CachingFsBlob<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CachingFsBlob")
            .field("blob_id", &self.blob.as_ref().map(|b| b.blob_id()))
            .finish()
    }
}

#[async_trait]
impl<B> AsyncDrop for CachingFsBlob<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        let blob = self.blob.take().expect("blob should be present");

        // Put the blob back in the cache
        self.cache.put(blob).await;

        self.cache.async_drop().await?;

        Ok(())
    }
}
