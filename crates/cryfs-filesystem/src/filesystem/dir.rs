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
    node::CryNode,
    node_info::{BlobDetails, NodeInfo},
    open_file::CryOpenFile,
    symlink::CrySymlink,
};
use crate::utils::fs_types;
use cryfs_blobstore::{BlobId, BlobStore, RemoveResult};
use cryfs_rustfs::{
    DirEntry, FsError, FsResult, Gid, Mode, NodeAttrs, NodeKind, PathComponent, Uid,
    object_based_api::Dir,
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard, flatten_async_drop, with_async_drop},
    with_async_drop_2,
};

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

        blob.async_drop().await?;
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

    async fn on_rename_overwrites_destination(
        &self,
        old_destination_blob_id: BlobId,
    ) -> FsResult<()> {
        let result = self
            .blobstore
            .remove_by_id(&old_destination_blob_id)
            .await
            .map_err(|err| {
                log::error!("Error removing blob: {:?}", err);
                FsError::UnknownError
            });
        match result {
            Ok(RemoveResult::SuccessfullyRemoved) => Ok(()),
            Ok(RemoveResult::NotRemovedBecauseItDoesntExist) => {
                log::error!("During rename->overwrite, tried to remove blob that doesn't exist");
                Err(FsError::UnknownError)
            }
            Err(err) => {
                log::error!("Error removing blob: {:?}", err);
                Err(FsError::UnknownError)
            }
        }
    }
}

#[async_trait]
impl<'a, B> Dir for CryDir<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    type Device = CryDevice<B>;

    fn as_node(&self) -> AsyncDropGuard<CryNode<B>> {
        CryNode::new_internal(
            AsyncDropArc::clone(&self.blobstore),
            Arc::clone(&self.node_info),
        )
    }

    async fn lookup_child(&self, name: &PathComponent) -> FsResult<AsyncDropGuard<CryNode<B>>> {
        let self_blob_id = self.node_info.blob_id(&self.blobstore).await?;
        let node_info = NodeInfo::new(
            self_blob_id,
            name.to_owned(),
            self.node_info.atime_update_behavior(),
        );
        Ok(CryNode::new(
            AsyncDropArc::clone(&self.blobstore),
            node_info,
        ))
    }

    async fn rename_child(&self, oldname: &PathComponent, newname: &PathComponent) -> FsResult<()> {
        let mut blob = self.load_blob().await?;
        let result = blob
            .rename_entry_by_name(oldname, newname.to_owned(), async |blob_id| {
                // TODO Is overwriting actually allowed here if the new entry already exists?
                self.on_rename_overwrites_destination(*blob_id).await
            })
            .await;
        blob.async_drop().await?;
        result
    }

    async fn move_child_to(
        &self,
        oldname: &PathComponent,
        newparent: Self,
        newname: &PathComponent,
    ) -> FsResult<()> {
        let (source_parent, dest_parent) = join!(self.load_blob(), newparent.load_blob());
        // TODO Use with_async_drop! for source_parent, dest_parent, self_blob
        let (mut source_parent, mut dest_parent) =
            flatten_async_drop(source_parent, dest_parent).await?;
        let entry = match source_parent.entry_by_name(oldname) {
            Some(entry) => entry,
            None => {
                // TODO Drop concurrently and drop latter even if first one fails
                source_parent.async_drop().await?;
                dest_parent.async_drop().await?;
                return Err(FsError::NodeDoesNotExist);
            }
        };
        let self_blob_id = entry.blob_id();
        // TODO In theory, we could load self_blob concurrently with dest_parent_blob. No need to only do it after dest_parent_blob loaded.
        //      But it likely has some dependency with source_parent_blob.
        let self_blob = self.blobstore.load(self_blob_id).await;
        let mut self_blob = match self_blob {
            Ok(Some(self_blob)) => self_blob,
            Ok(None) => {
                // TODO Drop concurrently and drop latter even if first one fails
                // TODO This branch means there was an entry in the parent dir but the blob itself doesn't exist. How should we handle this?
                source_parent.async_drop().await?;
                dest_parent.async_drop().await?;
                return Err(FsError::NodeDoesNotExist);
            }
            Err(err) => {
                // TODO Drop concurrently and drop latter even if first one fails
                source_parent.async_drop().await?;
                dest_parent.async_drop().await?;
                log::error!("Error loading blob: {:?}", err);
                return Err(FsError::UnknownError);
            }
        };

        let mut existing_dir_check = || {
            if let Some(existing_dest_entry) = dest_parent.entry_by_name_mut(newname) {
                if existing_dest_entry.entry_type() == EntryType::Dir {
                    let self_blob = self_blob.as_dir()
                        .map_err(|_| FsError::CorruptedFilesystem { message: format!("Blob {self_blob_id:?} is not a directory but its entry in its parent directory says it is") })?;
                    if self_blob.entries().len() > 0 {
                        return Err(FsError::CannotOverwriteNonEmptyDirectory);
                    }
                }
            }
            Ok(())
        };
        match existing_dir_check() {
            Ok(()) => (),
            Err(err) => {
                // TODO Drop concurrently and drop latter even if first one fails
                source_parent.async_drop().await?;
                dest_parent.async_drop().await?;
                self_blob.async_drop().await?;
                return Err(err);
            }
        }

        let res = source_parent.remove_entry_by_name(oldname);
        let entry = match res {
            Ok(entry) => entry,
            Err(err) => {
                // TODO Drop concurrently and drop latter even if first one fails
                source_parent.async_drop().await?;
                dest_parent.async_drop().await?;
                self_blob.async_drop().await?;
                log::error!("Error in add_or_overwrite_entry: {err:?}");
                return Err(FsError::UnknownError);
            }
        };
        let res = dest_parent
            .add_or_overwrite_entry(
                newname.to_owned(),
                *entry.blob_id(),
                entry.entry_type(),
                entry.mode(),
                entry.uid(),
                entry.gid(),
                entry.last_access_time(),
                entry.last_modification_time(),
                async |blob_id| {
                    // TODO Is overwriting actually allowed here if the new entry already exists?
                    self.on_rename_overwrites_destination(*blob_id).await
                },
            )
            .await;
        match res {
            Ok(()) => (),
            Err(err) => {
                // TODO Exception safety - we couldn't add the entry to the destination, but we already removed it from the source. We should probably re-add it to the source.
                // TODO Drop concurrently and drop latter even if first one fails
                source_parent.async_drop().await?;
                dest_parent.async_drop().await?;
                self_blob.async_drop().await?;
                log::error!("Error in add_or_overwrite_entry: {err:?}");
                return Err(FsError::UnknownError);
            }
        }

        let res = self_blob.set_parent(&dest_parent.blob_id()).await;
        match res {
            Ok(()) => (),
            Err(err) => {
                // TODO Exception safety - we already changed parent dir entries but couldn't update the parent pointer. We should probably try to undo the parent dir entry changes.
                // TODO Drop concurrently and drop latter even if first one fails
                source_parent.async_drop().await?;
                dest_parent.async_drop().await?;
                self_blob.async_drop().await?;
                log::error!("Error setting parent: {err:?}");
                return Err(FsError::UnknownError);
            }
        }
        // TODO Drop concurrently and drop latter even if first one fails
        self_blob.async_drop().await?;
        source_parent.async_drop().await?;
        dest_parent.async_drop().await?;
        Ok(())

        // TODO We need to update timestamps of the parent directories in the grandparent blobs.
    }

    async fn entries(&self) -> FsResult<Vec<DirEntry>> {
        let blob = self.load_blob().await?;
        with_async_drop(blob, |blob| {
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
        })
        .await
    }

    async fn create_child_dir(
        &self,
        name: &PathComponent,
        mode: Mode,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<(NodeAttrs, CryDir<'_, B>)> {
        let self_blob_id = self.node_info.blob_id(&self.blobstore).await?;
        let (blob, new_dir_blob_id) = join!(self.load_blob(), self.create_dir_blob(&self_blob_id));
        let blob = blob?;
        // TODO Is this possible without to_owned()?
        let name = name.to_owned();
        with_async_drop(blob, move |blob| {
            future::ready((move || {
                let new_dir_blob_id = new_dir_blob_id?;

                let atime = SystemTime::now();
                let mtime = atime;

                blob.add_entry_dir(
                    name.clone(),
                    new_dir_blob_id,
                    // TODO Don't convert between fs_types::xxx and cryfs_rustfs::xxx but reuse the same types
                    fs_types::Mode::from(u32::from(mode)),
                    fs_types::Uid::from(u32::from(uid)),
                    fs_types::Gid::from(u32::from(gid)),
                    atime,
                    mtime,
                )
                .map_err(|err| {
                    log::error!("Error adding dir entry: {err:?}");
                    FsError::UnknownError
                })?;

                // TODO Deduplicate this with the logic that looks up getattr for dir nodes and creates NodeAttrs from them there
                let attrs = NodeAttrs {
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
                };
                let node = CryDir::new(
                    &self.blobstore,
                    Arc::new(NodeInfo::new(
                        self_blob_id,
                        name,
                        self.node_info.atime_update_behavior(),
                    )),
                );
                Ok((attrs, node))
            })())
        })
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
                            Err(FsError::CorruptedFilesystem {
                                message: format!(
                                    "Removed entry {name} from directory but didn't find its blob {blob_id:?} to remove"
                                ),
                            })
                        }
                        Err(err) => {
                            log::error!("Error removing blob: {err:?}");
                            Err(FsError::UnknownError)
                        }
                    }
                }
            },
        };

        blob.async_drop().await?;

        result
    }

    async fn create_child_symlink(
        &self,
        name: &PathComponent,
        target: &str,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<(NodeAttrs, CrySymlink<B>)> {
        // TODO What should NumBytes be? Also, no unwrap?
        let num_bytes = NumBytes::from(u64::try_from(name.len()).unwrap());

        let self_blob_id = self.node_info.blob_id(&self.blobstore).await?;

        let (blob, new_symlink_blob_id) = join!(
            self.load_blob(),
            self.create_symlink_blob(target, &self_blob_id),
        );
        let blob = blob?;
        // TODO Is this possible without to_owned()?
        let name = name.to_owned();
        with_async_drop(blob, move |blob| {
            future::ready((move || {
                let new_symlink_blob_id = new_symlink_blob_id?;

                let atime = SystemTime::now();
                let mtime = atime;

                let result = blob.add_entry_symlink(
                    name.clone(),
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

                let node = CrySymlink::new(
                    &self.blobstore,
                    Arc::new(NodeInfo::new(
                        self_blob_id,
                        name,
                        self.node_info.atime_update_behavior(),
                    )),
                );

                // TODO Deduplicate this with the logic that looks up getattr for symlink nodes and creates NodeAttrs from them there
                let attrs = NodeAttrs {
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
                };
                Ok((attrs, node))
            })())
        })
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
                            Err(FsError::CorruptedFilesystem {
                                message: format!(
                                    "Removed entry {name} from directory but didn't find its blob {blob_id:?} to remove"
                                ),
                            })
                        }
                        Err(err) => {
                            log::error!("Error removing blob: {err:?}");
                            Err(FsError::UnknownError)
                        }
                    }
                }
            },
        };

        blob.async_drop().await?;

        result
    }

    async fn create_and_open_file(
        &self,
        name: &PathComponent,
        mode: Mode,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<(
        NodeAttrs,
        AsyncDropGuard<CryNode<B>>,
        AsyncDropGuard<CryOpenFile<B>>,
    )> {
        let blob_id = self.node_info.blob_id(&self.blobstore).await?;
        let (blob, new_file_blob_id) = join!(self.load_blob(), self.create_file_blob(&blob_id),);
        let mut blob = blob?;

        with_async_drop_2!(blob, {
            let new_file_blob_id = new_file_blob_id?;

            let atime = SystemTime::now();
            let mtime = atime;

            blob.add_entry_file(
                name.to_owned(),
                new_file_blob_id,
                // TODO Don't convert between fs_types::xxx and cryfs_rustfs::xxx but reuse the same types
                fs_types::Mode::from(u32::from(mode)),
                fs_types::Uid::from(u32::from(uid)),
                fs_types::Gid::from(u32::from(gid)),
                atime,
                mtime,
            )
            .map_err(|err| {
                log::error!("Error adding dir entry: {err:?}");
                FsError::UnknownError
            })?;

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

            let node_info = Arc::new(NodeInfo::IsNotRootDir {
                parent_blob_id: blob_id,
                name: name.to_owned(),
                blob_details: OnceCell::new_with(Some(BlobDetails {
                    blob_id: new_file_blob_id,
                    blob_type: BlobType::File,
                })),
                atime_update_behavior: self.node_info.atime_update_behavior(),
            });

            let node =
                CryNode::new_internal(AsyncDropArc::clone(self.blobstore), Arc::clone(&node_info));
            let open_file = CryOpenFile::new(AsyncDropArc::clone(self.blobstore), node_info);

            Ok((attrs, node, open_file))
        })
    }
}
