use anyhow::{Context, Result};
use async_trait::async_trait;
use cryfs_blockstore::RemoveResult;
use futures::join;
use maybe_owned::MaybeOwned;
use std::sync::Arc;
use std::{convert::Infallible, fmt::Debug};

use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_rustfs::{
    object_based_api::Device, AbsolutePath, FsError, FsResult, PathComponent, Statfs,
};
use cryfs_utils::{
    async_drop::{flatten_async_drop, AsyncDrop, AsyncDropArc, AsyncDropGuard},
    safe_panic, with_async_drop_2,
};

use super::{
    dir::CryDir, file::CryFile, node::CryNode, node_info::NodeInfo, open_file::CryOpenFile,
    symlink::CrySymlink,
};
use crate::filesystem::fsblobstore::{BlobType, DirBlob, EntryType, FsBlob, FsBlobStore};

pub struct CryDevice<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
{
    blobstore: AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
    root_blob_id: BlobId,
}

impl<B> CryDevice<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
{
    pub fn load_filesystem(blobstore: AsyncDropGuard<B>, root_blob_id: BlobId) -> Self {
        Self {
            blobstore: AsyncDropArc::new(FsBlobStore::new(blobstore)),
            root_blob_id,
        }
    }

    pub async fn create_new_filesystem(
        blobstore: AsyncDropGuard<B>,
        root_blob_id: BlobId,
    ) -> Result<Self> {
        let mut fsblobstore = FsBlobStore::new(blobstore);
        match fsblobstore.create_root_dir_blob(&root_blob_id).await {
            Ok(()) => Ok(Self {
                blobstore: AsyncDropArc::new(fsblobstore),
                root_blob_id,
            }),
            Err(err) => {
                fsblobstore.async_drop().await?;
                Err(err)
            }
        }
    }
}

impl<B> CryDevice<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
{
    async fn load_blob(
        &self,
        path: impl IntoIterator<Item = &PathComponent>,
    ) -> FsResult<AsyncDropGuard<FsBlob<'_, B>>> {
        let mut root_blob = self
            .blobstore
            .load(&self.root_blob_id)
            .await
            .map_err(|err| {
                log::error!("Failed to load root blob: {err:?}");
                FsError::Custom {
                    error_code: libc::EIO,
                }
            })?
            .ok_or_else(|| {
                log::error!("Root blob not found");
                FsError::Custom {
                    error_code: libc::EIO,
                }
            })?;
        if root_blob.blob_type() != BlobType::Dir {
            log::error!("Root blob is not a directory");
            root_blob.async_drop().await?;
            return Err(FsError::Custom {
                error_code: libc::EIO,
            });
        }
        self.load_blob_from_relative_path_owned(root_blob, path.into_iter())
            .await
    }

    async fn load_blob_from_relative_path_owned<'a, 'b>(
        &'a self,
        anchor: AsyncDropGuard<FsBlob<'a, B>>,
        relative_path: impl Iterator<Item = &PathComponent>,
    ) -> FsResult<AsyncDropGuard<FsBlob<'a, B>>> {
        match self.load_blob_from_relative_path(MaybeOwned::Owned(anchor), relative_path).await? {
            MaybeOwned::Owned(blob) => Ok(blob),
            MaybeOwned::Borrowed(blob) => panic!("Since we called `load_blob_from_relative_path` with an owned anchor, it should never return a borrowed blob"),
        }
    }

    // If `anchor` is borrowed and `relative_path` is empty, the returned blob will be borrowed.
    // If `anchor` is owned or `relative_path` is non-empty, the returned blob will be owned.
    async fn load_blob_from_relative_path<'a, 'b>(
        &'a self,
        anchor: MaybeOwned<'b, AsyncDropGuard<FsBlob<'a, B>>>,
        relative_path: impl Iterator<Item = &PathComponent>,
    ) -> FsResult<MaybeOwned<'b, AsyncDropGuard<FsBlob<'a, B>>>> {
        let mut current_blob = anchor;

        for path_component in relative_path {
            let dir_blob = match current_blob.as_dir() {
                Ok(dir_blob) => Ok(dir_blob),
                Err(err) => {
                    if let Some(current_blob) = current_blob.as_mut() {
                        current_blob.async_drop().await?;
                    } else {
                        // current_blob is borrowed. No need to drop it
                    }
                    // TODO This error mapping is weird. Probably better to have as_dir return the right error type.
                    Err(FsError::NodeIsNotADirectory)
                }
            }?;
            let entry = match dir_blob.entry_by_name(path_component) {
                Some(entry) => {
                    let blob_id = *entry.blob_id();
                    Ok(blob_id)
                }
                None => Err(FsError::NodeDoesNotExist),
            };
            if let Some(current_blob) = current_blob.as_mut() {
                current_blob.async_drop().await?;
            } else {
                // current_blob is borrowed. No need to drop it
            }
            let entry = entry?;
            current_blob = MaybeOwned::from(
                self.blobstore
                    .load(&entry)
                    .await
                    .map_err(|err| {
                        log::error!("Failed to load blob: {err:?}");
                        FsError::Custom {
                            error_code: libc::EIO,
                        }
                    })?
                    .ok_or_else(|| {
                        log::error!("Blob not found");
                        FsError::Custom {
                            error_code: libc::EIO,
                        }
                    })?,
            );
        }
        Ok(current_blob)
    }

    async fn load_two_blobs<'a>(
        &'a self,
        path1: &AbsolutePath,
        path2: &AbsolutePath,
    ) -> FsResult<LoadTwoBlobsResult<'a, B>> {
        let num_shared_path_components = path1
            .iter()
            .zip(path2.iter())
            .take_while(|(a, b)| a == b)
            .count();
        let shared_path = path1.iter().take(num_shared_path_components);
        let relative_path1 = path1.iter().skip(num_shared_path_components);
        let relative_path2 = path2.iter().skip(num_shared_path_components);
        let mut shared_blob = self.load_blob(shared_path).await?;

        let relative_path1_len = relative_path1.len();
        let relative_path2_len = relative_path2.len();

        let (blob1, blob2) = join!(
            self.load_blob_from_relative_path(MaybeOwned::Borrowed(&shared_blob), relative_path1),
            self.load_blob_from_relative_path(MaybeOwned::Borrowed(&shared_blob), relative_path2)
        );
        match (blob1, blob2) {
            (Err(err1), Err(err2)) => {
                shared_blob.async_drop().await?;
                // TODO Report both errors
                Err(err1)
            }
            (Err(err1), Ok(mut blob2)) => {
                // TODO async_drop blob2 and shared_blob concurrently
                if let Some(blob2) = blob2.as_mut() {
                    blob2.async_drop().await?;
                } else {
                    // blob2 is borrowed. No need to drop it
                }
                shared_blob.async_drop().await?;
                Err(err1)
            }
            (Ok(mut blob1), Err(err2)) => {
                // TODO async_drop blob1 and shared_blob concurrently
                if let Some(blob1) = blob1.as_mut() {
                    blob1.async_drop().await?;
                } else {
                    // blob1 is borrowed. No need to drop it
                }
                shared_blob.async_drop().await?;
                Err(err2)
            }
            (Ok(MaybeOwned::Borrowed(blob1)), Ok(MaybeOwned::Borrowed(blob2))) => {
                // Both blobs are borrowed, this means that both relative paths were empty
                assert_eq!(0, relative_path1_len);
                assert_eq!(0, relative_path2_len);
                Ok(LoadTwoBlobsResult::AreSameBlob { blob: shared_blob })
            }
            (Ok(MaybeOwned::Owned(blob1)), Ok(MaybeOwned::Owned(blob2))) => {
                // Both blobs are owned, this means that neither relative path was empty
                assert!(relative_path1_len > 0);
                assert!(relative_path2_len > 0);
                shared_blob.async_drop().await?;
                Ok(LoadTwoBlobsResult::AreDifferentBlobs { blob1, blob2 })
            }
            (Ok(MaybeOwned::Borrowed(blob1)), Ok(MaybeOwned::Owned(blob2))) => {
                // blob1 is borrowed, blob2 is owned, this means that relative_path1 is empty and relative_path2 is not
                assert_eq!(0, relative_path1_len);
                assert!(relative_path2_len > 0);
                Ok(LoadTwoBlobsResult::AreDifferentBlobs {
                    blob1: shared_blob,
                    blob2,
                })
            }
            (Ok(MaybeOwned::Owned(blob1)), Ok(MaybeOwned::Borrowed(blob2))) => {
                // blob1 is owned, blob2 is borrowed, this means that relative_path1 is not empty and relative_path2 is empty
                assert!(relative_path1_len > 0);
                assert_eq!(0, relative_path2_len);
                Ok(LoadTwoBlobsResult::AreDifferentBlobs {
                    blob1,
                    blob2: shared_blob,
                })
            }
        }
    }
}

#[async_trait]
impl<B> Device for CryDevice<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
{
    type Node = CryNode<B>;
    type Dir<'a> = CryDir<'a, B>;
    type Symlink<'a> = CrySymlink<'a, B>;
    type File<'a> = CryFile<'a, B>;
    type OpenFile = CryOpenFile<B>;

    async fn rootdir(&self) -> FsResult<Self::Dir<'_>> {
        let node_info = NodeInfo::new_rootdir(self.root_blob_id);
        Ok(CryDir::new(&self.blobstore, Arc::new(node_info)))
    }

    async fn rename(&self, source_path: &AbsolutePath, dest_path: &AbsolutePath) -> FsResult<()> {
        if source_path.is_ancestor_of(dest_path) {
            log::error!(
                "Tried to rename {source_path} into its descendant {dest_path}",
                source_path = source_path,
                dest_path = dest_path
            );
            return Err(FsError::InvalidOperation);
        }
        if dest_path.is_ancestor_of(source_path) {
            log::error!(
                "Tried to rename {source_path} into its ancestor {dest_path}",
                source_path = source_path,
                dest_path = dest_path
            );
            return Err(FsError::InvalidOperation);
        }

        let Some((source_parent, source_name)) = source_path.split_last() else {
            log::error!("Tried to rename the root directory {source_path} into {dest_path}");
            return Err(FsError::InvalidOperation);
        };
        let Some((dest_parent, dest_name)) = dest_path.split_last() else {
            log::error!("Tried to rename {source_path} to the root directory {dest_path}");
            return Err(FsError::InvalidOperation);
        };

        let on_overwritten = |blobid: &BlobId| {
            let blobid = *blobid;
            async move {
                let r = self.blobstore.remove_by_id(&blobid).await.map_err(|err| {
                    log::error!("Error removing blob: {:?}", err);
                    FsError::UnknownError
                });
                match r {
                    Ok(RemoveResult::SuccessfullyRemoved) => Ok(()),
                    Ok(RemoveResult::NotRemovedBecauseItDoesntExist) => {
                        log::error!(
                            "During rename->overwrite, tried to remove blob that doesn't exist"
                        );
                        Err(FsError::UnknownError)
                    }
                    Err(err) => {
                        log::error!("Error removing blob: {:?}", err);
                        Err(FsError::UnknownError)
                    }
                }
            }
        };

        let parents = self.load_two_blobs(source_parent, dest_parent).await?;
        match parents {
            LoadTwoBlobsResult::AreSameBlob { blob } => {
                let mut parent = FsBlob::into_dir(blob)
                    .await
                    .map_err(|_| FsError::NodeIsNotADirectory)?;
                // TODO Is overwriting allowed here if the new entry already exists?
                parent
                    .rename_entry_by_name(source_name, dest_name.to_owned(), on_overwritten)
                    .await?;
                parent.async_drop().await?;
                Ok(())
            }
            LoadTwoBlobsResult::AreDifferentBlobs {
                blob1: source_parent_blob,
                blob2: dest_parent_blob,
            } => {
                let (source_parent, dest_parent) = join!(
                    FsBlob::into_dir(source_parent_blob),
                    FsBlob::into_dir(dest_parent_blob),
                );
                let source_parent = source_parent.map_err(|err| {
                    // TODO No map_err but instead have into_dir return the right error
                    FsError::NodeIsNotADirectory
                });
                let dest_parent = dest_parent.map_err(|err| {
                    // TODO No map_err but instead have into_dir return the right error
                    FsError::NodeIsNotADirectory
                });
                // TODO Use with_async_drop! for source_parent, dest_parent, self_blob
                let (mut source_parent, mut dest_parent) =
                    // TODO Use flatten_async_drop instead of flatten_async_drop_err_map
                    flatten_async_drop(source_parent, dest_parent).await?;
                let entry = match source_parent.entry_by_name(source_name) {
                    Some(entry) => entry,
                    None => {
                        // TODO Drop concurrently
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
                        // TODO Drop concurrently
                        source_parent.async_drop().await?;
                        dest_parent.async_drop().await?;
                        return Err(FsError::NodeDoesNotExist);
                    }
                    Err(err) => {
                        // TODO Drop concurrently
                        source_parent.async_drop().await?;
                        dest_parent.async_drop().await?;
                        log::error!("Error loading blob: {:?}", err);
                        return Err(FsError::UnknownError);
                    }
                };

                let mut existing_dir_check = || {
                    if let Some(existing_dest_entry) = dest_parent.entry_by_name_mut(dest_name) {
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
                        // TODO Drop concurrently
                        source_parent.async_drop().await?;
                        dest_parent.async_drop().await?;
                        return Err(err);
                    }
                }

                let res = source_parent.remove_entry_by_name(source_name);
                let entry = match res {
                    Ok(entry) => entry,
                    Err(err) => {
                        // TODO Drop concurrently
                        source_parent.async_drop().await?;
                        dest_parent.async_drop().await?;
                        log::error!("Error in add_or_overwrite_entry: {err:?}");
                        return Err(FsError::UnknownError);
                    }
                };
                let res = dest_parent
                    .add_or_overwrite_entry(
                        dest_name.to_owned(),
                        *entry.blob_id(),
                        entry.entry_type(),
                        entry.mode(),
                        entry.uid(),
                        entry.gid(),
                        entry.last_access_time(),
                        entry.last_modification_time(),
                        on_overwritten,
                    )
                    .await;
                match res {
                    Ok(()) => (),
                    Err(err) => {
                        // TODO Drop concurrently
                        source_parent.async_drop().await?;
                        dest_parent.async_drop().await?;
                        log::error!("Error in add_or_overwrite_entry: {err:?}");
                        return Err(FsError::UnknownError);
                    }
                }

                let res = self_blob.set_parent(&dest_parent.blob_id()).await;
                match res {
                    Ok(()) => (),
                    Err(err) => {
                        // TODO Drop concurrently
                        source_parent.async_drop().await?;
                        dest_parent.async_drop().await?;
                        log::error!("Error setting parent: {err:?}");
                        return Err(FsError::UnknownError);
                    }
                }
                // TODO async_drop concurrently
                self_blob.async_drop().await?;
                source_parent.async_drop().await?;
                dest_parent.async_drop().await?;
                Ok(())

                // TODO We need to update timestamps of the parent directories in the grandparent blobs.
            }
        }
    }

    async fn statfs(&self) -> FsResult<Statfs> {
        let num_used_blocks = self.blobstore.num_blocks().await.map_err(|err| {
            log::error!("Failed to get num_blocks: {err:?}");
            FsError::UnknownError
        })?;
        let num_free_blocks = self
            .blobstore
            .estimate_space_for_num_blocks_left()
            .map_err(|err| {
                log::error!("Failed to get num_free_blocks: {err:?}");
                FsError::UnknownError
            })?;
        let num_total_blocks = num_used_blocks + num_free_blocks;
        let max_filename_length = 255; // We theoretically support unlimited file name length, but this is default for many Linux file systems, so probably also makes sense for CryFS.
        let blocksize = self.blobstore.virtual_block_size_bytes();

        Ok(Statfs {
            max_filename_length,
            blocksize,
            num_total_blocks,
            num_free_blocks,
            num_available_blocks: num_free_blocks,
            num_total_inodes: num_total_blocks,
            num_free_inodes: num_free_blocks,
        })
    }

    async fn destroy(mut self) {
        // TODO Can we do this without unwrap?
        self.blobstore.async_drop().await.unwrap();
    }
}

impl<B> Drop for CryDevice<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
{
    fn drop(&mut self) {
        if !self.blobstore.is_dropped() {
            safe_panic!("CryDevice dropped without calling destroy() first");
        }
    }
}

#[must_use = "Contains AsyncDropGuard, async_drop must be called"]
enum LoadTwoBlobsResult<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    AreSameBlob {
        blob: AsyncDropGuard<FsBlob<'a, B>>,
    },
    AreDifferentBlobs {
        blob1: AsyncDropGuard<FsBlob<'a, B>>,
        blob2: AsyncDropGuard<FsBlob<'a, B>>,
    },
}
