use anyhow::{Context as _, Result};
use async_trait::async_trait;
use atomic_time::AtomicInstant;
use cryfs_blockstore::RemoveResult;
use cryfs_rustfs::AtimeUpdateBehavior;
use cryfs_rustfs::object_based_api::Dir as _;
use cryfs_utils::with_async_drop_2;
use futures::join;
use maybe_owned::MaybeOwned;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::{fmt::Debug, time::Instant};

use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_rustfs::{
    AbsolutePath, FsError, FsResult, PathComponent, Statfs, object_based_api::Device,
};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};

use super::{
    dir::CryDir, file::CryFile, node::CryNode, node_info::NodeInfo, open_file::CryOpenFile,
    symlink::CrySymlink,
};
use crate::filesystem::concurrentfsblobstore::{ConcurrentFsBlob, ConcurrentFsBlobStore};
use crate::filesystem::fsblobstore::{BlobType, EntryType, FsBlob, FsBlobStore};

pub struct CryDevice<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    blobstore: AsyncDropGuard<AsyncDropArc<ConcurrentFsBlobStore<B>>>,
    root_blob_id: BlobId,

    atime_update_behavior: AtimeUpdateBehavior,

    /// Time of the latest operation that was executed on the filesystem
    last_access_time: Arc<AtomicInstant>,
}

impl<B> CryDevice<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    pub fn load_filesystem(
        blobstore: AsyncDropGuard<B>,
        root_blob_id: BlobId,
        atime_update_behavior: AtimeUpdateBehavior,
    ) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            blobstore: AsyncDropArc::new(ConcurrentFsBlobStore::new(FsBlobStore::new(blobstore))),
            root_blob_id,
            atime_update_behavior,
            last_access_time: Arc::new(AtomicInstant::now()),
        })
    }

    pub async fn create_new_filesystem(
        blobstore: AsyncDropGuard<B>,
        root_blob_id: BlobId,
        atime_update_behavior: AtimeUpdateBehavior,
    ) -> Result<AsyncDropGuard<Self>> {
        let mut fsblobstore = ConcurrentFsBlobStore::new(FsBlobStore::new(blobstore));
        match fsblobstore.create_root_dir_blob(&root_blob_id).await {
            Ok(()) => Ok(AsyncDropGuard::new(Self {
                blobstore: AsyncDropArc::new(fsblobstore),
                root_blob_id,
                atime_update_behavior,
                last_access_time: Arc::new(AtomicInstant::now()),
            })),
            Err(err) => {
                fsblobstore.async_drop().await?;
                Err(err)
            }
        }
    }

    pub async fn sanity_check(&self) -> Result<()> {
        // Make sure we can load the root dir and load its children
        let rootdir = self.rootdir().await.context("Didn't find root blob")?;
        with_async_drop_2!(rootdir, {
            rootdir.entries().await.context("Couldn't load root blob")?;
            Ok(())
        })
    }

    async fn load_blob(
        &self,
        path: impl IntoIterator<Item = &PathComponent>,
    ) -> FsResult<AsyncDropGuard<ConcurrentFsBlob<B>>> {
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
        if root_blob.blob_type().await != BlobType::Dir {
            log::error!("Root blob is not a directory");
            root_blob.async_drop().await?;
            return Err(FsError::Custom {
                error_code: libc::EIO,
            });
        }
        self.load_blob_from_relative_path_owned(root_blob, path.into_iter())
            .await
    }

    async fn load_blob_from_relative_path_owned(
        &self,
        anchor: AsyncDropGuard<ConcurrentFsBlob<B>>,
        relative_path: impl Iterator<Item = &PathComponent>,
    ) -> FsResult<AsyncDropGuard<ConcurrentFsBlob<B>>> {
        match self
            .load_blob_from_relative_path(MaybeOwned::Owned(anchor), relative_path)
            .await?
        {
            MaybeOwned::Owned(blob) => Ok(blob),
            MaybeOwned::Borrowed(_blob) => panic!(
                "Since we called `load_blob_from_relative_path` with an owned anchor, it should never return a borrowed blob"
            ),
        }
    }

    // If `anchor` is borrowed and `relative_path` is empty, the returned blob will be borrowed.
    // If `anchor` is owned or `relative_path` is non-empty, the returned blob will be owned.
    async fn load_blob_from_relative_path<'b>(
        &self,
        anchor: MaybeOwned<'b, AsyncDropGuard<ConcurrentFsBlob<B>>>,
        relative_path: impl Iterator<Item = &PathComponent>,
    ) -> FsResult<MaybeOwned<'b, AsyncDropGuard<ConcurrentFsBlob<B>>>> {
        let mut current_blob = anchor;

        for path_component in relative_path {
            let blob_id = current_blob
                .with_lock(async |blob| {
                    let dir_blob = blob.as_dir().map_err(|_err| FsError::NodeIsNotADirectory)?;
                    dir_blob.entry_by_name(path_component).map_or(
                        // TODO This error mapping is weird. Probably better to have as_dir return the right error type.
                        Err(FsError::NodeDoesNotExist),
                        |entry| Ok(*entry.blob_id()),
                    )
                })
                .await;

            if let Some(current_blob) = current_blob.as_mut() {
                current_blob.async_drop().await?;
            } else {
                // current_blob is borrowed. No need to drop it
            }

            let blob_id = blob_id?;
            current_blob = MaybeOwned::from(
                self.blobstore
                    .load(&blob_id)
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

    async fn load_two_blobs(
        &self,
        path1: &AbsolutePath,
        path2: &AbsolutePath,
    ) -> FsResult<LoadTwoBlobsResult<B>> {
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
            (Ok(MaybeOwned::Borrowed(_blob1)), Ok(MaybeOwned::Borrowed(_blob2))) => {
                // Both blobs are borrowed, this means that both relative paths were empty
                assert_eq!(0, relative_path1_len);
                assert_eq!(0, relative_path2_len);
                Ok(LoadTwoBlobsResult::AreSameBlob(shared_blob))
            }
            (Ok(MaybeOwned::Owned(blob1)), Ok(MaybeOwned::Owned(blob2))) => {
                // Both blobs are owned, this means that neither relative path was empty
                assert!(relative_path1_len > 0);
                assert!(relative_path2_len > 0);
                shared_blob.async_drop().await?;
                Ok(LoadTwoBlobsResult::AreDifferentBlobs(blob1, blob2))
            }
            (Ok(MaybeOwned::Borrowed(_blob1)), Ok(MaybeOwned::Owned(blob2))) => {
                // blob1 is borrowed, blob2 is owned, this means that relative_path1 is empty and relative_path2 is not
                assert_eq!(0, relative_path1_len);
                assert!(relative_path2_len > 0);
                Ok(LoadTwoBlobsResult::AreDifferentBlobs(shared_blob, blob2))
            }
            (Ok(MaybeOwned::Owned(blob1)), Ok(MaybeOwned::Borrowed(_blob2))) => {
                // blob1 is owned, blob2 is borrowed, this means that relative_path1 is not empty and relative_path2 is empty
                assert!(relative_path1_len > 0);
                assert_eq!(0, relative_path2_len);
                Ok(LoadTwoBlobsResult::AreDifferentBlobs(blob1, shared_blob))
            }
        }
    }

    /// Returns the last time the filesystem was accessed. This is updated on every operation.
    pub fn last_access_time(&self) -> Arc<AtomicInstant> {
        Arc::clone(&self.last_access_time)
    }
}

impl<B> Debug for CryDevice<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CryDevice")
            .field("root_blob_id", &self.root_blob_id)
            .finish()
    }
}

#[async_trait]
impl<B> Device for CryDevice<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    type Node = CryNode<B>;
    type Dir<'a> = CryDir<'a, B>;
    type Symlink<'a> = CrySymlink<'a, B>;
    type File<'a> = CryFile<'a, B>;
    type OpenFile = CryOpenFile<B>;

    async fn on_operation(&self) -> FsResult<()> {
        self.last_access_time
            .store(Instant::now(), Ordering::Relaxed);
        Ok(())
    }

    async fn rootdir(&self) -> FsResult<AsyncDropGuard<Self::Dir<'_>>> {
        let node_info = NodeInfo::new_rootdir(self.root_blob_id, self.atime_update_behavior);
        Ok(CryDir::new(&self.blobstore, AsyncDropArc::new(node_info)))
    }

    // TODO For some reason, async_trait doesn't work for `rename` and trying to do `async fn rename` fails, but the error message says that it's
    //      a known restriction that will be lifted in later rust versions. Check on later rust versions if it now works.
    fn rename(
        &self,
        source_path: &AbsolutePath,
        dest_path: &AbsolutePath,
    ) -> impl Future<Output = FsResult<()>> {
        async move {
            if source_path.is_ancestor_of(dest_path) {
                log::error!(
                    "Tried to rename {source_path} into its descendant {dest_path}",
                    source_path = source_path,
                    dest_path = dest_path
                );
                return Err(FsError::CannotMoveDirectoryIntoSubdirectoryOfItself);
            }
            if dest_path.is_ancestor_of(source_path) {
                // TODO Is this check actually necessary? We're checking that the target is non-empty anyways if it's a dir.
                log::error!(
                    "Tried to rename {source_path} into its ancestor {dest_path}",
                    source_path = source_path,
                    dest_path = dest_path
                );
                return Err(FsError::CannotOverwriteNonEmptyDirectory);
            }

            let Some((source_parent, source_name)) = source_path.split_last() else {
                log::error!("Tried to rename the root directory {source_path} into {dest_path}");
                return Err(FsError::InvalidOperation);
            };
            let Some((dest_parent, dest_name)) = dest_path.split_last() else {
                log::error!("Tried to rename {source_path} to the root directory {dest_path}");
                return Err(FsError::InvalidOperation);
            };

            let on_overwritten =
                async move |blobid: &BlobId| match self.blobstore.remove_by_id(&blobid).await {
                    // TODO If we figure out exception safety (see below), we can move the existing_dir_check into here.
                    //      Currently, some checks already happen inside of [add_or_overwrite_entry], e.g. that we don't overwrite dirs with non-dirs.
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
                };

            match self.load_two_blobs(source_parent, dest_parent).await? {
                LoadTwoBlobsResult::AreSameBlob(blob) => {
                    with_async_drop_2!(blob, {
                        blob.with_lock(async |blob| {
                            let parent = blob
                                .as_dir_mut()
                                .map_err(|_| FsError::NodeIsNotADirectory)?;
                            // TODO Don't allow overriding a non-empty directory
                            parent
                                .rename_entry_by_name(
                                    source_name,
                                    dest_name.to_owned(),
                                    on_overwritten,
                                )
                                .await?;
                            Ok(())
                        })
                        .await
                    })?;
                }
                LoadTwoBlobsResult::AreDifferentBlobs(source_parent_blob, dest_parent_blob) => {
                    // TODO We're currently locking, releasing and re-locking blobs multiple times. This introduces race conditions and is not optimal for performance either.
                    //      We should just lock each blob once and keep it locked until we're done. But we need to do it in a deadlock-free way, locking multiple
                    //      blobs at once is risky for deadlocks if not done in a consistent order.
                    // TODO Improve concurrency in this function

                    // TODO Concurrently drop source_parent_blob and dest_parent_blob
                    with_async_drop_2!(source_parent_blob, {
                        with_async_drop_2!(dest_parent_blob, {
                            let entry = source_parent_blob
                                .with_lock(async |source_parent: &mut FsBlob<B>| {
                                    source_parent
                                        .as_dir_mut()
                                        .map_err(|_| FsError::NodeIsNotADirectory)?
                                        .entry_by_name(source_name)
                                        .ok_or(FsError::NodeDoesNotExist)
                                        .cloned() // TODO No cloned?
                                })
                                .await?;
                            let self_blob_id = entry.blob_id();

                            // TODO In theory, we could load self_blob concurrently with dest_parent_blob. No need to only do it after dest_parent_blob loaded.
                            //      But it likely has some dependency with source_parent_blob.
                            let mut self_blob = self
                                .blobstore
                                .load(self_blob_id)
                                .await
                                .map_err(|err| {
                                    log::error!("Error loading blob: {:?}", err);
                                    FsError::UnknownError
                                })?
                                .ok_or(FsError::NodeDoesNotExist)?;

                            let existing_dir_check = async || {
                                let existing_dest_entry = dest_parent_blob
                                    .with_lock(async |dest_parent| {
                                        Ok(dest_parent
                                            .as_dir_mut()
                                            .map_err(|_| FsError::NodeIsNotADirectory)?
                                            .entry_by_name_mut(dest_name)
                                            .cloned()) // TODO No cloned?
                                    })
                                    .await?;
                                if let Some(existing_dest_entry) = existing_dest_entry {
                                    if existing_dest_entry.entry_type() == EntryType::Dir {
                                        // TODO Shouldn't we check the existing dest directory's entries instead of self_blob's entries?
                                        self_blob.with_lock(async |self_blob| {
                                    let self_blob = self_blob.as_dir()
                                        .map_err(|_| FsError::CorruptedFilesystem { message: format!("Blob {self_blob_id:?} is not a directory but its entry in its parent directory says it is") })?;
                                    if self_blob.entries().len() > 0 {
                                        return Err(FsError::CannotOverwriteNonEmptyDirectory);
                                    }
                                    Ok(())
                                }).await?;
                                    }
                                }
                                Ok(())
                            };
                            // TODO with_async_drop_2! for self_blob, and then remove the lambda around exist_dir_check
                            match existing_dir_check().await {
                                Ok(()) => (),
                                Err(err) => {
                                    self_blob.async_drop().await?;
                                    return Err(err);
                                }
                            }

                            let res = source_parent_blob
                                .with_lock(async |source_parent| {
                                    source_parent
                                        .as_dir_mut()
                                        .map_err(|_| FsError::NodeIsNotADirectory)?
                                        .remove_entry_by_name(source_name)
                                })
                                .await;
                            let entry = match res {
                                Ok(entry) => entry,
                                Err(err) => {
                                    log::error!("Error in add_or_overwrite_entry: {err:?}");
                                    self_blob.async_drop().await?;
                                    return Err(FsError::UnknownError);
                                }
                            };
                            let res = dest_parent_blob
                                .with_lock(async |source_parent| {
                                    source_parent
                                        .as_dir_mut()
                                        .map_err(|_| FsError::NodeIsNotADirectory)?
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
                                        .await
                                })
                                .await;
                            match res {
                                Ok(()) => (),
                                Err(err) => {
                                    // TODO Exception safety - we couldn't add the entry to the destination, but we already removed it from the source. We should probably re-add it to the source.
                                    self_blob.async_drop().await?;
                                    log::error!("Error in add_or_overwrite_entry: {err:?}");
                                    return Err(FsError::UnknownError);
                                }
                            }

                            let res = self_blob
                                .with_lock(async |self_blob| {
                                    self_blob.set_parent(&dest_parent_blob.blob_id()).await
                                })
                                .await;
                            match res {
                                Ok(()) => (),
                                Err(err) => {
                                    // TODO Exception safety - we already changed parent dir entries but couldn't update the parent pointer. We should probably try to undo the parent dir entry changes.
                                    self_blob.async_drop().await?;
                                    log::error!("Error setting parent: {err:?}");
                                    return Err(FsError::UnknownError);
                                }
                            }
                            self_blob.async_drop().await?;
                            Ok(())
                        })
                    })?;
                }
            }

            Ok(())
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
        //TODO Maybe we shouold increase max_filename_length?
        let max_filename_length = 255; // We theoretically support unlimited file name length, but this is default for many Linux file systems, so probably also makes sense for CryFS.
        let blocksize = self.blobstore.logical_block_size_bytes();

        Ok(Statfs {
            max_filename_length,
            blocksize: u32::try_from(blocksize.as_u64()).unwrap(),
            num_total_blocks,
            num_free_blocks,
            num_available_blocks: num_free_blocks,
            num_total_inodes: num_total_blocks,
            num_free_inodes: num_free_blocks,
        })
    }
}

#[async_trait]
impl<B> AsyncDrop for CryDevice<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.blobstore.async_drop().await
    }
}

enum LoadTwoBlobsResult<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    AreSameBlob(AsyncDropGuard<ConcurrentFsBlob<B>>),
    AreDifferentBlobs(
        AsyncDropGuard<ConcurrentFsBlob<B>>,
        AsyncDropGuard<ConcurrentFsBlob<B>>,
    ),
}
