use anyhow::{anyhow, Result};
use std::fmt::Debug;
use std::path::PathBuf;
use std::pin::Pin;
use futures::Stream;

use super::base_blob::BaseBlob;
use super::layout::BlobType;
use crate::blobstore::{BlobId, BlobStore};
use crate::blockstore::BlockId;

pub struct SymlinkBlob<'a, B>
where
    B: BlobStore + Debug + 'a,
{
    blob: BaseBlob<'a, B>,
}

impl<'a, B> SymlinkBlob<'a, B>
where
    B: BlobStore + Debug + 'a,
{
    pub(super) fn new(blob: BaseBlob<'a, B>) -> Self {
        Self { blob }
    }

    pub async fn create_blob(
        blobstore: &'a B,
        parent: &BlobId,
        target: &str,
    ) -> Result<SymlinkBlob<'a, B>> {
        Ok(Self {
            blob: BaseBlob::create(blobstore, BlobType::Symlink, parent, target.as_bytes()).await?,
        })
    }

    pub fn blob_id(&self) -> BlobId {
        self.blob.blob_id()
    }

    pub async fn target(&mut self) -> Result<PathBuf> {
        // TODO If blob.read_all() took &self instead of &mut self, we wouldn't have to make self "mut" in the parameter list above
        // TODO Should we cache the target and only read it once?
        let data = self.blob.read_all_data().await?;
        let target = std::str::from_utf8(&data)?;
        Ok(PathBuf::from(target))
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
        Ok(self.target().await?.to_str().ok_or_else(||anyhow!("Invalid UTF-8"))?.len() as u64)
    }

    pub async fn all_blocks(&self) -> Result<Box<dyn Stream<Item=Result<BlockId>> + Unpin + '_>> {
        self.blob.all_blocks().await
    }
}

impl<'a, B> Debug for SymlinkBlob<'a, B>
where
    B: BlobStore + Debug + 'a,
    <B as BlobStore>::ConcreteBlob<'a>: Send,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SymlinkBlob")
            .field("blob_id", &self.blob_id())
            .field("parent", &self.parent())
            .finish()
    }
}
