use anyhow::Result;
use async_trait::async_trait;
use std::{fmt::Debug, marker::PhantomData};

use super::FilesystemDriver;
use crate::fixture::request_info;
use cryfs_blobstore::{BlobStoreOnBlocks, TrackingBlobStore};
use cryfs_blockstore::{
    DynBlockStore, HLSharedBlockStore, HLTrackingBlockStore, LockingBlockStore,
};
use cryfs_filesystem::filesystem::CryDevice;
use cryfs_rustfs::{
    AbsolutePathBuf, FsResult, InodeNumber, Mode, PathComponent,
    low_level_api::AsyncFilesystemLL,
    object_based_api::{FUSE_ROOT_ID, ObjectBasedFsAdapterLL},
};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};

pub trait FuserCacheBehavior: Send + Sync {
    type NodeHandle: Debug + Clone;
    async fn load_inode<R>(
        node: &Option<Self::NodeHandle>,
        fs: &impl AsyncFilesystemLL,
        callback: impl AsyncFnOnce(InodeNumber) -> FsResult<R>,
    ) -> FsResult<R>;
    fn make_inode(
        parent: Option<Self::NodeHandle>,
        child_name: &PathComponent,
        child_ino: InodeNumber,
    ) -> Self::NodeHandle;
}

/// This is a version of the [FuserFilesystemDriver] that caches inodes, i.e. it simulates operation counts for the scenario where a filesystem operation
/// is run on a path that was already loaded into fuser's inode cache. This means the operation can directly run on that preloaded inode and doesn't have to load the whole path from the root dir to the node being operated on.
pub struct WithInodeCache;
impl FuserCacheBehavior for WithInodeCache {
    type NodeHandle = InodeNumber;
    async fn load_inode<R>(
        node: &Option<Self::NodeHandle>,
        _fs: &impl AsyncFilesystemLL,
        callback: impl AsyncFnOnce(InodeNumber) -> FsResult<R>,
    ) -> FsResult<R> {
        let inode = node.unwrap_or(FUSE_ROOT_ID);
        callback(inode).await
    }
    fn make_inode(
        _parent: Option<InodeNumber>,
        _child_name: &PathComponent,
        child_ino: InodeNumber,
    ) -> Self::NodeHandle {
        child_ino
    }
}
/// This is a version of the [FuserFilesystemDriver] that doesn't cache inodes, i.e. it simulates operation counts for the scenario where a filesystem operation
/// is run on a path that wasn't loaded into the inode cache yet. This means the operation has to load the whole path from the root dir to the node being operated on.
pub struct WithoutInodeCache;
impl FuserCacheBehavior for WithoutInodeCache {
    type NodeHandle = AbsolutePathBuf;
    async fn load_inode<R>(
        path: &Option<AbsolutePathBuf>,
        fs: &impl AsyncFilesystemLL,
        callback: impl AsyncFnOnce(InodeNumber) -> FsResult<R>,
    ) -> FsResult<R> {
        let path = path.clone().unwrap_or_else(AbsolutePathBuf::root);
        // Lookup every node on the path from the root to the given node
        let mut inos = vec![FUSE_ROOT_ID];
        for component in path.iter() {
            let parent_ino = *inos.last().unwrap();
            let child_ino = fs
                .lookup(&request_info(), parent_ino, component)
                .await?
                .ino
                .handle;
            inos.push(child_ino);
        }
        let result = callback(*inos.last().unwrap()).await?;
        for ino in inos.iter().skip(1).rev() {
            AsyncFilesystemLL::forget(fs, &request_info(), *ino, 1).await?;
        }
        Ok(result)
    }
    fn make_inode(
        parent: Option<AbsolutePathBuf>,
        child_name: &PathComponent,
        _child_ino: InodeNumber,
    ) -> Self::NodeHandle {
        parent
            .unwrap_or_else(AbsolutePathBuf::root)
            .join(child_name)
    }
}

pub struct FuserFilesystemDriver<C: FuserCacheBehavior> {
    fs: AsyncDropGuard<
        ObjectBasedFsAdapterLL<
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
    >,
    _c: PhantomData<C>,
}

impl<C: FuserCacheBehavior> Debug for FuserFilesystemDriver<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FuserFilesystemDriver")
    }
}

impl<C: FuserCacheBehavior> FilesystemDriver for FuserFilesystemDriver<C> {
    type NodeHandle = C::NodeHandle;

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
        AsyncDropGuard::new(FuserFilesystemDriver {
            fs: ObjectBasedFsAdapterLL::new(|_uid, _gid| device),
            _c: PhantomData,
        })
    }

    async fn init(&self) -> FsResult<()> {
        self.fs.init(&request_info()).await
    }

    async fn destroy(&self) {
        self.fs.destroy().await;
    }

    async fn mkdir(
        &self,
        parent: Option<Self::NodeHandle>,
        name: &PathComponent,
    ) -> FsResult<Self::NodeHandle> {
        let new_dir = C::load_inode(&parent, &*self.fs, async |parent_ino| {
            self.fs
                .mkdir(
                    &request_info(),
                    parent_ino,
                    name,
                    Mode::default().add_dir_flag(),
                    0,
                )
                .await
        })
        .await?;
        Ok(C::make_inode(parent, name, new_dir.ino.handle))
    }

    async fn create_and_open_file(
        &self,
        parent: Option<Self::NodeHandle>,
        name: &PathComponent,
    ) -> FsResult<Self::NodeHandle> {
        let new_file = C::load_inode(&parent, &*self.fs, async |parent_ino| {
            self.fs
                .create(
                    &request_info(),
                    parent_ino,
                    name,
                    Mode::default().add_file_flag(),
                    0,
                    0,
                )
                .await
        })
        .await?;
        Ok(C::make_inode(parent, name, new_file.ino.handle))
    }
}

#[async_trait]
impl<C: FuserCacheBehavior> AsyncDrop for FuserFilesystemDriver<C> {
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<()> {
        self.fs.async_drop().await?;
        Ok(())
    }
}
