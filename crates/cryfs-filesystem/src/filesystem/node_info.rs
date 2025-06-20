use cryfs_utils::with_async_drop_2;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::time::SystemTime;
use tokio::join;
use tokio::sync::OnceCell;

use super::fsblobstore::{DIR_LSTAT_SIZE, DirBlob, DirEntry, EntryType, FileBlob, FsBlob};
use crate::filesystem::concurrentfsblobstore::{ConcurrentFsBlob, ConcurrentFsBlobStore};
use crate::filesystem::fsblobstore::BlobType;
use crate::utils::fs_types;
use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_rustfs::{
    AtimeUpdateBehavior, FsError, FsResult, Mode, NodeAttrs, NumBytes, PathComponent,
    PathComponentBuf,
};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

// TODO The ancestor_checks_on_move feature implements checks that when moving a node to a different directory,
//      it doesn't get moved into an ancestor or child of itself. But it requires that `NodeInfo` remembers the blob ids
//      of all ancestors, from the root blob to the blob itself. This could be expensive? Or maybe not?
//      We need to test how expensive it is and if we should remove it. Note also that this check might be
//      unnecessary, because I think fuser already implements this check? But need to verify that they do, and
//      even if they do, maybe it's better to check ourselves as well and not fully trust fuser because it could
//      break file system consistency when broken?

// TODO Now that we have ConcurrentFsBlobStore, it's safe to have each NodeInfo store a reference to its parent.
//      We should do that instead of looking up parent blobs again when operations need it.

#[derive(Debug, Clone, Copy)]
pub struct BlobDetails {
    pub blob_id: BlobId,
    pub blob_type: BlobType,
}

#[derive(Debug)]
pub enum NodeInfo {
    IsRootDir {
        root_blob_id: BlobId,

        // TODO atime_update_behavior is in both IsRootDir and IsNotRootDir, maybe we should pull it out of an enum into a surrounding struct.
        atime_update_behavior: AtimeUpdateBehavior,
    },
    IsNotRootDir {
        #[cfg(not(feature = "ancestor_checks_on_move"))]
        parent_blob_id: BlobId,
        /// ancestors.first() is the root blob id, ancestors.last() is the immediate parent of this node.
        #[cfg(feature = "ancestor_checks_on_move")]
        ancestors: Box<[BlobId]>,

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
        //      Actually, it's probably OpenFile that's the problem since it stores a NodeInfo for as long as the file is open.
        //      Maybe we need to first implement ParallelAccessBlobStore ("CoalescingBlobStore"?) so that we can hold multiple
        //      instances of the blob. Or alternatively, we have to change OpenFile to not store a NodeInfo.
        blob_details: OnceCell<BlobDetails>,

        atime_update_behavior: AtimeUpdateBehavior,
    },
}

#[allow(async_fn_in_trait)]
pub trait CallbackWithParentBlob<B, R>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    async fn on_is_rootdir(self, root_blob: BlobId) -> FsResult<R>;
    async fn on_is_not_rootdir<'a, 'b, 'c>(
        self,
        parent_blob: &'a mut DirBlob<B>,
        name: &'b PathComponent,
        blob_details: &'c BlobDetails,
    ) -> FsResult<R>;
}

impl NodeInfo {
    pub fn new_rootdir(root_blob_id: BlobId, atime_update_behavior: AtimeUpdateBehavior) -> Self {
        Self::IsRootDir {
            root_blob_id,
            atime_update_behavior,
        }
    }

    pub fn new(
        #[cfg(not(feature = "ancestor_checks_on_move"))] parent_blob_id: BlobId,
        #[cfg(feature = "ancestor_checks_on_move")] ancestors: Box<[BlobId]>,
        name: PathComponentBuf,
        atime_update_behavior: AtimeUpdateBehavior,
    ) -> Self {
        Self::IsNotRootDir {
            #[cfg(not(feature = "ancestor_checks_on_move"))]
            parent_blob_id,
            #[cfg(feature = "ancestor_checks_on_move")]
            ancestors,
            name,
            blob_details: OnceCell::default(),
            atime_update_behavior,
        }
    }

    async fn _load_parent_blob<'a, B>(
        blobstore: &'a ConcurrentFsBlobStore<B>,
        parent_blob_id: &BlobId,
    ) -> FsResult<AsyncDropGuard<ConcurrentFsBlob<'a, B>>>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
    {
        blobstore
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
            })
    }

    pub async fn with_parent_blob<'a, 'b, 'c, B, R>(
        &'a self,
        blobstore: &'b ConcurrentFsBlobStore<B>,
        callback: impl CallbackWithParentBlob<B, R>,
    ) -> FsResult<R>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
    {
        match self {
            Self::IsRootDir {
                root_blob_id,
                atime_update_behavior: _,
            } => callback.on_is_rootdir(*root_blob_id).await,
            Self::IsNotRootDir {
                #[cfg(not(feature = "ancestor_checks_on_move"))]
                parent_blob_id,
                #[cfg(feature = "ancestor_checks_on_move")]
                ancestors,
                name,
                blob_details,
                atime_update_behavior: _,
            } => {
                #[cfg(feature = "ancestor_checks_on_move")]
                let parent_blob_id = ancestors.last().unwrap();

                let mut parent_blob = Self::_load_parent_blob(blobstore, parent_blob_id).await?;
                let result = parent_blob
                    .with_lock(async |parent_blob| {
                        let parent_blob_dir = parent_blob
                            .as_dir_mut()
                            .map_err(|_| FsError::NodeIsNotADirectory)?;

                        // Also store any information we have into self.blob_details so we don't have to load the parent blob again if blob_details get queried later
                        let blob_details = blob_details
                            .get_or_try_init(async || get_blob_details(parent_blob_dir, name))
                            .await?;

                        callback
                            .on_is_not_rootdir(parent_blob_dir, name, blob_details)
                            .await
                    })
                    .await;
                parent_blob.async_drop().await?;
                result
            }
        }
    }

    pub async fn blob_id<B>(&self, blobstore: &ConcurrentFsBlobStore<B>) -> FsResult<BlobId>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
    {
        self.blob_details(blobstore)
            .await
            .map(|blob_details| blob_details.blob_id)
    }

    #[cfg(feature = "ancestor_checks_on_move")]
    pub fn ancestors(&self) -> &[BlobId] {
        match self {
            Self::IsRootDir { .. } => {
                // Return an empty array for the root dir
                &[]
            }
            Self::IsNotRootDir { ancestors, .. } => {
                // Return the stored ancestors
                ancestors
            }
        }
    }

    #[cfg(feature = "ancestor_checks_on_move")]
    pub async fn ancestors_and_self<B>(
        &self,
        blobstore: &ConcurrentFsBlobStore<B>,
    ) -> FsResult<AncestorChain>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
    {
        // TODO Both self.blob_id and self.ancestors() match over Self::{IsRootDir/IsNotRootDir}, can we combine that into just one branch?
        let self_blob_id = self.blob_id(&blobstore).await?;
        let ancestors_and_self = self
            .ancestors()
            .iter()
            .copied()
            .chain(std::iter::once(self_blob_id))
            .collect::<Box<[BlobId]>>();
        Ok(AncestorChain::new(ancestors_and_self))
    }

    #[cfg(not(feature = "ancestor_checks_on_move"))]
    pub async fn ancestors_and_self<B>(&self, blobstore: &FsBlobStore<B>) -> FsResult<AncestorChain>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
    {
        // In this case, we just return the self blob id since ancestor checks are disabled
        // for move operations.
        Ok(AncestorChain::new(self.blob_id(&blobstore).await?))
    }

    pub async fn node_type<B>(&self, blobstore: &ConcurrentFsBlobStore<B>) -> FsResult<BlobType>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
    {
        self.blob_details(blobstore)
            .await
            .map(|blob_details| blob_details.blob_type)
    }

    pub async fn blob_details<B>(
        &self,
        blobstore: &ConcurrentFsBlobStore<B>,
    ) -> FsResult<BlobDetails>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
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
                #[cfg(not(feature = "ancestor_checks_on_move"))]
                parent_blob_id,
                #[cfg(feature = "ancestor_checks_on_move")]
                ancestors,
                name,
                blob_details,
                atime_update_behavior: _,
            } => {
                #[cfg(feature = "ancestor_checks_on_move")]
                let parent_blob_id = ancestors.last().unwrap();

                Ok(*blob_details
                    .get_or_try_init(async || {
                        let parent_blob =
                            Self::_load_parent_blob(blobstore, parent_blob_id).await?;
                        with_async_drop_2!(parent_blob, {
                            parent_blob
                                .with_lock(async |parent_blob| {
                                    let parent_blob = parent_blob
                                        .as_dir()
                                        .map_err(|_| FsError::NodeIsNotADirectory)?;
                                    get_blob_details(parent_blob, name)
                                })
                                .await
                        })
                    })
                    .await?)
            }
        }
    }

    pub async fn load_blob<'a, B>(
        &self,
        blobstore: &'a ConcurrentFsBlobStore<B>,
    ) -> FsResult<AsyncDropGuard<ConcurrentFsBlob<'a, B>>>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
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

    pub fn as_file_mut<'s, B>(blob: &'s mut FsBlob<B>) -> FsResult<&'s mut FileBlob<B>>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
    {
        let blob_id = blob.blob_id();
        blob.as_file_mut().map_err(|err| {
            FsError::CorruptedFilesystem {
                // TODO Add to message what it actually is
                message: format!("Blob {:?} is listed as a file in its parent directory but is actually not a file: {err:?}", blob_id),
            }
        })
    }

    async fn load_lstat_size<B>(&self, blobstore: &ConcurrentFsBlobStore<B>) -> FsResult<NumBytes>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
    {
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

    pub async fn getattr<B>(&self, blobstore: &ConcurrentFsBlobStore<B>) -> FsResult<NodeAttrs>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
    {
        struct Callback<'d, 'e, B>
        where
            B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
            <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
        {
            this: &'d NodeInfo,
            blobstore: &'e ConcurrentFsBlobStore<B>,
        }
        impl<'d, 'e, B> CallbackWithParentBlob<B, NodeAttrs> for Callback<'d, 'e, B>
        where
            B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
            <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
        {
            async fn on_is_rootdir(self, _root_blob: BlobId) -> FsResult<NodeAttrs> {
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

            async fn on_is_not_rootdir<'a, 'b, 'c>(
                self,
                parent_blob: &'a mut DirBlob<B>,
                name: &'b PathComponent,
                _blob_details: &'c BlobDetails,
            ) -> FsResult<NodeAttrs> {
                let lstat_size = self.this.load_lstat_size(self.blobstore).await?;
                let entry = parent_blob
                    .entry_by_name(name)
                    .ok_or_else(|| FsError::NodeDoesNotExist)?;
                let r = Ok(dir_entry_to_node_attrs(entry, lstat_size));
                r
            }
        }

        self.with_parent_blob(
            blobstore,
            Callback {
                blobstore,
                this: self,
            },
        )
        .await
    }

    pub async fn setattr<B>(
        &self,
        blobstore: &ConcurrentFsBlobStore<B>,
        mode: Option<Mode>,
        uid: Option<cryfs_rustfs::Uid>,
        gid: Option<cryfs_rustfs::Gid>,
        size: Option<NumBytes>,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
        ctime: Option<SystemTime>,
    ) -> FsResult<NodeAttrs>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
    {
        // TODO Or is setting ctime allowed? What would it mean?
        assert!(ctime.is_none(), "Cannot set ctime via setattr");
        // TODO Improve concurrency
        if let Some(size) = size {
            self.truncate_file(blobstore, size).await?;
        }
        struct Callback<'d, 'e, B>
        where
            B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
            <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
        {
            this: &'d NodeInfo,
            blobstore: &'e ConcurrentFsBlobStore<B>,
            mode: Option<Mode>,
            uid: Option<cryfs_rustfs::Uid>,
            gid: Option<cryfs_rustfs::Gid>,
            atime: Option<SystemTime>,
            mtime: Option<SystemTime>,
            size: Option<NumBytes>,
        }
        impl<'d, 'e, B> CallbackWithParentBlob<B, NodeAttrs> for Callback<'d, 'e, B>
        where
            B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
            <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
        {
            async fn on_is_rootdir(self, _root_blob: BlobId) -> FsResult<NodeAttrs> {
                // We're the root dir
                // TODO What should we do here?
                Err(FsError::InvalidOperation)
            }

            async fn on_is_not_rootdir<'a, 'b, 'c>(
                self,
                parent_blob: &'a mut DirBlob<B>,
                name: &'b PathComponent,
                _blob_details: &'c BlobDetails,
            ) -> FsResult<NodeAttrs> {
                // TODO No Mode/Uid/Gid conversion
                let mode = self.mode.map(|mode| fs_types::Mode::from(u32::from(mode)));
                let uid = self.uid.map(|uid| fs_types::Uid::from(u32::from(uid)));
                let gid = self.gid.map(|gid| fs_types::Gid::from(u32::from(gid)));
                let lstat_size = self.this.load_lstat_size(self.blobstore).await?;
                // TODO Don't look up the entry by name twice when we have attrs and size.is_some(). Looking it up once should be enough.
                let result = parent_blob
                    .set_attr_of_entry_by_name(name, mode, uid, gid, self.atime, self.mtime)
                    .map(|result| dir_entry_to_node_attrs(result, lstat_size));
                // Even if other fields are `None` (i.e. we don't run chmod, chown, utime), we still need to update the mtime in a truncate operation
                if self.size.is_some() {
                    parent_blob.update_modification_timestamp_by_name(name)?;
                }
                result
            }
        }
        self.with_parent_blob(
            blobstore,
            Callback {
                this: self,
                blobstore,
                mode,
                uid,
                gid,
                atime,
                mtime,
                size,
            },
        )
        .await
    }

    pub async fn truncate_file<B>(
        &self,
        blobstore: &ConcurrentFsBlobStore<B>,
        new_size: NumBytes,
    ) -> FsResult<()>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
    {
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

    pub async fn concurrently_maybe_update_access_timestamp_in_parent<B, F>(
        &self,
        blobstore: &ConcurrentFsBlobStore<B>,
        concurrent_fn: impl AsyncFnOnce() -> FsResult<F>,
    ) -> FsResult<F>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
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
        blobstore: &ConcurrentFsBlobStore<B>,
        concurrent_fn: impl AsyncFnOnce() -> FsResult<F>,
    ) -> FsResult<F>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
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
        blobstore: &ConcurrentFsBlobStore<B>,
        atime_update_behavior: AtimeUpdateBehavior,
    ) -> FsResult<()>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
    {
        self._update_in_parent(blobstore, |parent, blob_id| {
            parent.maybe_update_access_timestamp_of_entry(blob_id, atime_update_behavior)
        })
        .await
    }

    pub async fn update_modification_timestamp_in_parent<B>(
        &self,
        blobstore: &ConcurrentFsBlobStore<B>,
    ) -> FsResult<()>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
    {
        self._update_in_parent(blobstore, |parent, blob_id| {
            parent.update_modification_timestamp_of_entry(blob_id)
        })
        .await
    }

    async fn _update_in_parent<B>(
        &self,
        blobstore: &ConcurrentFsBlobStore<B>,
        update_fn: impl FnOnce(&mut DirBlob<B>, &BlobId) -> FsResult<()>,
    ) -> FsResult<()>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
    {
        // TODO Ideally we'd do this without loading the parent blob (we've already loaded it right before to be able to load the actual blob),
        //      but if not possible, the least we can do is remember the current atime in BlobDetails on the first load before calling into here,
        //      and decide if we need to update the timestamp based on that before loading the parent blob. Avoiding a load when it doesn't need to be updated.
        //      Or at the very least, short circuit noatime to not load the parent blob.

        struct Callback<B, F>
        where
            B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
            <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
            F: FnOnce(&mut DirBlob<B>, &BlobId) -> FsResult<()>,
        {
            update_fn: F,
            _phantom: PhantomData<B>,
        }
        impl<B, F> CallbackWithParentBlob<B, ()> for Callback<B, F>
        where
            B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
            <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
            F: FnOnce(&mut DirBlob<B>, &BlobId) -> FsResult<()>,
        {
            async fn on_is_rootdir(self, _root_blob: BlobId) -> FsResult<()> {
                // We're the root dir
                //TODO Instead of doing nothing when we're the root directory, handle timestamps in the root dir correctly (and delete isRootDir() function)
                Ok(())
            }

            async fn on_is_not_rootdir<'a, 'b, 'c>(
                self,
                parent_blob: &'a mut DirBlob<B>,
                _name: &'b PathComponent,
                blob_details: &'c BlobDetails,
            ) -> FsResult<()> {
                (self.update_fn)(parent_blob, &blob_details.blob_id)
            }
        }
        self.with_parent_blob(
            blobstore,
            Callback {
                update_fn,
                _phantom: PhantomData,
            },
        )
        .await?;
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

fn get_blob_details<B>(parent_blob: &DirBlob<B>, name: &PathComponent) -> FsResult<BlobDetails>
where
    // TODO Do we really need B: 'static ?
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
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

pub struct AncestorChain {
    // When `ancestor_checks_on_move` feature is disabled, we only need the self blob id, no ancestors
    #[cfg(not(feature = "ancestor_checks_on_move"))]
    self_blob_id: BlobId,
    // This is the list of ancestors from the root to this node, including this node itself.
    #[cfg(feature = "ancestor_checks_on_move")]
    ancestors_and_self: Box<[BlobId]>,
}

impl AncestorChain {
    pub fn new(
        #[cfg(not(feature = "ancestor_checks_on_move"))] self_blob_id: BlobId,
        #[cfg(feature = "ancestor_checks_on_move")] ancestors_and_self: Box<[BlobId]>,
    ) -> Self {
        #[cfg(feature = "ancestor_checks_on_move")]
        assert!(
            !ancestors_and_self.is_empty(),
            "Ancestors should not be empty, there must be at least the self node"
        );
        Self {
            #[cfg(not(feature = "ancestor_checks_on_move"))]
            self_blob_id,
            #[cfg(feature = "ancestor_checks_on_move")]
            ancestors_and_self,
        }
    }

    pub fn self_blob_id(&self) -> &BlobId {
        #[cfg(not(feature = "ancestor_checks_on_move"))]
        return &self.self_blob_id;

        #[cfg(feature = "ancestor_checks_on_move")]
        return self
            .ancestors_and_self
            .last()
            .expect("Ancestors should not be empty");
    }

    #[cfg(feature = "ancestor_checks_on_move")]
    pub fn ancestors_and_self(self) -> Box<[BlobId]> {
        // Return the stored ancestors
        self.ancestors_and_self
    }

    #[cfg(not(feature = "ancestor_checks_on_move"))]
    pub fn ancestors_and_self(self) -> BlobId {
        self.self_blob_id
    }
}
