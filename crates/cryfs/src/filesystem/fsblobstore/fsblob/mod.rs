use anyhow::{bail, Result};
use async_trait::async_trait;
use futures::stream::BoxStream;
use std::fmt::Debug;

use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_blockstore::BlockId;
use cryfs_rustfs::{FsError, FsResult};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

mod atime_update_behavior;
pub use atime_update_behavior::AtimeUpdateBehavior;

mod layout;
pub use layout::BlobType;

mod base_blob;
use base_blob::BaseBlob;

mod file_blob;
pub use file_blob::FileBlob;

mod dir_blob;
pub use dir_blob::{DirBlob, DIR_LSTAT_SIZE, MODE_NEW_SYMLINK};

mod symlink_blob;
pub use symlink_blob::SymlinkBlob;

mod dir_entries;
pub use dir_entries::{DirEntry, EntryType};

#[derive(Debug)]
pub enum FsBlob<'a, B>
where
    // TODO Do we really need B: 'static ?
    B: BlobStore + Debug + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send,
{
    File(FileBlob<'a, B>),
    Directory(AsyncDropGuard<DirBlob<'a, B>>),
    Symlink(SymlinkBlob<'a, B>),
}

impl<'a, B> FsBlob<'a, B>
where
    B: BlobStore + Debug + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send,
{
    pub async fn parse(blob: B::ConcreteBlob<'a>) -> Result<AsyncDropGuard<FsBlob<'a, B>>> {
        let blob = BaseBlob::parse(blob).await?;
        match blob.blob_type() {
            BlobType::Dir => Ok(AsyncDropGuard::new(Self::Directory(
                DirBlob::new(blob).await?,
            ))),
            BlobType::File => Ok(AsyncDropGuard::new(Self::File(FileBlob::new(blob)))),
            BlobType::Symlink => Ok(AsyncDropGuard::new(Self::Symlink(SymlinkBlob::new(blob)))),
        }
    }

    pub fn blob_id(&self) -> BlobId {
        match &self {
            Self::File(blob) => blob.blob_id(),
            Self::Directory(blob) => blob.blob_id(),
            Self::Symlink(blob) => blob.blob_id(),
        }
    }

    pub fn parent(&self) -> BlobId {
        match &self {
            Self::File(blob) => blob.parent(),
            Self::Directory(blob) => blob.parent(),
            Self::Symlink(blob) => blob.parent(),
        }
    }

    pub async fn set_parent(&mut self, parent: &BlobId) -> Result<()> {
        match self {
            Self::File(blob) => blob.set_parent(parent).await,
            Self::Directory(blob) => blob.set_parent(parent).await,
            Self::Symlink(blob) => blob.set_parent(parent).await,
        }
    }

    pub async fn remove(this: AsyncDropGuard<Self>) -> Result<()> {
        match this.unsafe_into_inner_dont_drop() {
            Self::File(blob) => blob.remove().await,
            Self::Directory(blob) => DirBlob::remove(blob).await,
            Self::Symlink(blob) => blob.remove().await,
        }
    }

    pub fn blob_type(&self) -> BlobType {
        match self {
            Self::File(_) => BlobType::File,
            Self::Directory(_) => BlobType::Dir,
            Self::Symlink(_) => BlobType::Symlink,
        }
    }

    pub async fn into_file(mut this: AsyncDropGuard<Self>) -> Result<FileBlob<'a, B>> {
        if !matches!(*this, Self::File(_)) {
            this.async_drop().await?;
            bail!("FsBlob is not a file");
        }
        // No need to call async_drop since we were a file
        let this = this.unsafe_into_inner_dont_drop();

        let Self::File(blob) = this else {
            panic!("Can't happen since we checked above that this is a file");
        };
        Ok(blob)
    }

    pub async fn into_dir(
        mut this: AsyncDropGuard<Self>,
    ) -> Result<AsyncDropGuard<DirBlob<'a, B>>> {
        if !matches!(*this, Self::Directory(_)) {
            this.async_drop().await?;
            // TODO These should probably return FsError instead of anyhow
            bail!("FsBlob is not a directory");
        }
        // No need to call async_drop since we are going to return an AsyncDropGuard
        let this = this.unsafe_into_inner_dont_drop();

        let Self::Directory(blob) = this else {
            panic!("Can't happen since we checked above that this is a directory");
        };
        Ok(blob)
    }

    pub fn as_dir(&self) -> Result<&'_ DirBlob<'a, B>> {
        match self {
            Self::Directory(blob) => Ok(blob),
            _ => bail!("FsBlob is not a directory"),
        }
    }

    pub async fn into_symlink(mut this: AsyncDropGuard<Self>) -> Result<SymlinkBlob<'a, B>> {
        if !matches!(*this, Self::Symlink(_)) {
            this.async_drop().await?;
            bail!("FsBlob is not a symlink");
        }
        // No need to call async_drop since we were a file
        let this = this.unsafe_into_inner_dont_drop();

        let Self::Symlink(blob) = this else {
            panic!("Can't happen since we checked above that this is a symlink");
        };
        Ok(blob)
    }

    pub async fn lstat_size(&mut self) -> Result<u64> {
        match self {
            Self::File(blob) => blob.lstat_size().await,
            Self::Directory(blob) => Ok(blob.lstat_size()),
            Self::Symlink(blob) => blob.lstat_size().await,
        }
    }

    pub fn all_blocks(&self) -> Result<BoxStream<'_, Result<BlockId>>> {
        match self {
            Self::File(blob) => blob.all_blocks(),
            Self::Directory(blob) => blob.all_blocks(),
            Self::Symlink(blob) => blob.all_blocks(),
        }
    }
}

#[async_trait]
impl<'a, B> AsyncDrop for FsBlob<'a, B>
where
    B: BlobStore + Debug + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> FsResult<()> {
        match &mut self {
            Self::File(_blob) => { /* do nothing */ }
            Self::Directory(blob) => {
                blob.async_drop().await?;
            }
            Self::Symlink(_blob) => { /* do nothing */ }
        }
        Ok(())
    }
}
