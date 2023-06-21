use async_trait::async_trait;
use cryfs_rustfs::NumBytes;
use futures::{future, join};
use std::fmt::Debug;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::OnceCell;

use super::fsblobstore::{BlobType, DirBlob, EntryType, FsBlob, FsBlobStore, MODE_NEW_SYMLINK};
use super::{
    device::CryDevice,
    node_info::{BlobDetails, NodeInfo},
    open_file::CryOpenFile,
};
use crate::utils::fs_types;
use cryfs_blobstore::{BlobId, BlobStore, RemoveResult};
use cryfs_rustfs::{
    object_based_api::Dir, DirEntry, FsError, FsResult, Gid, Mode, NodeAttrs, NodeKind,
    PathComponent, Uid,
};
use cryfs_utils::async_drop::{with_async_drop_err_map, AsyncDrop, AsyncDropArc, AsyncDropGuard};

pub struct CryDir<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    // TODO Here and in others, can we just store &FsBlobStore instead of &AsyncDropGuard?
    blobstore: &'a AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
    node_info: Arc<NodeInfo>,
}

impl<'a, B> CryDir<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    pub fn new(
        blobstore: &'a AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
        node_info: Arc<NodeInfo>,
    ) -> Self {
        Self {
            blobstore,
            node_info,
        }
    }

    async fn load_blob(&self) -> Result<AsyncDropGuard<DirBlob<'a, B>>, FsError> {
        let blob = self.node_info.load_blob(self.blobstore).await?;
        let blob_id = blob.blob_id();
        FsBlob::into_dir(blob).await.map_err(|err| {
            FsError::CorruptedFilesystem {
                // TODO Add to message what it actually is
                message: format!("Blob {:?} is listed as a directory in its parent directory but is actually not a directory: {err:?}", blob_id),
            }
        })
    }

    async fn create_dir_blob(&self, parent: &BlobId) -> Result<BlobId, FsError> {
        let mut blob = self
            .blobstore
            .create_dir_blob(parent)
            .await
            .map_err(|err| {
                log::error!("Error creating dir blob: {err:?}");
                FsError::UnknownError
            })?;
        let blob_id = blob.blob_id();

        // Make sure we flush this before the call site gets a chance to add this as an entry to its directory entry list.
        // This way, we make sure the filesystem stays consistent even if it crashes mid way.
        // Dropping by itself isn't enough to flush because it may go into a cache.
        // TODO Check if this is necessary or if create_dir_blob already flushes. Or maybe we should still keep this here but make sure
        // it is a no-op if a blob is not dirty.
        blob.flush().await.map_err(|err| {
            log::error!("Error flushing blob: {err:?}");
            FsError::UnknownError
        })?;

        blob.async_drop().await.map_err(|err| {
            log::error!("Error dropping blob: {err:?}");
            FsError::UnknownError
        })?;
        Ok(blob_id)
    }

    async fn create_file_blob(&self, parent: &BlobId) -> Result<BlobId, FsError> {
        let mut blob = self
            .blobstore
            .create_file_blob(parent)
            .await
            .map_err(|err| {
                log::error!("Error creating file blob: {err:?}");
                FsError::UnknownError
            })?;
        let blob_id = blob.blob_id();

        // Make sure we flush this before the call site gets a chance to add this as an entry to its directory entry list.
        // This way, we make sure the filesystem stays consistent even if it crashes mid way.
        // Dropping by itself isn't enough to flush because it may go into a cache.
        // TODO Check if this is necessary or if create_dir_blob already flushes. Or maybe we should still keep this here but make sure
        // it is a no-op if a blob is not dirty.
        blob.flush().await.map_err(|err| {
            log::error!("Error flushing blob: {err:?}");
            FsError::UnknownError
        })?;

        Ok(blob_id)
    }

    async fn create_symlink_blob(&self, target: &str, parent: &BlobId) -> Result<BlobId, FsError> {
        let mut blob = self
            .blobstore
            .create_symlink_blob(parent, target)
            .await
            .map_err(|err| {
                log::error!("Error creating symlink blob: {err:?}");
                FsError::UnknownError
            })?;
        let blob_id = blob.blob_id();

        // Make sure we flush this before the call site gets a chance to add this as an entry to its directory entry list.
        // This way, we make sure the filesystem stays consistent even if it crashes mid way.
        // Dropping by itself isn't enough to flush because it may go into a cache.
        // TODO Check if this is necessary or if create_dir_blob already flushes. Or maybe we should still keep this here but make sure
        // it is a no-op if a blob is not dirty.
        blob.flush().await.map_err(|err| {
            log::error!("Error flushing blob: {err:?}");
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
                        let name = entry.name().to_owned();
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
        name: &PathComponent,
        mode: Mode,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<NodeAttrs> {
        let blob_id = self.node_info.blob_id(&self.blobstore).await?;
        let (blob, new_dir_blob_id) = join!(self.load_blob(), self.create_dir_blob(&blob_id));
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
                        name,
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

    async fn remove_child_dir(&self, name: &PathComponent) -> FsResult<()> {
        let mut blob = self.load_blob().await?;

        // TODO Check the entry is actually a dir before removing it

        // First remove the entry, then flush that change, and only then remove the blob.
        // This is to make sure the file system doesn't end up in an invalid state
        // where the blob is removed but the entry is still there.
        let result = match blob.remove_entry_by_name(name) {
            Err(err) => Err(err),
            Ok(entry) => match blob.flush().await {
                Err(err) => {
                    log::error!("Error flushing blob: {err:?}");
                    Err(FsError::UnknownError)
                }
                Ok(()) => {
                    let blob_id = entry.blob_id();
                    let remove_result = self.blobstore.remove_by_id(blob_id).await;
                    match remove_result {
                            Ok(RemoveResult::SuccessfullyRemoved) => Ok(()),
                            Ok(RemoveResult::NotRemovedBecauseItDoesntExist) => {
                                Err(FsError::CorruptedFilesystem { message: format!("Removed entry {name} from directory but didn't find its blob {blob_id:?} to remove") })
                            }
                            Err(err) => {
                                log::error!("Error removing blob: {err:?}");
                                Err(FsError::UnknownError)
                            }
                        }
                }
            },
        };

        blob.async_drop().await.map_err(|err| {
            log::error!("Error dropping blob: {err:?}");
            FsError::UnknownError
        })?;

        result
    }

    async fn create_child_symlink(
        &self,
        name: &PathComponent,
        target: &str,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<NodeAttrs> {
        // TODO What should NumBytes be? Also, no unwrap?
        let num_bytes = NumBytes::from(u64::try_from(name.len()).unwrap());

        let blob_id = self.node_info.blob_id(&self.blobstore).await?;

        let (blob, new_symlink_blob_id) =
            join!(self.load_blob(), self.create_symlink_blob(target, &blob_id),);
        let blob = blob?;
        // TODO Is this possible without to_owned()?
        let name = name.to_owned();
        with_async_drop_err_map(
            blob,
            move |blob| {
                future::ready((move || {
                    let new_symlink_blob_id = new_symlink_blob_id?;

                    let atime = SystemTime::now();
                    let mtime = atime;

                    let result = blob.add_entry_symlink(
                        name,
                        new_symlink_blob_id,
                        // TODO Don't convert between fs_types::xxx and cryfs_rustfs::xxx but reuse the same types
                        fs_types::Uid::from(u32::from(uid)),
                        fs_types::Gid::from(u32::from(gid)),
                        atime,
                        mtime,
                    );

                    result.map_err(|err| {
                        log::error!("Error adding dir entry: {err:?}");
                        FsError::UnknownError
                    })?;

                    // TODO Deduplicate this with the logic that looks up getattr for symlink nodes and creates NodeAttrs from them there
                    Ok(NodeAttrs {
                        nlink: 1,
                        // TODO Don't convert mode but unify both classes
                        mode: cryfs_rustfs::Mode::from(u32::from(MODE_NEW_SYMLINK)),
                        uid,
                        gid,
                        num_bytes,
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

    async fn remove_child_file_or_symlink(&self, name: &PathComponent) -> FsResult<()> {
        let mut blob = self.load_blob().await?;

        // TODO Check the entry is actually a file or symlink before removing it

        // First remove the entry, then flush that change, and only then remove the blob.
        // This is to make sure the file system doesn't end up in an invalid state
        // where the blob is removed but the entry is still there.
        let result = match blob.remove_entry_by_name(name) {
            Err(err) => Err(err),
            Ok(entry) => match blob.flush().await {
                Err(err) => {
                    log::error!("Error flushing blob: {err:?}");
                    Err(FsError::UnknownError)
                }
                Ok(()) => {
                    let blob_id = entry.blob_id();
                    let remove_result = self.blobstore.remove_by_id(blob_id).await;
                    match remove_result {
                                Ok(RemoveResult::SuccessfullyRemoved) => Ok(()),
                                Ok(RemoveResult::NotRemovedBecauseItDoesntExist) => {
                                    Err(FsError::CorruptedFilesystem { message: format!("Removed entry {name} from directory but didn't find its blob {blob_id:?} to remove") })
                                }
                                Err(err) => {
                                    log::error!("Error removing blob: {err:?}");
                                    Err(FsError::UnknownError)
                                }
                            }
                }
            },
        };

        blob.async_drop().await.map_err(|err| {
            log::error!("Error dropping blob: {err:?}");
            FsError::UnknownError
        })?;

        result
    }

    async fn create_and_open_file(
        &self,
        name: &PathComponent,
        mode: Mode,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<(NodeAttrs, AsyncDropGuard<CryOpenFile<B>>)> {
        let blob_id = self.node_info.blob_id(&self.blobstore).await?;
        let (blob, new_file_blob_id) = join!(self.load_blob(), self.create_file_blob(&blob_id),);
        let mut blob = blob?;

        let new_file_blob_id = match new_file_blob_id {
            Ok(ok) => ok,
            Err(err) => {
                blob.async_drop().await.map_err(|err| {
                    log::error!("Error dropping Arc<FsBlobstore>: {err:?}");
                    FsError::UnknownError
                })?;
                return Err(err);
            }
        };

        let atime = SystemTime::now();
        let mtime = atime;

        let result = blob.add_entry_file(
            name.to_owned(),
            new_file_blob_id,
            // TODO Don't convert between fs_types::xxx and cryfs_rustfs::xxx but reuse the same types
            fs_types::Mode::from(u32::from(mode)),
            fs_types::Uid::from(u32::from(uid)),
            fs_types::Gid::from(u32::from(gid)),
            atime,
            mtime,
        );

        match result {
            Ok(()) => (),
            Err(err) => {
                log::error!("Error adding dir entry: {err:?}");
                blob.async_drop().await.map_err(|err| {
                    log::error!("Error dropping Arc<FsBlobstore>: {err:?}");
                    FsError::UnknownError
                })?;
                return Err(FsError::UnknownError);
            }
        }

        // TODO Deduplicate this with the logic that looks up getattr for symlink nodes and creates NodeAttrs from them there
        let attrs = NodeAttrs {
            nlink: 1,
            mode,
            uid,
            gid,
            num_bytes: NumBytes::from(0),
            num_blocks: None,
            atime,
            mtime,
            ctime: mtime,
        };

        let open_file = CryOpenFile::new(
            AsyncDropArc::clone(self.blobstore),
            Arc::new(NodeInfo::IsNotRootDir {
                parent_blob_id: blob_id,
                name: name.to_owned(),
                blob_details: OnceCell::new_with(Some(BlobDetails {
                    blob_id: new_file_blob_id,
                    blob_type: BlobType::File,
                })),
            }),
        );

        blob.async_drop().await.map_err(|err| {
            log::error!("Error dropping Arc<FsBlobstore>: {err:?}");
            FsError::UnknownError
        })?;

        Ok((attrs, open_file))
    }
}
