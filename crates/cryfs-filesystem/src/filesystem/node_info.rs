use async_trait::async_trait;
use cryfs_utils::with_async_drop_2;
use std::fmt::Debug;
use std::time::SystemTime;
use tokio::join;

use super::fsblobstore::{DIR_LSTAT_SIZE, DirBlob, DirEntry, FileBlob, FsBlob};
use crate::filesystem::concurrentfsblobstore::{ConcurrentFsBlob, ConcurrentFsBlobStore};
use crate::filesystem::fsblobstore::BlobType;
use crate::utils::fs_types;
use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_rustfs::{
    AtimeUpdateBehavior, FsError, FsResult, Mode, NodeAttrs, NumBytes, PathComponentBuf,
};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

// TODO The ancestor_checks_on_move feature implements checks that when moving a node to a different directory,
//      it doesn't get moved into an ancestor or child of itself. But it requires that `NodeInfo` remembers the blob ids
//      of all ancestors, from the root blob to the blob itself. This could be expensive? Or maybe not?
//      We need to test how expensive it is and if we should remove it. Note also that this check might be
//      unnecessary, because I think fuser already implements this check? But need to verify that they do, and
//      even if they do, maybe it's better to check ourselves as well and not fully trust fuser because it could
//      break file system consistency when broken?

#[derive(Debug)]
enum NodeInfoImpl<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    IsRootDir {
        root_blob_id: BlobId,

        // TODO atime_update_behavior is in both IsRootDir and IsNotRootDir, maybe we should pull it out of an enum into a surrounding struct.
        atime_update_behavior: AtimeUpdateBehavior,
    },
    IsNotRootDir {
        parent_blob: AsyncDropGuard<ConcurrentFsBlob<B>>,
        // TODO It probably makes sense to store the self_blob here as well, maybe optional (OnceCell?) so it isn't unnecessarily loaded, but once it's loaded it's cached and doesn't have to be re-loaded.
        //      But we need to be careful with deadlocks, given that NodeInfo is stored in fuser in the inode list and the open file list.
        /// ancestors.first() is the root blob id, ancestors.last() is the immediate parent of this node.
        #[cfg(feature = "ancestor_checks_on_move")]
        ancestors: Box<[BlobId]>,

        name: PathComponentBuf,

        blob_id: BlobId,

        blob_type: BlobType,

        atime_update_behavior: AtimeUpdateBehavior,
    },
}

#[derive(Debug)]
pub struct NodeInfo<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    inner: NodeInfoImpl<B>,
}

impl<B> NodeInfo<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    pub fn new_rootdir(
        root_blob_id: BlobId,
        atime_update_behavior: AtimeUpdateBehavior,
    ) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            inner: NodeInfoImpl::IsRootDir {
                root_blob_id,
                atime_update_behavior,
            },
        })
    }

    pub fn new_non_root_dir(
        parent_blob: AsyncDropGuard<ConcurrentFsBlob<B>>,
        #[cfg(feature = "ancestor_checks_on_move")] ancestors: Box<[BlobId]>,
        name: PathComponentBuf,
        blob_id: BlobId,
        blob_type: BlobType,
        atime_update_behavior: AtimeUpdateBehavior,
    ) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            inner: NodeInfoImpl::IsNotRootDir {
                parent_blob,
                #[cfg(feature = "ancestor_checks_on_move")]
                ancestors,
                name,
                blob_id,
                blob_type,
                atime_update_behavior,
            },
        })
    }

    pub fn blob_id(&self) -> &BlobId {
        match &self.inner {
            NodeInfoImpl::IsRootDir { root_blob_id, .. } => root_blob_id,
            NodeInfoImpl::IsNotRootDir { blob_id, .. } => blob_id,
        }
    }

    pub fn parent_blob(&self) -> Option<&AsyncDropGuard<ConcurrentFsBlob<B>>> {
        match &self.inner {
            NodeInfoImpl::IsRootDir { .. } => None,
            NodeInfoImpl::IsNotRootDir { parent_blob, .. } => Some(parent_blob),
        }
    }

    #[cfg(feature = "ancestor_checks_on_move")]
    pub fn ancestors(&self) -> &[BlobId] {
        match &self.inner {
            NodeInfoImpl::IsRootDir { .. } => {
                // Return an empty array for the root dir
                &[]
            }
            NodeInfoImpl::IsNotRootDir { ancestors, .. } => {
                // Return the stored ancestors
                ancestors
            }
        }
    }

    #[cfg(feature = "ancestor_checks_on_move")]
    pub fn ancestors_and_self(&self) -> AncestorChain {
        // TODO Both self.blob_id and self.ancestors() match over Self::{IsRootDir/IsNotRootDir}, can we combine that into just one branch?
        let self_blob_id = self.blob_id();
        let ancestors_and_self = self
            .ancestors()
            .iter()
            .copied()
            .chain(std::iter::once(*self_blob_id))
            .collect::<Box<[BlobId]>>();
        AncestorChain::new(ancestors_and_self)
    }

    pub fn node_type(&self) -> BlobType {
        match &self.inner {
            NodeInfoImpl::IsRootDir { .. } => BlobType::Dir,
            NodeInfoImpl::IsNotRootDir { blob_type, .. } => *blob_type,
        }
    }

    pub async fn load_blob(
        &self,
        blobstore: &ConcurrentFsBlobStore<B>,
    ) -> FsResult<AsyncDropGuard<ConcurrentFsBlob<B>>> {
        let blob_id = self.blob_id();
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

    pub async fn flush_if_cached(&self, blobstore: &ConcurrentFsBlobStore<B>) -> FsResult<()> {
        let blob_id = self.blob_id();
        blobstore.flush_if_cached(*blob_id).await.map_err(|err| {
            log::error!("Error flushing blob {:?}: {:?}", blob_id, err);
            FsError::UnknownError
        })
    }

    pub fn as_file_mut<'s>(blob: &'s mut FsBlob<B>) -> FsResult<&'s mut FileBlob<B>> {
        let blob_id = blob.blob_id();
        blob.as_file_mut().map_err(|err| {
            FsError::CorruptedFilesystem {
                // TODO Add to message what it actually is
                message: format!("Blob {:?} is listed as a file in its parent directory but is actually not a file: {err:?}", blob_id),
            }
        })
    }

    async fn load_lstat_size(&self, blobstore: &ConcurrentFsBlobStore<B>) -> FsResult<NumBytes> {
        let mut blob = self.load_blob(blobstore).await?;
        let lstat_size = blob.with_lock(async |blob| blob.lstat_size().await).await;
        let result = match lstat_size {
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

    pub async fn getattr(&self, blobstore: &ConcurrentFsBlobStore<B>) -> FsResult<NodeAttrs> {
        match &self.inner {
            NodeInfoImpl::IsRootDir {
                root_blob_id: _,
                atime_update_behavior: _,
            } => {
                // We're the root dir
                // TODO What should we do here?
                let now = SystemTime::now();
                Ok(NodeAttrs {
                    // TODO If possible without performance loss, then for a directory, st_nlink should return number of dir entries (including "." and "..")
                    nlink: 1,
                    // TODO Remove those conversions
                    mode: cryfs_rustfs::Mode::default()
                        .add_dir_flag()
                        .add_user_read_flag()
                        .add_user_write_flag()
                        .add_user_exec_flag(),
                    // TODO Windows doesn't have Uid/Gid, so we need to put something else here
                    uid: cryfs_rustfs::Uid::from(nix::unistd::Uid::current().as_raw()),
                    gid: cryfs_rustfs::Gid::from(nix::unistd::Gid::current().as_raw()),
                    num_bytes: cryfs_rustfs::NumBytes::from(DIR_LSTAT_SIZE),
                    // Setting num_blocks to none means it'll be automatically calculated for us
                    num_blocks: None,
                    atime: now,
                    mtime: now,
                    ctime: now,
                })
            }
            NodeInfoImpl::IsNotRootDir {
                parent_blob, name, ..
            } => {
                let lstat_size = self.load_lstat_size(blobstore).await?;
                let entry: DirEntry = parent_blob
                    .with_lock(async |blob| {
                        blob.as_dir()
                            .expect("Parent dir is not a directory")
                            .entry_by_name(name)
                            .cloned()
                            .ok_or_else(|| FsError::NodeDoesNotExist)
                    })
                    .await?;
                Ok(dir_entry_to_node_attrs(&entry, lstat_size))
            }
        }
    }

    pub async fn setattr(
        &self,
        blobstore: &ConcurrentFsBlobStore<B>,
        mode: Option<Mode>,
        uid: Option<cryfs_rustfs::Uid>,
        gid: Option<cryfs_rustfs::Gid>,
        size: Option<NumBytes>,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
        ctime: Option<SystemTime>,
    ) -> FsResult<NodeAttrs> {
        // TODO Or is setting ctime allowed? What would it mean?
        assert!(ctime.is_none(), "Cannot set ctime via setattr");
        // TODO Improve concurrency
        if let Some(size) = size {
            self.truncate_file(blobstore, size).await?;
        }

        match &self.inner {
            NodeInfoImpl::IsRootDir { .. } => {
                // We're the root dir
                // TODO What should we do here?
                Err(FsError::InvalidOperation)
            }
            NodeInfoImpl::IsNotRootDir {
                parent_blob, name, ..
            } => {
                // TODO No Mode/Uid/Gid conversion
                let mode = mode.map(|mode| fs_types::Mode::from(u32::from(mode)));
                let uid = uid.map(|uid| fs_types::Uid::from(u32::from(uid)));
                let gid = gid.map(|gid| fs_types::Gid::from(u32::from(gid)));
                let lstat_size = self.load_lstat_size(blobstore).await?;
                parent_blob
                    .with_lock(async |parent_blob| {
                        let parent_dir = parent_blob
                            .as_dir_mut()
                            .expect("Parent dir is not a directory");
                        let result = parent_dir
                            .set_attr_of_entry_by_name(name, mode, uid, gid, atime, mtime)
                            .map(|result| dir_entry_to_node_attrs(result, lstat_size));
                        // Even if other fields are `None` (i.e. we don't run chmod, chown, utime), we still need to update the mtime in a truncate operation
                        if size.is_some() {
                            // TODO Don't look up the entry by name twice when we have attrs and size.is_some(). Looking it up once should be enough.
                            parent_dir.update_modification_timestamp_by_name(name)?;
                        }
                        result
                    })
                    .await
            }
        }
    }

    pub async fn truncate_file(
        &self,
        blobstore: &ConcurrentFsBlobStore<B>,
        new_size: NumBytes,
    ) -> FsResult<()> {
        let blob = self.load_blob(blobstore).await?;
        with_async_drop_2!(blob, {
            blob.with_lock(async |mut blob| {
                let file = Self::as_file_mut(&mut blob)?;
                file.resize(new_size.into()).await.map_err(|err| {
                    log::error!("Error resizing file blob: {err:?}");
                    FsError::UnknownError
                })
            })
            .await
        })
    }

    pub async fn concurrently_maybe_update_access_timestamp_in_parent<F>(
        &self,
        concurrent_fn: impl AsyncFnOnce() -> FsResult<F>,
    ) -> FsResult<F> {
        let (update_result, fn_result) = join!(
            self.maybe_update_access_timestamp_in_parent(self.atime_update_behavior()),
            concurrent_fn(),
        );
        update_result?;
        fn_result
    }

    pub async fn concurrently_update_modification_timestamp_in_parent<F>(
        &self,
        concurrent_fn: impl AsyncFnOnce() -> FsResult<F>,
    ) -> FsResult<F> {
        let (update_result, fn_result) = join!(
            self.update_modification_timestamp_in_parent(),
            concurrent_fn(),
        );
        update_result?;
        fn_result
    }

    pub async fn maybe_update_access_timestamp_in_parent(
        &self,
        atime_update_behavior: AtimeUpdateBehavior,
    ) -> FsResult<()> {
        self._update_in_parent(|parent, blob_id| {
            parent.maybe_update_access_timestamp_of_entry(blob_id, atime_update_behavior)
        })
        .await
    }

    pub async fn update_modification_timestamp_in_parent(&self) -> FsResult<()> {
        self._update_in_parent(|parent, blob_id| {
            parent.update_modification_timestamp_of_entry(blob_id)
        })
        .await
    }

    async fn _update_in_parent(
        &self,
        update_fn: impl FnOnce(&mut DirBlob<B>, &BlobId) -> FsResult<()>,
    ) -> FsResult<()> {
        match &self.inner {
            NodeInfoImpl::IsRootDir { .. } => {
                // We're the root dir
                //TODO Instead of doing nothing when we're the root directory, handle timestamps in the root dir correctly (and delete isRootDir() function)
                Ok(())
            }
            NodeInfoImpl::IsNotRootDir {
                parent_blob,
                blob_id,
                ..
            } => {
                parent_blob
                    .with_lock(async |parent_blob| {
                        let parent_dir = parent_blob
                            .as_dir_mut()
                            .expect("Parent blob is not a directory");
                        update_fn(parent_dir, blob_id)
                    })
                    .await
            }
        }
    }

    pub fn atime_update_behavior(&self) -> AtimeUpdateBehavior {
        match &self.inner {
            NodeInfoImpl::IsRootDir {
                atime_update_behavior,
                ..
            } => *atime_update_behavior,
            NodeInfoImpl::IsNotRootDir {
                atime_update_behavior,
                ..
            } => *atime_update_behavior,
        }
    }

    pub async fn flush_metadata(&self) -> FsResult<()> {
        let Some(parent_blob) = self.parent_blob() else {
            // Can't flush metadata of root dir
            return Ok(());
        };
        parent_blob
            .with_lock(async |parent_blob| {
                parent_blob
                    .as_dir_mut()
                    .expect("Parent dir isn't a directory")
                    .flush()
                    .await
            })
            .await
            .map_err(|err| {
                log::error!("Failed to flush_metadata of parent blob: {err:?}");
                FsError::UnknownError
            })?;
        Ok(())
    }
}

#[async_trait]
impl<B> AsyncDrop for NodeInfo<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        match &mut self.inner {
            NodeInfoImpl::IsRootDir { .. } => (),
            NodeInfoImpl::IsNotRootDir { parent_blob, .. } => {
                parent_blob.async_drop().await?;
            }
        }
        Ok(())
    }
}

fn dir_entry_to_node_attrs(entry: &DirEntry, num_bytes: NumBytes) -> NodeAttrs {
    NodeAttrs {
        //TODO If possible without performance loss, then for a directory, nlink should return number of dir entries (including "." and "..")
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

#[cfg(feature = "ancestor_checks_on_move")]
pub struct AncestorChain {
    ancestors_and_self: Box<[BlobId]>,
}

#[cfg(feature = "ancestor_checks_on_move")]
impl AncestorChain {
    pub fn new(ancestors_and_self: Box<[BlobId]>) -> Self {
        assert!(
            !ancestors_and_self.is_empty(),
            "Ancestors should not be empty, there must be at least the self node"
        );
        Self { ancestors_and_self }
    }

    pub fn ancestors_and_self(self) -> Box<[BlobId]> {
        // Return the stored ancestors
        self.ancestors_and_self
    }
}
