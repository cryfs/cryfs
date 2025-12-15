use anyhow::{Result, bail};
use async_trait::async_trait;
use cryfs_rustfs::{FsError, FsResult};
use futures::stream::BoxStream;
use std::fmt::Debug;

use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_blockstore::BlockId;
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
pub use dir_entries::{AddOrOverwriteError, EntryType, RenameError, DirEntry};

// TODO Now that FileBlob, DirBlob and SymlinkBlob are only ever returned as references,
//      we can probably store BaseBlob directly in here and just have FileBlob, DirBlob and SymlinkBlob
//      store a reference to BaseBlob instead of owning BaseBlob.

#[derive(Debug)]
pub enum FsBlob<B>
where
    // TODO Do we really need B: 'static ?
    B: BlobStore + Debug + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    File(AsyncDropGuard<FileBlob<B>>),
    Directory(AsyncDropGuard<DirBlob<B>>),
    Symlink(AsyncDropGuard<SymlinkBlob<B>>),
}

impl<B> FsBlob<B>
where
    B: BlobStore + Debug + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    pub async fn parse(blob: AsyncDropGuard<B::ConcreteBlob>) -> Result<AsyncDropGuard<FsBlob<B>>> {
        let mut blob = BaseBlob::parse(blob).await?;
        match blob.blob_type() {
            Ok(BlobType::Dir) => Ok(AsyncDropGuard::new(Self::Directory(
                DirBlob::new(blob).await?,
            ))),
            Ok(BlobType::File) => Ok(AsyncDropGuard::new(Self::File(FileBlob::new(blob)))),
            Ok(BlobType::Symlink) => Ok(AsyncDropGuard::new(Self::Symlink(SymlinkBlob::new(blob)))),
            Err(e) => {
                blob.async_drop().await?;
                Err(e)
            }
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
            Self::File(blob) => FileBlob::remove(blob).await,
            Self::Directory(blob) => DirBlob::remove(blob).await,
            Self::Symlink(blob) => SymlinkBlob::remove(blob).await,
        }
    }

    pub fn blob_type(&self) -> BlobType {
        match self {
            Self::File(_) => BlobType::File,
            Self::Directory(_) => BlobType::Dir,
            Self::Symlink(_) => BlobType::Symlink,
        }
    }

    pub fn as_file(&self) -> Result<&'_ FileBlob<B>> {
        match self {
            Self::File(blob) => Ok(blob),
            _ => bail!("FsBlob is not a file"),
        }
    }

    pub fn as_file_mut(&mut self) -> Result<&'_ mut FileBlob<B>> {
        match self {
            Self::File(blob) => Ok(blob),
            _ => bail!("FsBlob is not a file"),
        }
    }

    pub fn as_dir(&self) -> Result<&'_ DirBlob<B>> {
        match self {
            Self::Directory(blob) => Ok(blob),
            _ => bail!("FsBlob is not a directory"),
        }
    }

    pub fn as_dir_mut(&mut self) -> Result<&'_ mut DirBlob<B>> {
        match self {
            Self::Directory(blob) => Ok(blob),
            _ => bail!("FsBlob is not a directory"),
        }
    }

    pub fn as_symlink(&self) -> Result<&'_ SymlinkBlob<B>> {
        match self {
            Self::Symlink(blob) => Ok(blob),
            _ => bail!("FsBlob is not a symlink"),
        }
    }

    pub fn as_symlink_mut(&mut self) -> Result<&'_ mut SymlinkBlob<B>> {
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

    pub async fn flush(&mut self) -> Result<()> {
        match self {
            Self::File(blob) => blob.flush().await,
            Self::Directory(blob) => blob.flush().await,
            Self::Symlink(blob) => blob.flush().await,
        }
    }

    #[cfg(any(test, feature = "testutils"))]
    pub async fn into_raw(this: AsyncDropGuard<Self>) -> Result<AsyncDropGuard<B::ConcreteBlob>> {
        match this.unsafe_into_inner_dont_drop() {
            Self::File(blob) => Ok(FileBlob::into_raw(blob)),
            Self::Directory(blob) => DirBlob::into_raw(blob).await,
            Self::Symlink(blob) => Ok(SymlinkBlob::into_raw(blob)),
        }
    }
}

#[async_trait]
impl<B> AsyncDrop for FsBlob<B>
where
    B: BlobStore + Debug + 'static,
    B::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> FsResult<()> {
        match &mut self {
            Self::File(blob) => {
                blob.async_drop().await?;
            }
            Self::Directory(blob) => {
                blob.async_drop().await?;
            }
            Self::Symlink(blob) => {
                blob.async_drop().await?;
            }
        }
        Ok(())
    }
}
