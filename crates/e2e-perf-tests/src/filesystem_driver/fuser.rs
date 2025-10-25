use anyhow::Result;
use async_trait::async_trait;
use futures::try_join;
use std::{
    fmt::Debug,
    marker::PhantomData,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use super::FilesystemDriver;
use super::common::request_info;
use cryfs_blobstore::{BlobStoreOnBlocks, TrackingBlobStore};
use cryfs_blockstore::{
    DynBlockStore, HLSharedBlockStore, HLTrackingBlockStore, LockingBlockStore,
};
use cryfs_filesystem::filesystem::CryDevice;
use cryfs_rustfs::{
    AbsolutePath, AbsolutePathBuf, Callback, FileHandle, FsResult, Gid, InodeNumber, Mode,
    NodeAttrs, NodeKind, NumBytes, OpenFlags, PathComponent, Statfs, Uid,
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
    async fn load_inodes<R>(
        node1: &Option<Self::NodeHandle>,
        node2: &Option<Self::NodeHandle>,
        fs: &impl AsyncFilesystemLL,
        callback: impl AsyncFnOnce(InodeNumber, InodeNumber) -> FsResult<R>,
    ) -> FsResult<R>;
    fn make_inode(
        parent: Option<Self::NodeHandle>,
        child_name: &PathComponent,
        child_ino: InodeNumber,
    ) -> Self::NodeHandle;
    async fn reset_cache(fs: &ObjectBasedFsAdapterLL<Device>);
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
    async fn load_inodes<R>(
        node1: &Option<Self::NodeHandle>,
        node2: &Option<Self::NodeHandle>,
        _fs: &impl AsyncFilesystemLL,
        callback: impl AsyncFnOnce(InodeNumber, InodeNumber) -> FsResult<R>,
    ) -> FsResult<R> {
        let ino1 = node1.unwrap_or(FUSE_ROOT_ID);
        let ino2 = node2.unwrap_or(FUSE_ROOT_ID);
        callback(ino1, ino2).await
    }
    fn make_inode(
        _parent: Option<InodeNumber>,
        _child_name: &PathComponent,
        child_ino: InodeNumber,
    ) -> Self::NodeHandle {
        child_ino
    }
    async fn reset_cache(_fs: &ObjectBasedFsAdapterLL<Device>) {
        // No-op for cached behavior
    }
}
/// This is a version of the [FuserFilesystemDriver] that doesn't cache inodes, i.e. it simulates operation counts for the scenario where a filesystem operation
/// is run on a path that wasn't loaded into the inode cache yet. This means the operation has to load the whole path from the root dir to the node being operated on.
pub struct WithoutInodeCache;
impl WithoutInodeCache {
    /// Lookup every node on the path from the relative root to the given node
    async fn load_inodes_on_path(
        relative_root: InodeNumber,
        path: impl Iterator<Item = &PathComponent>,
        fs: &impl AsyncFilesystemLL,
    ) -> FsResult<Vec<InodeNumber>> {
        let mut inos = vec![relative_root];
        for component in path {
            let parent_ino = *inos.last().unwrap();
            let child_ino = fs
                .lookup(&request_info(), parent_ino, component)
                .await?
                .ino
                .handle;
            inos.push(child_ino);
        }
        Ok(inos)
    }
}
impl FuserCacheBehavior for WithoutInodeCache {
    type NodeHandle = AbsolutePathBuf;
    async fn load_inode<R>(
        path: &Option<AbsolutePathBuf>,
        fs: &impl AsyncFilesystemLL,
        callback: impl AsyncFnOnce(InodeNumber) -> FsResult<R>,
    ) -> FsResult<R> {
        let path = path.clone().unwrap_or_else(AbsolutePathBuf::root);
        let inos = Self::load_inodes_on_path(FUSE_ROOT_ID, path.iter(), fs).await?;

        let result = callback(*inos.last().unwrap()).await;

        for ino in inos.iter().skip(1).rev() {
            AsyncFilesystemLL::forget(fs, &request_info(), *ino, 1).await?;
        }

        result
    }
    async fn load_inodes<R>(
        path1: &Option<AbsolutePathBuf>,
        path2: &Option<AbsolutePathBuf>,
        fs: &impl AsyncFilesystemLL,
        callback: impl AsyncFnOnce(InodeNumber, InodeNumber) -> FsResult<R>,
    ) -> FsResult<R> {
        let path1 = path1.clone().unwrap_or_else(AbsolutePathBuf::root);
        let path1 = path1.into_iter();
        let path2 = path2.clone().unwrap_or_else(AbsolutePathBuf::root);
        let path2 = path2.into_iter();
        let (common, relative1, relative2) = _split_common(path1, path2);

        let common_inodes = Self::load_inodes_on_path(FUSE_ROOT_ID, common.into_iter(), fs).await?;
        let (inodes1, inodes2) = try_join!(
            Self::load_inodes_on_path(*common_inodes.last().unwrap(), relative1, fs),
            Self::load_inodes_on_path(*common_inodes.last().unwrap(), relative2, fs),
        )?;

        let result = callback(*inodes1.last().unwrap(), *inodes2.last().unwrap()).await;

        let cleanup1 = async {
            for ino in inodes1.iter().skip(1).rev() {
                AsyncFilesystemLL::forget(fs, &request_info(), *ino, 1).await?;
            }
            Ok(())
        };
        let cleanup2 = async {
            for ino in inodes2.iter().skip(1).rev() {
                AsyncFilesystemLL::forget(fs, &request_info(), *ino, 1).await?;
            }
            Ok(())
        };
        let cleanup_common = async {
            for ino in common_inodes.iter().skip(1).rev() {
                AsyncFilesystemLL::forget(fs, &request_info(), *ino, 1).await?;
            }
            Ok(())
        };
        try_join!(cleanup1, cleanup2)?;
        cleanup_common.await?;

        result
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
    async fn reset_cache(fs: &ObjectBasedFsAdapterLL<Device>) {
        fs.reset_cache().await;
    }
}

/// Takes two iterators, returns common elements in the prefix in a Vector, and the remaining elements of both iterators as two separate iterators.
fn _split_common<T: PartialEq + Eq, I1: Iterator<Item = T>, I2: Iterator<Item = T>>(
    iter1: I1,
    iter2: I2,
) -> (Vec<T>, impl Iterator<Item = T>, impl Iterator<Item = T>) {
    let mut common = Vec::new();
    let mut iter1 = iter1.peekable();
    let mut iter2 = iter2.peekable();

    while iter1.peek() == iter2.peek() && iter1.peek().is_some() {
        let Some(item) = iter1.next() else {
            unreachable!();
        };
        iter2.next();
        common.push(item);
    }

    (common, iter1, iter2)
}

type Device = CryDevice<
    AsyncDropArc<
        TrackingBlobStore<
            BlobStoreOnBlocks<
                HLSharedBlockStore<HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>>,
            >,
        >,
    >,
>;

/// A [FilesystemDriver] implementation using the low-level Api from [rustfs], i.e. [ObjectBasedFsAdapterLL].
/// Its caching behavior can be configured using [FuserCacheBehavior].
pub struct FuserFilesystemDriver<C: FuserCacheBehavior> {
    fs: AsyncDropGuard<ObjectBasedFsAdapterLL<Device>>,
    _c: PhantomData<C>,
}

impl<C: FuserCacheBehavior> Debug for FuserFilesystemDriver<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FuserFilesystemDriver")
    }
}

impl<C: FuserCacheBehavior> FilesystemDriver for FuserFilesystemDriver<C> {
    type NodeHandle = C::NodeHandle;

    type FileHandle = FileHandle;

    async fn new(device: AsyncDropGuard<Device>) -> AsyncDropGuard<Self> {
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

    async fn reset_cache(&self) {
        C::reset_cache(&self.fs).await;
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

    async fn fgetattr(
        &self,
        node: Self::NodeHandle,
        open_file: &FileHandle,
    ) -> FsResult<NodeAttrs> {
        C::load_inode(&Some(node), &*self.fs, async |ino| {
            self.fs
                .getattr(&request_info(), ino, Some(*open_file))
                .await
                .map(|attrs| attrs.attr)
        })
        .await
    }

    async fn chmod(&self, node: Option<Self::NodeHandle>, mode: Mode) -> FsResult<()> {
        C::load_inode(&node, &*self.fs, async |ino| {
            self.fs
                .setattr(
                    &request_info(),
                    ino,
                    Some(mode),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .await?;
            Ok(())
        })
        .await
    }

    async fn fchmod(
        &self,
        node: Self::NodeHandle,
        open_file: &FileHandle,
        mode: Mode,
    ) -> FsResult<()> {
        C::load_inode(&Some(node), &*self.fs, async |ino| {
            self.fs
                .setattr(
                    &request_info(),
                    ino,
                    Some(mode),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(*open_file),
                    None,
                    None,
                    None,
                    None,
                )
                .await?;
            Ok(())
        })
        .await
    }

    async fn chown(
        &self,
        node: Option<Self::NodeHandle>,
        uid: Option<Uid>,
        gid: Option<Gid>,
    ) -> FsResult<()> {
        C::load_inode(&node, &*self.fs, async |ino| {
            self.fs
                .setattr(
                    &request_info(),
                    ino,
                    None,
                    uid,
                    gid,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .await?;
            Ok(())
        })
        .await
    }

    async fn fchown(
        &self,
        node: Self::NodeHandle,
        open_file: &FileHandle,
        uid: Option<Uid>,
        gid: Option<Gid>,
    ) -> FsResult<()> {
        C::load_inode(&Some(node), &*self.fs, async |ino| {
            self.fs
                .setattr(
                    &request_info(),
                    ino,
                    None,
                    uid,
                    gid,
                    None,
                    None,
                    None,
                    None,
                    Some(*open_file),
                    None,
                    None,
                    None,
                    None,
                )
                .await?;
            Ok(())
        })
        .await
    }

    async fn truncate(&self, node: Option<Self::NodeHandle>, size: NumBytes) -> FsResult<()> {
        C::load_inode(&node, &*self.fs, async |ino| {
            self.fs
                .setattr(
                    &request_info(),
                    ino,
                    None,
                    None,
                    None,
                    Some(size),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .await?;
            Ok(())
        })
        .await
    }

    async fn ftruncate(
        &self,
        node: Self::NodeHandle,
        open_file: &FileHandle,
        size: NumBytes,
    ) -> FsResult<()> {
        C::load_inode(&Some(node), &*self.fs, async |ino| {
            self.fs
                .setattr(
                    &request_info(),
                    ino,
                    None,
                    None,
                    None,
                    Some(size),
                    None,
                    None,
                    None,
                    Some(*open_file),
                    None,
                    None,
                    None,
                    None,
                )
                .await?;
            Ok(())
        })
        .await
    }

    async fn utimens(
        &self,
        node: Option<Self::NodeHandle>,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
    ) -> FsResult<()> {
        C::load_inode(&node, &*self.fs, async |ino| {
            self.fs
                .setattr(
                    &request_info(),
                    ino,
                    None,
                    None,
                    None,
                    None,
                    atime,
                    mtime,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .await?;
            Ok(())
        })
        .await
    }

    async fn futimens(
        &self,
        node: Self::NodeHandle,
        open_file: &FileHandle,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
    ) -> FsResult<()> {
        C::load_inode(&Some(node), &*self.fs, async |ino| {
            self.fs
                .setattr(
                    &request_info(),
                    ino,
                    None,
                    None,
                    None,
                    None,
                    atime,
                    mtime,
                    None,
                    Some(*open_file),
                    None,
                    None,
                    None,
                    None,
                )
                .await?;
            Ok(())
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
            // The fuse sequence for releasing a file in fuse is: first flush, then release
            self.fs.flush(&request_info(), ino, open_file, 0).await?;
            self.fs
                .release(&request_info(), ino, open_file, 0, None, false)
                .await
        })
        .await
    }

    async fn fsync(
        &self,
        node: Self::NodeHandle,
        open_file: &mut FileHandle,
        datasync: bool,
    ) -> FsResult<()> {
        C::load_inode(&Some(node), &*self.fs, async |ino| {
            self.fs
                .fsync(&request_info(), ino, *open_file, datasync)
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
    ) -> FsResult<Vec<(String, NodeKind)>> {
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

    async fn read(
        &self,
        node: Self::NodeHandle,
        open_file: &mut FileHandle,
        offset: NumBytes,
        size: NumBytes,
    ) -> FsResult<Vec<u8>> {
        let result = C::load_inode(&Some(node), &*self.fs, async |ino| {
            let result = Arc::new(Mutex::new(None));
            self.fs
                .read(
                    &request_info(),
                    ino,
                    *open_file,
                    offset,
                    size,
                    0,
                    None,
                    ReadCallbackImpl {
                        data: Arc::clone(&result),
                    },
                )
                .await;
            Arc::try_unwrap(result)
                .unwrap()
                .into_inner()
                .unwrap()
                .unwrap()
        })
        .await?;
        Ok(result)
    }

    async fn write(
        &self,
        node: Self::NodeHandle,
        open_file: &mut FileHandle,
        offset: NumBytes,
        data: Vec<u8>,
    ) -> FsResult<()> {
        let len = NumBytes::from(data.len() as u64);
        let reply = C::load_inode(&Some(node), &*self.fs, async |ino| {
            self.fs
                .write(&request_info(), ino, *open_file, offset, data, 0, 0, None)
                .await
        })
        .await?;
        assert_eq!(reply.written, len);
        Ok(())
    }

    async fn rename(
        &self,
        old_parent: Option<Self::NodeHandle>,
        old_name: &PathComponent,
        new_parent: Option<Self::NodeHandle>,
        new_name: &PathComponent,
    ) -> FsResult<()> {
        // Use [C::load_inodes] instead of [C::load_inode], because it loads the common ancestors only once.
        // Also, if both paths are identical, calling [Self::load_inode] twice may give them different
        // inode numbers, which is wrong since the fuser rename operation compares inode numbers to decide
        // if a file is moved between or within directories.
        C::load_inodes(
            &old_parent,
            &new_parent,
            &*self.fs,
            async |old_parent_ino, new_parent_ino| {
                self.fs
                    .rename(
                        &request_info(),
                        old_parent_ino,
                        old_name,
                        new_parent_ino,
                        new_name,
                        0, // flags - using 0 for default behavior
                    )
                    .await
            },
        )
        .await
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
    entries: Vec<(String, NodeKind)>,
}

impl ReplyDirectory for ReplyDirectoryImpl {
    fn add_self_reference(&mut self, _ino: InodeNumber, _offset: i64) -> ReplyDirectoryAddResult {
        self.entries.push((".".to_string(), NodeKind::Dir));
        ReplyDirectoryAddResult::NotFull
    }

    fn add_parent_reference(&mut self, _ino: InodeNumber, _offset: i64) -> ReplyDirectoryAddResult {
        self.entries.push(("..".to_string(), NodeKind::Dir));
        ReplyDirectoryAddResult::NotFull
    }

    fn add(
        &mut self,
        _ino: InodeNumber,
        _offset: i64,
        kind: NodeKind,
        name: &PathComponent,
    ) -> ReplyDirectoryAddResult {
        self.entries.push((name.to_string(), kind));
        ReplyDirectoryAddResult::NotFull
    }
}

#[derive(Default)]
struct ReadCallbackImpl {
    data: Arc<Mutex<Option<FsResult<Vec<u8>>>>>,
}

impl<'a> Callback<FsResult<&'a [u8]>, ()> for ReadCallbackImpl {
    fn call(self, result: FsResult<&'a [u8]>) {
        *self.data.lock().unwrap() = Some(result.map(|data| data.to_vec()));
    }
}
