use std::fmt::Debug;
use std::num::NonZeroUsize;

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

/// A simple LRU cache for blobs.
pub struct BlobCache<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    cache: LruCache<BlobId, CacheEntry<B>>,
}

impl<B> BlobCache<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    pub fn new(max_entries: NonZeroUsize) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            cache: LruCache::new(max_entries),
        })
    }

    /// Try to get a blob from the cache. If found, removes it from the cache and returns it.
    /// Returns None if not in cache.
    pub fn try_get(&mut self, blob_id: &BlobId) -> Option<AsyncDropGuard<ConcurrentFsBlob<B>>> {
        self.cache.pop(blob_id).map(|entry| entry.into_blob())
    }

    /// Put a blob into the cache.
    /// If the cache is full, the least recently used entry will be evicted and returned.
    /// If a blob with the same ID already exists, it will be replaced and the previous one returned.
    /// Caller is responsible for dropping the returned blob.
    pub fn put(
        &mut self,
        blob: AsyncDropGuard<ConcurrentFsBlob<B>>,
    ) -> Option<AsyncDropGuard<ConcurrentFsBlob<B>>> {
        self.cache
            .push(blob.blob_id(), CacheEntry::new(blob))
            .map(|(_, entry)| entry.into_blob())
    }

    /// Evict entries that are older than the given max_age.
    /// Returns the evicted blobs. Caller is responsible for dropping them.
    pub fn evict_old_entries(
        &mut self,
        max_age: Duration,
    ) -> Vec<AsyncDropGuard<ConcurrentFsBlob<B>>> {
        self._evict_lru_while(|entry| entry.age() > max_age)
    }

    /// Drain all entries from the cache for cleanup.
    /// Returns all blobs. Caller is responsible for dropping them.
    pub fn evict_all(&mut self) -> Vec<AsyncDropGuard<ConcurrentFsBlob<B>>> {
        self._evict_lru_while(|_| true)
    }

    fn _evict_lru_while(
        &mut self,
        mut cond: impl FnMut(&CacheEntry<B>) -> bool,
    ) -> Vec<AsyncDropGuard<ConcurrentFsBlob<B>>> {
        let mut evicted = Vec::new();
        while self
            .cache
            .peek_lru()
            .map_or(false, |(_, entry)| cond(entry))
        {
            let entry = self.cache.pop_lru().expect("entry should be present").1;
            evicted.push(entry.into_blob());
        }
        evicted
    }
}

impl<B> Debug for BlobCache<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlobCache")
            .field("len", &self.cache.len())
            .finish()
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
        let evicted = self.evict_all();
        for_each_unordered(evicted.into_iter(), |mut blob| async move {
            blob.async_drop().await
        })
        .await
    }
}
