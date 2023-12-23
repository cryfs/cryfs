use anyhow::Result;
use futures::stream::BoxStream;
use std::fmt::Debug;

use super::base_blob::BaseBlob;
use super::layout::BlobType;
use cryfs_blobstore::{BlobId, BlobStore, BlobStoreOnBlocks, DataNode};
use cryfs_blockstore::{BlockId, BlockStore};

pub struct SymlinkBlob<'a, B>
where
    B: BlobStore + Debug + 'a,
{
    blob: BaseBlob<'a, B>,
}

impl<'a, B> SymlinkBlob<'a, BlobStoreOnBlocks<B>>
where
    B: BlockStore + Send + Sync,
{
    // TODO We're duplicating a lot of the `BaseBlob` methods here and just passing them through. Might be better to offer a `.base_blob()` method and then the blob store can call those methods directly on the base blob.
    pub fn load_all_nodes(self) -> BoxStream<'a, Result<DataNode<B>, (BlockId, anyhow::Error)>> {
        self.blob.load_all_nodes()
    }
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

    pub async fn remove(self) -> Result<()> {
        self.blob.remove().await
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
