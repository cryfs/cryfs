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
    AbsolutePath, AbsolutePathBuf, Callback, FileHandle, FsResult, InodeNumber, Mode, NodeAttrs,
    NodeKind, NumBytes, OpenFlags, PathComponent, PathComponentBuf, Statfs,
    low_level_api::{AsyncFilesystemLL, ReplyDirectory, ReplyDirectoryAddResult},
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

    async fn create_file(
        &self,
        parent: Option<Self::NodeHandle>,
        name: &PathComponent,
    ) -> FsResult<Self::NodeHandle> {
        let new_file = C::load_inode(&parent, &*self.fs, async |parent_ino| {
            let open_file = self
                .fs
                .create(
                    &request_info(),
                    parent_ino,
                    name,
                    Mode::default().add_file_flag(),
                    0,
                    0,
                )
                .await?;
            self.fs
                .release(
                    &request_info(),
                    open_file.ino.handle,
                    open_file.fh,
                    0,
                    None,
                    false,
                )
                .await?;
            Ok(open_file.ino)
        })
        .await?;
        Ok(C::make_inode(parent, name, new_file.handle))
    }

    async fn create_and_open_file(
        &self,
        parent: Option<Self::NodeHandle>,
        name: &PathComponent,
    ) -> FsResult<(Self::NodeHandle, FileHandle)> {
        let new_file = C::load_inode(&parent, &*self.fs, async |parent_ino| {
            Ok(self
                .fs
                .create(
                    &request_info(),
                    parent_ino,
                    name,
                    Mode::default().add_file_flag(),
                    0,
                    0,
                )
                .await?)
        })
        .await?;
        Ok((
            C::make_inode(parent, name, new_file.ino.handle),
            new_file.fh,
        ))
    }

    async fn create_symlink(
        &self,
        parent: Option<Self::NodeHandle>,
        name: &PathComponent,
        target: &AbsolutePath,
    ) -> FsResult<Self::NodeHandle> {
        let new_dir = C::load_inode(&parent, &*self.fs, async |parent_ino| {
            self.fs
                .symlink(&request_info(), parent_ino, name, target)
                .await
        })
        .await?;
        Ok(C::make_inode(parent, name, new_dir.ino.handle))
    }

    async fn lookup(
        &self,
        parent: Option<Self::NodeHandle>,
        name: &PathComponent,
    ) -> FsResult<Self::NodeHandle> {
        let new_file = C::load_inode(&parent, &*self.fs, async |parent_ino| {
            self.fs.lookup(&request_info(), parent_ino, name).await
        })
        .await?;
        Ok(C::make_inode(parent, name, new_file.ino.handle))
    }

    async fn getattr(&self, node: Option<Self::NodeHandle>) -> FsResult<NodeAttrs> {
        C::load_inode(&node, &*self.fs, async |ino| {
            self.fs
                .getattr(&request_info(), ino, None)
                .await
                .map(|attrs| attrs.attr)
        })
        .await
    }

    async fn fgetattr(&self, node: Self::NodeHandle, open_file: FileHandle) -> FsResult<NodeAttrs> {
        C::load_inode(&Some(node), &*self.fs, async |ino| {
            self.fs
                .getattr(&request_info(), ino, Some(open_file))
                .await
                .map(|attrs| attrs.attr)
        })
        .await
    }

    async fn readlink(&self, node: Self::NodeHandle) -> FsResult<AbsolutePathBuf> {
        struct ToOwnedCallback;
        impl<'a> Callback<FsResult<&'a str>, FsResult<String>> for ToOwnedCallback {
            fn call(self, target: FsResult<&str>) -> FsResult<String> {
                target.map(|s| s.to_owned())
            }
        }
        let target = C::load_inode(&Some(node), &*self.fs, async |ino| {
            self.fs
                .readlink(&request_info(), ino, ToOwnedCallback)
                .await
        })
        .await?;
        Ok(AbsolutePathBuf::try_from_string(target).unwrap())
    }

    async fn unlink(&self, parent: Option<Self::NodeHandle>, name: &PathComponent) -> FsResult<()> {
        C::load_inode(&parent, &*self.fs, async |parent_ino| {
            self.fs.unlink(&request_info(), parent_ino, name).await
        })
        .await
    }

    async fn rmdir(&self, parent: Option<Self::NodeHandle>, name: &PathComponent) -> FsResult<()> {
        C::load_inode(&parent, &*self.fs, async |parent_ino| {
            self.fs.rmdir(&request_info(), parent_ino, name).await
        })
        .await
    }

    async fn open(&self, node: Self::NodeHandle) -> FsResult<FileHandle> {
        let open_file = C::load_inode(&Some(node), &*self.fs, async |ino| {
            self.fs
                .open(&request_info(), ino, OpenFlags::ReadWrite)
                .await
        })
        .await?;
        Ok(open_file.fh)
    }

    async fn release(&self, node: Self::NodeHandle, open_file: FileHandle) -> FsResult<()> {
        C::load_inode(&Some(node), &*self.fs, async |ino| {
            self.fs
                .release(&request_info(), ino, open_file, 0, None, false)
                .await
        })
        .await
    }

    async fn statfs(&self) -> FsResult<Statfs> {
        self.fs.statfs(&request_info(), FUSE_ROOT_ID).await
    }

    async fn readdir(
        &self,
        // TODO Instead of all these operations taking Option<NodeHandle>, would be nicer to just have a FilesystemDriver::rootdir() method that returns the root dir handle and then pass in the rootdir node handle
        node: Option<Self::NodeHandle>,
    ) -> FsResult<Vec<(PathComponentBuf, NodeKind)>> {
        let dir = C::load_inode(&node, &*self.fs, async |ino| {
            let fh = self.fs.opendir(&request_info(), ino, 0).await?.fh;
            let mut reply = ReplyDirectoryImpl::default();
            self.fs
                .readdir(&request_info(), ino, fh, 0, &mut reply)
                .await?;
            Ok(reply.entries)
        })
        .await?;
        Ok(dir)
    }

    async fn write(
        &self,
        node: Self::NodeHandle,
        open_file: FileHandle,
        offset: NumBytes,
        data: Vec<u8>,
    ) -> FsResult<()> {
        let len = NumBytes::from(data.len() as u64);
        let reply = C::load_inode(&Some(node), &*self.fs, async |ino| {
            self.fs
                .write(&request_info(), ino, open_file, offset, data, 0, 0, None)
                .await
        })
        .await?;
        assert_eq!(reply.written, len);
        Ok(())
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

#[derive(Default)]
struct ReplyDirectoryImpl {
    entries: Vec<(PathComponentBuf, NodeKind)>,
}

impl ReplyDirectory for ReplyDirectoryImpl {
    fn add(
        &mut self,
        _ino: InodeNumber,
        _offset: i64,
        kind: NodeKind,
        name: &PathComponent,
    ) -> ReplyDirectoryAddResult {
        self.entries.push((name.to_owned(), kind));
        ReplyDirectoryAddResult::NotFull
    }
}
