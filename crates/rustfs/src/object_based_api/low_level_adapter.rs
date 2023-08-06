use async_trait::async_trait;
use futures::stream::{FuturesOrdered, StreamExt};
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

use super::utils::MaybeInitializedFs;
use super::{Device, Dir, File, Node, OpenFile, Symlink};
use crate::common::{
    AbsolutePath, DirEntry, FileHandle, FileHandleWithGeneration, FsError, FsResult, Gid,
    HandleMap, Mode, NodeKind, NumBytes, OpenFlags, PathComponent, RequestInfo, Statfs, Uid,
};
use crate::low_level_api::{
    AsyncFilesystemLL, IntoFsLL, ReplyAttr, ReplyBmap, ReplyCreate, ReplyEntry, ReplyLock,
    ReplyLseek, ReplyOpen, ReplyWrite,
};
use cryfs_utils::{
    async_drop::{with_async_drop, AsyncDrop, AsyncDropArc, AsyncDropGuard},
    with_async_drop_2,
};
use fuser::{KernelConfig, ReplyDirectory, ReplyDirectoryPlus, ReplyIoctl, ReplyXattr};

// TODO What are good TTLs here?
const TTL_LOOKUP: Duration = Duration::from_secs(1);
const TTL_GETATTR: Duration = Duration::from_secs(1);
const TTL_CREATE: Duration = Duration::from_secs(1);

// TODO Can we share more code with [super::high_level_adapter::ObjectBasedFsAdapter]?
pub struct ObjectBasedFsAdapterLL<Fs: Device>
where
    // TODO Is this send+sync bound only needed because fuse_mt goes multi threaded or would it also be required for fuser?
    Fs::OpenFile: Send + Sync,
{
    // TODO We only need the Arc<RwLock<...>> because of initialization. Is there a better way to do that?
    fs: Arc<RwLock<MaybeInitializedFs<Fs>>>,

    // TODO Do we need Arc for inodes?
    inodes: Arc<RwLock<AsyncDropGuard<HandleMap<AsyncDropArc<Fs::Node>>>>>,

    // TODO Can we improve concurrency by locking less in open_files and instead making OpenFileList concurrency safe somehow?
    open_files: tokio::sync::RwLock<AsyncDropGuard<HandleMap<Fs::OpenFile>>>,
}

impl<Fs: Device> ObjectBasedFsAdapterLL<Fs>
where
    // TODO Is this send+sync bound only needed because fuse_mt goes multi threaded or would it also be required for fuser?
    Fs::OpenFile: Send + Sync,
{
    pub fn new(fs: impl FnOnce(Uid, Gid) -> Fs + Send + Sync + 'static) -> AsyncDropGuard<Self> {
        let mut inodes = HandleMap::new();
        // We need to block zero because fuse seems to dislike it.
        inodes.block_handle(FileHandle(0));
        // FUSE_ROOT_ID represents the root directory. We can't use it for other inodes.
        if fuser::FUSE_ROOT_ID != 0 {
            inodes.block_handle(FileHandle(fuser::FUSE_ROOT_ID));
        }
        let open_files = tokio::sync::RwLock::new(HandleMap::new());

        let inodes = Arc::new(RwLock::new(inodes));
        AsyncDropGuard::new(Self {
            fs: Arc::new(RwLock::new(MaybeInitializedFs::Uninitialized(Some(
                Box::new(fs),
            )))),
            inodes,
            open_files,
        })
    }

    // TODO &self instead of `fs`, `inodes`
    /// This function allows file system operations to abstract over whether a requested inode number is the root node or whether it is looked up from the inode table `inodes`.
    async fn get_inode(&self, ino: FileHandle) -> FsResult<AsyncDropGuard<AsyncDropArc<Fs::Node>>> {
        // TODO Once async closures are stable, we can - instead of returning an AsyncDropArc - take a callback parameter and pass &Fs::Node to it.
        //      That would simplify all the call sites (e.g. don't require them to call async_drop on the returned value anymore).
        //      See https://stackoverflow.com/questions/76625378/async-closure-holding-reference-over-await-point
        if ino == FileHandle::from(fuser::FUSE_ROOT_ID) {
            let fs = self.fs.read().await;
            let fs = fs.get();
            let node = fs.rootdir().await?;
            Ok(AsyncDropArc::new(node.as_node()))
        } else {
            let inodes = self.inodes.read().await;
            Ok(AsyncDropArc::clone(
                inodes.get(ino).expect("Error: Inode number unassigned"),
            ))
        }
    }

    async fn add_inode(
        &self,
        parent_ino: FileHandle,
        node: AsyncDropGuard<Fs::Node>,
        name: &PathComponent,
    ) -> FileHandleWithGeneration {
        let child_ino = self.inodes.write().await.add(AsyncDropArc::new(node));
        log::info!("New inode {child_ino:?}: parent={parent_ino:?}, name={name}");
        child_ino
    }
}

#[async_trait]
impl<Fs> AsyncFilesystemLL for ObjectBasedFsAdapterLL<Fs>
where
    // TODO Do we need those Send + Sync + 'static bounds?
    Fs: Device + Send + Sync + 'static,
    Fs::OpenFile: Send + Sync,
{
    async fn init(&self, req: &RequestInfo, config: &mut KernelConfig) -> FsResult<()> {
        log::info!("init");
        self.fs.write().await.initialize(req.uid, req.gid);
        Ok(())
    }

    async fn destroy(&self) {
        log::info!("destroy");
        self.open_files.write().await.async_drop().await.unwrap();
        self.inodes.write().await.async_drop().await.unwrap();
        self.fs.write().await.take().destroy().await;
        // Nothing.
    }

    async fn lookup(
        &self,
        _req: &RequestInfo,
        parent: FileHandle,
        name: &PathComponent,
    ) -> FsResult<ReplyEntry> {
        // TODO Will lookup() be called multiple times with the same parent+name and is it ok to give the second call a different inode while the first call is still ongoing?
        let parent_node = self.get_inode(parent).await?;
        let mut child = with_async_drop_2!(parent_node, {
            let parent_node_dir = parent_node
                .as_dir()
                .await
                .expect("Error: Inode number is not a directory");
            parent_node_dir.lookup_child(&name).await
        })?;

        // TODO async_drop parent_node concurrently with the child.getattr() call below.

        match child.getattr().await {
            Ok(attr) => {
                let ino = self.add_inode(parent, child, name).await;
                Ok(ReplyEntry {
                    ttl: TTL_LOOKUP,
                    attr,
                    ino: ino.handle,
                    generation: ino.generation,
                })
            }
            Err(err) => {
                child.async_drop().await?;
                Err(err)
            }
        }
    }

    async fn forget(&self, _req: &RequestInfo, ino: FileHandle, nlookup: u64) -> FsResult<()> {
        // From the fuser documentation:
        // ```
        // The nlookup parameter indicates the number of lookups previously performed on
        // this inode. If the filesystem implements inode lifetimes, it is recommended that
        // inodes acquire a single reference on each lookup, and lose nlookup references on
        // each forget. The filesystem may ignore forget calls, if the inodes don't need to
        // have a limited lifetime. On unmount it is not guaranteed, that all referenced
        // inodes will receive a forget message.
        // ```
        // But we don't reuse inode numbers so nlookup should always be 1.
        assert_eq!(
            1, nlookup,
            "We don't reuse inode numbers so nlookup should always be 1"
        );

        let mut entry = self.inodes.write().await.remove(ino);
        entry.async_drop().await?;
        Ok(())
    }

    async fn getattr(&self, _req: &RequestInfo, ino: FileHandle) -> FsResult<ReplyAttr> {
        let mut node = self.get_inode(ino).await?;
        let attr = node.getattr().await;
        node.async_drop().await?;
        let attr = attr?;
        Ok(ReplyAttr {
            ttl: TTL_GETATTR,
            attr,
            ino,
        })
    }

    async fn setattr(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        mode: Option<Mode>,
        uid: Option<Uid>,
        gid: Option<Gid>,
        size: Option<NumBytes>,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
        ctime: Option<SystemTime>,
        fh: Option<FileHandle>,
        crtime: Option<SystemTime>,
        chgtime: Option<SystemTime>,
        bkuptime: Option<SystemTime>,
        flags: Option<u32>,
    ) -> FsResult<ReplyAttr> {
        // TODO
        Err(FsError::NotImplemented)
    }

    async fn readlink<CallbackResult>(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        callback: impl Send + for<'a> FnOnce(FsResult<&'a str>) -> CallbackResult,
    ) -> CallbackResult {
        // TODO
        callback(Err(FsError::NotImplemented))
    }

    async fn mknod(
        &self,
        req: &RequestInfo,
        parent: FileHandle,
        name: &PathComponent,
        mode: Mode,
        umask: u32,
        rdev: u32,
    ) -> FsResult<ReplyEntry> {
        // TODO
        Err(FsError::NotImplemented)
    }

    async fn mkdir(
        &self,
        req: &RequestInfo,
        parent_ino: FileHandle,
        name: &PathComponent,
        mode: Mode,
        umask: u32,
    ) -> FsResult<ReplyEntry> {
        // TODO What to do with umask?
        let parent = self.get_inode(parent_ino).await?;
        let (attr, child) = with_async_drop_2!(parent, {
            let parent_dir = parent.as_dir().await?;
            let (attrs, child) = parent_dir
                .create_child_dir(&name, mode, req.uid, req.gid)
                .await?;
            Ok((attrs, child.as_node()))
        })?;
        let ino = self.add_inode(parent_ino, child, name).await;
        Ok(ReplyEntry {
            ttl: TTL_GETATTR,
            attr,
            ino: ino.handle,
            generation: ino.generation,
        })
    }

    async fn unlink(
        &self,
        req: &RequestInfo,
        parent: FileHandle,
        name: &PathComponent,
    ) -> FsResult<()> {
        // TODO
        Err(FsError::NotImplemented)
    }

    async fn rmdir(
        &self,
        req: &RequestInfo,
        parent: FileHandle,
        name: &PathComponent,
    ) -> FsResult<()> {
        // TODO
        Err(FsError::NotImplemented)
    }

    async fn symlink(
        &self,
        req: &RequestInfo,
        parent: FileHandle,
        name: &PathComponent,
        link: &str,
    ) -> FsResult<ReplyEntry> {
        // TODO
        Err(FsError::NotImplemented)
    }

    async fn rename(
        &self,
        req: &RequestInfo,
        parent: FileHandle,
        name: &PathComponent,
        newparent: FileHandle,
        newname: &PathComponent,
        flags: u32,
    ) -> FsResult<()> {
        // TODO
        Err(FsError::NotImplemented)
    }

    async fn link(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        newparent: FileHandle,
        newname: &PathComponent,
    ) -> FsResult<ReplyEntry> {
        // TODO
        Err(FsError::NotImplemented)
    }

    async fn open(&self, req: &RequestInfo, ino: FileHandle, flags: i32) -> FsResult<ReplyOpen> {
        // TODO
        Ok(ReplyOpen {
            fh: FileHandle(0),
            flags: 0,
        })
    }

    async fn read<CallbackResult>(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        fh: FileHandle,
        offset: NumBytes,
        size: NumBytes,
        flags: i32,
        lock_owner: Option<u64>,
        callback: impl Send + for<'a> FnOnce(FsResult<&'a [u8]>) -> CallbackResult,
    ) -> CallbackResult {
        // TODO
        callback(Err(FsError::NotImplemented))
    }

    async fn write(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        fh: FileHandle,
        offset: NumBytes,
        data: &[u8],
        write_flags: u32,
        flags: i32,
        lock_owner: Option<u64>,
    ) -> FsResult<ReplyWrite> {
        // TODO
        Err(FsError::NotImplemented)
    }

    async fn flush(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        fh: FileHandle,
        lock_owner: u64,
    ) -> FsResult<()> {
        // TODO
        Err(FsError::NotImplemented)
    }

    async fn release(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        fh: FileHandle,
        flags: i32,
        lock_owner: Option<u64>,
        flush: bool,
    ) -> FsResult<()> {
        // TODO
        Ok(())
    }

    async fn fsync(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        fh: FileHandle,
        datasync: bool,
    ) -> FsResult<()> {
        // TODO
        Err(FsError::NotImplemented)
    }

    async fn opendir(&self, req: &RequestInfo, ino: FileHandle, flags: i32) -> FsResult<ReplyOpen> {
        // TODO
        Ok(ReplyOpen {
            fh: FileHandle(0),
            flags: 0,
        })
    }

    async fn readdir(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        fh: FileHandle,
        offset: NumBytes,
        reply: ReplyDirectory,
    ) {
        let node = match self.get_inode(ino).await {
            Ok(node) => node,
            Err(err) => {
                reply.error(err.system_error_code());
                return;
            }
        };
        // TODO Possible without Arc+Mutex?
        let reply = Mutex::new(reply);
        let result: FsResult<()> = with_async_drop_2!(node, {
            let dir = node.as_dir().await?;
            let entries = dir.entries().await?;
            let dir = Arc::new(dir);
            let offset = usize::try_from(u64::try_from(offset).unwrap()).unwrap(); // TODO No unwrap
            let entries =
                entries
                    .into_iter()
                    .enumerate()
                    .skip(offset)
                    .map(move |(offset, entry)| {
                        // TODO Possible without Arc?
                        let dir = Arc::clone(&dir);
                        let inodes = Arc::clone(&self.inodes);
                        async move {
                            let child = dir.lookup_child(&entry.name).await.unwrap(); // TODO No unwrap

                            // TODO Check that readdir is actually supposed to register the inode and that [Self::forget] will be called for this inode
                            //      Note also that fuse-mt actually doesn't register the inode here and a comment there claims that fuse just ignores it, see https://github.com/wfraser/fuse-mt/blob/881d7320b4c73c0bfbcbca48a5faab2a26f3e9e8/src/fusemt.rs#L619
                            //      fuse documentation says it shouldn't lookup: https://libfuse.github.io/doxygen/structfuse__lowlevel__ops.html#af1ef8e59e0cb0b02dc0e406898aeaa51
                            //      (but readdirplus should? see https://github.com/libfuse/libfuse/blob/7b9e7eeec6c43a62ab1e02dfb6542e6bfb7f72dc/include/fuse_lowlevel.h#L1209 )
                            //      I think for readdir, the correct behavior might be: return ino if in cache, otherwise return -1. Or just always return -1. See the `readdir_ino` config of libfuse.
                            let child_ino = self.add_inode(ino, child, &entry.name).await;
                            (offset, child_ino.handle, entry)
                        }
                    });
            // TODO Does this need to be FuturesOrdered or can we reply them without order?
            let mut entries: FuturesOrdered<_> = entries.collect();
            while let Some((offset, ino, entry)) = entries.next().await {
                // offset+1 because fuser actually expects us to return the offset of the **next** entry
                let offset = i64::try_from(offset).unwrap() + 1; // TODO No unwrap
                let buffer_is_full = reply.lock().unwrap().add(
                    ino.into(),
                    offset,
                    convert_node_kind(entry.kind),
                    &entry.name,
                );
                if buffer_is_full {
                    // TODO Can we cancel the stream if the buffer is full?
                    break;
                }
            }

            Ok(())
        });
        let reply = reply.into_inner().unwrap();
        match result {
            Ok(()) => reply.ok(),
            Err(err) => reply.error(err.system_error_code()),
        }
    }

    async fn readdirplus(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        fh: FileHandle,
        offset: NumBytes,
        reply: ReplyDirectoryPlus,
    ) {
        // TODO
        reply.error(libc::ENOSYS)
    }

    async fn releasedir(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        fh: FileHandle,
        flags: i32,
    ) -> FsResult<()> {
        // TODO
        Ok(())
    }

    async fn fsyncdir(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        fh: FileHandle,
        datasync: bool,
    ) -> FsResult<()> {
        // TODO
        Err(FsError::NotImplemented)
    }

    async fn statfs(&self, req: &RequestInfo, ino: FileHandle) -> FsResult<Statfs> {
        // TODO
        Ok(Statfs {
            num_total_blocks: 0,
            num_free_blocks: 0,
            num_available_blocks: 0,
            num_total_inodes: 0,
            num_free_inodes: 0,
            blocksize: 512,
            max_filename_length: 255,
        })
    }

    async fn setxattr(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        name: &PathComponent,
        value: &[u8],
        flags: i32,
        position: NumBytes,
    ) -> FsResult<()> {
        // TODO
        Err(FsError::NotImplemented)
    }

    async fn getxattr(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        name: &PathComponent,
        size: NumBytes,
        reply: ReplyXattr,
    ) {
        // TODO
        reply.error(libc::ENOSYS)
    }

    async fn listxattr(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        size: NumBytes,
        reply: ReplyXattr,
    ) {
        // TODO
        reply.error(libc::ENOSYS)
    }

    async fn removexattr(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        name: &PathComponent,
    ) -> FsResult<()> {
        // TODO
        Err(FsError::NotImplemented)
    }

    async fn access(&self, req: &RequestInfo, ino: FileHandle, mask: i32) -> FsResult<()> {
        // TODO
        Err(FsError::NotImplemented)
    }

    async fn create(
        &self,
        req: &RequestInfo,
        parent_ino: FileHandle,
        name: &PathComponent,
        mode: Mode,
        umask: u32,
        flags: i32,
    ) -> FsResult<ReplyCreate> {
        let parent = self.get_inode(parent_ino).await?;
        with_async_drop_2!(parent, {
            let parent_dir = parent.as_dir().await?;
            let (attr, child_node, open_file) = parent_dir
                .create_and_open_file(&name, mode, req.uid, req.gid)
                .await?;

            // TODO Check that readdir is actually supposed to register the inode and that [Self::forget] will be called for this inode. If not, we probably don't need to return the child_node from create_and_open_file.
            //      Note also that fuse-mt actually doesn't register the inode here and a comment there claims that fuse just ignores it, see https://github.com/wfraser/fuse-mt/blob/881d7320b4c73c0bfbcbca48a5faab2a26f3e9e8/src/fusemt.rs#L619
            let child_ino = self.add_inode(parent_ino, child_node, name).await;

            let fh = self.open_files.write().await.add(open_file);
            Ok(ReplyCreate {
                ttl: TTL_CREATE,
                attr,
                ino: child_ino.handle,
                generation: child_ino.generation,
                fh: fh.handle,
                // TODO Do we need to change flags or is it ok to just return the flags passed in? If it's ok, then why do we have to return them?
                flags: u32::try_from(flags).unwrap(), // TODO No unwrap?
            })
        })
    }

    async fn getlk(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        fh: FileHandle,
        lock_owner: u64,
        start: u64,
        end: u64,
        typ: i32,
        pid: u32,
    ) -> FsResult<ReplyLock> {
        // TODO
        Err(FsError::NotImplemented)
    }

    async fn setlk(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        fh: FileHandle,
        lock_owner: u64,
        start: u64,
        end: u64,
        typ: i32,
        pid: u32,
        sleep: bool,
    ) -> FsResult<()> {
        // TODO
        Err(FsError::NotImplemented)
    }

    async fn bmap(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        blocksize: NumBytes,
        idx: u64,
    ) -> FsResult<ReplyBmap> {
        // TODO
        Err(FsError::NotImplemented)
    }

    /// control device
    async fn ioctl(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        fh: FileHandle,
        flags: u32,
        cmd: u32,
        in_data: &[u8],
        out_size: u32,
        reply: ReplyIoctl,
    ) {
        // TODO
        reply.error(libc::ENOSYS)
    }

    async fn fallocate(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        fh: FileHandle,
        offset: NumBytes,
        length: NumBytes,
        mode: Mode,
    ) -> FsResult<()> {
        // TODO
        Err(FsError::NotImplemented)
    }

    async fn lseek(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        fh: FileHandle,
        offset: NumBytes,
        whence: i32,
    ) -> FsResult<ReplyLseek> {
        // TODO
        Err(FsError::NotImplemented)
    }

    async fn copy_file_range(
        &self,
        req: &RequestInfo,
        ino_in: FileHandle,
        fh_in: FileHandle,
        offset_in: NumBytes,
        ino_out: FileHandle,
        fh_out: FileHandle,
        offset_out: NumBytes,
        len: NumBytes,
        flags: u32,
    ) -> FsResult<ReplyWrite> {
        // TODO
        Err(FsError::NotImplemented)
    }

    #[cfg(target_os = "macos")]
    async fn setvolname(&self, req: &RequestInfo, name: &str) -> FsResult<()> {
        // TODO
        Err(FsError::NotImplemented)
    }

    #[cfg(target_os = "macos")]
    async fn exchange(
        &self,
        req: &RequestInfo,
        parent: FileHandle,
        name: &PathComponent,
        newparent: FileHandle,
        newname: &PathComponent,
        options: u64,
    ) -> FsResult<()> {
        // TODO
        Err(FsError::NotImplemented)
    }

    #[cfg(target_os = "macos")]
    async fn getxtimes(&self, req: &RequestInfo, ino: FileHandle) -> FsResult<ReplyXTimes> {
        // TODO
        Err(FsError::NotImplemented)
    }
}

impl<Fn, D> IntoFsLL<ObjectBasedFsAdapterLL<D>> for Fn
where
    Fn: FnOnce(Uid, Gid) -> D + Send + Sync + 'static,
    D: Device + Send + Sync + 'static,
    D::OpenFile: Send + Sync,
{
    fn into_fs(self) -> AsyncDropGuard<ObjectBasedFsAdapterLL<D>> {
        ObjectBasedFsAdapterLL::new(self)
    }
}

impl<Fs: Device> Debug for ObjectBasedFsAdapterLL<Fs>
where
    Fs::OpenFile: Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObjectBasedFsAdapterLL")
            .field("open_files", &self.open_files)
            .finish()
    }
}

#[async_trait]
impl<Fs> AsyncDrop for ObjectBasedFsAdapterLL<Fs>
where
    Fs: Device + Send + Sync,
    Fs::OpenFile: Send + Sync,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        // TODO If the object was never used (e.g. destroy never called), we need to destroy members here.
        Ok(())
    }
}

fn convert_node_kind(kind: NodeKind) -> fuser::FileType {
    match kind {
        NodeKind::File => fuser::FileType::RegularFile,
        NodeKind::Dir => fuser::FileType::Directory,
        NodeKind::Symlink => fuser::FileType::Symlink,
    }
}
