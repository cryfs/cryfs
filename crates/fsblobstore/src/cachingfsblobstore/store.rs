use anyhow::Result;
use async_trait::async_trait;
use byte_unit::Byte;
use cryfs_utils::with_async_drop_2;
use std::fmt::Debug;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::time::Duration;

use cryfs_blobstore::{BlobId, BlobStore, RemoveResult};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard, AsyncDropWeak};
use cryfs_utils::periodic_task::PeriodicTask;

use super::blob::CachingFsBlob;
use super::cache::BlobCache;
use crate::concurrentfsblobstore::{ConcurrentFsBlob, ConcurrentFsBlobStore};
use crate::fsblobstore::FlushBehavior;

const MAX_CACHE_ENTRIES: NonZeroUsize = NonZeroUsize::new(1000).unwrap();
const EVICTION_INTERVAL: Duration = Duration::from_secs(1);
const MAX_ENTRY_AGE: Duration = Duration::from_secs(1);

/// A caching layer that sits above `ConcurrentFsBlobStore`.
///
/// This layer keeps recently-used blobs in a cache so that subsequent accesses
/// don't need to reload them from the underlying store. When a blob is released
/// by a consumer, it goes into the cache instead of being immediately dropped.
/// Periodically, old entries are evicted from the cache.
///
/// Blobs, when loaded or created, are removed from the cache and returned to the caller.
/// When the caller drops the blob, it is put back into the cache.
/// This means that if a blob is loaded while another task already loaded it, we don't have
/// the blob in the cache and request it again from the underlying store.
/// But that's ok because the underlying store is a ConcurrentFsBlobStore which will just
/// return a reference to the already loaded blob. When the first task drops the blob, it goes
/// into the cache, and when the second task drops its instance, it will replace the instance
/// in the cache.
#[derive(Debug)]
pub struct CachingFsBlobStore<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    underlying: AsyncDropGuard<ConcurrentFsBlobStore<B>>,
    cache: AsyncDropGuard<AsyncDropArc<BlobCache<B>>>,
    eviction_task: Option<AsyncDropGuard<PeriodicTask>>,
}

impl<B> CachingFsBlobStore<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    /// Create a new caching blob store
    pub fn new(underlying: AsyncDropGuard<ConcurrentFsBlobStore<B>>) -> AsyncDropGuard<Self> {
        let cache = AsyncDropArc::new(BlobCache::new(MAX_CACHE_ENTRIES));

        // Spawn the eviction task
        let cache_for_task = AsyncDropArc::downgrade(&cache);
        let eviction_task = PeriodicTask::spawn(
            "CachingFsBlobStore eviction",
            EVICTION_INTERVAL,
            move || {
                let cache = AsyncDropWeak::clone(&cache_for_task);
                async move {
                    let cache = cache.upgrade().expect("This can't happen because CachingFsBlobStore drops the PeriodicTask before it drops its strng reference to the cache");
                    with_async_drop_2!(cache, { cache.evict_old_entries(MAX_ENTRY_AGE).await })
                }
            },
        );

        AsyncDropGuard::new(Self {
            underlying,
            cache,
            eviction_task: Some(eviction_task),
        })
    }

    /// Create the root directory blob.
    pub async fn create_root_dir_blob(
        &self,
        root_blob_id: &BlobId,
    ) -> Result<(), Arc<anyhow::Error>> {
        self.underlying.create_root_dir_blob(root_blob_id).await
    }

    /// Create a new file blob.
    pub async fn create_file_blob(
        &self,
        parent: &BlobId,
        flush_behavior: FlushBehavior,
    ) -> Result<AsyncDropGuard<CachingFsBlob<B>>> {
        let blob = self
            .underlying
            .create_file_blob(parent, flush_behavior)
            .await?;
        Ok(CachingFsBlob::new(blob, AsyncDropArc::clone(&self.cache)))
    }

    /// Create a new directory blob.
    pub async fn create_dir_blob(
        &self,
        parent: &BlobId,
        flush_behavior: FlushBehavior,
    ) -> Result<AsyncDropGuard<CachingFsBlob<B>>> {
        let blob = self
            .underlying
            .create_dir_blob(parent, flush_behavior)
            .await?;
        Ok(CachingFsBlob::new(blob, AsyncDropArc::clone(&self.cache)))
    }

    /// Create a new symlink blob.
    pub async fn create_symlink_blob(
        &self,
        parent: &BlobId,
        target: &str,
        flush_behavior: FlushBehavior,
    ) -> Result<AsyncDropGuard<CachingFsBlob<B>>> {
        let blob = self
            .underlying
            .create_symlink_blob(parent, target, flush_behavior)
            .await?;
        Ok(CachingFsBlob::new(blob, AsyncDropArc::clone(&self.cache)))
    }

    /// Load a blob by ID.
    ///
    /// First checks the cache. If found, returns the cached blob.
    /// Otherwise loads from the underlying store.
    pub async fn load(
        &self,
        blob_id: &BlobId,
    ) -> Result<Option<AsyncDropGuard<CachingFsBlob<B>>>, Arc<anyhow::Error>> {
        // First check the cache
        if let Some(blob) = self.cache.try_get(blob_id) {
            return Ok(Some(CachingFsBlob::new(
                blob,
                AsyncDropArc::clone(&self.cache),
            )));
        }

        // Not in cache, load from underlying store
        let blob = self.underlying.load(blob_id).await?;
        Ok(blob.map(|b| CachingFsBlob::new(b, AsyncDropArc::clone(&self.cache))))
    }

    /// Get the number of blocks in the underlying store.
    pub async fn num_blocks(&self) -> Result<u64> {
        self.underlying.num_blocks().await
    }

    /// Estimate space for remaining blocks.
    pub fn estimate_space_for_num_blocks_left(&self) -> Result<u64> {
        self.underlying.estimate_space_for_num_blocks_left()
    }

    /// Get the logical block size.
    pub fn logical_block_size_bytes(&self) -> Byte {
        self.underlying.logical_block_size_bytes()
    }

    /// Remove a blob by ID.
    ///
    /// This removes the blob from both the cache and the underlying store.
    pub async fn remove_by_id(&self, id: &BlobId) -> Result<RemoveResult, Arc<anyhow::Error>> {
        if let Some(blob) = self.cache.remove(id) {
            // Note: If another task is currently using the blob, it will check is_removal_requested()
            // in CachingFsBlob::async_drop_impl and won't put it back in the cache but immediately drop it.
            ConcurrentFsBlob::remove(blob).await
        } else {
            self.underlying.remove_by_id(id).await.map_err(Arc::new)
        }
    }

    /// Flush a blob if it's loaded or cached.
    pub async fn flush_if_cached(&self, blob_id: BlobId) -> Result<(), Arc<anyhow::Error>> {
        // The underlying ConcurrentFsBlobStore will handle the flush
        self.underlying.flush_if_cached(blob_id).await
    }

    /// Clear all entries from the cache.
    /// This is primarily useful for testing.
    #[cfg(feature = "testutils")]
    pub async fn clear_cache(&self) -> Result<()> {
        self.cache.evict_all().await
    }
}

#[async_trait]
impl<B> AsyncDrop for CachingFsBlobStore<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        // First stop the eviction task
        self.eviction_task
            .take()
            .expect("already dropped")
            .async_drop()
            .await?;

        // Then drain and drop all cached blobs
        self.cache.async_drop().await?;

        // Finally drop the underlying store
        self.underlying.async_drop().await?;

        Ok(())
    }
}
