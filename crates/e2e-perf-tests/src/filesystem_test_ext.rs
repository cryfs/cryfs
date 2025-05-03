use cryfs_blobstore::{BlobStoreOnBlocks, TrackingBlobStore};
use cryfs_blockstore::{
    DynBlockStore, HLSharedBlockStore, HLTrackingBlockStore, LockingBlockStore,
};
use cryfs_filesystem::filesystem::CryDevice;
use cryfs_rustfs::{
    AbsolutePath, AbsolutePathBuf, FsError, FsResult, InodeNumber, Mode, PathComponent,
    high_level_api::AsyncFilesystem,
    low_level_api::AsyncFilesystemLL,
    object_based_api::{FUSE_ROOT_ID, ObjectBasedFsAdapter, ObjectBasedFsAdapterLL},
};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};
use std::fmt::Debug;

use crate::fixture::request_info;

/// An interface abstracting over [AsyncFilesystem] and [AsyncFilesystemLL], offering common file system operations.
pub trait FilesystemTestExt: AsyncDrop + Debug {
    async fn new(
        device: AsyncDropGuard<
            CryDevice<
                AsyncDropArc<
                    TrackingBlobStore<
                        BlobStoreOnBlocks<
                            HLSharedBlockStore<
                                HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>,
                            >,
                        >,
                    >,
                >,
            >,
        >,
    ) -> AsyncDropGuard<Self>
    where
        Self: Sized;

    /// A handle to a given file system node. Can be an InodeNumber (for the fuser backend) or just the path of the node (for the fuse-mt backend).
    type NodeHandle: Debug + Clone;

    async fn init(&self) -> Result<(), FsError>;
    async fn destroy(&self);
    async fn mkdir(
        &self,
        parent: Option<Self::NodeHandle>,
        name: &PathComponent,
    ) -> FsResult<Self::NodeHandle>;
    async fn mkdir_recursive(&self, path: &AbsolutePath) -> FsResult<Self::NodeHandle> {
        let mut current_node = None;
        for component in path.iter() {
            current_node = Some(self.mkdir(current_node, &component).await?);
        }
        Ok(current_node.unwrap())
    }
    async fn create_and_open_file(
        &self,
        parent: Option<Self::NodeHandle>,
        name: &PathComponent,
    ) -> FsResult<Self::NodeHandle>;
}

async fn load_parent_inode_from_path<R>(
    fs: &ObjectBasedFsAdapterLL<
        CryDevice<
            AsyncDropArc<
                TrackingBlobStore<
                    BlobStoreOnBlocks<
                        HLSharedBlockStore<HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>>,
                    >,
                >,
            >,
        >,
    >,
    path: &AbsolutePath,
    callback: impl AsyncFnOnce(InodeNumber, &PathComponent) -> Result<R, FsError>,
) -> Result<R, FsError> {
    let (parent_path, name) = path.split_last().unwrap();
    let mut inos = vec![FUSE_ROOT_ID];
    for component in parent_path.iter() {
        let parent_ino = *inos.last().unwrap();
        let child_ino = AsyncFilesystemLL::lookup(fs, &request_info(), parent_ino, component)
            .await?
            .ino
            .handle;
        inos.push(child_ino);
    }
    let result = callback(*inos.last().unwrap(), name).await?;
    for ino in inos.iter().skip(1).rev() {
        AsyncFilesystemLL::forget(fs, &request_info(), *ino, 1).await?;
    }
    Ok(result)
}

impl FilesystemTestExt
    for ObjectBasedFsAdapterLL<
        CryDevice<
            AsyncDropArc<
                TrackingBlobStore<
                    BlobStoreOnBlocks<
                        HLSharedBlockStore<HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>>,
                    >,
                >,
            >,
        >,
    >
{
    type NodeHandle = InodeNumber;

    async fn new(
        device: AsyncDropGuard<
            CryDevice<
                AsyncDropArc<
                    TrackingBlobStore<
                        BlobStoreOnBlocks<
                            HLSharedBlockStore<
                                HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>,
                            >,
                        >,
                    >,
                >,
            >,
        >,
    ) -> AsyncDropGuard<Self> {
        ObjectBasedFsAdapterLL::new(|_uid, _gid| device)
    }

    async fn init(&self) -> FsResult<()> {
        AsyncFilesystemLL::init(self, &request_info()).await
    }

    async fn destroy(&self) {
        AsyncFilesystemLL::destroy(self).await;
    }

    async fn mkdir(
        &self,
        parent: Option<InodeNumber>,
        name: &PathComponent,
    ) -> FsResult<InodeNumber> {
        Ok(AsyncFilesystemLL::mkdir(
            self,
            &request_info(),
            parent.unwrap_or(FUSE_ROOT_ID),
            name,
            Mode::default().add_dir_flag(),
            0,
        )
        .await?
        .ino
        .handle)
    }

    async fn create_and_open_file(
        &self,
        parent: Option<Self::NodeHandle>,
        name: &PathComponent,
    ) -> FsResult<InodeNumber> {
        Ok(AsyncFilesystemLL::create(
            self,
            &request_info(),
            parent.unwrap_or(FUSE_ROOT_ID),
            name,
            Mode::default().add_file_flag(),
            0,
            0,
        )
        .await?
        .ino
        .handle)
    }
}

impl FilesystemTestExt
    for ObjectBasedFsAdapter<
        CryDevice<
            AsyncDropArc<
                TrackingBlobStore<
                    BlobStoreOnBlocks<
                        HLSharedBlockStore<HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>>,
                    >,
                >,
            >,
        >,
    >
{
    type NodeHandle = AbsolutePathBuf;

    async fn new(
        device: AsyncDropGuard<
            CryDevice<
                AsyncDropArc<
                    TrackingBlobStore<
                        BlobStoreOnBlocks<
                            HLSharedBlockStore<
                                HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>,
                            >,
                        >,
                    >,
                >,
            >,
        >,
    ) -> AsyncDropGuard<Self> {
        ObjectBasedFsAdapter::new(|_uid, _gid| device)
    }

    async fn init(&self) -> FsResult<()> {
        AsyncFilesystem::init(self, request_info()).await
    }

    async fn destroy(&self) {
        AsyncFilesystem::destroy(self).await;
    }

    async fn mkdir(
        &self,
        parent: Option<AbsolutePathBuf>,
        name: &PathComponent,
    ) -> FsResult<AbsolutePathBuf> {
        let path = parent.unwrap_or_else(AbsolutePathBuf::root).join(name);
        AsyncFilesystem::mkdir(self, request_info(), &path, Mode::default().add_dir_flag()).await?;
        Ok(path)
    }

    async fn create_and_open_file(
        &self,
        parent: Option<AbsolutePathBuf>,
        name: &PathComponent,
    ) -> FsResult<AbsolutePathBuf> {
        let path = parent.unwrap_or_else(AbsolutePathBuf::root).join(name);
        AsyncFilesystem::create(
            self,
            request_info(),
            &path,
            Mode::default().add_file_flag(),
            0,
        )
        .await?;
        Ok(path)
    }
}
