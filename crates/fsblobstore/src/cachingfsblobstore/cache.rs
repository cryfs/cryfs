use std::fmt::Debug;
use std::num::NonZeroUsize;
use std::sync::Mutex;

use anyhow::Result;
use async_trait::async_trait;
use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    stream::for_each_unordered,
};
use lru::LruCache;
use tokio::time::{Duration, Instant};

use crate::concurrentfsblobstore::ConcurrentFsBlob;

/// A cache entry holding a blob guard and the time it was inserted.
struct CacheEntry<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    blob: AsyncDropGuard<ConcurrentFsBlob<B>>,
    inserted_at: Instant,
}

impl<B> CacheEntry<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    fn new(blob: AsyncDropGuard<ConcurrentFsBlob<B>>) -> Self {
        Self {
            blob,
            inserted_at: Instant::now(),
        }
    }

    fn into_blob(self) -> AsyncDropGuard<ConcurrentFsBlob<B>> {
        self.blob
    }

    fn age(&self) -> Duration {
        self.inserted_at.elapsed()
    }
}

/// A simple LRU cache for blobs, protected by a mutex.
pub struct BlobCache<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    cache: Mutex<LruCache<BlobId, CacheEntry<B>>>,
}

impl<B> BlobCache<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    pub fn new(max_entries: NonZeroUsize) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            cache: Mutex::new(LruCache::new(max_entries)),
        })
    }

    /// Try to get a blob from the cache. If found, removes it from the cache and returns it.
    /// Returns None if not in cache.
    pub fn try_get(&self, blob_id: &BlobId) -> Option<AsyncDropGuard<ConcurrentFsBlob<B>>> {
        let mut cache = self.cache.lock().unwrap();
        cache.pop(blob_id).map(|entry| entry.into_blob())
    }

    /// Put a blob into the cache.
    /// If the cache is full, the least recently used entry will be evicted.
    pub async fn put(&self, blob: AsyncDropGuard<ConcurrentFsBlob<B>>) {
        let evicted = {
            let mut cache = self.cache.lock().unwrap();

            // Push returns the evicted entry if cache was full
            cache
                .push(blob.blob_id(), CacheEntry::new(blob))
                .map(|(_, entry)| entry.into_blob())

            // Free the lock on self.cache before awaiting async drop
        };

        if let Some(evicted_blob) = evicted {
            let mut evicted_blob = evicted_blob;
            evicted_blob.async_drop().await.expect("async drop failed");
        }
    }

    /// Remove a specific blob from the cache if present.
    /// Returns the blob if it was in the cache.
    pub fn remove(&self, blob_id: &BlobId) -> Option<AsyncDropGuard<ConcurrentFsBlob<B>>> {
        self.try_get(blob_id)
    }

    /// Evict entries that are older than the given max_age.
    pub async fn evict_old_entries(&self, max_age: Duration) -> Result<()> {
        self._evict_lru_while(|entry| entry.age() > max_age).await
    }

    /// Drain all entries from the cache for cleanup.
    pub async fn evict_all(&self) -> Result<()> {
        self._evict_lru_while(|_| true).await
    }

    async fn _evict_lru_while(&self, mut cond: impl FnMut(&CacheEntry<B>) -> bool) -> Result<()> {
        let evicted = {
            let mut cache = self.cache.lock().unwrap();
            let mut evicted = Vec::new();

            while cache.peek_lru().map_or(false, |(_, entry)| cond(entry)) {
                let entry = cache.pop_lru().expect("entry should be present").1;
                evicted.push(entry.into_blob());
            }

            evicted

            // Free the lock on self.cache before awaiting async drop
        };

        for_each_unordered(evicted.into_iter(), |mut blob| async move {
            blob.async_drop().await
        })
        .await
    }
}

impl<B> Debug for BlobCache<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let len = self.cache.lock().unwrap().len();
        f.debug_struct("BlobCache").field("len", &len).finish()
    }
}

#[async_trait]
impl<B> AsyncDrop for BlobCache<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.evict_all().await
    }
}
