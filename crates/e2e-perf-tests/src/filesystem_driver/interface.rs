use std::fmt::Debug;

use cryfs_blobstore::{BlobStoreOnBlocks, TrackingBlobStore};
use cryfs_blockstore::{
    DynBlockStore, HLSharedBlockStore, HLTrackingBlockStore, LockingBlockStore,
};
use cryfs_filesystem::filesystem::CryDevice;
use cryfs_rustfs::{
    AbsolutePath, AbsolutePathBuf, FileHandle, FsError, FsResult, NodeAttrs, PathComponent, Statfs,
};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};

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
    ) -> FsResult<(Self::NodeHandle, FileHandle)>;

    async fn create_symlink(
        &self,
        parent: Option<Self::NodeHandle>,
        name: &PathComponent,
        target: &AbsolutePath,
    ) -> FsResult<Self::NodeHandle>;

    async fn lookup(
        &self,
        parent: Option<Self::NodeHandle>,
        name: &PathComponent,
    ) -> FsResult<Self::NodeHandle>;

    async fn getattr(&self, node: Option<Self::NodeHandle>) -> FsResult<NodeAttrs>;

    async fn fgetattr(&self, node: Self::NodeHandle, open_file: FileHandle) -> FsResult<NodeAttrs>;

    async fn readlink(&self, node: Self::NodeHandle) -> FsResult<AbsolutePathBuf>;

    async fn open(&self, node: Self::NodeHandle) -> FsResult<FileHandle>;

    async fn statfs(&self) -> FsResult<Statfs>;
}
