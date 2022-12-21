use anyhow::Result;
use futures::Stream;
use std::fmt::Debug;

use super::{base_blob::BaseBlob, layout::BlobType};
use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_blockstore::BlockId;

pub struct FileBlob<'a, B>
where
    B: BlobStore + Debug + 'a,
{
    blob: BaseBlob<'a, B>,
}

impl<'a, B> FileBlob<'a, B>
where
    B: BlobStore + Debug + 'a,
{
    pub(super) fn new(blob: BaseBlob<'a, B>) -> Self {
        Self { blob }
    }

    pub async fn create_blob(blobstore: &'a B, parent: &BlobId) -> Result<FileBlob<'a, B>> {
        Ok(Self {
            blob: BaseBlob::create(blobstore, BlobType::File, parent, &[]).await?,
        })
    }

    pub fn blob_id(&self) -> BlobId {
        self.blob.blob_id()
    }

    pub async fn num_bytes(&mut self) -> Result<u64> {
        self.blob.num_data_bytes().await
    }

    pub async fn resize(&mut self, new_num_bytes: u64) -> Result<()> {
        self.blob.resize_data(new_num_bytes).await
    }

    pub async fn try_read(&mut self, target: &mut [u8], offset: u64) -> Result<usize> {
        self.blob.try_read_data(target, offset).await
    }

    pub async fn write(&mut self, source: &[u8], offset: u64) -> Result<()> {
        self.blob.write_data(source, offset).await
    }

    pub async fn flush(&mut self) -> Result<()> {
        self.blob.flush().await
    }

    pub fn parent(&self) -> BlobId {
        self.blob.parent()
    }

    pub async fn set_parent(&mut self, new_parent: &BlobId) -> Result<()> {
        self.blob.set_parent(new_parent).await
    }

    pub async fn remove(self) -> Result<()> {
        self.blob.remove().await
    }

    pub async fn lstat_size(&mut self) -> Result<u64> {
        self.num_bytes().await
    }

    pub async fn all_blocks(&self) -> Result<Box<dyn Stream<Item = Result<BlockId>> + Unpin + '_>> {
        self.blob.all_blocks().await
    }
}

impl<'a, B> Debug for FileBlob<'a, B>
where
    B: BlobStore + Debug + 'a,
    <B as BlobStore>::ConcreteBlob<'a>: Send,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileBlob")
            .field("blob_id", &self.blob_id())
            .field("parent", &self.parent())
            .finish()
    }
}
