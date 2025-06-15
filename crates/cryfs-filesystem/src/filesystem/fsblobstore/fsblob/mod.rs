use anyhow::{Result, bail};
use async_trait::async_trait;
use futures::stream::BoxStream;
use std::fmt::Debug;

use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_blockstore::BlockId;
use cryfs_rustfs::{FsError, FsResult};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

mod layout;
pub use layout::BlobType;

mod base_blob;
use base_blob::BaseBlob;

mod file_blob;
pub use file_blob::FileBlob;

mod dir_blob;
pub use dir_blob::{DIR_LSTAT_SIZE, DirBlob, MODE_NEW_SYMLINK};

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
        match blob.blob_type()? {
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

    pub fn as_file(&self) -> Result<&'_ FileBlob<'a, B>> {
        match self {
            Self::File(blob) => Ok(blob),
            _ => bail!("FsBlob is not a file"),
        }
    }

    pub fn as_file_mut(&mut self) -> Result<&'_ mut FileBlob<'a, B>> {
        match self {
            Self::File(blob) => Ok(blob),
            _ => bail!("FsBlob is not a file"),
        }
    }

    pub fn as_dir(&self) -> Result<&'_ DirBlob<'a, B>> {
        match self {
            Self::Directory(blob) => Ok(blob),
            _ => bail!("FsBlob is not a directory"),
        }
    }

    pub fn as_dir_mut(&mut self) -> Result<&'_ mut DirBlob<'a, B>> {
        match self {
            Self::Directory(blob) => Ok(blob),
            _ => bail!("FsBlob is not a directory"),
        }
    }

    pub fn as_symlink_mut(&mut self) -> Result<&'_ mut SymlinkBlob<'a, B>> {
        match self {
            Self::Symlink(blob) => Ok(blob),
            _ => bail!("FsBlob is not a symlink"),
        }
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

    #[cfg(any(test, feature = "testutils"))]
    pub async fn into_raw(this: AsyncDropGuard<Self>) -> Result<B::ConcreteBlob<'a>> {
        match this.unsafe_into_inner_dont_drop() {
            Self::File(blob) => Ok(blob.into_raw()),
            Self::Directory(blob) => DirBlob::into_raw(blob).await,
            Self::Symlink(blob) => Ok(blob.into_raw()),
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
