use anyhow::Result;
use async_trait::async_trait;
use byte_unit::Byte;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use super::BlobStoreActionCounts;
use super::tracking_blob::TrackingBlob;
use crate::{BlobId, BlobStore, RemoveResult};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

#[derive(Debug)]
pub struct TrackingBlobStore<B>
where
    B: BlobStore + AsyncDrop + Debug + Send + Sync + 'static,
{
    underlying_store: AsyncDropGuard<B>,

    counts: Arc<Mutex<BlobStoreActionCounts>>,
}

impl<B> TrackingBlobStore<B>
where
    B: BlobStore + AsyncDrop + Debug + Send + Sync + 'static,
{
    pub fn new(underlying_store: AsyncDropGuard<B>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            underlying_store,
            counts: Arc::new(Mutex::new(BlobStoreActionCounts::ZERO)),
        })
    }

    pub fn counts(&self) -> BlobStoreActionCounts {
        *self.counts.lock().unwrap()
    }

    pub fn get_and_reset_counts(&self) -> BlobStoreActionCounts {
        std::mem::replace(
            &mut self.counts.lock().unwrap(),
            BlobStoreActionCounts::ZERO,
        )
    }
}

#[async_trait]
impl<B> BlobStore for TrackingBlobStore<B>
where
    B: BlobStore + AsyncDrop + Debug + Send + Sync + 'static,
{
    type ConcreteBlob<'a>
        = TrackingBlob<'a, B>
    where
        Self: 'a;

    async fn create(&self) -> Result<Self::ConcreteBlob<'_>> {
        self.counts.lock().unwrap().store_create += 1;
        let blob = self.underlying_store.create().await?;
        Ok(TrackingBlob::new(blob, &self.counts))
    }

    async fn try_create(&self, id: &BlobId) -> Result<Option<Self::ConcreteBlob<'_>>> {
        self.counts.lock().unwrap().store_try_create += 1;
        let maybe_blob = self.underlying_store.try_create(id).await?;
        Ok(maybe_blob.map(|b| TrackingBlob::new(b, &self.counts)))
    }

    async fn load(&self, id: &BlobId) -> Result<Option<Self::ConcreteBlob<'_>>> {
        self.counts.lock().unwrap().store_load += 1;
        let maybe_blob = self.underlying_store.load(id).await?;
        Ok(maybe_blob.map(|b| TrackingBlob::new(b, &self.counts)))
    }

    async fn remove_by_id(&self, id: &BlobId) -> Result<RemoveResult> {
        self.counts.lock().unwrap().store_remove_by_id += 1;
        self.underlying_store.remove_by_id(id).await
    }

    async fn num_nodes(&self) -> Result<u64> {
        self.counts.lock().unwrap().store_num_nodes += 1;
        self.underlying_store.num_nodes().await
    }

    fn estimate_space_for_num_blocks_left(&self) -> Result<u64> {
        self.counts
            .lock()
            .unwrap()
            .store_estimate_space_for_num_blocks_left += 1;
        self.underlying_store.estimate_space_for_num_blocks_left()
    }

    fn logical_block_size_bytes(&self) -> Byte {
        self.counts.lock().unwrap().store_logical_block_size_bytes += 1;
        self.underlying_store.logical_block_size_bytes()
    }

    #[cfg(any(test, feature = "testutils"))]
    async fn clear_cache_slow(&self) -> Result<()> {
        self.underlying_store.clear_cache_slow().await
    }

    #[cfg(test)]
    async fn all_blobs(&self) -> Result<Vec<BlobId>> {
        self.underlying_store.all_blobs().await
    }
}

#[async_trait]
impl<B> AsyncDrop for TrackingBlobStore<B>
where
    B: BlobStore + AsyncDrop + Debug + Send + Sync + 'static,
{
    type Error = B::Error;

    async fn async_drop_impl(&mut self) -> Result<(), B::Error> {
        self.underlying_store.async_drop().await
    }
}
