use anyhow::Result;
use async_trait::async_trait;
use std::fmt::Debug;

use crate::blobstore::{BlobId, BlobStore};
use crate::utils::async_drop::{AsyncDrop, AsyncDropGuard};

mod fsblob;

pub use fsblob::{DirBlob, DirEntry, EntryType, FileBlob, FsBlob, SymlinkBlob, AtimeUpdateBehavior, FsError};

#[derive(Debug)]
pub struct FsBlobStore<B>
where
    // TODO Do we really need B: 'static ?
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send,
{
    blobstore: AsyncDropGuard<B>,
}

impl<B> FsBlobStore<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send,
{
    pub fn new(blobstore: AsyncDropGuard<B>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self { blobstore })
    }

    pub async fn create_file_blob<'a>(
        &'a self,
        parent: &BlobId,
    ) -> Result<fsblob::FileBlob<'a, B>> {
        FileBlob::create_blob(&*self.blobstore, parent).await
    }

    pub async fn create_dir_blob<'a>(&'a self, parent: &BlobId) -> Result<AsyncDropGuard<fsblob::DirBlob<'a, B>>> {
        DirBlob::create_blob(&*self.blobstore, parent).await
    }

    pub async fn create_symlink_blob<'a>(
        &'a self,
        parent: &BlobId,
        target: &str,
    ) -> Result<fsblob::SymlinkBlob<'a, B>> {
        SymlinkBlob::create_blob(&*self.blobstore, parent, target).await
    }

    pub async fn load<'a>(&'a self, blob_id: &BlobId) -> Result<Option<AsyncDropGuard<FsBlob<'a, B>>>> {
        if let Some(blob) = self.blobstore.load(blob_id).await? {
            Ok(Some(FsBlob::parse(blob).await?))
        } else {
            Ok(None)
        }
    }

    pub async fn num_blocks(&self) -> Result<u64> {
        self.blobstore.num_nodes().await
    }

    pub fn estimate_space_for_num_blocks_left(&self) -> Result<u64> {
        self.blobstore.estimate_space_for_num_blocks_left()
    }

    // virtual means "space we can use" as opposed to "space it takes on the disk" (i.e. virtual is without headers, checksums, ...)
    pub fn virtual_block_size_bytes(&self) -> u32 {
        self.blobstore.virtual_block_size_bytes()
    }

    // async fn remove_by_id(&self, id: &BlobId) -> Result<RemoveResult>;
}

#[async_trait]
impl<B> AsyncDrop for FsBlobStore<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send,
{
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.blobstore.async_drop().await
    }
}
