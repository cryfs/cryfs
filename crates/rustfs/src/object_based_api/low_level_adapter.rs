use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

use super::utils::MaybeInitializedFs;
use super::{Device, Dir, File, Node, OpenFile, Symlink};
use crate::common::{
    AbsolutePath, DirEntry, FileHandle, FsError, FsResult, Gid, HandleMap, Mode, NumBytes,
    OpenFlags, PathComponent, RequestInfo, Statfs, Uid,
};
use crate::low_level_api::{
    AsyncFilesystemLL, ReplyAttr, ReplyBmap, ReplyCreate, ReplyEntry, ReplyLock, ReplyLseek,
    ReplyOpen, ReplyWrite,
};
use cryfs_utils::{
    async_drop::{with_async_drop, AsyncDrop, AsyncDropGuard},
    with_async_drop_2,
};
use fuser::{KernelConfig, ReplyDirectory, ReplyDirectoryPlus, ReplyIoctl, ReplyXattr};

// TODO Can we share more code with [super::high_level_adapter::ObjectBasedFsAdapter]?
//      If the adapter struct fields are exactly the same, we could just merge the structs and implement both traits for the struct.
pub struct ObjectBasedFsAdapterLL<Fs: Device>
where
    // TODO Is this send+sync bound only needed because fuse_mt goes multi threaded or would it also be required for fuser?
    Fs::OpenFile: Send + Sync,
{
    // TODO We only need the Arc<RwLock<...>> because of initialization. Is there a better way to do that?
    fs: Arc<RwLock<MaybeInitializedFs<Fs>>>,

    // TODO Can we improve concurrency by locking less in open_files and instead making OpenFileList concurrency safe somehow?
    open_files: tokio::sync::RwLock<AsyncDropGuard<HandleMap<Fs::OpenFile>>>,
}

impl<Fs: Device> ObjectBasedFsAdapterLL<Fs>
where
    // TODO Is this send+sync bound only needed because fuse_mt goes multi threaded or would it also be required for fuser?
    Fs::OpenFile: Send + Sync,
{
    pub fn new(fs: impl FnOnce(Uid, Gid) -> Fs + Send + Sync + 'static) -> AsyncDropGuard<Self> {
        let open_files = tokio::sync::RwLock::new(HandleMap::new());
        AsyncDropGuard::new(Self {
            fs: Arc::new(RwLock::new(MaybeInitializedFs::Uninitialized(Some(
                Box::new(fs),
            )))),
            open_files,
        })
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
        self.fs.write().await.take().destroy().await;
        // Nothing.
    }

    async fn lookup(
        &self,
        req: &RequestInfo,
        parent: FileHandle,
        name: &PathComponent,
    ) -> FsResult<ReplyEntry> {
        Err(FsError::NotImplemented)
    }

    async fn forget(&self, req: &RequestInfo, ino: FileHandle, nlookup: u64) -> FsResult<()> {
        Err(FsError::NotImplemented)
    }

    async fn getattr(&self, req: &RequestInfo, ino: FileHandle) -> FsResult<ReplyAttr> {
        Err(FsError::NotImplemented)
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
        Err(FsError::NotImplemented)
    }

    async fn readlink<CallbackResult>(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        callback: impl Send + for<'a> FnOnce(FsResult<&'a str>) -> CallbackResult,
    ) -> CallbackResult {
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
        Err(FsError::NotImplemented)
    }

    async fn mkdir(
        &self,
        req: &RequestInfo,
        parent: FileHandle,
        name: &PathComponent,
        mode: Mode,
        umask: u32,
    ) -> FsResult<ReplyEntry> {
        Err(FsError::NotImplemented)
    }

    async fn unlink(
        &self,
        req: &RequestInfo,
        parent: FileHandle,
        name: &PathComponent,
    ) -> FsResult<()> {
        Err(FsError::NotImplemented)
    }

    async fn rmdir(
        &self,
        req: &RequestInfo,
        parent: FileHandle,
        name: &PathComponent,
    ) -> FsResult<()> {
        Err(FsError::NotImplemented)
    }

    async fn symlink(
        &self,
        req: &RequestInfo,
        parent: FileHandle,
        name: &PathComponent,
        link: &str,
    ) -> FsResult<ReplyEntry> {
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
        Err(FsError::NotImplemented)
    }

    async fn link(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        newparent: FileHandle,
        newname: &PathComponent,
    ) -> FsResult<ReplyEntry> {
        Err(FsError::NotImplemented)
    }

    async fn open(&self, req: &RequestInfo, ino: FileHandle, flags: i32) -> FsResult<ReplyOpen> {
        Err(FsError::NotImplemented)
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
        Err(FsError::NotImplemented)
    }

    async fn flush(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        fh: FileHandle,
        lock_owner: u64,
    ) -> FsResult<()> {
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
        Err(FsError::NotImplemented)
    }

    async fn fsync(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        fh: FileHandle,
        datasync: bool,
    ) -> FsResult<()> {
        Err(FsError::NotImplemented)
    }

    async fn opendir(&self, req: &RequestInfo, ino: FileHandle, flags: i32) -> FsResult<ReplyOpen> {
        Err(FsError::NotImplemented)
    }

    async fn readdir(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        fh: FileHandle,
        offset: NumBytes,
        reply: ReplyDirectory,
    ) {
        reply.error(libc::ENOSYS)
    }

    async fn readdirplus(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        fh: FileHandle,
        offset: NumBytes,
        reply: ReplyDirectoryPlus,
    ) {
        reply.error(libc::ENOSYS)
    }

    async fn releasedir(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        fh: FileHandle,
        flags: i32,
    ) -> FsResult<()> {
        Err(FsError::NotImplemented)
    }

    async fn fsyncdir(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        fh: FileHandle,
        datasync: bool,
    ) -> FsResult<()> {
        Err(FsError::NotImplemented)
    }

    async fn statfs(&self, req: &RequestInfo, ino: FileHandle) -> FsResult<Statfs> {
        Err(FsError::NotImplemented)
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
        reply.error(libc::ENOSYS)
    }

    async fn listxattr(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        size: NumBytes,
        reply: ReplyXattr,
    ) {
        reply.error(libc::ENOSYS)
    }

    async fn removexattr(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        name: &PathComponent,
    ) -> FsResult<()> {
        Err(FsError::NotImplemented)
    }

    async fn access(&self, req: &RequestInfo, ino: FileHandle, mask: i32) -> FsResult<()> {
        Err(FsError::NotImplemented)
    }

    async fn create(
        &self,
        req: &RequestInfo,
        parent: FileHandle,
        name: &PathComponent,
        mode: Mode,
        umask: u32,
        flags: i32,
    ) -> FsResult<ReplyCreate> {
        Err(FsError::NotImplemented)
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
        Err(FsError::NotImplemented)
    }

    async fn bmap(
        &self,
        req: &RequestInfo,
        ino: FileHandle,
        blocksize: NumBytes,
        idx: u64,
    ) -> FsResult<ReplyBmap> {
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
        Err(FsError::NotImplemented)
    }

    #[cfg(target_os = "macos")]
    async fn setvolname(&self, req: &RequestInfo, name: &str) -> FsResult<()> {
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
        Err(FsError::NotImplemented)
    }

    #[cfg(target_os = "macos")]
    async fn getxtimes(&self, req: &RequestInfo, ino: FileHandle) -> FsResult<ReplyXTimes> {
        Err(FsError::NotImplemented)
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
        let mut v = self.open_files.write().await;
        v.async_drop().await?;
        Ok(())
    }
}
