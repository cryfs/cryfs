use async_trait::async_trait;
use futures::{
    join,
    stream::{FuturesOrdered, StreamExt},
};
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

use super::utils::MaybeInitializedFs;
use super::{Device, Dir, File, Node, OpenFile, Symlink};
use crate::common::{
    AbsolutePath, Callback, DirEntry, FileHandle, FsError, FsResult, Gid, HandleMap,
    HandleWithGeneration, InodeNumber, Mode, NodeKind, NumBytes, OpenFlags, PathComponent,
    RequestInfo, Statfs, Uid,
};
use crate::low_level_api::{
    AsyncFilesystemLL, IntoFsLL, ReplyAttr, ReplyBmap, ReplyCreate, ReplyEntry, ReplyLock,
    ReplyLseek, ReplyOpen, ReplyWrite,
};
use cryfs_utils::{
    async_drop::{flatten_async_drop, with_async_drop, AsyncDrop, AsyncDropArc, AsyncDropGuard},
    with_async_drop_2,
};
use fuser::{KernelConfig, ReplyDirectory, ReplyDirectoryPlus, ReplyIoctl, ReplyXattr};

const FUSE_ROOT_ID: InodeNumber = InodeNumber::from_const(fuser::FUSE_ROOT_ID);

// TODO What are good TTLs here?
const TTL_LOOKUP: Duration = Duration::from_secs(1);
const TTL_GETATTR: Duration = Duration::from_secs(1);
const TTL_CREATE: Duration = Duration::from_secs(1);
const TTL_SYMLINK: Duration = Duration::from_secs(1);

// TODO Can we share more code with [super::high_level_adapter::ObjectBasedFsAdapter]?
pub struct ObjectBasedFsAdapterLL<Fs: Device>
where
    // TODO Is this send+sync bound only needed because fuse_mt goes multi threaded or would it also be required for fuser?
    Fs: Device + Send + Sync + 'static,
    for<'a> Fs::File<'a>: Send,
    Fs::OpenFile: Send + Sync,
{
    // TODO We only need the Arc<RwLock<...>> because of initialization. Is there a better way to do that?
    fs: Arc<RwLock<MaybeInitializedFs<Fs>>>,

    // TODO Do we need Arc for inodes?
    inodes: Arc<RwLock<AsyncDropGuard<HandleMap<InodeNumber, AsyncDropArc<Fs::Node>>>>>,

    // TODO Can we improve concurrency by locking less in open_files and instead making OpenFileList concurrency safe somehow?
    open_files: tokio::sync::RwLock<AsyncDropGuard<HandleMap<FileHandle, Fs::OpenFile>>>,
}

impl<Fs: Device> ObjectBasedFsAdapterLL<Fs>
where
    // TODO Is this send+sync bound only needed because fuse_mt goes multi threaded or would it also be required for fuser?
    Fs: Device + Send + Sync + 'static,
    for<'a> Fs::File<'a>: Send,
    Fs::OpenFile: Send + Sync,
{
    pub fn new(fs: impl FnOnce(Uid, Gid) -> Fs + Send + Sync + 'static) -> AsyncDropGuard<Self> {
        let mut inodes = HandleMap::new();
        // We need to block zero because fuse seems to dislike it.
        inodes.block_handle(InodeNumber::from(0));
        // FUSE_ROOT_ID represents the root directory. We can't use it for other inodes.
        if fuser::FUSE_ROOT_ID != 0 {
            inodes.block_handle(FUSE_ROOT_ID);
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
    async fn get_inode(
        &self,
        ino: InodeNumber,
    ) -> FsResult<AsyncDropGuard<AsyncDropArc<Fs::Node>>> {
        // TODO Once async closures are stable, we can - instead of returning an AsyncDropArc - take a callback parameter and pass &Fs::Node to it.
        //      That would simplify all the call sites (e.g. don't require them to call async_drop on the returned value anymore).
        //      See https://stackoverflow.com/questions/76625378/async-closure-holding-reference-over-await-point
        if ino == FUSE_ROOT_ID {
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
        parent_ino: InodeNumber,
        node: AsyncDropGuard<Fs::Node>,
        name: &PathComponent,
    ) -> HandleWithGeneration<InodeNumber> {
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
    for<'a> Fs::File<'a>: Send,
    Fs::OpenFile: Send + Sync,
{
    async fn init(&self, req: &RequestInfo, _config: &mut KernelConfig) -> FsResult<()> {
        // TODO Allow implementations to change KernelConfig? Or at least parts of it?
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
        parent_ino: InodeNumber,
        name: &PathComponent,
    ) -> FsResult<ReplyEntry> {
        // TODO Will lookup() be called multiple times with the same parent+name and is it ok to give the second call a different inode while the first call is still ongoing?
        let parent_node = self.get_inode(parent_ino).await?;
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
                let ino = self.add_inode(parent_ino, child, name).await;
                Ok(ReplyEntry {
                    ttl: TTL_LOOKUP,
                    ino,
                    attr,
                })
            }
            Err(err) => {
                child.async_drop().await?;
                Err(err)
            }
        }
    }

    async fn forget(&self, _req: &RequestInfo, ino: InodeNumber, nlookup: u64) -> FsResult<()> {
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

    async fn getattr(&self, _req: &RequestInfo, ino: InodeNumber) -> FsResult<ReplyAttr> {
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
        _req: &RequestInfo,
        ino: InodeNumber,
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
        // TODO What to do with crtime, chgtime, bkuptime, flags?
        // TODO setattr based on fh?
        let mut node = self.get_inode(ino).await?;
        let attr = node
            .setattr(mode, uid, gid, size, atime, mtime, ctime)
            .await;
        node.async_drop().await?;
        let attr = attr?;
        Ok(ReplyAttr {
            ttl: TTL_GETATTR,
            attr,
            ino,
        })
    }

    async fn readlink<R, C>(&self, _req: &RequestInfo, ino: InodeNumber, callback: C) -> R
    where
        R: 'static,
        C: Send + 'static + for<'a> Callback<FsResult<&'a str>, R>,
    {
        let mut inode = match self.get_inode(ino).await {
            Ok(inode) => inode,
            Err(err) => return callback.call(Err(err)),
        };
        let target = match inode.as_symlink().await {
            Ok(inode_symlink) => {
                let target = inode_symlink.target();
                match target.await {
                    Ok(target) => Ok(target),
                    Err(err) => Err(err),
                }
            }
            Err(err) => Err(err),
        };
        let target = match target {
            Ok(ref target) => Ok(target.as_str()),
            Err(err) => Err(err),
        };
        let target = match inode.async_drop().await {
            Ok(()) => target,
            Err(err) => Err(err),
        };
        callback.call(target)
    }

    async fn mknod(
        &self,
        req: &RequestInfo,
        parent_ino: InodeNumber,
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
        parent_ino: InodeNumber,
        name: &PathComponent,
        mode: Mode,
        _umask: u32,
    ) -> FsResult<ReplyEntry> {
        // In my tests with fuser 0.12.0, umask is already auto-applied to mode and the `umask` argument is always `0`.
        // TODO see https://github.com/cberner/fuser/issues/256
        let parent = self.get_inode(parent_ino).await?;
        let (attr, child) = with_async_drop_2!(parent, {
            let parent_dir = parent.as_dir().await?;
            let (attrs, child) = parent_dir
                .create_child_dir(&name, mode, req.uid, req.gid)
                .await?;
            Ok((attrs, child.as_node()))
        })?;
        // TODO Are we supposed to add the inode to our inode list here?
        let ino = self.add_inode(parent_ino, child, name).await;
        Ok(ReplyEntry {
            ttl: TTL_GETATTR,
            attr,
            ino: ino,
        })
    }

    async fn unlink(
        &self,
        _req: &RequestInfo,
        parent_ino: InodeNumber,
        name: &PathComponent,
    ) -> FsResult<()> {
        let parent = self.get_inode(parent_ino).await?;
        with_async_drop_2!(parent, {
            let parent_dir = parent.as_dir().await?;
            parent_dir.remove_child_file_or_symlink(&name).await?;
            Ok(())
        })
    }

    async fn rmdir(
        &self,
        _req: &RequestInfo,
        parent_ino: InodeNumber,
        name: &PathComponent,
    ) -> FsResult<()> {
        let parent = self.get_inode(parent_ino).await?;
        with_async_drop_2!(parent, {
            let parent_dir = parent.as_dir().await?;
            parent_dir.remove_child_dir(&name).await?;
            Ok(())
        })
    }

    async fn symlink(
        &self,
        req: &RequestInfo,
        parent_ino: InodeNumber,
        name: &PathComponent,
        link: &str,
    ) -> FsResult<ReplyEntry> {
        let parent = self.get_inode(parent_ino).await?;
        let (attrs, child) = with_async_drop_2!(parent, {
            let parent_dir = parent.as_dir().await?;
            let (attrs, child) = parent_dir
                .create_child_symlink(&name, link, req.uid, req.gid)
                .await?;
            Ok((attrs, child.as_node()))
        })?;
        // TODO Are we supposed to add the inode to our inode list here?
        let ino = self.add_inode(parent_ino, child, name).await;
        Ok(ReplyEntry {
            ttl: TTL_SYMLINK,
            attr: attrs,
            ino,
        })
    }

    async fn rename(
        &self,
        _req: &RequestInfo,
        oldparent_ino: InodeNumber,
        oldname: &PathComponent,
        newparent_ino: InodeNumber,
        newname: &PathComponent,
        _flags: u32,
    ) -> FsResult<()> {
        // TODO Honor flags
        // TODO Check that oldparent+oldname/newparent+newname aren't ancestors of each other, or at least write a test that fuse already blocks that
        if oldparent_ino == newparent_ino {
            let shared_parent = self.get_inode(oldparent_ino).await?;
            with_async_drop_2!(shared_parent, {
                let parent_dir = shared_parent.as_dir().await?;
                parent_dir.rename_child(&oldname, &newname).await?;
                Ok(())
            })
        } else {
            let (oldparent, newparent) =
                join!(self.get_inode(oldparent_ino), self.get_inode(newparent_ino));
            let (mut oldparent, mut newparent) = flatten_async_drop(oldparent, newparent).await?;
            let result = (|| async {
                let oldparent_dir = oldparent.as_dir().await?;
                let newparent_dir = newparent.as_dir().await?;
                oldparent_dir
                    .move_child_to(oldname, newparent_dir, newname)
                    .await
            })()
            .await;
            // TODO Drop concurrently and drop latter even if first one fails
            oldparent.async_drop().await?;
            newparent.async_drop().await?;
            result
        }
    }

    async fn link(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        newparent_ino: InodeNumber,
        newname: &PathComponent,
    ) -> FsResult<ReplyEntry> {
        // TODO
        Err(FsError::NotImplemented)
    }

    async fn open(
        &self,
        _req: &RequestInfo,
        ino: InodeNumber,
        flags: OpenFlags,
    ) -> FsResult<ReplyOpen> {
        let inode = self.get_inode(ino).await?;
        with_async_drop_2!(inode, {
            let file = inode.as_file().await?;
            let open_file = file.open(flags);
            let open_file = open_file.await?;
            let fh = self.open_files.write().await.add(open_file);
            Ok(ReplyOpen {
                fh: fh.handle,
                // TODO What flags to return here? Just same as the argument?
                flags,
            })
        })
    }

    async fn read<R, C>(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        offset: NumBytes,
        size: NumBytes,
        flags: i32,
        lock_owner: Option<u64>,
        callback: C,
    ) -> R
    where
        R: 'static,
        C: Send + 'static + for<'a> Callback<FsResult<&'a [u8]>, R>,
    {
        let open_files = self.open_files.read().await;
        let Some(open_file) = open_files.get(fh) else {
            log::error!("read: no open file with handle {}", u64::from(fh));
            return callback.call(Err(FsError::InvalidFileDescriptor { fh: u64::from(fh) }));
        };

        let data = open_file.read(offset, size).await;
        let data = match data {
            Ok(ref data) => Ok(data.as_ref()),
            Err(err) => Err(err),
        };
        callback.call(data)
    }

    async fn write(
        &self,
        _req: &RequestInfo,
        _ino: InodeNumber,
        fh: FileHandle,
        offset: NumBytes,
        data: &[u8],
        _write_flags: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
    ) -> FsResult<ReplyWrite> {
        // TODO What to do with WriteFlags, flags, lock_owner?
        let open_files = self.open_files.read().await;
        let Some(open_file) = open_files.get(fh) else {
            log::error!("write: no open file with handle {}", u64::from(fh));
            return Err(FsError::InvalidFileDescriptor { fh: u64::from(fh) });
        };

        open_file.write(offset, data.to_vec().into()).await?;
        Ok(ReplyWrite {
            // TODO No unwrap
            written: u32::try_from(data.len()).unwrap(),
        })
    }

    async fn flush(
        &self,
        _req: &RequestInfo,
        _ino: InodeNumber,
        fh: FileHandle,
        _lock_owner: u64,
    ) -> FsResult<()> {
        // TODO What to do about lock_owner?
        let open_files = self.open_files.read().await;
        let Some(open_file) = open_files.get(fh) else {
            log::error!("write: no open file with handle {}", u64::from(fh));
            // TODO Add a self.get_open_file() function that deduplicates this logic of throwing InvalidFileDescriptor between file system operations
            return Err(FsError::InvalidFileDescriptor { fh: u64::from(fh) });
        };

        open_file.flush().await
    }

    async fn release(
        &self,
        _req: &RequestInfo,
        _ino: InodeNumber,
        fh: FileHandle,
        _flags: i32,
        _lock_owner: Option<u64>,
        flush: bool,
    ) -> FsResult<()> {
        // TODO Would it make sense to have `fh` always be equal to `ino`? Might simplify some things. Also, we could add an `assert_eq!(ino, fh)` here.

        // TODO What to do with flags, lock_owner?
        let open_file = self.open_files.write().await.remove(fh);
        with_async_drop_2!(open_file, {
            if flush {
                // TODO Is this actually what the `flush` parameter should do?
                open_file.flush().await?;
            }
            Ok(())
        })
    }

    async fn fsync(
        &self,
        _req: &RequestInfo,
        _ino: InodeNumber,
        fh: FileHandle,
        datasync: bool,
    ) -> FsResult<()> {
        // TODO What to do about lock_owner?
        let open_files = self.open_files.read().await;
        let Some(open_file) = open_files.get(fh) else {
            log::error!("write: no open file with handle {}", u64::from(fh));
            // TODO Add a self.get_open_file() function that deduplicates this logic of throwing InvalidFileDescriptor between file system operations
            return Err(FsError::InvalidFileDescriptor { fh: u64::from(fh) });
        };

        open_file.fsync(datasync).await
    }

    async fn opendir(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        flags: i32,
    ) -> FsResult<ReplyOpen> {
        // TODO
        Ok(ReplyOpen {
            fh: FileHandle::from(0),
            flags: OpenFlags::ReadWrite,
        })
    }

    async fn readdir(
        &self,
        _req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        offset: NumBytes,
        reply: ReplyDirectory,
    ) {
        // TODO Allow readdir on fh instead of ino?
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
        ino: InodeNumber,
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
        ino: InodeNumber,
        fh: FileHandle,
        flags: i32,
    ) -> FsResult<()> {
        // TODO
        Ok(())
    }

    async fn fsyncdir(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        datasync: bool,
    ) -> FsResult<()> {
        // TODO
        Err(FsError::NotImplemented)
    }

    async fn statfs(&self, _req: &RequestInfo, _ino: InodeNumber) -> FsResult<Statfs> {
        self.fs.read().await.get().statfs().await
    }

    async fn setxattr(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
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
        ino: InodeNumber,
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
        ino: InodeNumber,
        size: NumBytes,
        reply: ReplyXattr,
    ) {
        // TODO
        reply.error(libc::ENOSYS)
    }

    async fn removexattr(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        name: &PathComponent,
    ) -> FsResult<()> {
        // TODO
        Err(FsError::NotImplemented)
    }

    async fn access(&self, req: &RequestInfo, ino: InodeNumber, mask: i32) -> FsResult<()> {
        // TODO Should we implement access?
        Ok(())
    }

    async fn create(
        &self,
        req: &RequestInfo,
        parent_ino: InodeNumber,
        name: &PathComponent,
        mode: Mode,
        _umask: u32,
        flags: i32,
    ) -> FsResult<ReplyCreate> {
        // In my tests with fuser 0.12.0, umask is already auto-applied to mode and the `umask` argument is always `0`.
        // TODO see https://github.com/cberner/fuser/issues/256
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
                ino: child_ino,
                fh: fh.handle,
                // TODO Do we need to change flags or is it ok to just return the flags passed in? If it's ok, then why do we have to return them?
                flags: u32::try_from(flags).unwrap(), // TODO No unwrap?
            })
        })
    }

    async fn getlk(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
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
        ino: InodeNumber,
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
        ino: InodeNumber,
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
        ino: InodeNumber,
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
        ino: InodeNumber,
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
        ino: InodeNumber,
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
        ino_in: InodeNumber,
        fh_in: FileHandle,
        offset_in: NumBytes,
        ino_out: InodeNumber,
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
        parent_ino: InodeNumber,
        name: &PathComponent,
        newparent_ino: InodeNumber,
        newname: &PathComponent,
        options: u64,
    ) -> FsResult<()> {
        // TODO
        Err(FsError::NotImplemented)
    }

    #[cfg(target_os = "macos")]
    async fn getxtimes(&self, req: &RequestInfo, ino: InodeNumber) -> FsResult<ReplyXTimes> {
        // TODO
        Err(FsError::NotImplemented)
    }
}

impl<Fn, Fs> IntoFsLL<ObjectBasedFsAdapterLL<Fs>> for Fn
where
    Fn: FnOnce(Uid, Gid) -> Fs + Send + Sync + 'static,
    Fs: Device + Send + Sync + 'static,
    for<'a> Fs::File<'a>: Send,
    Fs::OpenFile: Send + Sync,
{
    fn into_fs(self) -> AsyncDropGuard<ObjectBasedFsAdapterLL<Fs>> {
        ObjectBasedFsAdapterLL::new(self)
    }
}

impl<Fs: Device> Debug for ObjectBasedFsAdapterLL<Fs>
where
    Fs: Device + Send + Sync + 'static,
    for<'a> Fs::File<'a>: Send,
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
    Fs: Device + Send + Sync + 'static,
    for<'a> Fs::File<'a>: Send,
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
