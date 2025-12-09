use async_trait::async_trait;
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropArc},
    path::PathComponent,
};
use mockall::mock;
use std::fmt::{self, Debug, Formatter};
use std::ops::Deref;
use std::time::SystemTime;

#[cfg(target_os = "macos")]
use crate::low_level_api::ReplyXTimes;
use crate::{
    common::{
        Callback, FileHandle, FsError, FsResult, Gid, InodeNumber, Mode, NumBytes, OpenInFlags,
        RequestInfo, Statfs, Uid,
    },
    low_level_api::{
        AsyncFilesystemLL, ReplyAttr, ReplyBmap, ReplyCreate, ReplyDirectory, ReplyDirectoryPlus,
        ReplyEntry, ReplyIoctl, ReplyLock, ReplyLseek, ReplyOpen, ReplyWrite,
    },
};

pub fn make_mock_filesystem() -> MockAsyncFilesystemLL {
    let mut mock = MockAsyncFilesystemLL::new();
    mock.expect_init().once().returning(|_| Ok(()));
    mock.expect_destroy().once().returning(|| ());
    mock.expect_async_drop_impl().once().returning(|| Ok(()));
    mock
}

mock! {
    pub AsyncFilesystemLL {}

    #[async_trait]
    impl AsyncFilesystemLL for AsyncFilesystemLL {
        async fn init(&self, req: &RequestInfo) -> FsResult<()>;

        async fn destroy(&self);

        async fn lookup(
            &self,
            req: &RequestInfo,
            parent: InodeNumber,
            name: &PathComponent,
        ) -> FsResult<ReplyEntry>;

        async fn forget(&self, req: &RequestInfo, ino: InodeNumber, nlookup: u64) -> FsResult<()>;

        async fn getattr(&self, req: &RequestInfo, ino: InodeNumber, fh: Option<FileHandle>) -> FsResult<ReplyAttr>;

        async fn setattr(
            &self,
            req: &RequestInfo,
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
        ) -> FsResult<ReplyAttr>;

        async fn readlink<R, C>(
            &self,
            req: &RequestInfo,
            ino: InodeNumber,
            callback: C,
        ) -> R
        where
            R: 'static,
            C: Send + 'static + for<'a> Callback<FsResult<&'a str>, R>;

        async fn mknod(
            &self,
            req: &RequestInfo,
            parent: InodeNumber,
            name: &PathComponent,
            mode: Mode,
            umask: u32,
            rdev: u32,
        ) -> FsResult<ReplyEntry>;

        async fn mkdir(
            &self,
            req: &RequestInfo,
            parent: InodeNumber,
            name: &PathComponent,
            mode: Mode,
            umask: u32,
        ) -> FsResult<ReplyEntry>;

        async fn unlink(
            &self,
            req: &RequestInfo,
            parent: InodeNumber,
            name: &PathComponent,
        ) -> FsResult<()>;

        async fn rmdir(
            &self,
            req: &RequestInfo,
            parent: InodeNumber,
            name: &PathComponent,
        ) -> FsResult<()>;

        async fn symlink(
            &self,
            req: &RequestInfo,
            parent: InodeNumber,
            name: &PathComponent,
            link: &str,
        ) -> FsResult<ReplyEntry>;

        async fn rename(
            &self,
            req: &RequestInfo,
            parent: InodeNumber,
            name: &PathComponent,
            newparent: InodeNumber,
            newname: &PathComponent,
            flags: u32,
        ) -> FsResult<()>;

        async fn link(
            &self,
            req: &RequestInfo,
            ino: InodeNumber,
            newparent: InodeNumber,
            newname: &PathComponent,
        ) -> FsResult<ReplyEntry>;

        async fn open(
            &self,
            req: &RequestInfo,
            ino: InodeNumber,
            flags: OpenInFlags,
        ) -> FsResult<ReplyOpen>;

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
            C: Send + 'static + for<'a> Callback<FsResult<&'a [u8]>, R>;

        async fn write(
            &self,
            req: &RequestInfo,
            ino: InodeNumber,
            fh: FileHandle,
            offset: NumBytes,
            data: Vec<u8>,
            write_flags: u32,
            flags: i32,
            lock_owner: Option<u64>,
        ) -> FsResult<ReplyWrite>;

        async fn flush(
            &self,
            req: &RequestInfo,
            ino: InodeNumber,
            fh: FileHandle,
            lock_owner: u64,
        ) -> FsResult<()>;

        async fn release(
            &self,
            req: &RequestInfo,
            ino: InodeNumber,
            fh: FileHandle,
            flags: OpenInFlags,
            lock_owner: Option<u64>,
            flush: bool,
        ) -> FsResult<()>;

        async fn fsync(
            &self,
            req: &RequestInfo,
            ino: InodeNumber,
            fh: FileHandle,
            datasync: bool,
        ) -> FsResult<()>;

        async fn opendir(
            &self,
            req: &RequestInfo,
            ino: InodeNumber,
            flags: OpenInFlags,
        ) -> FsResult<ReplyOpen>;

        async fn readdir<R: ReplyDirectory + Send + 'static>(
            &self,
            req: &RequestInfo,
            ino: InodeNumber,
            fh: FileHandle,
            offset: u64,
            reply: &mut R,
        ) -> FsResult<()> ;

        async fn readdirplus<R: ReplyDirectoryPlus + Send + 'static>(
            &self,
            req: &RequestInfo,
            ino: InodeNumber,
            fh: FileHandle,
            offset: u64,
            reply: &mut R,
        ) -> FsResult<()>;

        async fn releasedir(
            &self,
            req: &RequestInfo,
            ino: InodeNumber,
            fh: FileHandle,
            flags: OpenInFlags,
        ) -> FsResult<()>;

        async fn fsyncdir(
            &self,
            req: &RequestInfo,
            ino: InodeNumber,
            fh: FileHandle,
            datasync: bool,
        ) -> FsResult<()>;

        async fn statfs(&self, req: &RequestInfo, ino: InodeNumber) -> FsResult<Statfs>;

        async fn setxattr(
            &self,
            req: &RequestInfo,
            ino: InodeNumber,
            name: &PathComponent,
            value: &[u8],
            flags: i32,
            position: NumBytes,
        ) -> FsResult<()>;

        async fn getxattr_numbytes(
            &self,
            req: &RequestInfo,
            ino: InodeNumber,
            name: &PathComponent,
        ) -> FsResult<NumBytes>;

        async fn getxattr_data(
            &self,
            req: &RequestInfo,
            ino: InodeNumber,
            name: &PathComponent,
            max_bytes_to_read: NumBytes,
        ) -> FsResult<Vec<u8>>;

        async fn listxattr_numbytes(&self, req: &RequestInfo, ino: InodeNumber) -> FsResult<NumBytes>;

        async fn listxattr_data(
            &self,
            req: &RequestInfo,
            ino: InodeNumber,
            max_bytes_to_read: NumBytes,
        ) -> FsResult<Vec<u8>>;

        async fn removexattr(
            &self,
            req: &RequestInfo,
            ino: InodeNumber,
            name: &PathComponent,
        ) -> FsResult<()>;

        async fn access(
            &self,
            req: &RequestInfo,
            ino: InodeNumber,
            mask: i32,
        ) -> FsResult<()>;

        async fn create(
            &self,
            req: &RequestInfo,
            parent: InodeNumber,
            name: &PathComponent,
            mode: Mode,
            umask: u32,
            flags: OpenInFlags,
        ) -> FsResult<ReplyCreate>;

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
        ) -> FsResult<ReplyLock>;

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
        ) -> FsResult<()>;

        async fn bmap(
            &self,
            req: &RequestInfo,
            ino: InodeNumber,
            blocksize: NumBytes,
            idx: u64,
        ) -> FsResult<ReplyBmap>;

        async fn ioctl(
            &self,
            req: &RequestInfo,
            ino: InodeNumber,
            fh: FileHandle,
            flags: u32,
            cmd: u32,
            in_data: &[u8],
            out_size: u32,
        ) -> FsResult<ReplyIoctl>;

        async fn fallocate(
            &self,
            req: &RequestInfo,
            ino: InodeNumber,
            fh: FileHandle,
            offset: NumBytes,
            length: NumBytes,
            mode: Mode,
        ) -> FsResult<()>;

        async fn lseek(
            &self,
            req: &RequestInfo,
            ino: InodeNumber,
            fh: FileHandle,
            offset: NumBytes,
            whence: i32,
        ) -> FsResult<ReplyLseek>;

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
        ) -> FsResult<ReplyWrite>;

        #[cfg(target_os = "macos")]
        async fn setvolname(&self, req: &RequestInfo, name: &str) -> FsResult<()>;

        #[cfg(target_os = "macos")]
        async fn exchange(
            &self,
            req: &RequestInfo,
            parent: InodeNumber,
            name: &PathComponent,
            newparent: InodeNumber,
            newname: &PathComponent,
            options: u64,
        ) -> FsResult<()>;

        #[cfg(target_os = "macos")]
        async fn getxtimes(&self, req: &RequestInfo, ino: InodeNumber) -> FsResult<ReplyXTimes>;
    }

    #[async_trait]
    impl AsyncDrop for AsyncFilesystemLL {
        type Error = FsError;
        async fn async_drop_impl(&mut self) -> Result<(), FsError>;
    }
}

impl Debug for MockAsyncFilesystemLL {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("MockAsyncFilesystemLL").finish()
    }
}

/// We need to implement AsyncFilesystemLL for AsyncDropArc<MockAsyncFilesystemLL> because we need our code
/// to keep access to the mock (e.g. keep an Arc) while the mock is also being passed to the backend adapter
/// to run the file system.
#[async_trait]
impl AsyncFilesystemLL for AsyncDropArc<MockAsyncFilesystemLL> {
    async fn init(&self, req: &RequestInfo) -> FsResult<()> {
        self.deref().init(req).await
    }

    async fn destroy(&self) {
        self.deref().destroy().await
    }

    async fn lookup(
        &self,
        req: &RequestInfo,
        parent: InodeNumber,
        name: &PathComponent,
    ) -> FsResult<ReplyEntry> {
        self.deref().lookup(req, parent, name).await
    }

    async fn forget(&self, req: &RequestInfo, ino: InodeNumber, nlookup: u64) -> FsResult<()> {
        self.deref().forget(req, ino, nlookup).await
    }

    async fn getattr(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: Option<FileHandle>,
    ) -> FsResult<ReplyAttr> {
        self.deref().getattr(req, ino, fh).await
    }

    async fn setattr(
        &self,
        req: &RequestInfo,
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
        self.deref()
            .setattr(
                req, ino, mode, uid, gid, size, atime, mtime, ctime, fh, crtime, chgtime, bkuptime,
                flags,
            )
            .await
    }

    async fn readlink<R, C>(&self, req: &RequestInfo, ino: InodeNumber, callback: C) -> R
    where
        R: 'static,
        C: Send + 'static + for<'a> Callback<FsResult<&'a str>, R>,
    {
        self.deref().readlink(req, ino, callback).await
    }

    async fn mknod(
        &self,
        req: &RequestInfo,
        parent: InodeNumber,
        name: &PathComponent,
        mode: Mode,
        umask: u32,
        rdev: u32,
    ) -> FsResult<ReplyEntry> {
        self.deref()
            .mknod(req, parent, name, mode, umask, rdev)
            .await
    }

    async fn mkdir(
        &self,
        req: &RequestInfo,
        parent: InodeNumber,
        name: &PathComponent,
        mode: Mode,
        umask: u32,
    ) -> FsResult<ReplyEntry> {
        self.deref().mkdir(req, parent, name, mode, umask).await
    }

    async fn unlink(
        &self,
        req: &RequestInfo,
        parent: InodeNumber,
        name: &PathComponent,
    ) -> FsResult<()> {
        self.deref().unlink(req, parent, name).await
    }

    async fn rmdir(
        &self,
        req: &RequestInfo,
        parent: InodeNumber,
        name: &PathComponent,
    ) -> FsResult<()> {
        self.deref().rmdir(req, parent, name).await
    }

    async fn symlink(
        &self,
        req: &RequestInfo,
        parent: InodeNumber,
        name: &PathComponent,
        link: &str,
    ) -> FsResult<ReplyEntry> {
        self.deref().symlink(req, parent, name, link).await
    }

    async fn rename(
        &self,
        req: &RequestInfo,
        parent: InodeNumber,
        name: &PathComponent,
        newparent: InodeNumber,
        newname: &PathComponent,
        flags: u32,
    ) -> FsResult<()> {
        self.deref()
            .rename(req, parent, name, newparent, newname, flags)
            .await
    }

    async fn link(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        newparent: InodeNumber,
        newname: &PathComponent,
    ) -> FsResult<ReplyEntry> {
        self.deref().link(req, ino, newparent, newname).await
    }

    async fn open(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        flags: OpenInFlags,
    ) -> FsResult<ReplyOpen> {
        self.deref().open(req, ino, flags).await
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
        self.deref()
            .read(req, ino, fh, offset, size, flags, lock_owner, callback)
            .await
    }

    async fn write(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        offset: NumBytes,
        data: Vec<u8>,
        write_flags: u32,
        flags: i32,
        lock_owner: Option<u64>,
    ) -> FsResult<ReplyWrite> {
        self.deref()
            .write(req, ino, fh, offset, data, write_flags, flags, lock_owner)
            .await
    }

    async fn flush(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        lock_owner: u64,
    ) -> FsResult<()> {
        self.deref().flush(req, ino, fh, lock_owner).await
    }

    async fn release(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        flags: OpenInFlags,
        lock_owner: Option<u64>,
        flush: bool,
    ) -> FsResult<()> {
        self.deref()
            .release(req, ino, fh, flags, lock_owner, flush)
            .await
    }

    async fn fsync(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        datasync: bool,
    ) -> FsResult<()> {
        self.deref().fsync(req, ino, fh, datasync).await
    }

    async fn opendir(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        flags: OpenInFlags,
    ) -> FsResult<ReplyOpen> {
        self.deref().opendir(req, ino, flags).await
    }

    async fn readdir<R: ReplyDirectory + Send + 'static>(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        offset: u64,
        reply: &mut R,
    ) -> FsResult<()> {
        self.deref().readdir(req, ino, fh, offset, reply).await
    }

    async fn readdirplus<R: ReplyDirectoryPlus + Send + 'static>(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        offset: u64,
        reply: &mut R,
    ) -> FsResult<()> {
        self.deref().readdirplus(req, ino, fh, offset, reply).await
    }

    async fn releasedir(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        flags: OpenInFlags,
    ) -> FsResult<()> {
        self.deref().releasedir(req, ino, fh, flags).await
    }

    async fn fsyncdir(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        datasync: bool,
    ) -> FsResult<()> {
        self.deref().fsyncdir(req, ino, fh, datasync).await
    }

    async fn statfs(&self, req: &RequestInfo, ino: InodeNumber) -> FsResult<Statfs> {
        self.deref().statfs(req, ino).await
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
        self.deref()
            .setxattr(req, ino, name, value, flags, position)
            .await
    }

    async fn getxattr_numbytes(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        name: &PathComponent,
    ) -> FsResult<NumBytes> {
        self.deref().getxattr_numbytes(req, ino, name).await
    }

    async fn getxattr_data(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        name: &PathComponent,
        max_bytes_to_read: NumBytes,
    ) -> FsResult<Vec<u8>> {
        self.deref()
            .getxattr_data(req, ino, name, max_bytes_to_read)
            .await
    }

    async fn listxattr_numbytes(&self, req: &RequestInfo, ino: InodeNumber) -> FsResult<NumBytes> {
        self.deref().listxattr_numbytes(req, ino).await
    }

    async fn listxattr_data(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        max_bytes_to_read: NumBytes,
    ) -> FsResult<Vec<u8>> {
        self.deref()
            .listxattr_data(req, ino, max_bytes_to_read)
            .await
    }

    async fn removexattr(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        name: &PathComponent,
    ) -> FsResult<()> {
        self.deref().removexattr(req, ino, name).await
    }

    async fn access(&self, req: &RequestInfo, ino: InodeNumber, mask: i32) -> FsResult<()> {
        self.deref().access(req, ino, mask).await
    }

    async fn create(
        &self,
        req: &RequestInfo,
        parent: InodeNumber,
        name: &PathComponent,
        mode: Mode,
        umask: u32,
        flags: OpenInFlags,
    ) -> FsResult<ReplyCreate> {
        self.deref()
            .create(req, parent, name, mode, umask, flags)
            .await
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
        self.deref()
            .getlk(req, ino, fh, lock_owner, start, end, typ, pid)
            .await
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
        self.deref()
            .setlk(req, ino, fh, lock_owner, start, end, typ, pid, sleep)
            .await
    }

    async fn bmap(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        blocksize: NumBytes,
        idx: u64,
    ) -> FsResult<ReplyBmap> {
        self.deref().bmap(req, ino, blocksize, idx).await
    }

    async fn ioctl(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        flags: u32,
        cmd: u32,
        in_data: &[u8],
        out_size: u32,
    ) -> FsResult<ReplyIoctl> {
        self.deref()
            .ioctl(req, ino, fh, flags, cmd, in_data, out_size)
            .await
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
        self.deref()
            .fallocate(req, ino, fh, offset, length, mode)
            .await
    }

    async fn lseek(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        offset: NumBytes,
        whence: i32,
    ) -> FsResult<ReplyLseek> {
        self.deref().lseek(req, ino, fh, offset, whence).await
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
        self.deref()
            .copy_file_range(
                req, ino_in, fh_in, offset_in, ino_out, fh_out, offset_out, len, flags,
            )
            .await
    }

    #[cfg(target_os = "macos")]
    async fn setvolname(&self, req: &RequestInfo, name: &str) -> FsResult<()> {
        self.deref().setvolname(req, name).await
    }

    #[cfg(target_os = "macos")]
    async fn exchange(
        &self,
        req: &RequestInfo,
        parent: InodeNumber,
        name: &PathComponent,
        newparent: InodeNumber,
        newname: &PathComponent,
        options: u64,
    ) -> FsResult<()> {
        self.deref()
            .exchange(req, parent, name, newparent, newname, options)
            .await
    }

    #[cfg(target_os = "macos")]
    async fn getxtimes(&self, req: &RequestInfo, ino: InodeNumber) -> FsResult<ReplyXTimes> {
        self.deref().getxtimes(req, ino).await
    }
}
