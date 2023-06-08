use async_trait::async_trait;
use cryfs_rustfs::NumBytes;
use futures::{future, join};
use std::fmt::Debug;
use std::path::Path;
use std::time::SystemTime;

use crate::utils::fs_types;

use super::fsblobstore::{DirBlob, EntryType, FsBlob};
use super::{device::CryDevice, node::CryNode, open_file::CryOpenFile};
use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_rustfs::{
    object_based_api::Dir, DirEntry, FsError, FsResult, Gid, Mode, NodeAttrs, NodeKind, Uid,
};
use cryfs_utils::async_drop::{
    with_async_drop, with_async_drop_err_map, AsyncDrop, AsyncDropGuard,
};

pub struct CryDir<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    node: CryNode<'a, B>,
}

impl<'a, B> CryDir<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    pub fn new(node: CryNode<'a, B>) -> Self {
        Self { node }
    }

    async fn load_blob(&self) -> Result<AsyncDropGuard<DirBlob<'a, B>>, FsError> {
        let blob = self.node.load_blob().await?;
        FsBlob::into_dir(blob).await.map_err(|err| {
            FsError::CorruptedFilesystem {
                // TODO Add to message what it actually is
                message: format!("Blob {:?} is listed as a directory in its parent directory but is actually not a directory: {err:?}", self.node.blob_id()),
            }
        })
    }

    async fn create_dir_blob(&self, parent: &BlobId) -> Result<BlobId, FsError> {
        let mut blob = self
            .node
            .blobstore()
            .create_dir_blob(parent)
            .await
            .map_err(|err| {
                log::error!("Error creating dir blob: {err:?}");
                FsError::UnknownError
            })?;
        let blob_id = blob.blob_id();
        blob.async_drop().await.map_err(|err| {
            log::error!("Error dropping blob: {err:?}");
            FsError::UnknownError
        })?;
        Ok(blob_id)
    }
}

#[async_trait]
impl<'a, B> Dir for CryDir<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    type Device = CryDevice<B>;

    async fn entries(&self) -> FsResult<Vec<DirEntry>> {
        let blob = self.load_blob().await?;
        with_async_drop_err_map(
            blob,
            |blob| {
                future::ready((move || {
                    let entries = blob.entries();

                    let mut result = Vec::with_capacity(entries.len());
                    for entry in entries {
                        let name = match entry.name() {
                            Err(err) => {
                                return Err(FsError::CorruptedFilesystem {
                                    message: format!("Entry name is not valid UTF-8: {:?}", err),
                                });
                            }
                            Ok(ok) => ok.to_owned(),
                        };
                        let kind = match entry.entry_type() {
                            EntryType::Dir => NodeKind::Dir,
                            EntryType::File => NodeKind::File,
                            EntryType::Symlink => NodeKind::Symlink,
                        };
                        result.push(cryfs_rustfs::DirEntry { name, kind });
                    }
                    Ok(result)
                })())
            },
            |err| {
                log::error!("Error dropping blob: {err:?}");
                FsError::UnknownError
            },
        )
        .await
    }

    async fn create_child_dir(
        &self,
        name: &str,
        mode: Mode,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<NodeAttrs> {
        let (blob, new_dir_blob_id) =
            join!(self.load_blob(), self.create_dir_blob(self.node.blob_id()));
        let blob = blob?;
        // TODO Is this possible without to_owned()?
        let name = name.to_owned();
        with_async_drop_err_map(
            blob,
            move |blob| {
                future::ready((move || {
                    let new_dir_blob_id = new_dir_blob_id?;

                    let atime = SystemTime::now();
                    let mtime = atime;

                    let result = blob.add_entry_dir(
                        &name,
                        new_dir_blob_id,
                        // TODO Don't convert between fs_types::xxx and cryfs_rustfs::xxx but reuse the same types
                        fs_types::Mode::from(u32::from(mode)),
                        fs_types::Uid::from(u32::from(uid)),
                        fs_types::Gid::from(u32::from(gid)),
                        atime,
                        mtime,
                    );

                    result.map_err(|err| {
                        log::error!("Error adding dir entry: {err:?}");
                        FsError::UnknownError
                    })?;

                    // TODO Deduplicate this with the logic that looks up getattr for dir nodes and creates NodeAttrs from them there
                    Ok(NodeAttrs {
                        nlink: 1,
                        mode,
                        uid,
                        gid,
                        // TODO What should NumBytes be?
                        num_bytes: NumBytes::from(0),
                        num_blocks: None,
                        atime,
                        mtime,
                        ctime: mtime,
                    })
                })())
            },
            |err| {
                log::error!("Error dropping blob: {err:?}");
                FsError::UnknownError
            },
        )
        .await
    }

    async fn remove_child_dir(&self, name: &str) -> FsResult<()> {
        let blob = self.load_blob().await?;
        with_async_drop_err_map(
            blob,
            move |blob| async move {
                // First remove the entry, then flush that change, and only then drop the blob.
                // This is to make sure the file system doesn't end up in an invalid state
                // where the blob is removed but the entry is still there.
                let entry = blob.remove_entry_by_name(name).map_err(|utf8_err| {
                    FsError::CorruptedFilesystem {
                        message: "Directory entry has an entry name that is not utf-8".to_string(),
                    }
                })?;
                blob.flush().await;
                let blob_id = entry.blob_id();
                self.node
                    .blobstore()
                    .remove_by_id(blob_id)
                    .await
                    .map_err(|err| {
                        log::error!("Error removing blob: {err:?}");
                        FsError::UnknownError
                    })?;
                Ok(())
            },
            |err| {
                log::error!("Error dropping blob: {err:?}");
                FsError::UnknownError
            },
        )
        .await
    }

    async fn create_child_symlink(
        &self,
        name: &str,
        target: &Path,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<NodeAttrs> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }

    async fn remove_child_file_or_symlink(&self, name: &str) -> FsResult<()> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }

    async fn create_and_open_file(
        &self,
        name: &str,
        mode: Mode,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<(NodeAttrs, AsyncDropGuard<CryOpenFile<B>>)> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }

    async fn rename_child(&self, old_name: &str, new_path: &Path) -> FsResult<()> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }
}

/// Flattens two Result values that contain AsyncDropGuards, making sure that we correctly drop things if errors happen.
async fn flatten<E, T, E1, U, E2>(
    first: Result<AsyncDropGuard<T>, E1>,
    second: Result<AsyncDropGuard<U>, E2>,
) -> Result<(AsyncDropGuard<T>, AsyncDropGuard<U>), E>
where
    T: AsyncDrop + Debug,
    U: AsyncDrop + Debug,
    E: From<<T as AsyncDrop>::Error> + From<<U as AsyncDrop>::Error> + From<E1> + From<E2>,
{
    match (first, second) {
        (Ok(first), Ok(second)) => Ok((first, second)),
        (Ok(mut first), Err(second)) => {
            first.async_drop().await?;
            Err(second.into())
        }
        (Err(first), Ok(mut second)) => {
            second.async_drop().await?;
            Err(first.into())
        }
        (Err(first), Err(second)) => {
            // TODO Report both errors
            Err(first.into())
        }
    }
}
