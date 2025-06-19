use anyhow::Result;
use async_trait::async_trait;
use cryfs_rustfs::FsError;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};
use futures::stream::BoxStream;
use std::fmt::Debug;

use super::base_blob::BaseBlob;
use super::layout::BlobType;
use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_blockstore::BlockId;

pub struct SymlinkBlob<B>
where
    B: BlobStore + Debug,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    blob: AsyncDropGuard<BaseBlob<B>>,
}

impl<B> SymlinkBlob<B>
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
        target: &str,
    ) -> Result<AsyncDropGuard<SymlinkBlob<B>>> {
        Ok(AsyncDropGuard::new(Self {
            blob: BaseBlob::create(blobstore, BlobType::Symlink, parent, target.as_bytes()).await?,
        }))
    }

    pub fn blob_id(&self) -> BlobId {
        self.blob.blob_id()
    }

    pub async fn target(&mut self) -> Result<String> {
        // TODO If blob.read_all() took &self instead of &mut self, we wouldn't have to make self "mut" in the parameter list above
        // TODO Should we cache the target and only read it once?
        let data = self.blob.read_all_data().await?;
        let target = String::from_utf8(data.into_vec())?;
        Ok(target)
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
        Ok(self.target().await?.len() as u64)
    }

    pub async fn flush(&mut self) -> Result<()> {
        self.blob.flush().await
    }

    pub fn all_blocks(&self) -> Result<BoxStream<'_, Result<BlockId>>> {
        self.blob.all_blocks()
    }

    #[cfg(any(test, feature = "testutils"))]
    pub async fn num_nodes(&mut self) -> Result<u64> {
        self.blob.num_nodes().await
    }

    #[cfg(any(test, feature = "testutils"))]
    pub fn into_raw(this: AsyncDropGuard<Self>) -> AsyncDropGuard<B::ConcreteBlob> {
        BaseBlob::into_raw(this.unsafe_into_inner_dont_drop().blob)
    }
}

impl<B> Debug for SymlinkBlob<B>
where
    B: BlobStore + Debug,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SymlinkBlob")
            .field("blob_id", &self.blob_id())
            .field("parent", &self.parent())
            .finish()
    }
}

#[async_trait]
impl<B> AsyncDrop for SymlinkBlob<B>
where
    B: BlobStore + Debug,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.blob
            .async_drop()
            .await
            .map_err(|err| FsError::InternalError {
                error: err.context("Error in SymlinkBlob::async_drop_impl"),
            })?;
        Ok(())
    }
}
