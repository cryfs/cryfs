use anyhow::Result;
use async_trait::async_trait;
use futures::stream::BoxStream;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use super::BlobStoreActionCounts;
use crate::BlobStore;
use crate::{Blob, BlobId};
use cryfs_blockstore::BlockId;
use cryfs_utils::async_drop::AsyncDrop;
use cryfs_utils::data::Data;

#[derive(Debug)]
pub struct TrackingBlob<'a, B>
where
    B: BlobStore + AsyncDrop + Debug + 'static,
{
    blob: B::ConcreteBlob<'a>,
    counts: Arc<Mutex<BlobStoreActionCounts>>,
}

impl<'a, B> TrackingBlob<'a, B>
where
    B: BlobStore + AsyncDrop + Debug + 'static,
{
    pub fn new(blob: B::ConcreteBlob<'a>, counts: &Arc<Mutex<BlobStoreActionCounts>>) -> Self {
        Self {
            blob,
            counts: Arc::clone(counts),
        }
    }
}

#[async_trait]
impl<'a, B> Blob for TrackingBlob<'a, B>
where
    B: BlobStore + AsyncDrop + Debug + 'static,
{
    fn id(&self) -> BlobId {
        self.blob.id()
    }

    async fn num_bytes(&mut self) -> Result<u64> {
        self.counts.lock().unwrap().blob_num_bytes += 1;
        self.blob.num_bytes().await
    }

    async fn resize(&mut self, new_num_bytes: u64) -> Result<()> {
        self.counts.lock().unwrap().blob_resize += 1;
        self.blob.resize(new_num_bytes).await
    }

    async fn read_all(&mut self) -> Result<Data> {
        self.counts.lock().unwrap().blob_read_all += 1;
        self.blob.read_all().await
    }

    async fn read(&mut self, target: &mut [u8], offset: u64) -> Result<()> {
        self.counts.lock().unwrap().blob_read += 1;
        self.blob.read(target, offset).await
    }

    async fn try_read(&mut self, target: &mut [u8], offset: u64) -> Result<usize> {
        self.counts.lock().unwrap().blob_try_read += 1;
        self.blob.try_read(target, offset).await
    }

    async fn write(&mut self, source: &[u8], offset: u64) -> Result<()> {
        self.counts.lock().unwrap().blob_write += 1;
        self.blob.write(source, offset).await
    }

    async fn flush(&mut self) -> Result<()> {
        self.counts.lock().unwrap().blob_flush += 1;
        self.blob.flush().await
    }

    async fn num_nodes(&mut self) -> Result<u64> {
        self.counts.lock().unwrap().blob_num_nodes += 1;
        self.blob.num_nodes().await
    }

    async fn remove(self) -> Result<()> {
        self.counts.lock().unwrap().blob_remove += 1;
        self.blob.remove().await
    }

    fn all_blocks(&self) -> Result<BoxStream<'_, Result<BlockId>>> {
        self.counts.lock().unwrap().blob_all_blocks += 1;
        self.blob.all_blocks()
    }
}
