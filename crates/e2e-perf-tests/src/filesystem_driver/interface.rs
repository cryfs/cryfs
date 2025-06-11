use std::fmt::Debug;

use cryfs_blobstore::{BlobStoreOnBlocks, TrackingBlobStore};
use cryfs_blockstore::{
    DynBlockStore, HLSharedBlockStore, HLTrackingBlockStore, LockingBlockStore,
};
use cryfs_filesystem::filesystem::CryDevice;
use cryfs_rustfs::{
    AbsolutePath, AbsolutePathBuf, FsError, FsResult, Gid, Mode, NodeAttrs, NodeKind, NumBytes,
    PathComponent, Statfs, Uid,
};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};
use std::time::SystemTime;

/// An interface abstracting over [AsyncFilesystem] and [AsyncFilesystemLL], offering common file system operations.
pub trait FilesystemDriver: AsyncDrop + Debug {
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

    // A handle to an open file
    type FileHandle;

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

    async fn create_file(
        &self,
        parent: Option<Self::NodeHandle>,
        name: &PathComponent,
    ) -> FsResult<Self::NodeHandle>;

    async fn create_and_open_file(
        &self,
        parent: Option<Self::NodeHandle>,
        name: &PathComponent,
    ) -> FsResult<(Self::NodeHandle, Self::FileHandle)>;

    async fn create_symlink(
        &self,
        parent: Option<Self::NodeHandle>,
        name: &PathComponent,
        target: &AbsolutePath,
    ) -> FsResult<Self::NodeHandle>;

    async fn unlink(&self, parent: Option<Self::NodeHandle>, name: &PathComponent) -> FsResult<()>;

    async fn rmdir(&self, parent: Option<Self::NodeHandle>, name: &PathComponent) -> FsResult<()>;

    async fn lookup(
        &self,
        parent: Option<Self::NodeHandle>,
        name: &PathComponent,
    ) -> FsResult<Self::NodeHandle>;

    async fn getattr(&self, node: Option<Self::NodeHandle>) -> FsResult<NodeAttrs>;

    async fn fgetattr(
        &self,
        node: Self::NodeHandle,
        open_file: &Self::FileHandle,
    ) -> FsResult<NodeAttrs>;

    async fn chmod(&self, node: Option<Self::NodeHandle>, mode: Mode) -> FsResult<()>;

    async fn fchmod(
        &self,
        node: Self::NodeHandle,
        open_file: &Self::FileHandle,
        mode: Mode,
    ) -> FsResult<()>;

    async fn chown(
        &self,
        node: Option<Self::NodeHandle>,
        uid: Option<Uid>,
        gid: Option<Gid>,
    ) -> FsResult<()>;

    async fn fchown(
        &self,
        node: Self::NodeHandle,
        open_file: &Self::FileHandle,
        uid: Option<Uid>,
        gid: Option<Gid>,
    ) -> FsResult<()>;

    async fn truncate(&self, node: Option<Self::NodeHandle>, size: NumBytes) -> FsResult<()>;

    async fn ftruncate(
        &self,
        node: Self::NodeHandle,
        open_file: &Self::FileHandle,
        size: NumBytes,
    ) -> FsResult<()>;

    async fn utimens(
        &self,
        node: Option<Self::NodeHandle>,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
    ) -> FsResult<()>;

    async fn futimens(
        &self,
        node: Self::NodeHandle,
        open_file: &Self::FileHandle,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
    ) -> FsResult<()>;

    async fn readlink(&self, node: Self::NodeHandle) -> FsResult<AbsolutePathBuf>;

    async fn open(&self, node: Self::NodeHandle) -> FsResult<Self::FileHandle>;

    async fn release(&self, node: Self::NodeHandle, open_file: Self::FileHandle) -> FsResult<()>;

    async fn statfs(&self) -> FsResult<Statfs>;

    async fn rename(
        &self,
        old_parent: Option<Self::NodeHandle>,
        old_name: &PathComponent,
        new_parent: Option<Self::NodeHandle>,
        new_name: &PathComponent,
    ) -> FsResult<()>;

    async fn readdir(&self, node: Option<Self::NodeHandle>) -> FsResult<Vec<(String, NodeKind)>>;

    async fn read(
        &self,
        node: Self::NodeHandle,
        open_file: &mut Self::FileHandle,
        offset: NumBytes,
        size: NumBytes,
    ) -> FsResult<Vec<u8>>;

    async fn write(
        &self,
        node: Self::NodeHandle,
        open_file: &mut Self::FileHandle,
        offset: NumBytes,
        data: Vec<u8>,
    ) -> FsResult<()>;

    async fn fsync(
        &self,
        node: Self::NodeHandle,
        open_file: &mut Self::FileHandle,
        datasync: bool,
    ) -> FsResult<()>;
}
