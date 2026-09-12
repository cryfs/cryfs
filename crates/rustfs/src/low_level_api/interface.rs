use derive_more::Debug;
use std::time::{Duration, SystemTime};

use crate::{
    NodeKind,
    common::{
        Callback, FileHandle, FsResult, Gid, HandleWithGeneration, InodeNumber, Mode, NodeAttrs,
        NumBytes, OpenInFlags, OpenOutFlags, RequestInfo, Statfs, Uid,
    },
};
use cryfs_utils::path::PathComponent;

// TODO Can we deduplicate some of these Reply types with the high level Response types? Also, unify naming. Reply+Response are one name too many.

#[derive(Clone, Copy, Debug)]
pub struct ReplyEntry {
    #[debug("{ino}")]
    pub ino: HandleWithGeneration<InodeNumber>,
    pub attr: NodeAttrs,
    pub ttl: Duration,
}

#[derive(Clone, Copy, Debug)]
pub struct ReplyAttr {
    #[debug("{ino}")]
    pub ino: InodeNumber,
    pub attr: NodeAttrs,
    pub ttl: Duration,
}

#[derive(Clone, Copy, Debug)]
pub struct ReplyOpen {
    #[debug("{fh}")]
    pub fh: FileHandle,
    pub flags: OpenOutFlags,
}

#[derive(Clone, Copy, Debug)]
pub struct ReplyWrite {
    #[debug("{written}")]
    pub written: NumBytes,
}

#[derive(Clone, Copy, Debug)]
pub struct ReplyCreate {
    pub ttl: Duration,
    #[debug("{ino}")]
    pub ino: HandleWithGeneration<InodeNumber>,
    pub attr: NodeAttrs,
    #[debug("{fh}")]
    pub fh: FileHandle,
    pub flags: OpenOutFlags,
}

#[derive(Clone, Copy, Debug)]
pub struct ReplyLock {
    #[debug("{start}")]
    pub start: NumBytes,
    #[debug("{end}")]
    pub end: NumBytes,
    // TODO Wrapper type for typ and pid
    pub typ: i32,
    pub pid: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct ReplyBmap {
    // TODO What is block? Add a wrapper type?
    pub block: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct ReplyLseek {
    // TODO In fuser, this was i64. Why?
    #[debug("{offset}")]
    pub offset: NumBytes,
}

#[cfg(target_os = "macos")]
#[derive(Clone, Copy, Debug)]
pub struct ReplyXTimes {
    pub bkuptime: SystemTime,
    pub crtime: SystemTime,
}

#[derive(Clone, Debug)]
pub struct ReplyIoctl {
    pub result: i32,
    // TOOD It'd be better to not force a clone of the bytes, but instead use a callback type similar to read() or readdir().
    #[debug("[{} bytes]", data.len())]
    pub data: Box<[u8]>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReplyDirectoryAddResult {
    /// The buffer is full, no need to add more entries
    Full,
    /// The buffer is not full, more entries can be added
    NotFull,
}

pub trait ReplyDirectory {
    // TODO Come up with a better way to handle '.' and '..', enforcing correct usage of offsets and full buffers in the API.

    /// Add the '.' entry to the directory reply buffer. Returns whether the buffer is full.
    #[must_use]
    fn add_self_reference(&mut self, ino: InodeNumber, offset: i64) -> ReplyDirectoryAddResult;

    /// Add the '..' entry to the directory reply buffer. Returns whether the buffer is full.
    #[must_use]
    fn add_parent_reference(&mut self, ino: InodeNumber, offset: i64) -> ReplyDirectoryAddResult;

    /// Add an entry to the directory reply buffer. Returns whether the buffer is full.
    /// A transparent offset value can be provided for each entry. The kernel uses these
    /// value to request the next entries in further readdir calls
    #[must_use]
    fn add(
        &mut self,
        ino: InodeNumber,
        offset: i64,
        kind: NodeKind,
        name: &PathComponent,
    ) -> ReplyDirectoryAddResult;
}

pub trait ReplyDirectoryPlus {
    /// Add an entry to the directory reply buffer. Returns whether the buffer is full.
    /// A transparent offset value can be provided for each entry. The kernel uses these
    /// value to request the next entries in further readdir calls
    #[must_use]
    fn add(
        &mut self,
        ino: InodeNumber,
        offset: i64,
        name: &PathComponent,
        ttl: &Duration,
        attr: &NodeAttrs,
        generation: u64,
    ) -> ReplyDirectoryAddResult;
}

pub trait AsyncFilesystemLL {
    /// Initialize filesystem.
    /// Called before any other filesystem method.
    /// The kernel module connection can be configured using the KernelConfig object
    fn init(&self, req: &RequestInfo) -> impl Future<Output = FsResult<()>> + Send;

    /// Clean up filesystem.
    /// Called on filesystem exit.
    fn destroy(&self) -> impl Future<Output = ()> + Send;

    /// Look up a directory entry by name and get its attributes.
    fn lookup(
        &self,
        req: &RequestInfo,
        parent: InodeNumber,
        name: &PathComponent,
    ) -> impl Future<Output = FsResult<ReplyEntry>> + Send;

    /// Forget about an inode.
    /// The nlookup parameter indicates the number of lookups previously performed on
    /// this inode. If the filesystem implements inode lifetimes, it is recommended that
    /// inodes acquire a single reference on each lookup, and lose nlookup references on
    /// each forget. The filesystem may ignore forget calls, if the inodes don't need to
    /// have a limited lifetime. On unmount it is not guaranteed, that all referenced
    /// inodes will receive a forget message.
    fn forget(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        nlookup: u64,
    ) -> impl Future<Output = FsResult<()>> + Send;

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
    fn getattr(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: Option<FileHandle>,
    ) -> impl Future<Output = FsResult<ReplyAttr>> + Send;

    /// Set file attributes.
    fn setattr(
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
    ) -> impl Future<Output = FsResult<ReplyAttr>> + Send;

    /// Read symbolic link.
    fn readlink<R, C>(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        callback: C,
    ) -> impl Future<Output = R> + Send
    where
        R: 'static,
        C: Send + 'static + for<'a> Callback<FsResult<&'a str>, R>;

    /// Create file node.
    /// Create a regular file, character device, block device, fifo or socket node.
    fn mknod(
        &self,
        req: &RequestInfo,
        parent: InodeNumber,
        name: &PathComponent,
        mode: Mode,
        // TODO Which type for umask?
        umask: u32,
        // TODO What is rdev?
        rdev: u32,
    ) -> impl Future<Output = FsResult<ReplyEntry>> + Send;

    /// Create a directory.
    fn mkdir(
        &self,
        req: &RequestInfo,
        parent: InodeNumber,
        name: &PathComponent,
        mode: Mode,
        // TODO Which type for umask?
        umask: u32,
    ) -> impl Future<Output = FsResult<ReplyEntry>> + Send;

    /// Remove a file.
    fn unlink(
        &self,
        req: &RequestInfo,
        parent: InodeNumber,
        name: &PathComponent,
    ) -> impl Future<Output = FsResult<()>> + Send;

    /// Remove a directory.
    fn rmdir(
        &self,
        req: &RequestInfo,
        parent: InodeNumber,
        name: &PathComponent,
    ) -> impl Future<Output = FsResult<()>> + Send;

    /// Create a symbolic link.
    fn symlink(
        &self,
        req: &RequestInfo,
        parent: InodeNumber,
        name: &PathComponent,
        link: &str,
    ) -> impl Future<Output = FsResult<ReplyEntry>> + Send;

    /// Rename a file.
    ///
    /// `flags` carries the `renameat2()` flags (`RENAME_NOREPLACE`, `RENAME_EXCHANGE`,
    /// `RENAME_WHITEOUT`); it is zero for a plain `rename()`. An implementation that doesn't honor
    /// them must reject a non-zero `flags` with FsError::InvalidOperation rather than ignore it -
    /// performing a plain rename in response to a `RENAME_EXCHANGE` reports success while
    /// destroying one of the two files.
    fn rename(
        &self,
        req: &RequestInfo,
        parent: InodeNumber,
        name: &PathComponent,
        newparent: InodeNumber,
        newname: &PathComponent,
        // TODO Which type for flags?
        flags: u32,
    ) -> impl Future<Output = FsResult<()>> + Send;

    /// Create a hard link.
    fn link(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        newparent: InodeNumber,
        newname: &PathComponent,
    ) -> impl Future<Output = FsResult<ReplyEntry>> + Send;

    /// Open a file.
    /// Open flags (with the exception of O_CREAT, O_EXCL, O_NOCTTY and O_TRUNC) are
    /// available in flags. Filesystem may store an arbitrary file handle (pointer, index,
    /// etc) in fh, and use this in other all other file operations (read, write, flush,
    /// release, fsync). Filesystem may also implement stateless file I/O and not store
    /// anything in fh. There are also some flags (direct_io, keep_cache) which the
    /// filesystem may set, to change the way the file is opened. See fuse_file_info
    /// structure in <fuse_common.h> for more details.
    fn open(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        flags: OpenInFlags,
    ) -> impl Future<Output = FsResult<ReplyOpen>> + Send;

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
    fn read<R, C>(
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
    ) -> impl Future<Output = R> + Send
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
    fn write(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        // TODO offset was i32 not u32 in fuser, why?
        offset: NumBytes,
        data: Vec<u8>,
        // TODO Wrapper type for write_flags
        write_flags: u32,
        // TODO Wrapper type for flags
        flags: i32,
        // TODO What is lock_owner?
        lock_owner: Option<u64>,
    ) -> impl Future<Output = FsResult<ReplyWrite>> + Send;

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
    fn flush(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        // TODO What is lock_owner?
        lock_owner: u64,
    ) -> impl Future<Output = FsResult<()>> + Send;

    /// Release an open file.
    /// Release is called when there are no more references to an open file: all file
    /// descriptors are closed and all memory mappings are unmapped. For every open
    /// call there will be exactly one release call. The filesystem may reply with an
    /// error, but error values are not returned to close() or munmap() which triggered
    /// the release. fh will contain the value set by the open method, or will be undefined
    /// if the open method didn't set any value. flags will contain the same flags as for
    /// open.
    fn release(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        // TODO Wrapper type for flags
        flags: OpenInFlags,
        // TODO What is lock_owner?
        lock_owner: Option<u64>,
        flush: bool,
    ) -> impl Future<Output = FsResult<()>> + Send;

    /// Synchronize file contents.
    /// If the datasync parameter is non-zero, then only the user data should be flushed,
    /// not the meta data.
    fn fsync(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        datasync: bool,
    ) -> impl Future<Output = FsResult<()>> + Send;

    /// Open a directory.
    /// Filesystem may store an arbitrary file handle (pointer, index, etc) in fh, and
    /// use this in other all other directory stream operations (readdir, releasedir,
    /// fsyncdir). Filesystem may also implement stateless directory I/O and not store
    /// anything in fh, though that makes it impossible to implement standard conforming
    /// directory stream operations in case the contents of the directory can change
    /// between opendir and releasedir.
    fn opendir(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        flags: OpenInFlags,
    ) -> impl Future<Output = FsResult<ReplyOpen>> + Send;

    /// Read directory.
    /// Send a buffer filled using buffer.fill(), with size not exceeding the
    /// requested size. Send an empty buffer on end of stream. fh will contain the
    /// value set by the opendir method, or will be undefined if the opendir method
    /// didn't set any value.
    fn readdir<R: ReplyDirectory + Send + 'static>(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        // TODO In fuser, offset was i64. Why?
        offset: u64,
        // TODO Can we do this via a callback that takes an iterator
        reply: &mut R,
    ) -> impl Future<Output = FsResult<()>> + Send;

    /// Read directory.
    /// Send a buffer filled using buffer.fill(), with size not exceeding the
    /// requested size. Send an empty buffer on end of stream. fh will contain the
    /// value set by the opendir method, or will be undefined if the opendir method
    /// didn't set any value.
    fn readdirplus<R: ReplyDirectoryPlus + Send + 'static>(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        // TODO In fuser, offset was i64. Why?
        offset: u64,
        // TODO Can we do this via a callback that takes an iterator
        reply: &mut R,
    ) -> impl Future<Output = FsResult<()>> + Send;

    /// Release an open directory.
    /// For every opendir call there will be exactly one releasedir call. fh will
    /// contain the value set by the opendir method, or will be undefined if the
    /// opendir method didn't set any value.
    fn releasedir(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        flags: OpenInFlags,
    ) -> impl Future<Output = FsResult<()>> + Send;

    /// Synchronize directory contents.
    /// If the datasync parameter is set, then only the directory contents should
    /// be flushed, not the meta data. fh will contain the value set by the opendir
    /// method, or will be undefined if the opendir method didn't set any value.
    fn fsyncdir(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        datasync: bool,
    ) -> impl Future<Output = FsResult<()>> + Send;

    /// Get file system statistics.
    fn statfs(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
    ) -> impl Future<Output = FsResult<Statfs>> + Send;

    /// Set an extended attribute.
    fn setxattr(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        // TODO Different wrapper type for name that isn't PathComponent? Are the rules the same for xattr names and path components?
        name: &PathComponent,
        value: &[u8],
        // TODO Wrapper type for flags
        flags: i32,
        position: NumBytes,
    ) -> impl Future<Output = FsResult<()>> + Send;

    /// Get the size of a file extended attribute.
    fn getxattr_numbytes(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        // TODO Different wrapper type for name that isn't PathComponent? Are the rules the same for xattr names and path components?
        name: &PathComponent,
    ) -> impl Future<Output = FsResult<NumBytes>> + Send;

    /// Get the data stored in a file extended attribute.
    /// Return FsError::XattrBufferTooSmall if `max_bytes_to_read` is too small.
    ///
    /// TODO Should we change the API to a callback based one, similar to how `read` works? Could reduce amount of copies needed
    fn getxattr_data(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        // TODO Different wrapper type for name that isn't PathComponent? Are the rules the same for xattr names and path components?
        name: &PathComponent,
        max_bytes_to_read: NumBytes,
    ) -> impl Future<Output = FsResult<Vec<u8>>> + Send;

    /// Return the number of bytes that would be returned by a call to [Self::listxattr_data].
    ///
    /// See [Self::listxattr_data] for a definition of what it returns.
    fn listxattr_numbytes(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
    ) -> impl Future<Output = FsResult<NumBytes>> + Send;

    /// List extended attributes for a file. Return all the null-terminated attribute names.
    /// Return FsError::XattrBufferTooSmall if `max_bytes_to_read` is too small.
    ///
    /// // TODO Come up with a better way to handle this return, and its combination with listxattr_numbytes.
    fn listxattr_data(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        max_bytes_to_read: NumBytes,
    ) -> impl Future<Output = FsResult<Vec<u8>>> + Send;

    /// Remove an extended attribute.
    fn removexattr(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        // TODO Different wrapper type for name that isn't PathComponent? Are the rules the same for xattr names and path components?
        name: &PathComponent,
    ) -> impl Future<Output = FsResult<()>> + Send;

    /// Check file access permissions.
    /// This will be called for the access() system call. If the 'default_permissions'
    /// mount option is given, this method is not called. This method is not called
    /// under Linux kernel versions 2.4.x
    fn access(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        // TODO Wrapper task for mask
        mask: i32,
    ) -> impl Future<Output = FsResult<()>> + Send;

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
        &self,
        req: &RequestInfo,
        parent: InodeNumber,
        name: &PathComponent,
        mode: Mode,
        // TODO Wrapper type for umask
        umask: u32,
        flags: OpenInFlags,
    ) -> impl Future<Output = FsResult<ReplyCreate>> + Send;

    /// Test for a POSIX file lock.
    fn getlk(
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
    ) -> impl Future<Output = FsResult<ReplyLock>> + Send;

    /// Acquire, modify or release a POSIX file lock.
    /// For POSIX threads (NPTL) there's a 1-1 relation between pid and owner, but
    /// otherwise this is not always the case.  For checking lock ownership,
    /// 'fi->owner' must be used. The l_pid field in 'struct flock' should only be
    /// used to fill in this field in getlk(). Note: if the locking methods are not
    /// implemented, the kernel will still allow file locking to work locally.
    /// Hence these are only interesting for network filesystems and similar.
    fn setlk(
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
    ) -> impl Future<Output = FsResult<()>> + Send;

    /// Map block index within file to block index within device.
    /// Note: This makes sense only for block device backed filesystems mounted
    /// with the 'blkdev' option
    fn bmap(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        blocksize: NumBytes,
        // TODO What is idx?
        idx: u64,
    ) -> impl Future<Output = FsResult<ReplyBmap>> + Send;

    /// control device
    fn ioctl(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        // TODO Wrapper types for remaining args
        flags: u32,
        cmd: u32,
        in_data: &[u8],
        out_size: u32,
    ) -> impl Future<Output = FsResult<ReplyIoctl>> + Send;

    /// Preallocate or deallocate space to a file
    fn fallocate(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        // TODO offset and length in fuser was i64. Why?
        offset: NumBytes,
        length: NumBytes,
        mode: Mode,
    ) -> impl Future<Output = FsResult<()>> + Send;

    /// Reposition read/write file offset
    fn lseek(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        // TODO offset was i64 in fuser. Why?
        offset: NumBytes,
        // TODO What is whence?
        whence: i32,
    ) -> impl Future<Output = FsResult<ReplyLseek>> + Send;

    // TODO Some below (and maybe some above) aren't actually needed and fuse allows returning ENOSYS as a "not-implemented" marker, see https://www.youtube.com/watch?v=id0Kkq4VHDo
    //     See also which ones are actually implemented in https://github.com/wfraser/fuse-mt/blob/master/src/fusemt.rs

    /// Copy the specified range from the source inode to the destination inode
    fn copy_file_range(
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
        flags: u64,
    ) -> impl Future<Output = FsResult<ReplyWrite>> + Send;

    /// macOS only: Rename the volume. Set fuse_init_out.flags during init to
    /// FUSE_VOL_RENAME to enable
    #[cfg(target_os = "macos")]
    fn setvolname(
        &self,
        req: &RequestInfo,
        name: &str,
    ) -> impl Future<Output = FsResult<()>> + Send;

    /// macOS only (undocumented)
    #[cfg(target_os = "macos")]
    fn exchange(
        &self,
        req: &RequestInfo,
        parent: InodeNumber,
        name: &PathComponent,
        newparent: InodeNumber,
        newname: &PathComponent,
        // TODO Wrapper type for options
        options: u64,
    ) -> impl Future<Output = FsResult<()>> + Send;

    /// macOS only: Query extended times (bkuptime and crtime). Set fuse_init_out.flags
    /// during init to FUSE_XTIMES to enable
    #[cfg(target_os = "macos")]
    fn getxtimes(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
    ) -> impl Future<Output = FsResult<ReplyXTimes>> + Send;
}
