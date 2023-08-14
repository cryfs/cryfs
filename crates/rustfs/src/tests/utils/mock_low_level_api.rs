use async_trait::async_trait;
use cryfs_utils::async_drop::AsyncDrop;
use fuser::{KernelConfig, ReplyDirectory, ReplyDirectoryPlus, ReplyIoctl, ReplyXattr};
use mockall::mock;
use std::fmt::{self, Debug, Formatter};
use std::time::SystemTime;

use crate::{
    common::{
        Callback, FileHandle, FsError, FsResult, Gid, InodeNumber, Mode, NumBytes, OpenFlags,
        PathComponent, RequestInfo, Statfs, Uid,
    },
    low_level_api::{
        AsyncFilesystemLL, ReplyAttr, ReplyBmap, ReplyCreate, ReplyEntry, ReplyLock, ReplyLseek,
        ReplyOpen, ReplyWrite,
    },
};

mock! {
    pub AsyncFilesystemLL {}

    #[async_trait]
    impl AsyncFilesystemLL for AsyncFilesystemLL {
        async fn init(&self, req: &RequestInfo, config: &mut KernelConfig) -> FsResult<()>;

        async fn destroy(&self);

        async fn lookup(
            &self,
            req: &RequestInfo,
            parent: InodeNumber,
            name: &PathComponent,
        ) -> FsResult<ReplyEntry>;

        async fn forget(&self, req: &RequestInfo, ino: InodeNumber, nlookup: u64) -> FsResult<()>;

        async fn getattr(&self, req: &RequestInfo, ino: InodeNumber) -> FsResult<ReplyAttr>;

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
            flags: OpenFlags,
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
            data: &[u8],
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
            flags: i32,
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
            flags: i32,
        ) -> FsResult<ReplyOpen>;

        async fn readdir(
            &self,
            req: &RequestInfo,
            ino: InodeNumber,
            fh: FileHandle,
            offset: NumBytes,
            reply: ReplyDirectory,
        );

        async fn readdirplus(
            &self,
            req: &RequestInfo,
            ino: InodeNumber,
            fh: FileHandle,
            offset: NumBytes,
            reply: ReplyDirectoryPlus,
        );

        async fn releasedir(
            &self,
            req: &RequestInfo,
            ino: InodeNumber,
            fh: FileHandle,
            flags: i32,
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

        async fn getxattr(
            &self,
            req: &RequestInfo,
            ino: InodeNumber,
            name: &PathComponent,
            size: NumBytes,
            reply: ReplyXattr,
        );

        async fn listxattr(
            &self,
            req: &RequestInfo,
            ino: InodeNumber,
            size: NumBytes,
            reply: ReplyXattr,
        );

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
            flags: i32,
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
            reply: ReplyIoctl,
        );

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

pub fn make_mock_filesystem() -> MockAsyncFilesystemLL {
    let mut mock = MockAsyncFilesystemLL::new();
    mock.expect_init().once().returning(|_, _| Ok(()));
    mock.expect_destroy().once().returning(|| ());
    mock.expect_async_drop_impl().once().returning(|| Ok(()));
    mock
}
