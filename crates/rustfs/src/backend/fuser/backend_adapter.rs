#[cfg(target_os = "macos")]
use fuser::ReplyXTimes;
use fuser::{
    Filesystem, KernelConfig, ReplyAttr, ReplyBmap, ReplyCreate, ReplyData, ReplyDirectory,
    ReplyDirectoryPlus, ReplyEmpty, ReplyEntry, ReplyIoctl, ReplyLock, ReplyLseek, ReplyOpen,
    ReplyStatfs, ReplyWrite, ReplyXattr, Request, TimeOrNow,
};
use libc::c_int;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::future::Future;
use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

use crate::common::{
    FileHandle, FsError, FsResult, Gid, InodeNumber, Mode, NodeAttrs, NodeKind, NumBytes,
    OpenFlags, PathComponentBuf, RequestInfo, Statfs, Uid,
};
use crate::low_level_api::{self, AsyncFilesystemLL};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

// TODO Fuse has a requirement that (inode, generation) tuples are unique throughout the lifetime of the filesystem, not just the lifetime of the mount.
//      See https://github.com/libfuse/libfuse/blob/d92bf83c152ff88c2d92bd852752d4c326004400/include/fuse_lowlevel.h#L69-L81 and https://github.com/wfraser/fuse-mt/issues/19
//      This means currently, CryFS can't be used over NFS. We should fix this.

pub struct BackendAdapter<Fs>
where
    // TODO Send + Sync + 'static needed?
    Fs: AsyncFilesystemLL + AsyncDrop<Error = FsError> + Debug + Send + Sync + 'static,
{
    // TODO RwLock is only needed for async drop. Can we remove it? init() and destroy() are called on &mut self so they should be exclusive anyways.
    fs: Arc<tokio::sync::RwLock<AsyncDropGuard<Fs>>>,

    runtime: tokio::runtime::Handle,
}

impl<Fs> Debug for BackendAdapter<Fs>
where
    Fs: AsyncFilesystemLL + AsyncDrop<Error = FsError> + Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BackendAdapter").finish()
    }
}

impl<Fs> BackendAdapter<Fs>
where
    Fs: AsyncFilesystemLL + AsyncDrop<Error = FsError> + Debug + Send + Sync + 'static,
{
    pub fn new(fs: AsyncDropGuard<Fs>, runtime: tokio::runtime::Handle) -> Self {
        Self {
            fs: Arc::new(RwLock::new(fs)),
            runtime,
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

    fn run_async_no_reply<F>(
        &self,
        log_msg: String,
        func: impl Send + 'static + FnOnce(Arc<RwLock<AsyncDropGuard<Fs>>>) -> F,
    ) where
        F: Future<Output = FsResult<()>> + Send,
    {
        let fs = Arc::clone(&self.fs);
        self.runtime.spawn(async move {
            log::info!("{}...", log_msg);
            match func(fs).await {
                Ok(()) => {
                    log::info!("{}...done", log_msg);
                }
                Err(err) => {
                    log::info!("{}...failed: {}", log_msg, err);
                }
            }
        });
    }

    fn run_async_reply_empty<F>(
        &self,
        log_msg: String,
        fuser_reply: fuser::ReplyEmpty,
        func: impl Send + 'static + FnOnce(Arc<RwLock<AsyncDropGuard<Fs>>>) -> F,
    ) where
        F: Future<Output = FsResult<()>> + Send,
    {
        let fs = Arc::clone(&self.fs);
        self.runtime.spawn(async move {
            log::info!("{}...", log_msg);
            match func(fs).await {
                Ok(()) => {
                    log::info!("{}...done", log_msg);
                    fuser_reply.ok();
                }
                Err(err) => {
                    log::info!("{}...failed: {}", log_msg, err);
                    fuser_reply.error(err.system_error_code())
                }
            }
        });
    }

    fn run_async_reply_entry<F>(
        &self,
        log_msg: String,
        fuser_reply: fuser::ReplyEntry,
        func: impl Send + 'static + FnOnce(Arc<RwLock<AsyncDropGuard<Fs>>>) -> F,
    ) where
        F: Future<Output = FsResult<low_level_api::ReplyEntry>> + Send,
    {
        let fs = Arc::clone(&self.fs);
        self.runtime.spawn(async move {
            log::info!("{}...", log_msg);
            match func(fs).await {
                Ok(reply) => {
                    log::info!("{}...done", log_msg);
                    fuser_reply.entry(
                        &reply.ttl,
                        &convert_node_attrs(reply.attr, reply.ino.handle),
                        reply.ino.generation,
                    );
                }
                Err(err) => {
                    log::info!("{}...failed: {}", log_msg, err);
                    fuser_reply.error(err.system_error_code())
                }
            }
        });
    }

    fn run_async_reply_attr<F>(
        &self,
        log_msg: String,
        fuser_reply: fuser::ReplyAttr,
        func: impl Send + 'static + FnOnce(Arc<RwLock<AsyncDropGuard<Fs>>>) -> F,
    ) where
        F: Future<Output = FsResult<low_level_api::ReplyAttr>> + Send,
    {
        let fs = Arc::clone(&self.fs);
        self.runtime.spawn(async move {
            log::info!("{}...", log_msg);
            match func(fs).await {
                Ok(reply) => {
                    log::info!("{}...done", log_msg);
                    fuser_reply.attr(&reply.ttl, &convert_node_attrs(reply.attr, reply.ino));
                }
                Err(err) => {
                    log::info!("{}...failed: {}", log_msg, err);
                    fuser_reply.error(err.system_error_code())
                }
            }
        });
    }

    fn run_async_reply_open<F>(
        &self,
        log_msg: String,
        fuser_reply: fuser::ReplyOpen,
        func: impl Send + 'static + FnOnce(Arc<RwLock<AsyncDropGuard<Fs>>>) -> F,
    ) where
        F: Future<Output = FsResult<low_level_api::ReplyOpen>> + Send,
    {
        let fs = Arc::clone(&self.fs);
        self.runtime.spawn(async move {
            log::info!("{}...", log_msg);
            match func(fs).await {
                Ok(reply) => {
                    log::info!("{}...done", log_msg);
                    let flags = convert_openflags(reply.flags);
                    // TODO Why u32 and not i32?
                    let flags = u32::try_from(flags).unwrap();
                    fuser_reply.opened(reply.fh.into(), flags);
                }
                Err(err) => {
                    log::info!("{}...failed: {}", log_msg, err);
                    fuser_reply.error(err.system_error_code())
                }
            }
        });
    }

    fn run_async_reply_write<F>(
        &self,
        log_msg: String,
        fuser_reply: fuser::ReplyWrite,
        func: impl Send + 'static + FnOnce(Arc<RwLock<AsyncDropGuard<Fs>>>) -> F,
    ) where
        F: Future<Output = FsResult<low_level_api::ReplyWrite>> + Send,
    {
        let fs = Arc::clone(&self.fs);
        self.runtime.spawn(async move {
            log::info!("{}...", log_msg);
            match func(fs).await {
                Ok(reply) => {
                    log::info!("{}...done", log_msg);
                    fuser_reply.written(reply.written);
                }
                Err(err) => {
                    log::info!("{}...failed: {}", log_msg, err);
                    fuser_reply.error(err.system_error_code())
                }
            }
        });
    }

    fn run_async_reply_statfs<F>(
        &self,
        log_msg: String,
        fuser_reply: fuser::ReplyStatfs,
        func: impl Send + 'static + FnOnce(Arc<RwLock<AsyncDropGuard<Fs>>>) -> F,
    ) where
        F: Future<Output = FsResult<Statfs>> + Send,
    {
        let fs = Arc::clone(&self.fs);
        self.runtime.spawn(async move {
            log::info!("{}...", log_msg);
            match func(fs).await {
                Ok(reply) => {
                    log::info!("{}...done", log_msg);
                    let blocks = reply.num_total_blocks;
                    let bfree = reply.num_free_blocks;
                    let bavail = reply.num_available_blocks;
                    let files = reply.num_total_inodes;
                    let ffree = reply.num_free_inodes;
                    let bsize = reply.blocksize;
                    let namelen = reply.max_filename_length;
                    // TODO What is fragment size? Should it be different to blocksize?
                    let frsize = reply.blocksize;
                    fuser_reply.statfs(blocks, bfree, bavail, files, ffree, bsize, namelen, frsize);
                }
                Err(err) => {
                    log::info!("{}...failed: {}", log_msg, err);
                    fuser_reply.error(err.system_error_code())
                }
            }
        });
    }

    fn run_async_reply_create<F>(
        &self,
        log_msg: String,
        fuser_reply: fuser::ReplyCreate,
        func: impl Send + 'static + FnOnce(Arc<RwLock<AsyncDropGuard<Fs>>>) -> F,
    ) where
        F: Future<Output = FsResult<low_level_api::ReplyCreate>> + Send,
    {
        let fs = Arc::clone(&self.fs);
        self.runtime.spawn(async move {
            log::info!("{}...", log_msg);
            match func(fs).await {
                Ok(reply) => {
                    log::info!("{}...done", log_msg);
                    fuser_reply.created(
                        &reply.ttl,
                        &convert_node_attrs(reply.attr, reply.ino.handle),
                        reply.ino.generation,
                        reply.fh.into(),
                        reply.flags,
                    );
                }
                Err(err) => {
                    log::info!("{}...failed: {}", log_msg, err);
                    fuser_reply.error(err.system_error_code())
                }
            }
        });
    }

    fn run_async_reply_lock<F>(
        &self,
        log_msg: String,
        fuser_reply: fuser::ReplyLock,
        func: impl Send + 'static + FnOnce(Arc<RwLock<AsyncDropGuard<Fs>>>) -> F,
    ) where
        F: Future<Output = FsResult<low_level_api::ReplyLock>> + Send,
    {
        let fs = Arc::clone(&self.fs);
        self.runtime.spawn(async move {
            log::info!("{}...", log_msg);
            match func(fs).await {
                Ok(reply) => {
                    log::info!("{}...done", log_msg);
                    fuser_reply.locked(reply.start.into(), reply.end.into(), reply.typ, reply.pid);
                }
                Err(err) => {
                    log::info!("{}...failed: {}", log_msg, err);
                    fuser_reply.error(err.system_error_code())
                }
            }
        });
    }

    fn run_async_reply_bmap<F>(
        &self,
        log_msg: String,
        fuser_reply: fuser::ReplyBmap,
        func: impl Send + 'static + FnOnce(Arc<RwLock<AsyncDropGuard<Fs>>>) -> F,
    ) where
        F: Future<Output = FsResult<low_level_api::ReplyBmap>> + Send,
    {
        let fs = Arc::clone(&self.fs);
        self.runtime.spawn(async move {
            log::info!("{}...", log_msg);
            match func(fs).await {
                Ok(reply) => {
                    log::info!("{}...done", log_msg);
                    fuser_reply.bmap(reply.block);
                }
                Err(err) => {
                    log::info!("{}...failed: {}", log_msg, err);
                    fuser_reply.error(err.system_error_code())
                }
            }
        });
    }

    fn run_async_reply_lseek<F>(
        &self,
        log_msg: String,
        fuser_reply: fuser::ReplyLseek,
        func: impl Send + 'static + FnOnce(Arc<RwLock<AsyncDropGuard<Fs>>>) -> F,
    ) where
        F: Future<Output = FsResult<low_level_api::ReplyLseek>> + Send,
    {
        let fs = Arc::clone(&self.fs);
        self.runtime.spawn(async move {
            log::info!("{}...", log_msg);
            match func(fs).await {
                Ok(reply) => {
                    log::info!("{}...done", log_msg);
                    let offset: u64 = reply.offset.into();
                    // TODO Why does fuse use i64 instead of u64?
                    let offset: i64 = i64::try_from(offset).unwrap(); // TODO No unwrap
                    fuser_reply.offset(offset);
                }
                Err(err) => {
                    log::info!("{}...failed: {}", log_msg, err);
                    fuser_reply.error(err.system_error_code())
                }
            }
        });
    }

    #[cfg(target_os = "macos")]
    fn run_async_reply_xtimes<F>(
        &self,
        log_msg: String,
        fuser_reply: fuser::ReplyXTimes,
        func: impl Send + 'static + FnOnce(Arc<RwLock<AsyncDropGuard<Fs>>>) -> F,
    ) where
        F: Future<Output = FsResult<low_level_api::ReplyXTimes>> + Send,
    {
        let fs = Arc::clone(&self.fs);
        self.runtime.spawn(async move {
            log::info!("{}...", log_msg);
            match func(fs).await {
                Ok(reply) => {
                    log::info!("{}...done", log_msg);
                    fuser_reply.xtimes(reply.bkuptime, reply.crtime);
                }
                Err(err) => {
                    log::info!("{}...failed: {}", log_msg, err);
                    fuser_reply.error(err.system_error_code())
                }
            }
        });
    }

    fn run_async_reply_data<F>(
        &self,
        log_msg: String,
        reply: fuser::ReplyData,
        // TODO If we could do `for <Callback: FnOnce> impl ...`, we wouldn't need the DataCallback class
        func: impl Send + 'static + FnOnce(Arc<RwLock<AsyncDropGuard<Fs>>>, DataCallback) -> F,
    ) where
        F: Future<Output = ()> + Send,
    {
        let fs = Arc::clone(&self.fs);
        self.runtime.spawn(async move {
            log::info!("{}...", log_msg);
            func(fs, DataCallback { reply, log_msg });
        });
    }
}

impl<Fs> Filesystem for BackendAdapter<Fs>
where
    Fs: AsyncFilesystemLL + AsyncDrop<Error = FsError> + Debug + Send + Sync + 'static,
{
    fn init(&mut self, req: &Request<'_>, config: &mut KernelConfig) -> Result<(), c_int> {
        Self::run_blocking(&self.runtime, &format!("init"), || async {
            self.fs
                .write()
                .await
                .init(&RequestInfo::from(req), config)
                .await
        })
    }

    fn destroy(&mut self) {
        Self::run_blocking(&self.runtime, &format!("destroy"), || async {
            let mut fs = self.fs.write().await;
            fs.destroy().await;
            fs.async_drop().await.unwrap();
            Ok(())
        })
        .expect("failed to drop file system");

        // TODO Is there a way to do the above without a call to expect()?
    }

    fn lookup(
        &mut self,
        req: &Request<'_>,
        parent_ino: u64,
        name: &OsStr,
        reply: fuser::ReplyEntry,
    ) {
        // TODO Is this possible without name.to_owned()?
        let req = RequestInfo::from(req);
        let name = name.to_owned();
        let parent_ino = InodeNumber::from(parent_ino);
        self.run_async_reply_entry(
            format!("lookup(parent={parent_ino:?}, name={name:?}"),
            reply,
            move |fs| async move {
                let name: PathComponentBuf = name.try_into().map_err(|err| FsError::InvalidPath)?;
                fs.read().await.lookup(&req, parent_ino, &name).await
            },
        );
    }

    fn forget(&mut self, req: &Request, ino: u64, nlookup: u64) {
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        self.run_async_no_reply(format!("forget(ino={ino:?})"), move |fs| async move {
            fs.read().await.forget(&req, ino, nlookup).await
        });
    }

    // TODO Do we want this? It seems to be gated by an "abi-7-16" feature but what is that?
    // fn batch_forget(&mut self, req: &Request<'_>, nodes: &[fuse_forget_one]) {
    //     todo!()
    // }

    fn getattr(&mut self, req: &Request<'_>, ino: u64, reply: ReplyAttr) {
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        self.run_async_reply_attr(
            format!("getattr(ino={ino:?})"),
            reply,
            move |fs| async move { fs.read().await.getattr(&req, ino).await },
        );
    }

    fn setattr(
        &mut self,
        req: &Request<'_>,
        ino: u64,
        mode: Option<u32>,
        uid: Option<u32>,
        gid: Option<u32>,
        size: Option<u64>,
        atime: Option<TimeOrNow>,
        mtime: Option<TimeOrNow>,
        ctime: Option<SystemTime>,
        fh: Option<u64>,
        crtime: Option<SystemTime>,
        chgtime: Option<SystemTime>,
        bkuptime: Option<SystemTime>,
        flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        let mode = mode.map(Mode::from);
        let uid = uid.map(Uid::from);
        let gid = gid.map(Gid::from);
        let size = size.map(NumBytes::from);
        let atime = atime.map(parse_time);
        let mtime = mtime.map(parse_time);
        let fh = fh.map(FileHandle::from);
        self.run_async_reply_attr(
            format!("setattr(ino={ino:?}, mode={mode:?}, uid={uid:?}, gid={gid:?}, size={size:?}, atime={atime:?}, mtime={mtime:?}, ctime={ctime:?}, fh={fh:?}, crtime={crtime:?}, chgtime={chgtime:?}, bkuptime={bkuptime:?}, flags={flags:?}"),
            reply,
            move |fs| async move {
                fs.read().await.setattr(&req, ino, mode, uid, gid, size, atime, mtime, ctime, fh, crtime, chgtime, bkuptime, flags).await
            });
    }

    fn readlink(&mut self, req: &Request<'_>, ino: u64, reply: ReplyData) {
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        self.run_async_reply_data(
            format!("readlink(ino={ino:?})"),
            reply,
            move |fs, callback| async move {
                fs.read()
                    .await
                    .readlink(&req, ino, |result| callback.call(result.map(str::as_bytes)))
                    .await
            },
        );
    }

    fn mknod(
        &mut self,
        req: &Request<'_>,
        parent_ino: u64,
        name: &OsStr,
        mode: u32,
        umask: u32,
        rdev: u32,
        reply: ReplyEntry,
    ) {
        let req = RequestInfo::from(req);
        let parent_ino = InodeNumber::from(parent_ino);
        let name = name.to_owned();
        let mode = Mode::from(mode);
        self.run_async_reply_entry(
            format!("mknod(parent={parent_ino:?}, name={name:?}, mode={mode:?}, umask={umask:?}, rdev={rdev:?})"),
            reply,
            move |fs| async move {
                let name: PathComponentBuf = name.try_into().map_err(|err| FsError::InvalidPath)?;
                fs.read().await.mknod(&req, parent_ino, &name, mode, umask, rdev).await
            },
        );
    }

    fn mkdir(
        &mut self,
        req: &Request<'_>,
        parent_ino: u64,
        name: &OsStr,
        mode: u32,
        umask: u32,
        reply: ReplyEntry,
    ) {
        // TODO Assert that file/symlink flags aren't set
        let req = RequestInfo::from(req);
        let parent_ino = InodeNumber::from(parent_ino);
        let name = name.to_owned();
        let mode = Mode::from(mode).add_dir_flag();
        self.run_async_reply_entry(
            format!("mkdir(parent={parent_ino:?}, name={name:?}, mode={mode:?}, umask={umask:?})"),
            reply,
            move |fs| async move {
                let name: PathComponentBuf = name.try_into().map_err(|err| FsError::InvalidPath)?;
                fs.read()
                    .await
                    .mkdir(&req, parent_ino, &name, mode, umask)
                    .await
            },
        );
    }

    fn unlink(&mut self, req: &Request<'_>, parent_ino: u64, name: &OsStr, reply: ReplyEmpty) {
        let req = RequestInfo::from(req);
        let parent_ino = InodeNumber::from(parent_ino);
        let name = name.to_owned();
        self.run_async_reply_empty(
            format!("unlink(parent={parent_ino:?}, name={name:?})"),
            reply,
            move |fs| async move {
                let name: PathComponentBuf = name.try_into().map_err(|err| FsError::InvalidPath)?;
                fs.read().await.unlink(&req, parent_ino, &name).await
            },
        );
    }

    fn rmdir(&mut self, req: &Request<'_>, parent_ino: u64, name: &OsStr, reply: ReplyEmpty) {
        let req = RequestInfo::from(req);
        let parent_ino = InodeNumber::from(parent_ino);
        let name = name.to_owned();
        self.run_async_reply_empty(
            format!("rmdir(parent={parent_ino:?}, name={name:?})"),
            reply,
            move |fs| async move {
                let name: PathComponentBuf = name.try_into().map_err(|err| FsError::InvalidPath)?;
                fs.read().await.rmdir(&req, parent_ino, &name).await
            },
        );
    }

    fn symlink(
        &mut self,
        req: &Request<'_>,
        parent_ino: u64,
        name: &OsStr,
        link: &Path,
        reply: ReplyEntry,
    ) {
        let req = RequestInfo::from(req);
        let parent_ino = InodeNumber::from(parent_ino);
        let name = name.to_owned();
        let link = link.to_owned();
        self.run_async_reply_entry(
            format!("symlink(parent={parent_ino:?}, name={name:?}, link={link:?})"),
            reply,
            move |fs| async move {
                let name: PathComponentBuf = name.try_into().map_err(|err| FsError::InvalidPath)?;
                let link = link
                    .into_os_string()
                    .into_string()
                    .map_err(|err| FsError::InvalidPath)?;
                fs.read()
                    .await
                    .symlink(&req, parent_ino, &name, &link)
                    .await
            },
        );
    }

    fn rename(
        &mut self,
        req: &Request<'_>,
        parent_ino: u64,
        name: &OsStr,
        newparent: u64,
        newname: &OsStr,
        flags: u32,
        reply: ReplyEmpty,
    ) {
        let req = RequestInfo::from(req);
        let parent_ino = InodeNumber::from(parent_ino);
        let name = name.to_owned();
        let newparent = InodeNumber::from(newparent);
        let newname = newname.to_owned();
        self.run_async_reply_empty(
            format!("rename(parent={parent_ino:?}, name={name:?}, newparent={newparent:?}, newname={newname:?}, flags={flags:?})"),
            reply,
            move |fs| async move {
                let name: PathComponentBuf = name.try_into().map_err(|err| FsError::InvalidPath)?;
                let newname: PathComponentBuf =
                    newname.try_into().map_err(|err| FsError::InvalidPath)?;
                fs.read().await.rename(&req, parent_ino, &name, newparent, &newname, flags)
                    .await
            },
        );
    }

    fn link(
        &mut self,
        req: &Request<'_>,
        ino: u64,
        newparent: u64,
        newname: &OsStr,
        reply: ReplyEntry,
    ) {
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        let newparent = InodeNumber::from(newparent);
        let newname = newname.to_owned();
        self.run_async_reply_entry(
            format!("link(ino={ino:?}, newparent={newparent:?}, newname={newname:?})"),
            reply,
            move |fs| async move {
                let newname: PathComponentBuf =
                    newname.try_into().map_err(|err| FsError::InvalidPath)?;
                fs.read().await.link(&req, ino, newparent, &newname).await
            },
        );
    }

    fn open(&mut self, req: &Request<'_>, ino: u64, flags: i32, reply: ReplyOpen) {
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        let flags = parse_openflags(flags);
        self.run_async_reply_open(
            format!("open(ino={ino:?}, flags={flags:?})"),
            reply,
            move |fs| async move { fs.read().await.open(&req, ino, flags).await },
        );
    }

    fn read(
        &mut self,
        req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        size: u32,
        flags: i32,
        lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        let fh = FileHandle::from(fh);
        let offset = NumBytes::from(u64::try_from(offset).unwrap()); // TODO No unwrap?
        let size = NumBytes::from(u64::from(size));
        self.run_async_reply_data(
            format!("read(ino={ino:?}, fh={fh:?}, offset={offset:?}, size={size:?}, flags={flags:?}, lock_owner={lock_owner:?})"),
            reply,
            move |fs, callback| async move {
                fs.read().await.read(
                    &req,
                    ino,
                    fh,
                    offset,
                    size,
                    flags,
                    lock_owner,
                    |result| callback.call(result.map(|data| data.as_ref())),
                )
                .await
            },
        );
    }

    fn write(
        &mut self,
        req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        data: &[u8],
        write_flags: u32,
        flags: i32,
        lock_owner: Option<u64>,
        reply: ReplyWrite,
    ) {
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        let fh = FileHandle::from(fh);
        let offset = NumBytes::from(u64::try_from(offset).unwrap()); // TODO No unwrap?
        let data = data.to_owned();
        self.run_async_reply_write(
            format!("write(ino={ino:?}, fh={fh:?}, offset={offset:?}, data={data:?}, write_flags={write_flags:?}, flags={flags:?}, lock_owner={lock_owner:?})"),
            reply,
            move |fs| async move {
                fs.read().await.write(
                    &req,
                    ino,
                    fh,
                    offset,
                    &data,
                    write_flags,
                    flags,
                    lock_owner,
                )
                .await
            },
        );
    }

    fn flush(&mut self, req: &Request<'_>, ino: u64, fh: u64, lock_owner: u64, reply: ReplyEmpty) {
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        let fh = FileHandle::from(fh);
        self.run_async_reply_empty(
            format!("flush(ino={ino:?}, fh={fh:?}, lock_owner={lock_owner:?})"),
            reply,
            move |fs| async move { fs.read().await.flush(&req, ino, fh, lock_owner).await },
        );
    }

    fn release(
        &mut self,
        req: &Request<'_>,
        ino: u64,
        fh: u64,
        flags: i32,
        lock_owner: Option<u64>,
        flush: bool,
        reply: ReplyEmpty,
    ) {
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        let fh = FileHandle::from(fh);
        self.run_async_reply_empty(
            format!("release(ino={ino:?}, fh={fh:?}, flags={flags:?}, lock_owner={lock_owner:?}, flush={flush:?})"),
            reply,
            move |fs| async move { fs.read().await.release(&req, ino, fh, flags, lock_owner, flush).await },
        );
    }

    fn fsync(&mut self, req: &Request<'_>, ino: u64, fh: u64, datasync: bool, reply: ReplyEmpty) {
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        let fh = FileHandle::from(fh);
        self.run_async_reply_empty(
            format!("fsync(ino={ino:?}, fh={fh:?}, datasync={datasync:?})"),
            reply,
            move |fs| async move { fs.read().await.fsync(&req, ino, fh, datasync).await },
        );
    }

    fn opendir(&mut self, req: &Request<'_>, ino: u64, flags: i32, reply: ReplyOpen) {
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        self.run_async_reply_open(
            format!("opendir(ino={ino:?}, flags={flags:?})"),
            reply,
            move |fs| async move { fs.read().await.opendir(&req, ino, flags).await },
        );
    }

    fn readdir(
        &mut self,
        req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        reply: ReplyDirectory,
    ) {
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        let fh = FileHandle::from(fh);
        let offset = NumBytes::from(u64::try_from(offset).unwrap()); // TODO No unwrap?
        self.run_async_no_reply(
            format!("readdir(ino={ino:?}, fh={fh:?}, offset={offset:?})"),
            move |fs| async move {
                fs.read().await.readdir(&req, ino, fh, offset, reply).await;
                Ok(())
            },
        );
    }

    fn readdirplus(
        &mut self,
        req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        reply: ReplyDirectoryPlus,
    ) {
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        let fh = FileHandle::from(fh);
        let offset = NumBytes::from(u64::try_from(offset).unwrap()); // TODO No unwrap?
        self.run_async_no_reply(
            format!("readdirplus(ino={ino:?}, fh={fh:?}, offset={offset:?})"),
            move |fs| async move {
                fs.read()
                    .await
                    .readdirplus(&req, ino, fh, offset, reply)
                    .await;
                Ok(())
            },
        );
    }

    fn releasedir(&mut self, req: &Request<'_>, ino: u64, fh: u64, flags: i32, reply: ReplyEmpty) {
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        let fh = FileHandle::from(fh);
        self.run_async_reply_empty(
            format!("releasedir(ino={ino:?}, fh={fh:?}, flags={flags:?})"),
            reply,
            move |fs| async move { fs.read().await.releasedir(&req, ino, fh, flags).await },
        );
    }

    fn fsyncdir(
        &mut self,
        req: &Request<'_>,
        ino: u64,
        fh: u64,
        datasync: bool,
        reply: ReplyEmpty,
    ) {
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        let fh = FileHandle::from(fh);
        self.run_async_reply_empty(
            format!("fsyncdir(ino={ino:?}, fh={fh:?}, datasync={datasync:?})"),
            reply,
            move |fs| async move { fs.read().await.fsyncdir(&req, ino, fh, datasync).await },
        );
    }

    fn statfs(&mut self, req: &Request<'_>, ino: u64, reply: ReplyStatfs) {
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        self.run_async_reply_statfs(
            format!("statfs(ino={ino:?})"),
            reply,
            move |fs| async move { fs.read().await.statfs(&req, ino).await },
        );
    }

    fn setxattr(
        &mut self,
        req: &Request<'_>,
        ino: u64,
        name: &OsStr,
        value: &[u8],
        flags: i32,
        position: u32,
        reply: ReplyEmpty,
    ) {
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        let name = name.to_owned();
        let value = value.to_owned();
        let position = NumBytes::from(u64::from(position));
        self.run_async_reply_empty(
            format!("setxattr(ino={ino:?}, name={name:?}, value={value:?}, flags={flags:?}, position={position:?})"),
            reply,
            move |fs| async move {
                // TODO InvalidPath is probably the wrong error here
                let name = PathComponentBuf::try_from(name).map_err(|err| FsError::InvalidPath)?;
                fs.read().await.setxattr(&req, ino, &name, &value, flags, position)
                    .await
            },
        );
    }

    fn getxattr(
        &mut self,
        req: &Request<'_>,
        ino: u64,
        name: &OsStr,
        size: u32,
        reply: ReplyXattr,
    ) {
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        let name = name.to_owned();
        let size = NumBytes::from(u64::from(size));
        self.run_async_no_reply(
            format!("getxattr(ino={ino:?}, name={name:?}, size={size:?})"),
            move |fs| async move {
                // TODO InvalidPath is probably the wrong error here
                let name = PathComponentBuf::try_from(name).map_err(|err| FsError::InvalidPath)?;
                fs.read()
                    .await
                    .getxattr(&req, ino, &name, size, reply)
                    .await;
                Ok(())
            },
        );
    }

    fn listxattr(&mut self, req: &Request<'_>, ino: u64, size: u32, reply: ReplyXattr) {
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        let size = NumBytes::from(u64::from(size));
        self.run_async_no_reply(
            format!("listxattr(ino={ino:?}, size={size:?})"),
            move |fs| async move {
                fs.read().await.listxattr(&req, ino, size, reply).await;
                Ok(())
            },
        );
    }

    fn removexattr(&mut self, _req: &Request<'_>, ino: u64, name: &OsStr, reply: ReplyEmpty) {
        let req = RequestInfo::from(_req);
        let ino = InodeNumber::from(ino);
        let name = name.to_owned();
        self.run_async_reply_empty(
            format!("removexattr(ino={ino:?}, name={name:?})"),
            reply,
            move |fs| async move {
                // TODO InvalidPath is probably the wrong error here
                let name = PathComponentBuf::try_from(name).map_err(|err| FsError::InvalidPath)?;
                fs.read().await.removexattr(&req, ino, &name).await
            },
        );
    }

    fn access(&mut self, req: &Request<'_>, ino: u64, mask: i32, reply: ReplyEmpty) {
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        self.run_async_reply_empty(
            format!("access(ino={ino:?}, mask={mask:?})"),
            reply,
            move |fs| async move { fs.read().await.access(&req, ino, mask).await },
        );
    }

    fn create(
        &mut self,
        req: &Request<'_>,
        parent_ino: u64,
        name: &OsStr,
        mode: u32,
        umask: u32,
        flags: i32,
        reply: ReplyCreate,
    ) {
        // TODO Assert that dir/symlink flags aren't set
        let req = RequestInfo::from(req);
        let parent_ino = InodeNumber::from(parent_ino);
        let name = name.to_owned();
        let mode = Mode::from(mode).add_file_flag();
        self.run_async_reply_create(
            format!("create(parent={parent_ino:?}, name={name:?}, mode={mode:?}, umask={umask:?}, flags={flags:?})"),
            reply,
            move |fs| async move {
                let name: PathComponentBuf = name.try_into().map_err(|err| FsError::InvalidPath)?;
                fs.read().await.create(&req, parent_ino, &name, mode, umask, flags)
                    .await
            },
        );
    }

    fn getlk(
        &mut self,
        req: &Request<'_>,
        ino: u64,
        fh: u64,
        lock_owner: u64,
        start: u64,
        end: u64,
        typ: i32,
        pid: u32,
        reply: ReplyLock,
    ) {
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        let fh = FileHandle::from(fh);
        self.run_async_reply_lock(
            format!("getlk(ino={ino:?}, fh={fh:?}, lock_owner={lock_owner:?}, start={start:?}, end={end:?}, typ={typ:?}, pid={pid:?})"),
            reply,
            move |fs| async move {
                fs.read().await.getlk(&req, ino, fh, lock_owner, start, end, typ, pid)
                    .await
            },
        );
    }

    fn setlk(
        &mut self,
        req: &Request<'_>,
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
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        let fh = FileHandle::from(fh);
        self.run_async_reply_empty(
            format!("setlk(ino={ino:?}, fh={fh:?}, lock_owner={lock_owner:?}, start={start:?}, end={end:?}, typ={typ:?}, pid={pid:?}, sleep={sleep:?})"),
            reply,
            move |fs| async move {
                fs.read().await.setlk(&req, ino, fh, lock_owner, start, end, typ, pid, sleep)
                    .await
            },
        );
    }

    fn bmap(&mut self, req: &Request<'_>, ino: u64, blocksize: u32, idx: u64, reply: ReplyBmap) {
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        let blocksize = NumBytes::from(u64::from(blocksize));
        self.run_async_reply_bmap(
            format!("bmap(ino={ino:?}, blocksize={blocksize:?}, idx={idx:?})"),
            reply,
            move |fs| async move { fs.read().await.bmap(&req, ino, blocksize, idx).await },
        );
    }

    fn ioctl(
        &mut self,
        req: &Request<'_>,
        ino: u64,
        fh: u64,
        flags: u32,
        cmd: u32,
        in_data: &[u8],
        out_size: u32,
        reply: ReplyIoctl,
    ) {
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        let fh = FileHandle::from(fh);
        let in_data = in_data.to_owned();
        self.run_async_no_reply(
            format!("ioctl(ino={ino:?}, fh={fh:?}, flags={flags:?}, cmd={cmd:?}, in_data={in_data:?}, out_size={out_size:?})"),
            move |fs| async move {
                fs.read().await.ioctl(&req, ino, fh, flags, cmd, &in_data, out_size, reply)
                    .await;
                Ok(())
            },
        );
    }

    fn fallocate(
        &mut self,
        req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        length: i64,
        mode: i32,
        reply: ReplyEmpty,
    ) {
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        let fh = FileHandle::from(fh);
        let offset = NumBytes::from(u64::try_from(offset).unwrap()); // TODO No unwrap?
        let length = NumBytes::from(u64::try_from(length).unwrap()); // TODO No unwrap?

        // TODO Why does fuser use i32 instead of u32? for mode?
        let mode = Mode::from(u32::try_from(mode).unwrap());
        self.run_async_reply_empty(
            format!("fallocate(ino={ino:?}, fh={fh:?}, offset={offset:?}, length={length:?}, mode={mode:?})"),
            reply,
            move |fs| async move {
                fs.read().await.fallocate(&req, ino, fh, offset, length, mode)
                    .await
            },
        );
    }

    fn lseek(
        &mut self,
        req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        whence: i32,
        reply: ReplyLseek,
    ) {
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        let fh = FileHandle::from(fh);
        let offset = NumBytes::from(u64::try_from(offset).unwrap()); // TODO No unwrap?
        self.run_async_reply_lseek(
            format!("lseek(ino={ino:?}, fh={fh:?}, offset={offset:?}, whence={whence:?})"),
            reply,
            move |fs| async move { fs.read().await.lseek(&req, ino, fh, offset, whence).await },
        );
    }

    fn copy_file_range(
        &mut self,
        req: &Request<'_>,
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
        let req = RequestInfo::from(req);
        let ino_in = InodeNumber::from(ino_in);
        let fh_in = FileHandle::from(fh_in);
        let offset_in = NumBytes::from(u64::try_from(offset_in).unwrap()); // TODO No unwrap?
        let ino_out = InodeNumber::from(ino_out);
        let fh_out = FileHandle::from(fh_out);
        let offset_out = NumBytes::from(u64::try_from(offset_out).unwrap()); // TODO No unwrap?
        let len = NumBytes::from(len);
        self.run_async_reply_write(
            format!("copy_file_range(ino_in={ino_in:?}, fh_in={fh_in:?}, offset_in={offset_in:?}, ino_out={ino_out:?}, fh_out={fh_out:?}, offset_out={offset_out:?}, len={len:?}, flags={flags:?})"),
            reply,
            move |fs| async move {
                fs.read().await.copy_file_range(
                    &req,
                    ino_in,
                    fh_in,
                    offset_in,
                    ino_out,
                    fh_out,
                    offset_out,
                    len,
                    flags,
                )
                .await
            },
        );
    }

    #[cfg(target_os = "macos")]
    fn setvolname(&mut self, req: &Request<'_>, name: &OsStr, reply: ReplyEmpty) {
        let req = RequestInfo::from(req);
        let name = name.to_owned();
        self.run_async_reply_empty(
            format!("setvolname(name={name:?})"),
            reply,
            move |fs| async move {
                // TODO InvalidPath is the wrong error here
                let name = name
                    .into_os_string()
                    .into_string()
                    .map_err(|err| FsError::InvalidPath)?;
                fs.read().await.setvolname(&req, &name).await
            },
        );
    }

    /// macOS only (undocumented)
    #[cfg(target_os = "macos")]
    fn exchange(
        &mut self,
        req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        newparent: u64,
        newname: &OsStr,
        options: u64,
        reply: ReplyEmpty,
    ) {
        let req = RequestInfo::from(req);
        let parent_ino = InodeNumber::from(parent_ino);
        let name = name.to_owned();
        let newparent = FileHandle::from(newparent);
        let newname = newname.to_owned();
        self.run_async_reply_empty(
            format!("exchange(parent={parent:?}, name={name:?}, newparent={newparent:?}, newname={newname:?}, options={options:?})"),
            reply,
            move |fs| async move {
                // TODO InvalidPath is the wrong error here
                let name: PathComponentBuf =
                    name.try_into().map_err(|err| FsError::InvalidPath)?;
                // TODO InvalidPath is the wrong error here
                let newname: PathComponentBuf =
                    newname.try_into().map_err(|err| FsError::InvalidPath)?;
                fs.read().await.exchange(&req, parent, &name, newparent, &newname, options)
                    .await
            },
        );
    }

    /// macOS only: Query extended times (bkuptime and crtime). Set fuse_init_out.flags
    /// during init to FUSE_XTIMES to enable
    #[cfg(target_os = "macos")]
    fn getxtimes(&mut self, req: &Request<'_>, ino: u64, reply: ReplyXTimes) {
        let req = RequestInfo::from(req);
        let ino = InodeNumber::from(ino);
        self.run_async_reply_xtimes(
            format!("getxtimes(ino={ino:?})"),
            reply,
            move |fs| async move { fs.read().await.getxtimes(&req, ino).await },
        );
    }
}

impl<'a> From<&fuser::Request<'a>> for crate::common::RequestInfo {
    fn from(value: &fuser::Request<'a>) -> Self {
        Self {
            unique: value.unique(),
            uid: Uid::from(value.uid()),
            gid: Gid::from(value.gid()),
            pid: value.pid(),
        }
    }
}

fn parse_time(time: TimeOrNow) -> SystemTime {
    match time {
        TimeOrNow::SpecificTime(time) => time,
        TimeOrNow::Now => SystemTime::now(),
    }
}

fn parse_openflags(flags: i32) -> OpenFlags {
    // TODO Is this the right way to parse openflags? Are there other flags than just Read+Write?
    //      https://docs.rs/fuser/latest/fuser/trait.Filesystem.html#method.open seems to suggest so.
    // TODO This is duplicate between fuser and fuse_mt
    match flags & libc::O_ACCMODE {
        libc::O_RDONLY => OpenFlags::Read,
        libc::O_WRONLY => OpenFlags::Write,
        libc::O_RDWR => OpenFlags::ReadWrite,
        _ => panic!("invalid flags: {flags}"),
    }
}

fn convert_openflags(flags: OpenFlags) -> i32 {
    // TODO Is this the right way to convert openflags? Are there other flags than just Read+Write?
    //      https://docs.rs/fuser/latest/fuser/trait.Filesystem.html#method.open seems to suggest so.
    // TODO This is duplicate between fuser and fuse_mt
    match flags {
        OpenFlags::Read => libc::O_RDONLY,
        OpenFlags::Write => libc::O_WRONLY,
        OpenFlags::ReadWrite => libc::O_RDWR,
    }
}

fn convert_node_attrs(attrs: NodeAttrs, ino: InodeNumber) -> fuser::FileAttr {
    let size: u64 = attrs.num_bytes.into();
    fuser::FileAttr {
        ino: ino.into(),
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

struct DataCallback {
    log_msg: String,
    reply: fuser::ReplyData,
}

impl DataCallback {
    pub fn call(self, data: FsResult<&[u8]>) {
        match data {
            Ok(data) => {
                log::info!("{}...done", self.log_msg);
                self.reply.data(data);
            }
            Err(err) => {
                log::info!("{}...failed: {}", self.log_msg, err);
                self.reply.error(err.system_error_code())
            }
        }
    }
}
