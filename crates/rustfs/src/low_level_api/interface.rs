use async_trait::async_trait;
use std::time::{Duration, SystemTime};

use crate::common::{
    Callback, FileHandle, FsResult, Gid, HandleWithGeneration, InodeNumber, Mode, NodeAttrs,
    NumBytes, OpenFlags, PathComponent, RequestInfo, Statfs, Uid,
};

// TODO Remove asterisk import
use fuser::*;

// TODO Can we deduplicate some of these Reply types with the high level Response types? Also, unify naming. Reply+Response are one name too many.

#[derive(Clone, Copy)]
pub struct ReplyEntry {
    pub ino: HandleWithGeneration<InodeNumber>,
    pub attr: NodeAttrs,
    pub ttl: Duration,
}

#[derive(Clone, Copy)]
pub struct ReplyAttr {
    pub ino: InodeNumber,
    pub attr: NodeAttrs,
    pub ttl: Duration,
}

#[derive(Clone, Copy)]
pub struct ReplyOpen {
    pub fh: FileHandle,
    pub flags: OpenFlags,
}

#[derive(Clone, Copy)]
pub struct ReplyWrite {
    pub written: u32,
}

#[derive(Clone, Copy)]
pub struct ReplyCreate {
    pub ttl: Duration,
    pub ino: HandleWithGeneration<InodeNumber>,
    pub attr: NodeAttrs,
    pub fh: FileHandle,
    // TODO Wrapper type for flags
    pub flags: u32,
}

#[derive(Clone, Copy)]
pub struct ReplyLock {
    pub start: NumBytes,
    pub end: NumBytes,
    // TODO Wrapper type for typ and pid
    pub typ: i32,
    pub pid: u32,
}

#[derive(Clone, Copy)]
pub struct ReplyBmap {
    // TODO What is block? Add a wrapper type?
    pub block: u64,
}

#[derive(Clone, Copy)]
pub struct ReplyLseek {
    // TODO In fuser, this was i64. Why?
    pub offset: NumBytes,
}

#[cfg(target_os = "macos")]
#[derive(Clone, Copy)]
pub struct ReplyXTimes {
    pub bkuptime: SystemTime,
    pub crtime: SystemTime,
}

#[async_trait]
pub trait AsyncFilesystemLL {
    /// Initialize filesystem.
    /// Called before any other filesystem method.
    /// The kernel module connection can be configured using the KernelConfig object
    async fn init(&self, req: &RequestInfo, config: &mut KernelConfig) -> FsResult<()>;

    /// Clean up filesystem.
    /// Called on filesystem exit.
    async fn destroy(&self);

    /// Look up a directory entry by name and get its attributes.
    async fn lookup(
        &self,
        req: &RequestInfo,
        parent: InodeNumber,
        name: &PathComponent,
    ) -> FsResult<ReplyEntry>;

    /// Forget about an inode.
    /// The nlookup parameter indicates the number of lookups previously performed on
    /// this inode. If the filesystem implements inode lifetimes, it is recommended that
    /// inodes acquire a single reference on each lookup, and lose nlookup references on
    /// each forget. The filesystem may ignore forget calls, if the inodes don't need to
    /// have a limited lifetime. On unmount it is not guaranteed, that all referenced
    /// inodes will receive a forget message.
    async fn forget(&self, req: &RequestInfo, ino: InodeNumber, nlookup: u64) -> FsResult<()>;

    // TODO Do we want this? It seems to be gated by an "abi-7-16" feature but what is that?
    // /// Like forget, but take multiple forget requests at once for performance. The default
    // /// implementation will fallback to forget.
    // #[cfg(feature = "abi-7-16")]
    // async fn batch_forget(&self, req: &RequestInfo, nodes: &[fuse_forget_one]) {
    //     for node in nodes {
    //         self.forget(req, node.nodeid, node.nlookup);
    //     }
    // }

    /// Get file attributes.
    async fn getattr(&self, req: &RequestInfo, ino: InodeNumber) -> FsResult<ReplyAttr>;

    /// Set file attributes.
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
        // TODO Custom type for flags
        flags: Option<u32>,
    ) -> FsResult<ReplyAttr>;

    /// Read symbolic link.
    async fn readlink<R, C>(&self, req: &RequestInfo, ino: InodeNumber, callback: C) -> R
    where
        R: 'static,
        C: Send + 'static + for<'a> Callback<FsResult<&'a str>, R>;

    /// Create file node.
    /// Create a regular file, character device, block device, fifo or socket node.
    async fn mknod(
        &self,
        req: &RequestInfo,
        parent: InodeNumber,
        name: &PathComponent,
        mode: Mode,
        // TODO Which type for umask?
        umask: u32,
        // TODO What is rdev?
        rdev: u32,
    ) -> FsResult<ReplyEntry>;

    /// Create a directory.
    async fn mkdir(
        &self,
        req: &RequestInfo,
        parent: InodeNumber,
        name: &PathComponent,
        mode: Mode,
        // TODO Which type for umask?
        umask: u32,
    ) -> FsResult<ReplyEntry>;

    /// Remove a file.
    async fn unlink(
        &self,
        req: &RequestInfo,
        parent: InodeNumber,
        name: &PathComponent,
    ) -> FsResult<()>;

    /// Remove a directory.
    async fn rmdir(
        &self,
        req: &RequestInfo,
        parent: InodeNumber,
        name: &PathComponent,
    ) -> FsResult<()>;

    /// Create a symbolic link.
    async fn symlink(
        &self,
        req: &RequestInfo,
        parent: InodeNumber,
        name: &PathComponent,
        link: &str,
    ) -> FsResult<ReplyEntry>;

    /// Rename a file.
    async fn rename(
        &self,
        req: &RequestInfo,
        parent: InodeNumber,
        name: &PathComponent,
        newparent: InodeNumber,
        newname: &PathComponent,
        // TODO Which type for flags?
        flags: u32,
    ) -> FsResult<()>;

    /// Create a hard link.
    async fn link(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        newparent: InodeNumber,
        newname: &PathComponent,
    ) -> FsResult<ReplyEntry>;

    /// Open a file.
    /// Open flags (with the exception of O_CREAT, O_EXCL, O_NOCTTY and O_TRUNC) are
    /// available in flags. Filesystem may store an arbitrary file handle (pointer, index,
    /// etc) in fh, and use this in other all other file operations (read, write, flush,
    /// release, fsync). Filesystem may also implement stateless file I/O and not store
    /// anything in fh. There are also some flags (direct_io, keep_cache) which the
    /// filesystem may set, to change the way the file is opened. See fuse_file_info
    /// structure in <fuse_common.h> for more details.
    async fn open(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        flags: OpenFlags,
    ) -> FsResult<ReplyOpen>;

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
    async fn read<R, C>(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        // TODO offset was i32 not u32 in fuser, why?
        offset: NumBytes,
        size: NumBytes,
        // TODO Wrapper type for flags
        flags: i32,
        // TODO What is lock_owner?
        lock_owner: Option<u64>,
        // TODO Here and in other places, add documentation saying that `CallbackResult` is just a way to ensure that the implementation actually calls callback.
        callback: C,
    ) -> R
    where
        R: 'static,
        C: Send + 'static + for<'a> Callback<FsResult<&'a [u8]>, R>;

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
    async fn write(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        // TODO offset was i32 not u32 in fuser, why?
        offset: NumBytes,
        data: &[u8],
        // TODO Wrapper type for write_flags
        write_flags: u32,
        // TODO Wrapper type for flags
        flags: i32,
        // TODO What is lock_owner?
        lock_owner: Option<u64>,
    ) -> FsResult<ReplyWrite>;

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
    async fn flush(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        // TODO What is lock_owner?
        lock_owner: u64,
    ) -> FsResult<()>;

    /// Release an open file.
    /// Release is called when there are no more references to an open file: all file
    /// descriptors are closed and all memory mappings are unmapped. For every open
    /// call there will be exactly one release call. The filesystem may reply with an
    /// error, but error values are not returned to close() or munmap() which triggered
    /// the release. fh will contain the value set by the open method, or will be undefined
    /// if the open method didn't set any value. flags will contain the same flags as for
    /// open.
    async fn release(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        // TODO Wrapper type for flags
        flags: i32,
        // TODO What is lock_owner?
        lock_owner: Option<u64>,
        flush: bool,
    ) -> FsResult<()>;

    /// Synchronize file contents.
    /// If the datasync parameter is non-zero, then only the user data should be flushed,
    /// not the meta data.
    async fn fsync(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        datasync: bool,
    ) -> FsResult<()>;

    /// Open a directory.
    /// Filesystem may store an arbitrary file handle (pointer, index, etc) in fh, and
    /// use this in other all other directory stream operations (readdir, releasedir,
    /// fsyncdir). Filesystem may also implement stateless directory I/O and not store
    /// anything in fh, though that makes it impossible to implement standard conforming
    /// directory stream operations in case the contents of the directory can change
    /// between opendir and releasedir.
    async fn opendir(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        // TODO Wrapper type for flags
        flags: i32,
    ) -> FsResult<ReplyOpen>;

    /// Read directory.
    /// Send a buffer filled using buffer.fill(), with size not exceeding the
    /// requested size. Send an empty buffer on end of stream. fh will contain the
    /// value set by the opendir method, or will be undefined if the opendir method
    /// didn't set any value.
    async fn readdir(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        // TODO In fuser, offset was i64. Why?
        offset: NumBytes,
        // TODO We probably want to do this via a callback that takes an iterator
        reply: ReplyDirectory,
    );

    /// Read directory.
    /// Send a buffer filled using buffer.fill(), with size not exceeding the
    /// requested size. Send an empty buffer on end of stream. fh will contain the
    /// value set by the opendir method, or will be undefined if the opendir method
    /// didn't set any value.
    async fn readdirplus(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        // TODO In fuser, offset was i64. Why?
        offset: NumBytes,
        // TODO We probably want to do this via a callback that takes an iterator
        reply: ReplyDirectoryPlus,
    );

    /// Release an open directory.
    /// For every opendir call there will be exactly one releasedir call. fh will
    /// contain the value set by the opendir method, or will be undefined if the
    /// opendir method didn't set any value.
    async fn releasedir(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        // TODO Wrapper type for flags
        flags: i32,
    ) -> FsResult<()>;

    /// Synchronize directory contents.
    /// If the datasync parameter is set, then only the directory contents should
    /// be flushed, not the meta data. fh will contain the value set by the opendir
    /// method, or will be undefined if the opendir method didn't set any value.
    async fn fsyncdir(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        datasync: bool,
    ) -> FsResult<()>;

    /// Get file system statistics.
    async fn statfs(&self, req: &RequestInfo, ino: InodeNumber) -> FsResult<Statfs>;

    /// Set an extended attribute.
    async fn setxattr(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        // TODO Different wrapper type for name that isn't PathComponent? Are the rules the same for xattr names and path components?
        name: &PathComponent,
        value: &[u8],
        // TODO Wrapper type for flags
        flags: i32,
        position: NumBytes,
    ) -> FsResult<()>;

    /// Get an extended attribute.
    /// If `size` is 0, the size of the value should be sent with `reply.size()`.
    /// If `size` is not 0, and the value fits, send it with `reply.data()`, or
    /// `reply.error(ERANGE)` if it doesn't.
    async fn getxattr(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        // TODO Different wrapper type for name that isn't PathComponent? Are the rules the same for xattr names and path components?
        name: &PathComponent,
        size: NumBytes,
        // TODO Return this instead of passing in a Reply type
        reply: ReplyXattr,
    );

    /// List extended attribute names.
    /// If `size` is 0, the size of the value should be sent with `reply.size()`.
    /// If `size` is not 0, and the value fits, send it with `reply.data()`, or
    /// `reply.error(ERANGE)` if it doesn't.
    async fn listxattr(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        size: NumBytes,
        // TODO Return this instead of passing in a Reply type
        reply: ReplyXattr,
    );

    /// Remove an extended attribute.
    async fn removexattr(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        // TODO Different wrapper type for name that isn't PathComponent? Are the rules the same for xattr names and path components?
        name: &PathComponent,
    ) -> FsResult<()>;

    /// Check file access permissions.
    /// This will be called for the access() system call. If the 'default_permissions'
    /// mount option is given, this method is not called. This method is not called
    /// under Linux kernel versions 2.4.x
    async fn access(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        // TODO Wrapper task for mask
        mask: i32,
    ) -> FsResult<()>;

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
    async fn create(
        &self,
        req: &RequestInfo,
        parent: InodeNumber,
        name: &PathComponent,
        mode: Mode,
        // TODO Wrapper type for umask
        umask: u32,
        // TODO Wrapper type for flags
        flags: i32,
    ) -> FsResult<ReplyCreate>;

    /// Test for a POSIX file lock.
    async fn getlk(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        // TODO What is lock_owner?
        lock_owner: u64,
        // TODO Wrapper types for remaining arguments
        start: u64,
        end: u64,
        typ: i32,
        pid: u32,
    ) -> FsResult<ReplyLock>;

    /// Acquire, modify or release a POSIX file lock.
    /// For POSIX threads (NPTL) there's a 1-1 relation between pid and owner, but
    /// otherwise this is not always the case.  For checking lock ownership,
    /// 'fi->owner' must be used. The l_pid field in 'struct flock' should only be
    /// used to fill in this field in getlk(). Note: if the locking methods are not
    /// implemented, the kernel will still allow file locking to work locally.
    /// Hence these are only interesting for network filesystems and similar.
    async fn setlk(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        // TODO What is lock_owner?
        lock_owner: u64,
        // TODO Wrapper type for remaining arguments
        start: u64,
        end: u64,
        typ: i32,
        pid: u32,
        sleep: bool,
    ) -> FsResult<()>;

    /// Map block index within file to block index within device.
    /// Note: This makes sense only for block device backed filesystems mounted
    /// with the 'blkdev' option
    async fn bmap(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        blocksize: NumBytes,
        // TODO What is idx?
        idx: u64,
    ) -> FsResult<ReplyBmap>;

    /// control device
    async fn ioctl(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        // TODO Wrapper types for remaining args
        flags: u32,
        cmd: u32,
        in_data: &[u8],
        out_size: u32,
        // TODO Return this instead of taking it as a parameter?
        reply: ReplyIoctl,
    );

    /// Preallocate or deallocate space to a file
    async fn fallocate(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        // TODO offset and length in fuser was i64. Why?
        offset: NumBytes,
        length: NumBytes,
        mode: Mode,
    ) -> FsResult<()>;

    /// Reposition read/write file offset
    async fn lseek(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        // TODO offset was i64 in fuser. Why?
        offset: NumBytes,
        // TODO What is whence?
        whence: i32,
    ) -> FsResult<ReplyLseek>;

    // TODO Some below (and maybe some above) aren't actually needed and fuse allows returning ENOSYS as a "not-implemented" marker, see https://www.youtube.com/watch?v=id0Kkq4VHDo
    //     See also which ones are actually implemented in https://github.com/wfraser/fuse-mt/blob/master/src/fusemt.rs

    /// Copy the specified range from the source inode to the destination inode
    async fn copy_file_range(
        &self,
        req: &RequestInfo,
        ino_in: InodeNumber,
        fh_in: FileHandle,
        // TODO offset_in was i64 in fuser. Why?
        offset_in: NumBytes,
        ino_out: InodeNumber,
        fh_out: FileHandle,
        // TODO offset_out was i64 in fuser. Why?
        offset_out: NumBytes,
        len: NumBytes,
        // TODO Wrapper type for flags
        flags: u32,
    ) -> FsResult<ReplyWrite>;

    /// macOS only: Rename the volume. Set fuse_init_out.flags during init to
    /// FUSE_VOL_RENAME to enable
    #[cfg(target_os = "macos")]
    async fn setvolname(&self, req: &RequestInfo, name: &str) -> FsResult<()>;

    /// macOS only (undocumented)
    #[cfg(target_os = "macos")]
    async fn exchange(
        &self,
        req: &RequestInfo,
        parent: InodeNumber,
        name: &PathComponent,
        newparent: InodeNumber,
        newname: &PathComponent,
        // TODO Wrapper type for options
        options: u64,
    ) -> FsResult<()>;

    /// macOS only: Query extended times (bkuptime and crtime). Set fuse_init_out.flags
    /// during init to FUSE_XTIMES to enable
    #[cfg(target_os = "macos")]
    async fn getxtimes(&self, req: &RequestInfo, ino: InodeNumber) -> FsResult<ReplyXTimes>;
}
