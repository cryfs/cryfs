use anyhow::Result;
use async_trait::async_trait;
use std::fmt::Debug;

use cryfs_blobstore::{BlobId, BlobStore, RemoveResult};
use cryfs_rustfs::{FsError, FsResult};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

mod fsblob;

pub use fsblob::{
    AtimeUpdateBehavior, BlobType, DirBlob, DirEntry, EntryType, FileBlob, FsBlob, SymlinkBlob,
    DIR_LSTAT_SIZE, MODE_NEW_SYMLINK,
};

// TODO With an adapter we can run block store tests on this, similar to how we do it for BlobStore
// TODO Add a CachingFsBlobStore, that currently only exists in C++.

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

    pub async fn create_root_dir_blob(&self, root_blob_id: &BlobId) -> Result<()> {
        DirBlob::create_root_dir_blob(&*self.blobstore, root_blob_id).await
    }

    pub async fn create_file_blob<'a>(
        &'a self,
        parent: &BlobId,
    ) -> Result<fsblob::FileBlob<'a, B>> {
        FileBlob::create_blob(&*self.blobstore, parent).await
    }

    pub async fn create_dir_blob<'a>(
        &'a self,
        parent: &BlobId,
    ) -> Result<AsyncDropGuard<fsblob::DirBlob<'a, B>>> {
        DirBlob::create_blob(&*self.blobstore, parent).await
    }

    pub async fn create_symlink_blob<'a>(
        &'a self,
        parent: &BlobId,
        target: &str,
    ) -> Result<fsblob::SymlinkBlob<'a, B>> {
        SymlinkBlob::create_blob(&*self.blobstore, parent, target).await
    }

    pub async fn load<'a>(
        &'a self,
        blob_id: &BlobId,
    ) -> Result<Option<AsyncDropGuard<FsBlob<'a, B>>>> {
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

    pub async fn load_block_depth(
        &self,
        block_id: &cryfs_blockstore::BlockId,
    ) -> Result<Option<u8>> {
        self.blobstore.load_block_depth(block_id).await
    }

    pub async fn remove_by_id(&self, id: &BlobId) -> Result<RemoveResult> {
        self.blobstore.remove_by_id(id).await
    }

    pub fn into_inner_blobstore(this: AsyncDropGuard<Self>) -> AsyncDropGuard<B> {
        this.unsafe_into_inner_dont_drop().blobstore
    }

    #[cfg(any(test, feature="testutils"))]
    pub async fn clear_cache_slow(&self) -> Result<()> {
        self.blobstore.clear_cache_slow().await
    }
}

#[async_trait]
impl<B> AsyncDrop for FsBlobStore<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> FsResult<()> {
        self.blobstore
            .async_drop()
            .await
            .map_err(|error| FsError::InternalError { error })
    }
}
