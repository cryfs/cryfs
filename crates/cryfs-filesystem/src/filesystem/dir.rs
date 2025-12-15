use async_trait::async_trait;
use cryfs_rustfs::{NumBytes, OpenInFlags};
use futures::join;
use std::fmt::Debug;
use std::time::SystemTime;

use crate::filesystem::device::check_entry_overwrite_allowed;

use super::{
    device::CryDevice, node::CryNode, node_info::NodeInfo, open_file::CryOpenFile,
    symlink::CrySymlink,
};
use cryfs_blobstore::{BlobId, BlobStore, RemoveResult};
use cryfs_fsblobstore::concurrentfsblobstore::{ConcurrentFsBlob, ConcurrentFsBlobStore};
use cryfs_fsblobstore::fsblobstore::{AddOrOverwriteError, FlushBehavior, RenameError};
use cryfs_fsblobstore::fsblobstore::{BlobType, DirBlob, EntryType, FsBlob, MODE_NEW_SYMLINK};
use cryfs_fsblobstore::{Gid, Mode, Uid};
use cryfs_rustfs::{DirEntry, FsError, FsResult, NodeAttrs, NodeKind, object_based_api::Dir};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard, flatten_async_drop},
    path::PathComponent,
    with_async_drop_2,
};

#[derive(Debug)]
pub struct CryDir<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    // TODO Here and in others, can we just store &FsBlobStore instead of &AsyncDropGuard?
    blobstore: &'a AsyncDropGuard<AsyncDropArc<ConcurrentFsBlobStore<B>>>,
    node_info: AsyncDropGuard<AsyncDropArc<NodeInfo<B>>>,
}

impl<'a, B> CryDir<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    pub fn new(
        blobstore: &'a AsyncDropGuard<AsyncDropArc<ConcurrentFsBlobStore<B>>>,
        node_info: AsyncDropGuard<AsyncDropArc<NodeInfo<B>>>,
    ) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            blobstore,
            node_info,
        })
    }

    async fn load_blob(&self) -> Result<AsyncDropGuard<ConcurrentFsBlob<B>>, FsError> {
        self.node_info.load_blob(self.blobstore).await
    }

    fn blob_as_dir<'b>(blob: &'b FsBlob<B>) -> Result<&'b DirBlob<B>, FsError> {
        blob.as_dir().map_err(|err| {
            let blob_id = blob.blob_id();
            FsError::CorruptedFilesystem {
                // TODO Add to message what it actually is
                message: format!("Blob {:?} is listed as a directory in its parent directory but is actually not a directory: {err:?}", blob_id),
            }
        })
    }

    fn blob_as_dir_mut<'b>(blob: &'b mut FsBlob<B>) -> Result<&'b mut DirBlob<B>, FsError> {
        let blob_id = blob.blob_id();
        blob.as_dir_mut().map_err(|err| {
            FsError::CorruptedFilesystem {
                // TODO Add to message what it actually is
                message: format!("Blob {:?} is listed as a directory in its parent directory but is actually not a directory: {err:?}", blob_id),
            }
        })
    }

    async fn create_dir_blob(&self, parent: &BlobId) -> Result<BlobId, FsError> {
        let mut blob = self
            .blobstore
            .create_dir_blob(
                parent,
                // Make sure we flush this before the call site gets a chance to add this as an entry to its directory entry list.
                // This way, we make sure the filesystem stays consistent even if it crashes mid way.
                // Dropping by itself isn't enough to flush because it may go into a cache.
                FlushBehavior::FlushImmediately,
            )
            .await
            .map_err(|err| {
                log::error!("Error creating dir blob: {err:?}");
                FsError::UnknownError
            })?;

        let blob_id = blob.blob_id();
        blob.async_drop().await?;
        Ok(blob_id)
    }

    async fn create_file_blob(&self, parent: &BlobId) -> Result<BlobId, FsError> {
        let mut blob = self
            .blobstore
            .create_file_blob(
                parent,
                // Make sure we flush this before the call site gets a chance to add this as an entry to its directory entry list.
                // This way, we make sure the filesystem stays consistent even if it crashes mid way.
                // Dropping by itself isn't enough to flush because it may go into a cache.
                FlushBehavior::FlushImmediately,
            )
            .await
            .map_err(|err| {
                log::error!("Error creating file blob: {err:?}");
                FsError::UnknownError
            })?;

        let blob_id = blob.blob_id();
        blob.async_drop().await?;
        Ok(blob_id)
    }

    async fn create_symlink_blob(&self, target: &str, parent: &BlobId) -> Result<BlobId, FsError> {
        let mut blob = self
            .blobstore
            .create_symlink_blob(
                parent,
                target,
                // Make sure we flush this before the call site gets a chance to add this as an entry to its directory entry list.
                // This way, we make sure the filesystem stays consistent even if it crashes mid way.
                // Dropping by itself isn't enough to flush because it may go into a cache.
                FlushBehavior::FlushImmediately,
            )
            .await
            .map_err(|err| {
                log::error!("Error creating symlink blob: {err:?}");
                FsError::UnknownError
            })?;

        let blob_id = blob.blob_id();
        blob.async_drop().await?;
        Ok(blob_id)
    }

    async fn on_rename_overwrites_destination(
        &self,
        old_destination_blob_id: BlobId,
    ) -> FsResult<()> {
        let result = self
            .blobstore
            .remove_by_id(&old_destination_blob_id)
            .await?;
        match result {
            RemoveResult::SuccessfullyRemoved => Ok(()),
            RemoveResult::NotRemovedBecauseItDoesntExist => {
                log::error!("During rename->overwrite, tried to remove blob that doesn't exist");
                Err(FsError::UnknownError)
            }
        }
    }

    // TODO Add tests for this ancestor check
    #[cfg(feature = "ancestor_checks_on_move")]
    async fn validate_move_doesnt_cause_cycle(
        &self,
        child_to_move: &BlobId,
        newparent: &Self,
    ) -> FsResult<()> {
        let dest_ancestors = newparent.node_info.ancestors_and_self();

        // Check we're not moving a directory into itself or one of its own subdirectories
        // TODO Do we handle moving /path/to/file to /path/to/file/newname correctly? Or does it only work with /path/to/dir ?
        if dest_ancestors.ancestors_and_self().contains(&child_to_move) {
            Err(FsError::CannotMoveDirectoryIntoSubdirectoryOfItself)
        } else {
            Ok(())
        }
    }

    async fn remove_just_created_blob(&self, blob_id: BlobId) {
        // TODO This is used in functions like create_child_dir, create_child_symlink and create_and_open_file,
        //      to remove a blob that was created but then we failed to add it to its parent directory.
        //      It might be more performant if we don't even create it if we know it already exists.
        //      But then we can't do self.load_blob() and self.create_dir_blob() above concurrently anymore.
        //      Maybe this works in a world where self.load_blob() is already preloaded and stored in the CryDir object?
        if let Err(err) = self.blobstore.remove_by_id(&blob_id).await {
            log::error!("Error removing just created dir blob: {err:?}");
        }
    }

    async fn flush_dir_contents(&self) -> FsResult<()> {
        // Only flush the blob if it is loaded. If it isn't even loaded/cached, there's nothing we need to do.
        self.node_info.flush_if_cached(&self.blobstore).await
    }
}

#[async_trait]
impl<'a, B> Dir for CryDir<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    type Device = CryDevice<B>;

    fn into_node(this: AsyncDropGuard<Self>) -> AsyncDropGuard<CryNode<B>> {
        let this = this.unsafe_into_inner_dont_drop();
        CryNode::new_internal(AsyncDropArc::clone(&this.blobstore), this.node_info)
    }

    async fn lookup_child(&self, name: &PathComponent) -> FsResult<AsyncDropGuard<CryNode<B>>> {
        // TODO We shouldn't have to reload the self blob here, that's weird
        let mut self_blob = self.load_blob().await?;

        let blob_details = self_blob
            .with_lock(async |self_blob| {
                let self_dir = self_blob.as_dir().expect("Parent blob is not a directory");
                let entry = self_dir
                    .entry_by_name(name)
                    .ok_or_else(|| FsError::NodeDoesNotExist)?;
                let blob_id = *entry.blob_id();
                let blob_type = match entry.entry_type() {
                    EntryType::File => BlobType::File,
                    EntryType::Dir => BlobType::Dir,
                    EntryType::Symlink => BlobType::Symlink,
                };
                Ok((blob_id, blob_type))
            })
            .await;
        let (blob_id, blob_type) = match blob_details {
            Ok(blob_details) => blob_details,
            Err(err) => {
                self_blob.async_drop().await?;
                return Err(err);
            }
        };

        let node_info = NodeInfo::new_non_root_dir(
            self_blob,
            #[cfg(feature = "ancestor_checks_on_move")]
            self.node_info.ancestors_and_self().ancestors_and_self(),
            name.to_owned(),
            blob_id,
            blob_type,
            self.node_info.atime_update_behavior(),
        );
        Ok(CryNode::new(
            AsyncDropArc::clone(&self.blobstore),
            node_info,
        ))
    }

    async fn rename_child(&self, oldname: &PathComponent, newname: &PathComponent) -> FsResult<()> {
        self.node_info
            .concurrently_update_modification_timestamp_in_parent(async || {
                let blob = self.load_blob().await?;
                with_async_drop_2!(blob, {
                    blob.with_lock(async |blob| {
                        Self::blob_as_dir_mut(&mut *blob)?
                            .rename_entry_by_name(
                                oldname,
                                newname.to_owned(),
                                async |source_blob_type,
                                       overwritten_blob_type,
                                       overwritten_blobid| {
                                    check_entry_overwrite_allowed(
                                        &self.blobstore,
                                        source_blob_type,
                                        overwritten_blob_type,
                                        overwritten_blobid,
                                    )
                                    .await?;
                                    self.on_rename_overwrites_destination(*overwritten_blobid)
                                        .await
                                },
                            )
                            .await
                            .map_err(|err| match err {
                                RenameError::NodeDoesNotExist => FsError::NodeDoesNotExist,
                                RenameError::OnOverwriteError(e) => e,
                            })
                    })
                    .await
                })
            })
            .await
    }

    async fn move_child_to(
        &self,
        oldname: &PathComponent,
        newparent: AsyncDropGuard<Self>,
        newname: &PathComponent,
    ) -> FsResult<()> {
        // TODO We're currently locking, releasing and re-locking blobs multiple times. This introduces race conditions and is not optimal for performance either.
        //      We should just lock each blob once and keep it locked until we're done. But we need to do it in a deadlock-free way, locking multiple
        //      blobs at once is risky for deadlocks if not done in a consistent order.

        // TODO Improve concurrency in this function
        with_async_drop_2!(newparent, {
            let (source_parent, dest_parent) = join!(self.load_blob(), newparent.load_blob());
            let (source_parent, dest_parent) =
                flatten_async_drop::<FsError, _, _, _, _>(source_parent, dest_parent).await?;
            // TODO Drop source_parent, dest_parent, newparent and self_blob concurrently
            with_async_drop_2!(source_parent, {
                with_async_drop_2!(dest_parent, {
                    let entry = source_parent
                        .with_lock(async |source_parent_dir| {
                            let source_parent_dir = Self::blob_as_dir_mut(&mut *source_parent_dir)?;
                            let entry = source_parent_dir
                                .entry_by_name(oldname)
                                .ok_or(FsError::NodeDoesNotExist)?;
                            Ok::<_, FsError>(entry.clone()) // TODO No clone
                        })
                        .await?;

                    let self_blob_id = entry.blob_id();
                    #[cfg(feature = "ancestor_checks_on_move")]
                    {
                        // TODO This can happen concurrently with the load_blob above
                        self.validate_move_doesnt_cause_cycle(self_blob_id, &newparent)
                            .await?;
                    }

                    // TODO In theory, we could load self_blob concurrently with dest_parent_blob. No need to only do it after dest_parent_blob loaded.
                    //      But it likely has some dependency with source_parent_blob.
                    let self_blob = self
                        .blobstore
                        .load(self_blob_id)
                        .await
                        .map_err(|err| {
                            log::error!("Error loading blob: {:?}", err);
                            FsError::UnknownError
                        })?
                        .ok_or(
                            // TODO This branch means there was an entry in the parent dir but the blob itself doesn't exist. How should we handle this?
                            FsError::NodeDoesNotExist,
                        )?;
                    with_async_drop_2!(self_blob, {
                        let entry = source_parent
                            .with_lock(async |source_parent_dir| {
                                Self::blob_as_dir_mut(source_parent_dir)?
                                    .remove_entry_by_name(oldname)
                            })
                            .await
                            .map_err(|err| {
                                log::error!("Error in remove_entry_by_name: {err:?}");
                                FsError::UnknownError
                            })?;
                        dest_parent
                            .with_lock(async |dest_parent_dir| {
                                Self::blob_as_dir_mut(dest_parent_dir)?
                                    .add_or_overwrite_entry(
                                        newname.to_owned(),
                                        *entry.blob_id(),
                                        entry.entry_type(),
                                        entry.mode(),
                                        entry.uid(),
                                        entry.gid(),
                                        entry.last_access_time(),
                                        entry.last_modification_time(),
                                        async |source_blob_type, overwritten_blob_type, overwritten_blobid| {
                                            check_entry_overwrite_allowed(
                                                &self.blobstore,
                                                source_blob_type,
                                                overwritten_blob_type,
                                                overwritten_blobid,
                                            )
                                            .await?;
                                            self.on_rename_overwrites_destination(
                                                *overwritten_blobid,
                                            )
                                            .await
                                        },
                                    )
                                    .await
                                    .map_err(|err| {
                                        // TODO Exception safety - we couldn't add the entry to the destination, but we already removed it from the source. We should probably re-add it to the source.
                                        match err {
                                            AddOrOverwriteError::ValidationFailed(fs_err) => {
                                                log::error!("Error in add_or_overwrite_entry validation: {fs_err:?}");
                                                FsError::internal_error(fs_err.into()) // This shouldn't happen because we are moving an already validated entry
                                            }
                                            AddOrOverwriteError::OnOverwriteError(err) => {
                                                log::error!("Error in add_or_overwrite_entry on_overwritten: {err:?}");
                                                err
                                            }
                                        }
                                    })
                            }).await?;

                        self_blob
                            .with_lock(async |self_blob| {
                                self_blob.set_parent(&dest_parent.blob_id()).await
                            })
                            .await
                            .map_err(|err| {
                                // TODO Exception safety - we already changed parent dir entries but couldn't update the parent pointer. We should probably try to undo the parent dir entry changes.
                                log::error!("Error setting parent: {err:?}");
                                FsError::UnknownError
                            })?;

                        // TODO We can probably do this concurrently with the other modifications further up
                        // TODO This requires loading the grandparent blobs so we can update the parent blob's timestamps.
                        //      Can this cause a deadlock? What if one of the grandparents is already loaded as one of the parents?
                        let (source_update, dest_update) = join!(
                            self.node_info.update_modification_timestamp_in_parent(),
                            newparent
                                .node_info
                                .update_modification_timestamp_in_parent(),
                        );

                        source_update?;
                        dest_update?;

                        Ok::<(), FsError>(())
                    })
                })
            })?;

            Ok(())
        })
    }

    async fn entries(&self) -> FsResult<Vec<DirEntry>> {
        // TODO Can we return an iterator instead of a Vec from here? But it'll need state with an async destructor. Probably needs async generators.
        self.node_info
            .concurrently_maybe_update_access_timestamp_in_parent(async || {
                let blob = self.load_blob().await?;
                with_async_drop_2!(blob, {
                    blob.with_lock(async |blob| {
                        let blob = Self::blob_as_dir(blob)?;
                        let result = blob
                            .entries()
                            .map(|entry| cryfs_rustfs::DirEntry {
                                name: entry.name().to_owned(),
                                kind: match entry.entry_type() {
                                    EntryType::Dir => NodeKind::Dir,
                                    EntryType::File => NodeKind::File,
                                    EntryType::Symlink => NodeKind::Symlink,
                                },
                            })
                            .collect();
                        Ok(result)
                    })
                    .await
                })
            })
            .await
    }

    async fn create_child_dir(
        &self,
        name: &PathComponent,
        mode: cryfs_rustfs::Mode,
        uid: cryfs_rustfs::Uid,
        gid: cryfs_rustfs::Gid,
    ) -> FsResult<(NodeAttrs, AsyncDropGuard<CryDir<'_, B>>)> {
        self.node_info
            .concurrently_update_modification_timestamp_in_parent(async || {
                let self_blob_id = self.node_info.blob_id();
                let (blob, new_dir_blob_id) =
                    join!(self.load_blob(), self.create_dir_blob(&self_blob_id));
                let mut blob = match blob {
                    Ok(blob) => blob,
                    Err(err) => {
                        log::error!("Error loading blob: {err:?}");
                        if let Ok(new_dir_blob_id) = new_dir_blob_id {
                            self.remove_just_created_blob(new_dir_blob_id).await;
                        }
                        return Err(err);
                    }
                };
                // TODO Is this possible without to_owned()?
                let name = name.to_owned();
                let new_dir_blob_id = match new_dir_blob_id {
                    Ok(new_dir_blob_id) => new_dir_blob_id,
                    Err(err) => {
                        blob.async_drop().await?;
                        return Err(err);
                    }
                };

                let atime = SystemTime::now();
                let mtime = atime;

                let attrs: FsResult<NodeAttrs> = blob
                    .with_lock(async |blob| {
                        let blob = Self::blob_as_dir_mut(&mut *blob)?;

                        blob.add_entry_dir(
                            name.clone(),
                            new_dir_blob_id,
                            // TODO Don't convert between fs_types::xxx and cryfs_rustfs::xxx but reuse the same types
                            Mode::from(u32::from(mode)),
                            Uid::from(u32::from(uid)),
                            Gid::from(u32::from(gid)),
                            atime,
                            mtime,
                        )?;

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
                    })
                    .await;
                let attrs = match attrs {
                    Ok(attrs) => attrs,
                    Err(err) => {
                        log::error!("Error adding dir entry: {err:?}");
                        self.remove_just_created_blob(new_dir_blob_id).await;
                        blob.async_drop().await?;
                        return Err(FsError::UnknownError);
                    }
                };
                let node = CryDir::new(
                    &self.blobstore,
                    AsyncDropArc::new(NodeInfo::new_non_root_dir(
                        blob,
                        #[cfg(feature = "ancestor_checks_on_move")]
                        self.node_info.ancestors_and_self().ancestors_and_self(),
                        name,
                        new_dir_blob_id,
                        BlobType::Dir,
                        self.node_info.atime_update_behavior(),
                    )),
                );
                Ok((attrs, node))
            })
            .await
    }

    async fn remove_child_dir(&self, name: &PathComponent) -> FsResult<()> {
        self.node_info
            .concurrently_update_modification_timestamp_in_parent( async || {
                let self_blob = self.load_blob().await?;
                with_async_drop_2!(self_blob, {
                    let child_id = self_blob
                        .with_lock(async |self_blob| {
                            let self_blob = Self::blob_as_dir(&*self_blob)?;
                            let child_entry = self_blob
                                .entry_by_name(name)
                                .ok_or_else(|| FsError::NodeDoesNotExist)?;
                            if child_entry.entry_type() != EntryType::Dir {
                                Err(FsError::NodeIsNotADirectory)?;
                            }
                            Ok::<_, FsError>(*child_entry.blob_id())
                        })
                        .await?;

                    let mut child_blob = self.blobstore.load(&child_id).await.map_err(|_| FsError::NodeDoesNotExist)?.ok_or_else(|| FsError::NodeDoesNotExist)?;

                    let entries_check = child_blob.with_lock(async |child_blob| {
                        let child_blob_dir = Self::blob_as_dir(&child_blob).map_err(|err| {
                            FsError::CorruptedFilesystem {
                                // TODO Add to message what it actually is
                                message: format!("Blob {:?} is listed as a directory in its parent directory but is actually not a directory: {err:?}", child_id),
                            }
                        })?;
                        if child_blob_dir.entries().len() > 0 {
                            return Err(FsError::CannotRemoveNonEmptyDirectory)
                        }
                        Ok(())
                    }).await;
                    if let Err(err) = entries_check {
                        child_blob.async_drop().await?;
                        return Err(err);
                    }

                    // TODO We released the lock on self_blob above and are now re-locking it. There is a race condition here.

                    // First remove the entry, then flush that change, and only then remove the blob.
                    // This is to make sure the file system doesn't end up in an invalid state
                    // where the blob is removed but the entry is still there.

                    let removed_entry = self_blob.with_lock(async |self_blob| {
                        let self_blob = Self::blob_as_dir_mut(&mut *self_blob)?;
                        let entry = self_blob.remove_entry_by_name(name)?;
                        match self_blob.flush().await {
                            Err(err) => {
                                log::error!("Error flushing blob: {err:?}");
                                Err(FsError::UnknownError)
                            }
                            Ok(()) => {
                                Ok(entry)
                            }
                        }
                    }).await;
                    let removed_entry = match removed_entry {
                        Ok(removed_entry) => removed_entry,
                        Err(err) => {
                            child_blob.async_drop().await?;
                            return Err(err);
                        }
                    };
                    assert_eq!(*removed_entry.blob_id(), child_blob.blob_id());

                    let remove_result = ConcurrentFsBlob::remove(child_blob).await;
                    match remove_result {
                        Ok(RemoveResult::SuccessfullyRemoved) => Ok(()),
                        Ok(RemoveResult::NotRemovedBecauseItDoesntExist) => {
                            Err(FsError::CorruptedFilesystem {
                                message: format!(
                                    "Removed entry {name} from directory but didn't find its blob {child_id:?} to remove"
                                ),
                            })
                        }
                        Err(err) => {
                            log::error!("Error removing blob: {err:?}");
                            Err(FsError::UnknownError)
                        }
                    }
                })
            })
            .await
    }

    async fn create_child_symlink(
        &self,
        name: &PathComponent,
        target: &str,
        uid: cryfs_rustfs::Uid,
        gid: cryfs_rustfs::Gid,
    ) -> FsResult<(NodeAttrs, AsyncDropGuard<CrySymlink<B>>)> {
        self.node_info
            .concurrently_update_modification_timestamp_in_parent(async || {
                // TODO What should NumBytes be? Also, no unwrap?
                let num_bytes = NumBytes::from(u64::try_from(name.len()).unwrap());

                let self_blob_id = self.node_info.blob_id();

                let (blob, new_symlink_blob_id) = join!(
                    self.load_blob(),
                    self.create_symlink_blob(target, self_blob_id),
                );
                let mut blob = match blob {
                    Ok(blob) => blob,
                    Err(err) => {
                        log::error!("Error loading blob: {err:?}");
                        if let Ok(new_symlink_blob_id) = new_symlink_blob_id {
                            self.remove_just_created_blob(new_symlink_blob_id).await;
                        }
                        return Err(err);
                    }
                };
                // TODO Is this possible without to_owned()?
                let name = name.to_owned();
                let new_symlink_blob_id = match new_symlink_blob_id {
                    Ok(id) => id,
                    Err(err) => {
                        log::error!("Error creating symlink blob: {err:?}");
                        blob.async_drop().await?;
                        return Err(err);
                    }
                };

                let atime = SystemTime::now();
                let mtime = atime;

                let attrs: FsResult<NodeAttrs> = blob
                    .with_lock(async |blob| {
                        let blob = Self::blob_as_dir_mut(&mut *blob)?;

                        blob.add_entry_symlink(
                            name.clone(),
                            new_symlink_blob_id,
                            // TODO Don't convert between fs_types::xxx and cryfs_rustfs::xxx but reuse the same types
                            Uid::from(u32::from(uid)),
                            Gid::from(u32::from(gid)),
                            atime,
                            mtime,
                        )?;

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
                    })
                    .await;
                let attrs = match attrs {
                    Ok(attrs) => attrs,
                    Err(err) => {
                        log::error!("Error adding dir entry: {err:?}");
                        self.remove_just_created_blob(new_symlink_blob_id).await;
                        blob.async_drop().await?;
                        return Err(FsError::UnknownError);
                    }
                };
                let node = CrySymlink::new(
                    &self.blobstore,
                    AsyncDropArc::new(NodeInfo::new_non_root_dir(
                        blob,
                        #[cfg(feature = "ancestor_checks_on_move")]
                        self.node_info.ancestors_and_self().ancestors_and_self(),
                        name,
                        new_symlink_blob_id,
                        BlobType::Symlink,
                        self.node_info.atime_update_behavior(),
                    )),
                );
                Ok((attrs, node))
            })
            .await
    }

    async fn remove_child_file_or_symlink(&self, name: &PathComponent) -> FsResult<()> {
        self.node_info.concurrently_update_modification_timestamp_in_parent( async || {
            let blob = self.load_blob().await?;

            with_async_drop_2!(blob, {
                let removed = blob.with_lock(async |blob| {
                    let blob = Self::blob_as_dir_mut(&mut *blob)?;
                    // First remove the entry, then flush that change, and only then remove the blob.
                    // This is to make sure the file system doesn't end up in an invalid state
                    // where the blob is removed but the entry is still there.
                    let removed = blob.remove_entry_by_name(name)?;
                    blob.flush().await.map_err(|err| {
                        log::error!("Error flushing blob: {err:?}");
                        FsError::UnknownError
                    })?;
                    Ok::<_, FsError>(removed)
                }).await?;

                let blob_id = removed.blob_id();
                match removed.entry_type() {
                    EntryType::Dir => {
                        Err(FsError::NodeIsADirectory)
                    }
                    EntryType::File | EntryType::Symlink => {
                        let remove_result = self.blobstore.remove_by_id(blob_id).await?;
                        match remove_result {
                            RemoveResult::SuccessfullyRemoved => Ok(()),
                            RemoveResult::NotRemovedBecauseItDoesntExist => {
                                Err(FsError::CorruptedFilesystem {
                                    message: format!(
                                        "Removed entry {name} from directory but didn't find its blob {blob_id:?} to remove"
                                    ),
                                })
                            }
                        }
                    }
                }
            })
        }).await
    }

    async fn create_and_open_file(
        &self,
        name: &PathComponent,
        mode: cryfs_rustfs::Mode,
        uid: cryfs_rustfs::Uid,
        gid: cryfs_rustfs::Gid,
        _flags: OpenInFlags, // TODO Use flags
    ) -> FsResult<(
        NodeAttrs,
        AsyncDropGuard<CryNode<B>>,
        AsyncDropGuard<CryOpenFile<B>>,
    )> {
        self.node_info
            .concurrently_update_modification_timestamp_in_parent(async || {
                let self_blob_id = self.node_info.blob_id();
                let (blob, new_file_blob_id) =
                    join!(self.load_blob(), self.create_file_blob(&self_blob_id),);
                let mut blob = match blob {
                    Ok(blob) => blob,
                    Err(err) => {
                        log::error!("Error loading blob: {err:?}");
                        if let Ok(new_file_blob_id) = new_file_blob_id {
                            self.remove_just_created_blob(new_file_blob_id).await;
                        }
                        return Err(err);
                    }
                };

                let new_file_blob_id = match new_file_blob_id {
                    Ok(id) => id,
                    Err(err) => {
                        blob.async_drop().await?;
                        return Err(err);
                    }
                };

                let atime = SystemTime::now();
                let mtime = atime;

                let attrs: FsResult<NodeAttrs> = blob
                    .with_lock(async |blob| {
                        let blob = Self::blob_as_dir_mut(&mut *blob)?;

                        blob.add_entry_file(
                            name.to_owned(),
                            new_file_blob_id,
                            // TODO Don't convert between fs_types::xxx and cryfs_rustfs::xxx but reuse the same types
                            Mode::from(u32::from(mode)),
                            Uid::from(u32::from(uid)),
                            Gid::from(u32::from(gid)),
                            atime,
                            mtime,
                        )?;

                        // TODO Deduplicate this with the logic that looks up getattr for symlink nodes and creates NodeAttrs from them there
                        Ok(NodeAttrs {
                            nlink: 1,
                            mode,
                            uid,
                            gid,
                            num_bytes: NumBytes::from(0),
                            num_blocks: None,
                            atime,
                            mtime,
                            ctime: mtime,
                        })
                    })
                    .await;
                let attrs = match attrs {
                    Ok(attrs) => attrs,
                    Err(err) => {
                        log::error!("Error adding dir entry: {err:?}");
                        self.remove_just_created_blob(new_file_blob_id).await;
                        blob.async_drop().await?;
                        return Err(FsError::UnknownError);
                    }
                };
                let node_info = AsyncDropArc::new(NodeInfo::new_non_root_dir(
                    blob,
                    #[cfg(feature = "ancestor_checks_on_move")]
                    self.node_info.ancestors_and_self().ancestors_and_self(),
                    name.to_owned(),
                    new_file_blob_id,
                    BlobType::File,
                    self.node_info.atime_update_behavior(),
                ));

                let node = CryNode::new_internal(
                    AsyncDropArc::clone(self.blobstore),
                    AsyncDropArc::clone(&node_info),
                );
                let open_file = CryOpenFile::new(AsyncDropArc::clone(self.blobstore), node_info);

                Ok((attrs, node, open_file))
            })
            .await
    }

    async fn fsync(&self, datasync: bool) -> FsResult<()> {
        if datasync {
            self.flush_dir_contents().await?;
        } else {
            let (r1, r2) = join!(self.flush_dir_contents(), self.node_info.flush_metadata());
            // TODO Report both errors if both happen
            r1?;
            r2?;
        }
        Ok(())
    }
}

#[async_trait]
impl<'a, B> AsyncDrop for CryDir<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    type Error = FsError;
    async fn async_drop_impl(&mut self) -> Result<(), FsError> {
        self.node_info.async_drop().await
    }
}
