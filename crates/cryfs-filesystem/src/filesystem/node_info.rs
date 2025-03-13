use std::fmt::Debug;
use std::time::SystemTime;
use tokio::join;
use tokio::sync::OnceCell;

use super::fsblobstore::{DIR_LSTAT_SIZE, DirBlob, DirEntry, EntryType, FileBlob, FsBlob};
use crate::filesystem::fsblobstore::{BlobType, FsBlobStore};
use crate::utils::fs_types;
use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_rustfs::{
    AtimeUpdateBehavior, FsError, FsResult, Gid, Mode, NodeAttrs, NumBytes, PathComponent,
    PathComponentBuf, Uid,
};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

#[derive(Debug, Clone, Copy)]
pub struct BlobDetails {
    pub blob_id: BlobId,
    pub blob_type: BlobType,
}

#[derive(Debug, Clone)]
pub enum NodeInfo {
    IsRootDir {
        root_blob_id: BlobId,

        // TODO atime_update_behavior is in both IsRootDir and IsNotRootDir, maybe we should pull it out of an enum into a surrounding struct.
        atime_update_behavior: AtimeUpdateBehavior,
    },
    IsNotRootDir {
        parent_blob_id: BlobId,
        name: PathComponentBuf,

        // While fields in [parent_blob_id] and [name] are always set, a [CryNode]/[NodeInfo] object can exist even before we
        // actually looked it up from the parent directory. In that case, [blob_details] is not initialized yet.
        // Once it was looked up, this will be initialized.
        // The reason for this is that we cannot hold a reference to the loaded parent blob in here because that would
        // lock it and prevent other threads from loading it, potentially leading to a deadlock. But some operations in here
        // (e.g. getattr, chmod, chown) need to load the parent blob. To prevent loading it multiple times, we avoid loading
        // the parent blob before instantiating the [CryNode]/[NodeInfo] instance.
        // TODO Check if any of the Mutex'es used in the whole repository could be replaced by OnceCell, Cell, RefCell or similar
        // TODO Maybe it's actually ok to store a Blob here? Except for rename, all operations should just operate on one node
        //      and not depend on another node so we shouldn't cause any deadlocks. This would simplify what we're doing here
        //      and would also allow us to avoid potential double loads where [load_parent_blob] is called multiple times.
        //      For example, each update to atime or mtime currently loads the parent blob for a second time, which is slow and
        //      means basically every operation does it.
        blob_details: OnceCell<BlobDetails>,

        atime_update_behavior: AtimeUpdateBehavior,
    },
}

pub enum LoadParentBlobResult<'a, 'b, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'c> <B as BlobStore>::ConcreteBlob<'c>: Send + Sync,
{
    IsRootDir {
        root_blob: BlobId,
    },
    IsNotRootDir {
        parent_blob: AsyncDropGuard<DirBlob<'b, B>>,
        name: &'a PathComponent,
        blob_details: &'a BlobDetails,
    },
}

impl NodeInfo {
    pub fn new_rootdir(root_blob_id: BlobId, atime_update_behavior: AtimeUpdateBehavior) -> Self {
        Self::IsRootDir {
            root_blob_id,
            atime_update_behavior,
        }
    }

    pub fn new(
        parent_blob_id: BlobId,
        name: PathComponentBuf,
        atime_update_behavior: AtimeUpdateBehavior,
    ) -> Self {
        Self::IsNotRootDir {
            parent_blob_id,
            name,
            blob_details: OnceCell::default(),
            atime_update_behavior,
        }
    }

    async fn _load_parent_blob<'a, B>(
        blobstore: &'a FsBlobStore<B>,
        parent_blob_id: &BlobId,
    ) -> FsResult<AsyncDropGuard<DirBlob<'a, B>>>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
    {
        let parent_blob = blobstore
            .load(parent_blob_id)
            .await
            .map_err(|err| {
                log::error!("Error loading blob {:?}: {:?}", parent_blob_id, err);
                FsError::UnknownError
            })?
            .ok_or_else(|| {
                log::error!("Parent blob {:?} not found", parent_blob_id);
                FsError::CorruptedFilesystem {
                    message: format!("Didn't find parent blob {:?}", parent_blob_id),
                }
            })?;
        FsBlob::into_dir(parent_blob)
            .await
            .map_err(|_err| FsError::NodeIsNotADirectory)
    }

    pub async fn load_parent_blob<'a, 'b, B>(
        &'a self,
        blobstore: &'b FsBlobStore<B>,
    ) -> FsResult<LoadParentBlobResult<'a, 'b, B>>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        for<'c> <B as BlobStore>::ConcreteBlob<'c>: Send + Sync,
    {
        match self {
            Self::IsRootDir {
                root_blob_id,
                atime_update_behavior: _,
            } => Ok(LoadParentBlobResult::IsRootDir {
                root_blob: *root_blob_id,
            }),
            Self::IsNotRootDir {
                parent_blob_id,
                name,
                blob_details,
                atime_update_behavior: _,
            } => {
                let mut parent_blob = Self::_load_parent_blob(blobstore, parent_blob_id).await?;

                // Also store any information we have into self.blob_details so we don't have to load the parent blob again if blob_details get queried later
                let blob_details = blob_details
                    .get_or_try_init(async || get_blob_details(&mut parent_blob, name))
                    .await;
                let blob_details = match blob_details {
                    Ok(blob_details) => Ok(blob_details),
                    Err(err) => {
                        parent_blob.async_drop().await?;
                        Err(err)
                    }
                }?;

                Ok(LoadParentBlobResult::IsNotRootDir {
                    parent_blob,
                    name,
                    blob_details,
                })
            }
        }
    }

    pub async fn blob_id<B>(&self, blobstore: &FsBlobStore<B>) -> FsResult<BlobId>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
    {
        self.blob_details(blobstore)
            .await
            .map(|blob_details| blob_details.blob_id)
    }

    pub async fn node_type<B>(&self, blobstore: &FsBlobStore<B>) -> FsResult<BlobType>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
    {
        self.blob_details(blobstore)
            .await
            .map(|blob_details| blob_details.blob_type)
    }

    pub async fn blob_details<B>(&self, blobstore: &FsBlobStore<B>) -> FsResult<BlobDetails>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
    {
        match self {
            Self::IsRootDir {
                root_blob_id,
                atime_update_behavior: _,
            } => Ok(BlobDetails {
                blob_id: root_blob_id.clone(),
                blob_type: BlobType::Dir,
            }),
            Self::IsNotRootDir {
                parent_blob_id,
                name,
                blob_details,
                atime_update_behavior: _,
            } => Ok(*blob_details
                .get_or_try_init(async || {
                    let mut parent_blob =
                        Self::_load_parent_blob(blobstore, parent_blob_id).await?;
                    let blob_details = get_blob_details(&mut parent_blob, name);
                    parent_blob.async_drop().await?;
                    blob_details
                })
                .await?),
        }
    }

    pub async fn load_blob<'a, B>(
        &self,
        blobstore: &'a FsBlobStore<B>,
    ) -> FsResult<AsyncDropGuard<FsBlob<'a, B>>>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
    {
        let blob_id = self.blob_details(blobstore).await?.blob_id;
        blobstore
            .load(&blob_id)
            .await
            .map_err(|err| {
                log::error!("Error loading blob {:?}: {:?}", blob_id, err);
                FsError::UnknownError
            })?
            .ok_or_else(|| {
                log::error!("Blob {:?} not found", blob_id);
                FsError::CorruptedFilesystem {
                    message: format!("Didn't find blob {:?}", blob_id),
                }
            })
    }

    pub async fn load_file_blob<'a, B>(
        &self,
        blobstore: &'a FsBlobStore<B>,
    ) -> Result<FileBlob<'a, B>, FsError>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
    {
        let blob = self.load_blob(blobstore).await?;
        let blob_id = blob.blob_id();
        FsBlob::into_file(blob).await.map_err(|err| {
            FsError::CorruptedFilesystem {
                // TODO Add to message what it actually is
                message: format!("Blob {:?} is listed as a directory in its parent directory but is actually not a directory: {err:?}", blob_id),
            }
        })
    }

    async fn load_lstat_size<B>(&self, blobstore: &FsBlobStore<B>) -> FsResult<NumBytes>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
    {
        let mut blob = self.load_blob(blobstore).await?;
        let result = match blob.lstat_size().await {
            // TODO Return NumBytes from blob.lstat_size() instead of converting it here
            Ok(size) => Ok(NumBytes::from(size)),
            Err(err) => {
                log::error!("Error getting lstat size: {:?}", err);
                Err(FsError::UnknownError)
            }
        };
        blob.async_drop().await?;
        result
    }

    pub async fn getattr<B>(&self, blobstore: &FsBlobStore<B>) -> FsResult<NodeAttrs>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
    {
        match self.load_parent_blob(blobstore).await? {
            LoadParentBlobResult::IsRootDir { .. } => {
                // We're the root dir
                // TODO What should we do here?
                Ok(NodeAttrs {
                    nlink: 1,
                    // TODO Remove those conversions
                    mode: cryfs_rustfs::Mode::default()
                        .add_dir_flag()
                        .add_user_read_flag()
                        .add_user_write_flag()
                        .add_user_exec_flag(),
                    uid: cryfs_rustfs::Uid::from(1000),
                    gid: cryfs_rustfs::Gid::from(1000),
                    num_bytes: cryfs_rustfs::NumBytes::from(DIR_LSTAT_SIZE),
                    // Setting num_blocks to none means it'll be automatically calculated for us
                    num_blocks: None,
                    atime: SystemTime::now(),
                    mtime: SystemTime::now(),
                    ctime: SystemTime::now(),
                })
            }
            LoadParentBlobResult::IsNotRootDir {
                name,
                mut parent_blob,
                ..
            } => {
                let result = (async || {
                    let lstat_size = self.load_lstat_size(blobstore).await?;
                    let entry = parent_blob
                        .entry_by_name(name)
                        .ok_or_else(|| FsError::NodeDoesNotExist)?;
                    Ok(dir_entry_to_node_attrs(entry, lstat_size))
                })()
                .await;
                parent_blob.async_drop().await?;
                result
            }
        }
    }

    pub async fn setattr<B>(
        &self,
        blobstore: &FsBlobStore<B>,
        mode: Option<Mode>,
        uid: Option<Uid>,
        gid: Option<Gid>,
        size: Option<NumBytes>,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
        ctime: Option<SystemTime>,
    ) -> FsResult<NodeAttrs>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
    {
        // TODO Or is setting ctime allowed? What would it mean?
        assert!(ctime.is_none(), "Cannot set ctime via setattr");
        // TODO Improve concurrency
        if let Some(size) = size {
            self.truncate_file(blobstore, size).await?;
        }
        match self.load_parent_blob(blobstore).await? {
            LoadParentBlobResult::IsRootDir { .. } => {
                // We're the root dir
                // TODO What should we do here?
                Err(FsError::InvalidOperation)
            }
            LoadParentBlobResult::IsNotRootDir {
                name,
                mut parent_blob,
                ..
            } => {
                // TODO No Mode/Uid/Gid conversion
                let mode = mode.map(|mode| fs_types::Mode::from(u32::from(mode)));
                let uid = uid.map(|uid| fs_types::Uid::from(u32::from(uid)));
                let gid = gid.map(|gid| fs_types::Gid::from(u32::from(gid)));
                let lstat_size = self.load_lstat_size(blobstore).await?;
                let result = parent_blob
                    .set_attr_of_entry_by_name(name, mode, uid, gid, atime, mtime)
                    .map(|result| dir_entry_to_node_attrs(result, lstat_size));
                parent_blob.async_drop().await?;
                result
            }
        }
    }

    pub async fn truncate_file<B>(
        &self,
        blobstore: &FsBlobStore<B>,
        new_size: NumBytes,
    ) -> FsResult<()>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
    {
        let mut blob = self.load_file_blob(blobstore).await?;
        blob.resize(new_size.into()).await.map_err(|err| {
            log::error!("Error resizing file blob: {err:?}");
            FsError::UnknownError
        })
    }

    pub async fn concurrently_maybe_update_access_timestamp_in_parent<B, F>(
        &self,
        blobstore: &FsBlobStore<B>,
        concurrent_fn: impl AsyncFnOnce() -> FsResult<F>,
    ) -> FsResult<F>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
    {
        let (update_result, fn_result) = join!(
            self.maybe_update_access_timestamp_in_parent(blobstore, self.atime_update_behavior()),
            concurrent_fn(),
        );
        update_result?;
        fn_result
    }

    pub async fn concurrently_update_modification_timestamp_in_parent<B, F>(
        &self,
        blobstore: &FsBlobStore<B>,
        concurrent_fn: impl AsyncFnOnce() -> FsResult<F>,
    ) -> FsResult<F>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
    {
        let (update_result, fn_result) = join!(
            self.update_modification_timestamp_in_parent(blobstore),
            concurrent_fn(),
        );
        update_result?;
        fn_result
    }

    pub async fn maybe_update_access_timestamp_in_parent<B>(
        &self,
        blobstore: &FsBlobStore<B>,
        atime_update_behavior: AtimeUpdateBehavior,
    ) -> FsResult<()>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
    {
        self._update_in_parent(blobstore, |parent, blob_id| {
            parent.maybe_update_access_timestamp_of_entry(blob_id, atime_update_behavior)
        })
        .await
    }

    pub async fn update_modification_timestamp_in_parent<B>(
        &self,
        blobstore: &FsBlobStore<B>,
    ) -> FsResult<()>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
    {
        self._update_in_parent(blobstore, |parent, blob_id| {
            parent.update_modification_timestamp_of_entry(blob_id)
        })
        .await
    }

    async fn _update_in_parent<B>(
        &self,
        blobstore: &FsBlobStore<B>,
        update_fn: impl FnOnce(&mut DirBlob<B>, &BlobId) -> FsResult<()>,
    ) -> FsResult<()>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
    {
        // TODO Ideally we'd do this without loading the parent blob (we've already loaded it right before to be able to load the actual blob),
        //      but if not possible, the least we can do is remember the current atime in BlobDetails on the first load before calling into here,
        //      and decide if we need to update the timestamp based on that before loading the parent blob. Avoiding a load when it doesn't need to be updated.
        //      Or at the very least, short circuit noatime to not load the parent blob.
        let parent = self.load_parent_blob(blobstore).await?;
        match parent {
            LoadParentBlobResult::IsRootDir { .. } => {
                //TODO Instead of doing nothing when we're the root directory, handle timestamps in the root dir correctly (and delete isRootDir() function)
            }
            LoadParentBlobResult::IsNotRootDir {
                name: _,
                mut parent_blob,
                blob_details,
            } => {
                update_fn(&mut parent_blob, &blob_details.blob_id)?;
                parent_blob.async_drop().await?;
            }
        }

        Ok(())
    }

    pub fn atime_update_behavior(&self) -> AtimeUpdateBehavior {
        match self {
            Self::IsRootDir {
                atime_update_behavior,
                ..
            } => *atime_update_behavior,
            Self::IsNotRootDir {
                atime_update_behavior,
                ..
            } => *atime_update_behavior,
        }
    }
}

fn get_blob_details<'a, B>(
    parent_blob: &DirBlob<'a, B>,
    name: &PathComponent,
) -> FsResult<BlobDetails>
where
    // TODO Do we really need B: 'static ?
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    let entry = parent_blob
        .entry_by_name(name)
        .ok_or_else(|| FsError::NodeDoesNotExist)?;
    let blob_id = *entry.blob_id();
    let blob_type = match entry.entry_type() {
        EntryType::File => BlobType::File,
        EntryType::Dir => BlobType::Dir,
        EntryType::Symlink => BlobType::Symlink,
    };
    Ok(BlobDetails { blob_id, blob_type })
}

fn dir_entry_to_node_attrs(entry: &DirEntry, num_bytes: NumBytes) -> NodeAttrs {
    NodeAttrs {
        nlink: 1,
        // TODO Remove those conversions
        mode: cryfs_rustfs::Mode::from(u32::from(entry.mode())),
        uid: cryfs_rustfs::Uid::from(u32::from(entry.uid())),
        gid: cryfs_rustfs::Gid::from(u32::from(entry.gid())),
        num_bytes,
        // Setting num_blocks to none means it'll be automatically calculated for us
        num_blocks: None,
        atime: entry.last_access_time(),
        mtime: entry.last_modification_time(),
        ctime: entry.last_metadata_change_time(),
    }
}
