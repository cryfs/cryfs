use anyhow::Result;
use async_trait::async_trait;
use byte_unit::Byte;
use std::fmt::Debug;

use cryfs_blobstore::{BlobId, BlobStore, RemoveResult};
use cryfs_rustfs::{FsError, FsResult};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

mod fsblob;

pub use fsblob::{
    BlobType, DIR_LSTAT_SIZE, DirBlob, DirEntry, EntryType, FileBlob, FsBlob, MODE_NEW_SYMLINK,
    SymlinkBlob,
};

// TODO With an adapter we can run block store tests on this, similar to how we do it for BlobStore
// TODO Add a CachingFsBlobStore, that currently only exists in C++.

#[derive(Debug)]
pub struct FsBlobStore<B>
where
    // TODO Do we really need B: 'static ?
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    blobstore: AsyncDropGuard<B>,
}

impl<B> FsBlobStore<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    pub fn new(blobstore: AsyncDropGuard<B>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self { blobstore })
    }

    pub async fn create_root_dir_blob(
        &self,
        root_blob_id: &BlobId,
    ) -> Result<AsyncDropGuard<fsblob::FsBlob<B>>> {
        Ok(AsyncDropGuard::new(FsBlob::Directory(
            DirBlob::create_root_dir_blob(&*self.blobstore, root_blob_id).await?,
        )))
    }

    pub async fn create_file_blob<'a>(
        &'a self,
        parent: &BlobId,
        flush_behavior: FlushBehavior,
    ) -> Result<AsyncDropGuard<fsblob::FsBlob<B>>> {
        let mut file_blob = FileBlob::create_blob(&*self.blobstore, parent).await?;
        match flush_behavior {
            FlushBehavior::FlushImmediately => {
                file_blob.flush().await?;
            }
            FlushBehavior::DontFlush => {}
        }
        Ok(AsyncDropGuard::new(FsBlob::File(file_blob)))
    }

    pub async fn create_dir_blob<'a>(
        &'a self,
        parent: &BlobId,
        flush_behavior: FlushBehavior,
    ) -> Result<AsyncDropGuard<fsblob::FsBlob<B>>> {
        let mut dir_blob = DirBlob::create_blob(&*self.blobstore, parent).await?;
        match flush_behavior {
            FlushBehavior::FlushImmediately => {
                dir_blob.flush().await?;
            }
            FlushBehavior::DontFlush => {}
        }
        Ok(AsyncDropGuard::new(FsBlob::Directory(dir_blob)))
    }

    pub async fn create_symlink_blob<'a>(
        &'a self,
        parent: &BlobId,
        target: &str,
        flush_behavior: FlushBehavior,
    ) -> Result<AsyncDropGuard<fsblob::FsBlob<B>>> {
        let mut symlink_blob = SymlinkBlob::create_blob(&*self.blobstore, parent, target).await?;
        match flush_behavior {
            FlushBehavior::FlushImmediately => {
                symlink_blob.flush().await?;
            }
            FlushBehavior::DontFlush => {}
        }
        Ok(AsyncDropGuard::new(FsBlob::Symlink(symlink_blob)))
    }

    pub async fn load<'a>(&'a self, blob_id: &BlobId) -> Result<Option<AsyncDropGuard<FsBlob<B>>>> {
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

    // logical means "space we can use" as opposed to "space it takes on the disk" (i.e. logical is without headers, checksums, ...)
    pub fn logical_block_size_bytes(&self) -> Byte {
        self.blobstore.logical_block_size_bytes()
    }

    pub async fn remove_by_id(&self, id: &BlobId) -> Result<RemoveResult> {
        self.blobstore.remove_by_id(id).await
    }

    pub fn into_inner_blobstore(this: AsyncDropGuard<Self>) -> AsyncDropGuard<B> {
        this.unsafe_into_inner_dont_drop().blobstore
    }

    #[cfg(any(test, feature = "testutils"))]
    pub async fn clear_cache_slow(&self) -> Result<()> {
        self.blobstore.clear_cache_slow().await
    }
}

pub enum FlushBehavior {
    FlushImmediately,
    DontFlush,
}

#[async_trait]
impl<B> AsyncDrop for FsBlobStore<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> FsResult<()> {
        self.blobstore
            .async_drop()
            .await
            .map_err(|error| FsError::InternalError { error })
    }
}
