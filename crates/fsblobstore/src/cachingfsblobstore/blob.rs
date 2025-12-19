use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use cryfs_blobstore::{BlobId, BlobStore, RemoveResult};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard, AsyncDropTokioMutex};

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
    cache: AsyncDropGuard<AsyncDropArc<AsyncDropTokioMutex<BlobCache<B>>>>,
}

impl<B> CachingFsBlob<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    pub(super) fn new(
        blob: AsyncDropGuard<ConcurrentFsBlob<B>>,
        cache: AsyncDropGuard<AsyncDropArc<AsyncDropTokioMutex<BlobCache<B>>>>,
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
        let mut inner = this.unsafe_into_inner_dont_drop();
        let blob = inner.blob.expect("blob should be present");
        let blob_id = blob.blob_id();
        let cache = &inner.cache;

        // Hold the lock until the removal flag is set and we removed from the cache.
        // This ensures that any concurrent task calling async_drop or remove for the same blob will either:
        // - Complete before we take the lock (we'll remove the cached blob)
        // - Wait until we release the lock (will see is_removal_requested() == true)
        let (remove_future, to_drop) = {
            let mut cache = cache.lock().await;
            // Now remove - this sets the removal flag while we hold the lock
            let remove_future = ConcurrentFsBlob::request_removal(blob).await;

            // If there are any cached blob, we need to drop it as well
            let from_cache = cache.try_get(&blob_id);
            (remove_future, from_cache)

            // Lock released here, now both the removal flag is set and we removed from the cache
        };

        // First drop it from the cache so we don't deadlock (removal waits for all users to release the blob)
        if let Some(mut blob_to_drop) = to_drop {
            blob_to_drop.async_drop().await?;
        }

        inner.cache.async_drop().await?;

        // Then wait for the removal to complete
        let remove_result = remove_future?.await?;
        Ok(remove_result)
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

        // Hold the lock while checking the removal flag AND putting in cache.
        // This ensures that if a removal is in progress, we'll see is_removal_requested() == true.
        // Other tasks are blocked from requesting removals or dropping the blob until we're done.
        let to_drop = {
            let mut cache = self.cache.lock().await;
            if blob.is_removal_requested() {
                // Don't cache, return blob to be dropped
                Some(blob)
            } else {
                // Put in cache, return evicted blob if any
                cache.put(blob)
            }
            // Lock released here
        };

        if let Some(mut blob_to_drop) = to_drop {
            blob_to_drop.async_drop().await?;
        }

        self.cache.async_drop().await?;

        Ok(())
    }
}
