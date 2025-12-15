use anyhow::Result;
use async_trait::async_trait;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};
use futures::stream::BoxStream;
use std::fmt::Debug;

use super::{base_blob::BaseBlob, layout::BlobType};
use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_blockstore::BlockId;

pub struct FileBlob<B>
where
    B: BlobStore + Debug,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    blob: AsyncDropGuard<BaseBlob<B>>,
}

impl<B> FileBlob<B>
where
    B: BlobStore + Debug,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    pub(super) fn new(blob: AsyncDropGuard<BaseBlob<B>>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self { blob })
    }

    pub async fn create_blob(
        blobstore: &B,
        parent: &BlobId,
    ) -> Result<AsyncDropGuard<FileBlob<B>>> {
        Ok(AsyncDropGuard::new(Self {
            blob: BaseBlob::create(blobstore, BlobType::File, parent, &[]).await?,
        }))
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

    pub async fn remove(this: AsyncDropGuard<Self>) -> Result<()> {
        BaseBlob::remove(this.unsafe_into_inner_dont_drop().blob).await
    }

    pub async fn lstat_size(&mut self) -> Result<u64> {
        self.num_bytes().await
    }

    pub fn all_blocks(&self) -> Result<BoxStream<'_, Result<BlockId>>> {
        self.blob.all_blocks()
    }

    // TODO Remove `num_nodes()` and let call sites call it after a call to `into_raw()` gives them a `BlobOnBlocks`
    #[cfg(any(test, feature = "testutils"))]
    pub async fn num_nodes(&mut self) -> Result<u64> {
        self.blob.num_nodes().await
    }

    #[cfg(any(test, feature = "testutils"))]
    pub fn into_raw(this: AsyncDropGuard<Self>) -> AsyncDropGuard<B::ConcreteBlob> {
        BaseBlob::into_raw(this.unsafe_into_inner_dont_drop().blob)
    }
}

impl<B> Debug for FileBlob<B>
where
    B: BlobStore + Debug,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
    <B as BlobStore>::ConcreteBlob: Send,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileBlob")
            .field("blob_id", &self.blob_id())
            .field("parent", &self.parent())
            .finish()
    }
}

#[async_trait]
impl<B> AsyncDrop for FileBlob<B>
where
    B: BlobStore + Debug,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.blob.async_drop().await?;
        Ok(())
    }
}
