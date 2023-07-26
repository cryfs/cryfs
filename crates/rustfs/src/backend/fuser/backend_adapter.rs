use cryfs_utils::async_drop::with_async_drop;
use cryfs_utils::with_async_drop_2;
#[cfg(target_os = "macos")]
use fuser::ReplyXTimes;
use fuser::{
    Filesystem, KernelConfig, Reply, ReplyAttr, ReplyBmap, ReplyCreate, ReplyData, ReplyDirectory,
    ReplyDirectoryPlus, ReplyEmpty, ReplyEntry, ReplyIoctl, ReplyLock, ReplyLseek, ReplyOpen,
    ReplyStatfs, ReplyWrite, ReplyXattr, Request, TimeOrNow,
};
use futures::join;
use futures::stream::{FuturesUnordered, StreamExt};
use libc::{c_int, ENOSYS, EPERM};
use std::ffi::OsStr;
use std::fmt::Debug;
use std::future::Future;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use std::time::SystemTime;
use tokio::sync::RwLock;

use crate::common::{
    AbsolutePath, AbsolutePathBuf, DirEntry, FileHandle, FileHandleWithGeneration, FsError,
    FsResult, Gid, HandleMap, HandlePool, Mode, NodeAttrs, NodeKind, NumBytes, OpenFlags,
    PathComponent, PathComponentBuf, Statfs, Uid,
};
use crate::object_based_api::{adapter::MaybeInitializedFs, Device, Dir, Node};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard},
    stream::for_each_unordered,
};

// TODO What are good TTLs here?
const TTL_LOOKUP: Duration = Duration::from_secs(1);
const TTL_GETATTR: Duration = Duration::from_secs(1);

// TODO Fuse has a requirement that (inode, generation) tuples are unique throughout the lifetime of the filesystem, not just the lifetime of the mount.
//      See https://github.com/libfuse/libfuse/blob/d92bf83c152ff88c2d92bd852752d4c326004400/include/fuse_lowlevel.h#L69-L81 and https://github.com/wfraser/fuse-mt/issues/19
//      This means currently, CryFS can't be used over NFS. We should fix this.

pub struct BackendAdapter<Fs>
where
    Fs: Device + Send + Sync + 'static,
    <Fs as Device>::Node: Send + Sync,
    for<'a> <Fs as Device>::Dir<'a>: Send + Sync,
{
    // TODO RwLock is only needed for initialize, destroy and async drop. Can we remove it?
    fs: Arc<RwLock<MaybeInitializedFs<Fs>>>,

    // TODO Do we need Arc for inodes?
    inodes: Arc<RwLock<AsyncDropGuard<HandleMap<AsyncDropArc<Fs::Node>>>>>,

    runtime: tokio::runtime::Handle,
}

impl<Fs> Debug for BackendAdapter<Fs>
where
    Fs: Device + Send + Sync + 'static,
    <Fs as Device>::Node: Send + Sync,
    for<'a> <Fs as Device>::Dir<'a>: Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BackendAdapter").finish()
    }
}

impl<Fs> BackendAdapter<Fs>
where
    Fs: Device + Send + Sync + 'static,
    <Fs as Device>::Node: Send + Sync,
    for<'a> <Fs as Device>::Dir<'a>: Send + Sync,
{
    pub fn new(
        fs: impl FnOnce(Uid, Gid) -> Fs + Send + Sync + 'static,
        runtime: tokio::runtime::Handle,
    ) -> Self {
        let mut inodes = HandleMap::new();
        // We need to block zero because fuse seems to dislike it.
        inodes.block_handle(FileHandle(0));
        // FUSE_ROOT_ID represents the root directory. We can't use it for other inodes.
        if fuser::FUSE_ROOT_ID != 0 {
            inodes.block_handle(FileHandle(fuser::FUSE_ROOT_ID));
        }

        let inodes = Arc::new(RwLock::new(inodes));
        Self {
            fs: Arc::new(RwLock::new(MaybeInitializedFs::Uninitialized(Some(
                Box::new(fs),
            )))),
            inodes,
            runtime,
        }
    }

    // TODO &self instead of `fs`, `inodes`
    /// This function allows file system operations to abstract over whether a requested inode number is the root node or whether it is looked up from the inode table `inodes`.
    async fn get_inode(
        fs: &RwLock<MaybeInitializedFs<Fs>>,
        inodes: &RwLock<AsyncDropGuard<HandleMap<AsyncDropArc<Fs::Node>>>>,
        ino: FileHandle,
    ) -> FsResult<AsyncDropGuard<AsyncDropArc<Fs::Node>>> {
        // TODO Once async closures are stable, we can - instead of returning an AsyncDropArc - take a callback parameter and pass &Fs::Node to it.
        //      That would simplify all the call sites (e.g. don't require them to call async_drop on the returned value anymore).
        //      See https://stackoverflow.com/questions/76625378/async-closure-holding-reference-over-await-point
        if ino == FileHandle::from(fuser::FUSE_ROOT_ID) {
            let fs = fs.read().await;
            let fs = fs.get();
            let node = fs.rootdir().await?;
            Ok(AsyncDropArc::new(node.as_node()))
        } else {
            let inodes = inodes.read().await;
            Ok(AsyncDropArc::clone(
                inodes.get(ino).expect("Error: Inode number unassigned"),
            ))
        }
    }

    fn run_blocking<R, F>(
        runtime: &tokio::runtime::Handle,
        log_msg: &str,
        func: impl FnOnce() -> F,
    ) -> Result<R, libc::c_int>
    where
        F: Future<Output = FsResult<R>>,
    {
        // TODO Is it ok to call block_on concurrently for multiple fs operations?
        runtime.block_on(async move {
            log::info!("{}...", log_msg);
            let result = func().await;
            match result {
                Ok(ok) => {
                    log::info!("{}...done", log_msg);
                    Ok(ok)
                }
                Err(err) => {
                    log::info!("{}...failed: {}", log_msg, err);
                    Err(err.system_error_code())
                }
            }
        })
    }

    // TODO Can we unify `run_async_reply_{entry,attr,...}` ?

    fn run_async_reply_entry(
        runtime: &tokio::runtime::Handle,
        log_msg: String,
        reply: ReplyEntry,
        func: impl Future<Output = FsResult<(Duration, NodeAttrs, FileHandleWithGeneration)>>
            + Send
            + 'static,
    ) {
        runtime.spawn(async move {
            log::info!("{}...", log_msg);
            match func.await {
                Ok((ttl, attrs, ino)) => {
                    log::info!("{}...done", log_msg);
                    reply.entry(&ttl, &convert_node_attrs(attrs, ino.handle), ino.generation);
                }
                Err(err) => {
                    log::info!("{}...failed: {}", log_msg, err);
                    reply.error(err.system_error_code())
                }
            }
        });
    }

    fn run_async_reply_attr(
        runtime: &tokio::runtime::Handle,
        log_msg: String,
        reply: ReplyAttr,
        func: impl Future<Output = FsResult<(Duration, NodeAttrs, FileHandle)>> + Send + 'static,
    ) {
        runtime.spawn(async move {
            log::info!("{}...", log_msg);
            match func.await {
                Ok((ttl, attrs, ino)) => {
                    log::info!("{}...done", log_msg);
                    reply.attr(&ttl, &convert_node_attrs(attrs, ino));
                }
                Err(err) => {
                    log::info!("{}...failed: {}", log_msg, err);
                    reply.error(err.system_error_code())
                }
            }
        });
    }

    fn run_async_reply_directory<'a, I>(
        runtime: &tokio::runtime::Handle,
        log_msg: String,
        mut reply: ReplyDirectory,
        func: impl Future<Output = FsResult<(usize, I)>> + Send + 'static,
    ) where
        I: Iterator<Item = (FileHandle, DirEntry)> + Send,
    {
        runtime.spawn(async move {
            log::info!("{}...", log_msg);
            match func.await {
                Ok((offset_base, entries)) => {
                    log::info!("{}...done", log_msg);
                    for (entry_offset, entry) in entries.enumerate() {
                        let offset = i64::try_from(offset_base + entry_offset).unwrap();
                        let (handle, entry) = entry;
                        let buffer_full = reply.add(
                            handle.0,
                            // Return offset + 1 because the fuse API expects this to be the offset of the **next** entry, see https://libfuse.github.io/doxygen/fuse__lowlevel_8h.html#ad1957bcc8ece8c90f16c42c4daf3053f
                            offset + 1,
                            convert_node_kind(entry.kind),
                            entry.name,
                        );
                        if buffer_full {
                            break;
                        }
                    }
                    reply.ok();
                }
                Err(err) => {
                    log::info!("{}...failed: {}", log_msg, err);
                    reply.error(err.system_error_code())
                }
            }
        });
    }
}

impl<Fs> Filesystem for BackendAdapter<Fs>
where
    // TODO Is both Send + Sync needed here?
    Fs: Device + Send + Sync + 'static,
    <Fs as Device>::Node: Send + Sync,
    for<'a> <Fs as Device>::Dir<'a>: Send + Sync,
{
    fn init(&mut self, req: &Request<'_>, _config: &mut KernelConfig) -> Result<(), c_int> {
        Self::run_blocking(&self.runtime, &format!("init"), || async {
            self.fs
                .write()
                .await
                .initialize(Uid::from(req.uid()), Gid::from(req.gid()));
            Ok(())
        })
    }

    fn destroy(&mut self) {
        Self::run_blocking(&self.runtime, &format!("destroy"), || async {
            self.inodes.write().await.async_drop().await.unwrap();
            let fs = self.fs.write().await.take();
            fs.destroy().await;
            Ok(())
        })
        .expect("failed to drop file system");

        // TODO Is there a way to do the above without a call to expect()?
    }

    fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEntry) {
        // TODO Is this possible without name.to_owned()?
        // TODO Is this possible without the Arc::clone here? Also in other functions below.
        let name = name.to_owned();
        let fs = Arc::clone(&self.fs);
        let inodes = Arc::clone(&self.inodes);
        let parent = FileHandle::from(parent);
        Self::run_async_reply_entry(
            &self.runtime,
            format!("lookup(parent={parent:?}, name={name:?}"),
            reply,
            async move {
                let name: PathComponentBuf = name.try_into().map_err(|err| FsError::InvalidPath)?;
                let name_clone = name.clone();

                let mut parent_node = Self::get_inode(&fs, &inodes, parent).await?;
                let child = async {
                    let parent_node_dir = parent_node
                        .as_dir()
                        .await
                        .expect("Error: Inode number is not a directory");
                    let child = parent_node_dir.lookup_child(&name);
                    child.await
                }
                .await;
                // TODO async_drop concurrently with the child.getattr() call below.
                parent_node.async_drop().await?;
                let mut child = child?;

                match child.getattr().await {
                    Ok(attrs) => {
                        let ino = inodes.write().await.add(AsyncDropArc::new(child));
                        log::info!("New inode {ino:?}: parent={parent:?}, name={name_clone}");
                        Ok((TTL_LOOKUP, attrs, ino))
                    }
                    Err(err) => {
                        child.async_drop().await?;
                        Err(err)
                    }
                }
            },
        )
    }

    fn forget(&mut self, _req: &Request<'_>, ino: u64, nlookup: u64) {
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
        Self::run_blocking(&self.runtime, &format!("forget(ino={ino})"), || async {
            let mut entry = self.inodes.write().await.remove(ino.into());
            entry.async_drop().await?;
            Ok(())
        })
        .expect("failed to forget about an inode");
    }

    // TODO Do we want this? It seems to be gated by an "abi-7-16" feature but what is that?
    // fn batch_forget(&mut self, req: &Request<'_>, nodes: &[fuse_forget_one]) {
    //     assert_eq!(1, nlookup, "We don't reuse inode numbers so nlookup should always be 1");
    //     Self::run_blocking(&self.runtime, &format!("batch_forget({nodes:?})"), || async {
    //         let inodes = self.inodes.write().await;
    //         for_each_unordered(
    //             nodes,
    //             |node| {
    //                 let mut entry = inodes.remove(node.into());
    //                 entry.async_drop()
    //             }
    //         ).await?;
    //         Ok(())
    //     })
    //     .expect("failed to forget about an inode");
    // }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, reply: ReplyAttr) {
        let inodes = Arc::clone(&self.inodes);
        let fs = Arc::clone(&self.fs);
        Self::run_async_reply_attr(
            &self.runtime,
            format!("getattr(ino={ino})"),
            reply,
            async move {
                let ino = FileHandle::from(ino);
                let mut node = Self::get_inode(&fs, &inodes, ino).await?;
                let attrs = node.getattr().await;
                node.async_drop().await?;
                let attrs = attrs?;
                Ok((TTL_GETATTR, attrs, ino))
            },
        );
    }

    /// Set file attributes.
    fn setattr(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        mode: Option<u32>,
        uid: Option<u32>,
        gid: Option<u32>,
        size: Option<u64>,
        _atime: Option<TimeOrNow>,
        _mtime: Option<TimeOrNow>,
        _ctime: Option<SystemTime>,
        fh: Option<u64>,
        _crtime: Option<SystemTime>,
        _chgtime: Option<SystemTime>,
        _bkuptime: Option<SystemTime>,
        flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        log::debug!(
            "[Not Implemented] setattr(ino: {:#x?}, mode: {:?}, uid: {:?}, \
                gid: {:?}, size: {:?}, fh: {:?}, flags: {:?})",
            ino,
            mode,
            uid,
            gid,
            size,
            fh,
            flags
        );
        reply.error(ENOSYS);
    }

    /// Read symbolic link.
    fn readlink(&mut self, _req: &Request<'_>, ino: u64, reply: ReplyData) {
        log::debug!("[Not Implemented] readlink(ino: {:#x?})", ino);
        reply.error(ENOSYS);
    }

    /// Create file node.
    /// Create a regular file, character device, block device, fifo or socket node.
    fn mknod(
        &mut self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        mode: u32,
        umask: u32,
        rdev: u32,
        reply: ReplyEntry,
    ) {
        log::debug!(
            "[Not Implemented] mknod(parent: {:#x?}, name: {:?}, mode: {}, \
                umask: {:#x?}, rdev: {})",
            parent,
            name,
            mode,
            umask,
            rdev
        );
        reply.error(ENOSYS);
    }

    /// Create a directory.
    fn mkdir(
        &mut self,
        req: &Request<'_>,
        parent_ino: u64,
        name: &OsStr,
        mode: u32,
        _umask: u32,
        reply: ReplyEntry,
    ) {
        // TODO What to do with umask?
        let uid = Uid::from(req.uid());
        let gid = Gid::from(req.gid());
        let mode = Mode::from(mode).add_dir_flag();
        let inodes = Arc::clone(&self.inodes);
        let fs = Arc::clone(&self.fs);
        let name = name.to_owned();
        Self::run_async_reply_entry(
            &self.runtime,
            format!("mkdir(parent={parent_ino}, name={name:?}, mode={mode:?})"),
            reply,
            async move {
                let name: PathComponentBuf = name.try_into().map_err(|err| FsError::InvalidPath)?;

                let mut parent =
                    Self::get_inode(&fs, &inodes, FileHandle::from(parent_ino)).await?;
                let res = {
                    let parent_dir = parent.as_dir().await;
                    match parent_dir {
                        Ok(parent_dir) => {
                            let child = parent_dir.create_child_dir(&name, mode, uid, gid);
                            let child = child.await;
                            match child {
                                Ok((attrs, child)) => Ok((attrs, child.as_node())),
                                Err(err) => Err(err),
                            }
                        }
                        Err(err) => Err(err),
                    }
                };
                parent.async_drop().await?;
                let (attrs, child) = res?;
                let ino = inodes.write().await.add(AsyncDropArc::new(child));
                log::info!("New inode {ino:?}: parent={parent_ino:?}, name={name}");
                Ok((TTL_GETATTR, attrs, ino))
            },
        );
    }

    /// Remove a file.
    fn unlink(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        log::debug!(
            "[Not Implemented] unlink(parent: {:#x?}, name: {:?})",
            parent,
            name,
        );
        reply.error(ENOSYS);
    }

    /// Remove a directory.
    fn rmdir(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        log::debug!(
            "[Not Implemented] rmdir(parent: {:#x?}, name: {:?})",
            parent,
            name,
        );
        reply.error(ENOSYS);
    }

    /// Create a symbolic link.
    fn symlink(
        &mut self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        link: &Path,
        reply: ReplyEntry,
    ) {
        log::debug!(
            "[Not Implemented] symlink(parent: {:#x?}, name: {:?}, link: {:?})",
            parent,
            name,
            link,
        );
        reply.error(EPERM);
    }

    /// Rename a file.
    fn rename(
        &mut self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        newparent: u64,
        newname: &OsStr,
        flags: u32,
        reply: ReplyEmpty,
    ) {
        log::debug!(
            "[Not Implemented] rename(parent: {:#x?}, name: {:?}, newparent: {:#x?}, \
                newname: {:?}, flags: {})",
            parent,
            name,
            newparent,
            newname,
            flags,
        );
        reply.error(ENOSYS);
    }

    /// Create a hard link.
    fn link(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        newparent: u64,
        newname: &OsStr,
        reply: ReplyEntry,
    ) {
        log::debug!(
            "[Not Implemented] link(ino: {:#x?}, newparent: {:#x?}, newname: {:?})",
            ino,
            newparent,
            newname
        );
        reply.error(EPERM);
    }

    /// Open a file.
    /// Open flags (with the exception of O_CREAT, O_EXCL, O_NOCTTY and O_TRUNC) are
    /// available in flags. Filesystem may store an arbitrary file handle (pointer, index,
    /// etc) in fh, and use this in other all other file operations (read, write, flush,
    /// release, fsync). Filesystem may also implement stateless file I/O and not store
    /// anything in fh. There are also some flags (direct_io, keep_cache) which the
    /// filesystem may set, to change the way the file is opened. See fuse_file_info
    /// structure in <fuse_common.h> for more details.
    fn open(&mut self, _req: &Request<'_>, _ino: u64, _flags: i32, reply: ReplyOpen) {
        reply.opened(0, 0);
    }

    /// Read data.
    /// Read should send exactly the number of bytes requested except on EOF or error,
    /// otherwise the rest of the data will be substituted with zeroes. An exception to
    /// this is when the file has been opened in 'direct_io' mode, in which case the
    /// return value of the read system call will reflect the return value of this
    /// operation. fh will contain the value set by the open method, or will be undefined
    /// if the open method didn't set any value.
    ///
    /// flags: these are the file flags, such as O_SYNC. Only supported with ABI >= 7.9
    /// lock_owner: only supported with ABI >= 7.9
    fn read(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        size: u32,
        flags: i32,
        lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        log::warn!(
            "[Not Implemented] read(ino: {:#x?}, fh: {}, offset: {}, size: {}, \
                flags: {:#x?}, lock_owner: {:?})",
            ino,
            fh,
            offset,
            size,
            flags,
            lock_owner
        );
        reply.error(ENOSYS);
    }

    /// Write data.
    /// Write should return exactly the number of bytes requested except on error. An
    /// exception to this is when the file has been opened in 'direct_io' mode, in
    /// which case the return value of the write system call will reflect the return
    /// value of this operation. fh will contain the value set by the open method, or
    /// will be undefined if the open method didn't set any value.
    ///
    /// write_flags: will contain FUSE_WRITE_CACHE, if this write is from the page cache. If set,
    /// the pid, uid, gid, and fh may not match the value that would have been sent if write cachin
    /// is disabled
    /// flags: these are the file flags, such as O_SYNC. Only supported with ABI >= 7.9
    /// lock_owner: only supported with ABI >= 7.9
    fn write(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        data: &[u8],
        write_flags: u32,
        flags: i32,
        lock_owner: Option<u64>,
        reply: ReplyWrite,
    ) {
        log::debug!(
            "[Not Implemented] write(ino: {:#x?}, fh: {}, offset: {}, data.len(): {}, \
                write_flags: {:#x?}, flags: {:#x?}, lock_owner: {:?})",
            ino,
            fh,
            offset,
            data.len(),
            write_flags,
            flags,
            lock_owner
        );
        reply.error(ENOSYS);
    }

    /// Flush method.
    /// This is called on each close() of the opened file. Since file descriptors can
    /// be duplicated (dup, dup2, fork), for one open call there may be many flush
    /// calls. Filesystems shouldn't assume that flush will always be called after some
    /// writes, or that if will be called at all. fh will contain the value set by the
    /// open method, or will be undefined if the open method didn't set any value.
    /// NOTE: the name of the method is misleading, since (unlike fsync) the filesystem
    /// is not forced to flush pending writes. One reason to flush data, is if the
    /// filesystem wants to return write errors. If the filesystem supports file locking
    /// operations (setlk, getlk) it should remove all locks belonging to 'lock_owner'.
    fn flush(&mut self, _req: &Request<'_>, ino: u64, fh: u64, lock_owner: u64, reply: ReplyEmpty) {
        log::debug!(
            "[Not Implemented] flush(ino: {:#x?}, fh: {}, lock_owner: {:?})",
            ino,
            fh,
            lock_owner
        );
        reply.error(ENOSYS);
    }

    /// Release an open file.
    /// Release is called when there are no more references to an open file: all file
    /// descriptors are closed and all memory mappings are unmapped. For every open
    /// call there will be exactly one release call. The filesystem may reply with an
    /// error, but error values are not returned to close() or munmap() which triggered
    /// the release. fh will contain the value set by the open method, or will be undefined
    /// if the open method didn't set any value. flags will contain the same flags as for
    /// open.
    fn release(
        &mut self,
        _req: &Request<'_>,
        _ino: u64,
        _fh: u64,
        _flags: i32,
        _lock_owner: Option<u64>,
        _flush: bool,
        reply: ReplyEmpty,
    ) {
        reply.ok();
    }

    /// Synchronize file contents.
    /// If the datasync parameter is non-zero, then only the user data should be flushed,
    /// not the meta data.
    fn fsync(&mut self, _req: &Request<'_>, ino: u64, fh: u64, datasync: bool, reply: ReplyEmpty) {
        log::debug!(
            "[Not Implemented] fsync(ino: {:#x?}, fh: {}, datasync: {})",
            ino,
            fh,
            datasync
        );
        reply.error(ENOSYS);
    }

    /// Open a directory.
    /// Filesystem may store an arbitrary file handle (pointer, index, etc) in fh, and
    /// use this in other all other directory stream operations (readdir, releasedir,
    /// fsyncdir). Filesystem may also implement stateless directory I/O and not store
    /// anything in fh, though that makes it impossible to implement standard conforming
    /// directory stream operations in case the contents of the directory can change
    /// between opendir and releasedir.
    /// TODO Make tis standard-confirming
    fn opendir(&mut self, _req: &Request<'_>, _ino: u64, _flags: i32, reply: ReplyOpen) {
        reply.opened(0, 0);
    }

    fn readdir(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        reply: ReplyDirectory,
    ) {
        // TODO Can we optimize this so we don't lookup all entries for every batch if fuse requests them in batches with `offset`?

        let inodes = Arc::clone(&self.inodes);
        let fs = Arc::clone(&self.fs);
        Self::run_async_reply_directory(
            &self.runtime,
            format!("readdir(ino={ino}, fh={fh}, offset={offset})"),
            reply,
            async move {
                let ino = FileHandle::from(ino);
                let node = Self::get_inode(&fs, &inodes, ino).await?;
                with_async_drop_2!(node, {
                    let dir = node.as_dir().await?;
                    let entries = dir.entries();
                    let entries = entries.await?;
                    let dir = Arc::new(dir);
                    let offset = usize::try_from(offset).unwrap();
                    let entries = entries.into_iter().skip(offset).map(move |entry| {
                        // TODO Possible without Arc?
                        let dir = Arc::clone(&dir);
                        let inodes = Arc::clone(&inodes);
                        async move {
                            let child = dir.lookup_child(&entry.name);
                            // TODO No unwrap
                            let child = child.await.unwrap();
                            // TODO Check that readdir is actually supposed to register the inode and that [Self::forget] will be called for this inode
                            //      Note also that fuse-mt actually doesn't register the inode here and a comment there claims that fuse just ignores it, see https://github.com/wfraser/fuse-mt/blob/881d7320b4c73c0bfbcbca48a5faab2a26f3e9e8/src/fusemt.rs#L619
                            let child_ino = inodes.write().await.add(AsyncDropArc::new(child));
                            log::info!(
                                "New inode {child_ino:?}: parent={ino:?}, name={name}",
                                name = entry.name
                            );
                            (child_ino.handle, entry)
                        }
                    });
                    // TODO Possible without collecting into a Vec, maybe by returning an iterator over futures?
                    let entries: FuturesUnordered<_> = entries.collect();
                    let entries: Vec<(FileHandle, DirEntry)> = entries.collect().await;

                    Ok((offset, entries.into_iter()))
                })
            },
        );
    }

    /// Read directory.
    /// Send a buffer filled using buffer.fill(), with size not exceeding the
    /// requested size. Send an empty buffer on end of stream. fh will contain the
    /// value set by the opendir method, or will be undefined if the opendir method
    /// didn't set any value.
    fn readdirplus(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        reply: ReplyDirectoryPlus,
    ) {
        log::debug!(
            "[Not Implemented] readdirplus(ino: {:#x?}, fh: {}, offset: {})",
            ino,
            fh,
            offset
        );
        reply.error(ENOSYS);
    }

    /// Release an open directory.
    /// For every opendir call there will be exactly one releasedir call. fh will
    /// contain the value set by the opendir method, or will be undefined if the
    /// opendir method didn't set any value.
    fn releasedir(
        &mut self,
        _req: &Request<'_>,
        _ino: u64,
        _fh: u64,
        _flags: i32,
        reply: ReplyEmpty,
    ) {
        reply.ok();
    }

    /// Synchronize directory contents.
    /// If the datasync parameter is set, then only the directory contents should
    /// be flushed, not the meta data. fh will contain the value set by the opendir
    /// method, or will be undefined if the opendir method didn't set any value.
    fn fsyncdir(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        fh: u64,
        datasync: bool,
        reply: ReplyEmpty,
    ) {
        log::debug!(
            "[Not Implemented] fsyncdir(ino: {:#x?}, fh: {}, datasync: {})",
            ino,
            fh,
            datasync
        );
        reply.error(ENOSYS);
    }

    /// Get file system statistics.
    fn statfs(&mut self, _req: &Request<'_>, _ino: u64, reply: ReplyStatfs) {
        reply.statfs(0, 0, 0, 0, 0, 512, 255, 0);
    }

    /// Set an extended attribute.
    fn setxattr(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        name: &OsStr,
        _value: &[u8],
        flags: i32,
        position: u32,
        reply: ReplyEmpty,
    ) {
        log::debug!(
            "[Not Implemented] setxattr(ino: {:#x?}, name: {:?}, flags: {:#x?}, position: {})",
            ino,
            name,
            flags,
            position
        );
        reply.error(ENOSYS);
    }

    /// Get an extended attribute.
    /// If `size` is 0, the size of the value should be sent with `reply.size()`.
    /// If `size` is not 0, and the value fits, send it with `reply.data()`, or
    /// `reply.error(ERANGE)` if it doesn't.
    fn getxattr(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        name: &OsStr,
        size: u32,
        reply: ReplyXattr,
    ) {
        log::debug!(
            "[Not Implemented] getxattr(ino: {:#x?}, name: {:?}, size: {})",
            ino,
            name,
            size
        );
        reply.error(ENOSYS);
    }

    /// List extended attribute names.
    /// If `size` is 0, the size of the value should be sent with `reply.size()`.
    /// If `size` is not 0, and the value fits, send it with `reply.data()`, or
    /// `reply.error(ERANGE)` if it doesn't.
    fn listxattr(&mut self, _req: &Request<'_>, ino: u64, size: u32, reply: ReplyXattr) {
        log::debug!(
            "[Not Implemented] listxattr(ino: {:#x?}, size: {})",
            ino,
            size
        );
        reply.error(ENOSYS);
    }

    /// Remove an extended attribute.
    fn removexattr(&mut self, _req: &Request<'_>, ino: u64, name: &OsStr, reply: ReplyEmpty) {
        log::debug!(
            "[Not Implemented] removexattr(ino: {:#x?}, name: {:?})",
            ino,
            name
        );
        reply.error(ENOSYS);
    }

    /// Check file access permissions.
    /// This will be called for the access() system call. If the 'default_permissions'
    /// mount option is given, this method is not called. This method is not called
    /// under Linux kernel versions 2.4.x
    fn access(&mut self, _req: &Request<'_>, ino: u64, mask: i32, reply: ReplyEmpty) {
        log::debug!("[Not Implemented] access(ino: {:#x?}, mask: {})", ino, mask);
        reply.error(ENOSYS);
    }

    /// Create and open a file.
    /// If the file does not exist, first create it with the specified mode, and then
    /// open it. Open flags (with the exception of O_NOCTTY) are available in flags.
    /// Filesystem may store an arbitrary file handle (pointer, index, etc) in fh,
    /// and use this in other all other file operations (read, write, flush, release,
    /// fsync). There are also some flags (direct_io, keep_cache) which the
    /// filesystem may set, to change the way the file is opened. See fuse_file_info
    /// structure in <fuse_common.h> for more details. If this method is not
    /// implemented or under Linux kernel versions earlier than 2.6.15, the mknod()
    /// and open() methods will be called instead.
    fn create(
        &mut self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        mode: u32,
        umask: u32,
        flags: i32,
        reply: ReplyCreate,
    ) {
        log::debug!(
            "[Not Implemented] create(parent: {:#x?}, name: {:?}, mode: {}, umask: {:#x?}, \
                flags: {:#x?})",
            parent,
            name,
            mode,
            umask,
            flags
        );
        reply.error(ENOSYS);
    }

    /// Test for a POSIX file lock.
    fn getlk(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        fh: u64,
        lock_owner: u64,
        start: u64,
        end: u64,
        typ: i32,
        pid: u32,
        reply: ReplyLock,
    ) {
        log::debug!(
            "[Not Implemented] getlk(ino: {:#x?}, fh: {}, lock_owner: {}, start: {}, \
                end: {}, typ: {}, pid: {})",
            ino,
            fh,
            lock_owner,
            start,
            end,
            typ,
            pid
        );
        reply.error(ENOSYS);
    }

    /// Acquire, modify or release a POSIX file lock.
    /// For POSIX threads (NPTL) there's a 1-1 relation between pid and owner, but
    /// otherwise this is not always the case.  For checking lock ownership,
    /// 'fi->owner' must be used. The l_pid field in 'struct flock' should only be
    /// used to fill in this field in getlk(). Note: if the locking methods are not
    /// implemented, the kernel will still allow file locking to work locally.
    /// Hence these are only interesting for network filesystems and similar.
    fn setlk(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        fh: u64,
        lock_owner: u64,
        start: u64,
        end: u64,
        typ: i32,
        pid: u32,
        sleep: bool,
        reply: ReplyEmpty,
    ) {
        log::debug!(
            "[Not Implemented] setlk(ino: {:#x?}, fh: {}, lock_owner: {}, start: {}, \
                end: {}, typ: {}, pid: {}, sleep: {})",
            ino,
            fh,
            lock_owner,
            start,
            end,
            typ,
            pid,
            sleep
        );
        reply.error(ENOSYS);
    }

    /// Map block index within file to block index within device.
    /// Note: This makes sense only for block device backed filesystems mounted
    /// with the 'blkdev' option
    fn bmap(&mut self, _req: &Request<'_>, ino: u64, blocksize: u32, idx: u64, reply: ReplyBmap) {
        log::debug!(
            "[Not Implemented] bmap(ino: {:#x?}, blocksize: {}, idx: {})",
            ino,
            blocksize,
            idx,
        );
        reply.error(ENOSYS);
    }

    /// control device
    fn ioctl(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        fh: u64,
        flags: u32,
        cmd: u32,
        in_data: &[u8],
        out_size: u32,
        reply: ReplyIoctl,
    ) {
        log::debug!(
            "[Not Implemented] ioctl(ino: {:#x?}, fh: {}, flags: {}, cmd: {}, \
                in_data.len(): {}, out_size: {})",
            ino,
            fh,
            flags,
            cmd,
            in_data.len(),
            out_size,
        );
        reply.error(ENOSYS);
    }

    /// Preallocate or deallocate space to a file
    fn fallocate(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        length: i64,
        mode: i32,
        reply: ReplyEmpty,
    ) {
        log::debug!(
            "[Not Implemented] fallocate(ino: {:#x?}, fh: {}, offset: {}, \
                length: {}, mode: {})",
            ino,
            fh,
            offset,
            length,
            mode
        );
        reply.error(ENOSYS);
    }

    /// Reposition read/write file offset
    fn lseek(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        whence: i32,
        reply: ReplyLseek,
    ) {
        log::debug!(
            "[Not Implemented] lseek(ino: {:#x?}, fh: {}, offset: {}, whence: {})",
            ino,
            fh,
            offset,
            whence
        );
        reply.error(ENOSYS);
    }

    /// Copy the specified range from the source inode to the destination inode
    fn copy_file_range(
        &mut self,
        _req: &Request<'_>,
        ino_in: u64,
        fh_in: u64,
        offset_in: i64,
        ino_out: u64,
        fh_out: u64,
        offset_out: i64,
        len: u64,
        flags: u32,
        reply: ReplyWrite,
    ) {
        log::debug!(
            "[Not Implemented] copy_file_range(ino_in: {:#x?}, fh_in: {}, \
                offset_in: {}, ino_out: {:#x?}, fh_out: {}, offset_out: {}, \
                len: {}, flags: {})",
            ino_in,
            fh_in,
            offset_in,
            ino_out,
            fh_out,
            offset_out,
            len,
            flags
        );
        reply.error(ENOSYS);
    }

    /// macOS only: Rename the volume. Set fuse_init_out.flags during init to
    /// FUSE_VOL_RENAME to enable
    #[cfg(target_os = "macos")]
    fn setvolname(&mut self, _req: &Request<'_>, name: &OsStr, reply: ReplyEmpty) {
        log::debug!("[Not Implemented] setvolname(name: {:?})", name);
        reply.error(ENOSYS);
    }

    /// macOS only (undocumented)
    #[cfg(target_os = "macos")]
    fn exchange(
        &mut self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        newparent: u64,
        newname: &OsStr,
        options: u64,
        reply: ReplyEmpty,
    ) {
        log::debug!(
            "[Not Implemented] exchange(parent: {:#x?}, name: {:?}, newparent: {:#x?}, \
                newname: {:?}, options: {})",
            parent,
            name,
            newparent,
            newname,
            options
        );
        reply.error(ENOSYS);
    }

    /// macOS only: Query extended times (bkuptime and crtime). Set fuse_init_out.flags
    /// during init to FUSE_XTIMES to enable
    #[cfg(target_os = "macos")]
    fn getxtimes(&mut self, _req: &Request<'_>, ino: u64, reply: ReplyXTimes) {
        log::debug!("[Not Implemented] getxtimes(ino: {:#x?})", ino);
        reply.error(ENOSYS);
    }
}

impl<'a> From<&fuser::Request<'a>> for crate::high_level_api::RequestInfo {
    fn from(value: &fuser::Request<'a>) -> Self {
        Self {
            unique: value.unique(),
            uid: Uid::from(value.uid()),
            gid: Gid::from(value.gid()),
            pid: value.pid(),
        }
    }
}

fn convert_node_attrs(attrs: NodeAttrs, ino: FileHandle) -> fuser::FileAttr {
    let size: u64 = attrs.num_bytes.into();
    fuser::FileAttr {
        ino: ino.0,
        size,
        blocks: attrs.num_blocks.unwrap_or(size / 512),
        atime: attrs.atime,
        mtime: attrs.mtime,
        ctime: attrs.ctime,
        crtime: attrs.ctime, // TODO actually store and compute crtime
        kind: convert_node_kind(attrs.mode.node_kind()),
        perm: convert_permission_bits(attrs.mode),
        nlink: attrs.nlink,
        uid: attrs.uid.into(),
        gid: attrs.gid.into(),
        /// Device ID (if special file)
        rdev: 0, // TODO What to do about this?
        /// Flags (macOS only; see chflags(2))
        flags: 0, // TODO What to do about this?
        blksize: 4096, // TODO What to do about this?
    }
}

fn convert_node_kind(kind: NodeKind) -> fuser::FileType {
    match kind {
        NodeKind::File => fuser::FileType::RegularFile,
        NodeKind::Dir => fuser::FileType::Directory,
        NodeKind::Symlink => fuser::FileType::Symlink,
    }
}

fn convert_permission_bits(mode: Mode) -> u16 {
    let mode_bits: u32 = mode.into();
    // TODO Is 0o777 the right mask or do we need 0o7777?
    let perm_bits = mode_bits & 0o777;
    perm_bits as u16
}
