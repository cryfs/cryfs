use async_trait::async_trait;
use derive_more::{From, Into};
use std::time::{Duration, SystemTime};

use crate::common::{
    AbsolutePath, DirEntry, FsResult, Gid, Mode, NodeAttrs, NumBytes, OpenFlags, Statfs, Uid,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct RequestInfo {
    /// The unique ID assigned to this request by FUSE.
    pub unique: u64,
    /// The user ID of the process making the request.
    pub uid: Uid,
    /// The group ID of the process making the request.
    pub gid: Gid,
    /// The process ID of the process making the request.
    /// // TODO Make a Pid type instead of using u32
    pub pid: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, From, Into)]
pub struct FileHandle(pub u64);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AttrResponse {
    pub attrs: NodeAttrs,
    pub ttl: Duration,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct OpenResponse {
    pub fh: FileHandle,
    pub flags: OpenFlags,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct OpendirResponse {
    pub fh: FileHandle,
    // TODO Wrap flags into its own type, or reuse OpenFlags?
    pub flags: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CreateResponse {
    pub ttl: Duration,
    pub attrs: NodeAttrs,
    pub fh: FileHandle,
    // TODO Wrap flags into its own type, or reuse OpenFlags?
    pub flags: i32,
}

#[async_trait(?Send)]
pub trait AsyncFilesystem {
    /// Called on mount, before any other function.
    async fn init(&self, req: RequestInfo) -> FsResult<()>;

    /// Called on filesystem unmount.
    async fn destroy(&self);

    /// Get the attributes of a filesystem entry.
    ///
    /// * `fh`: a file handle if this is called on an open file.
    async fn getattr(
        &self,
        req: RequestInfo,
        path: &AbsolutePath,
        fh: Option<FileHandle>,
    ) -> FsResult<AttrResponse>;

    /// Change the mode of a filesystem entry.
    ///
    /// * `fh`: a file handle if this is called on an open file.
    /// * `mode`: the mode to change the file to.
    async fn chmod(
        &self,
        req: RequestInfo,
        path: &AbsolutePath,
        fh: Option<FileHandle>,
        mode: Mode,
    ) -> FsResult<()>;

    /// Change the owner UID and/or group GID of a filesystem entry.
    ///
    /// * `fh`: a file handle if this is called on an open file.
    /// * `uid`: user ID to change the file's owner to. If `None`, leave the UID unchanged.
    /// * `gid`: group ID to change the file's group to. If `None`, leave the GID unchanged.
    async fn chown(
        &self,
        req: RequestInfo,
        path: &AbsolutePath,
        fh: Option<FileHandle>,
        uid: Option<Uid>,
        gid: Option<Gid>,
    ) -> FsResult<()>;

    /// Set the length of a file.
    ///
    /// * `fh`: a file handle if this is called on an open file.
    /// * `size`: size in bytes to set as the file's length.
    async fn truncate(
        &self,
        req: RequestInfo,
        path: &AbsolutePath,
        fh: Option<FileHandle>,
        size: NumBytes,
    ) -> FsResult<()>;

    /// Set timestamps of a filesystem entry.
    ///
    /// * `fh`: a file handle if this is called on an open file.
    /// * `atime`: the time of last access.
    /// * `mtime`: the time of last modification.
    async fn utimens(
        &self,
        req: RequestInfo,
        path: &AbsolutePath,
        fh: Option<FileHandle>,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
    ) -> FsResult<()>;

    /// Set timestamps of a filesystem entry (with extra options only used on MacOS).
    async fn utimens_macos(
        &self,
        req: RequestInfo,
        path: &AbsolutePath,
        fh: Option<FileHandle>,
        crtime: Option<SystemTime>,
        chgtime: Option<SystemTime>,
        bkuptime: Option<SystemTime>,
        // TODO What are those flags? Should we wrap them into a custom type?
        flags: Option<u32>,
    ) -> FsResult<()>;

    /// Read a symbolic link.
    /// TODO Use custom type for absolute-or-relative paths as the return type
    async fn readlink(&self, req: RequestInfo, path: &AbsolutePath) -> FsResult<String>;

    /// Create a special file.
    ///
    /// * `path`: path of the file to create
    /// * `mode`: mode for the new entry.
    /// * `rdev`: if mode has the bits `S_IFCHR` or `S_IFBLK` set, this is the major and minor numbers for the device file. Otherwise it should be ignored.
    async fn mknod(
        &self,
        req: RequestInfo,
        path: &AbsolutePath,
        mode: Mode,
        // TODO What to do with rdev? Should we wrap it into a custom type?
        rdev: u32,
    ) -> FsResult<AttrResponse>;

    /// Create a directory.
    ///
    /// * `path`: path of the directory to create
    /// * `mode`: permissions for the new directory.
    async fn mkdir(
        &self,
        req: RequestInfo,
        path: &AbsolutePath,
        mode: Mode,
    ) -> FsResult<AttrResponse>;

    /// Remove a file.
    ///
    /// * `path`: path of the file or symlink to delete
    async fn unlink(&self, req: RequestInfo, path: &AbsolutePath) -> FsResult<()>;

    /// Remove a directory.
    ///
    /// * `path`: path of the directory to delete
    async fn rmdir(&self, req: RequestInfo, path: &AbsolutePath) -> FsResult<()>;

    /// Create a symbolic link.
    ///
    /// * `path`: path of the symlink to create
    /// * `target`: path (may be relative or absolute) to the target of the link.
    async fn symlink(
        &self,
        req: RequestInfo,
        path: &AbsolutePath,
        // TODO We may want to introduce a separate `Path` type for paths that can be either relative or absolute
        target: &str,
    ) -> FsResult<AttrResponse>;

    /// Rename a filesystem entry.
    ///
    /// * `oldpath`: path to the existing entry
    /// * `newpath`: path the entry should be reachable at after the rename/move operation
    async fn rename(
        &self,
        req: RequestInfo,
        oldpath: &AbsolutePath,
        newpath: &AbsolutePath,
    ) -> FsResult<()>;

    /// Create a hard link.
    ///
    /// * `path`: path to an existing file.
    /// * `newpath`: path to the new hardlink under which the file should now also be reachable.
    async fn link(
        &self,
        req: RequestInfo,
        oldpath: &AbsolutePath,
        newpath: &AbsolutePath,
    ) -> FsResult<AttrResponse>;

    /// Open a file.
    ///
    /// * `path`: path to the file.
    /// * `flags`: one of `O_RDONLY`, `O_WRONLY`, or `O_RDWR`, plus maybe additional flags.
    ///
    /// Return a struct with file handle and flags. The file handle will be passed to any subsequent
    /// calls that operate on the file, and can be any value you choose, though it should allow
    /// your filesystem to identify the file opened even without any path info.
    async fn open(
        &self,
        req: RequestInfo,
        path: &AbsolutePath,
        flags: OpenFlags,
    ) -> FsResult<OpenResponse>;

    /// Read from a file.
    ///
    /// Note that it is not an error for this call to request to read past the end of the file, and
    /// you should only return data up to the end of the file (i.e. the number of bytes returned
    /// will be fewer than requested; possibly even zero). Do not extend the file in this case.
    ///
    /// * `path`: path to the file.
    /// * `fh`: file handle returned from the `open` call.
    /// * `offset`: offset into the file to start reading.
    /// * `size`: number of bytes to read.
    /// * `callback`: a callback that must be invoked to return the result of the operation: either
    ///    the result data as a slice, or an error code.
    ///
    /// Return the return value from the `callback` function.
    async fn read<CallbackResult>(
        &self,
        req: RequestInfo,
        path: &AbsolutePath,
        fh: FileHandle,
        offset: NumBytes,
        size: NumBytes,
        callback: impl for<'a> FnOnce(FsResult<&'a [u8]>) -> CallbackResult,
    ) -> CallbackResult;

    /// Write to a file.
    ///
    /// * `path`: path to the file.
    /// * `fh`: file handle returned from the `open` call.
    /// * `offset`: offset into the file to start writing.
    /// * `data`: the data to write
    /// * `flags`:
    ///
    /// Return the number of bytes written.
    async fn write(
        &self,
        req: RequestInfo,
        path: &AbsolutePath,
        fh: FileHandle,
        offset: NumBytes,
        data: Vec<u8>,
        // TODO What is the flags parameter for? Should we use a type wrapper instead of u32?
        flags: u32,
    ) -> FsResult<NumBytes>;

    /// Called each time a program calls `close` on an open file.
    ///
    /// Note that because file descriptors can be duplicated (by `dup`, `dup2`, `fork`) this may be
    /// called multiple times for a given file handle. The main use of this function is if the
    /// filesystem would like to return an error to the `close` call. Note that most programs
    /// ignore the return value of `close`, though.
    ///
    /// * `path`: path to the file.
    /// * `fh`: file handle returned from the `open` call.
    /// * `lock_owner`: if the filesystem supports locking (`setlk`, `getlk`), remove all locks
    ///   belonging to this lock owner.
    async fn flush(
        &self,
        req: RequestInfo,
        path: &AbsolutePath,
        fh: FileHandle,
        lock_owner: u64,
    ) -> FsResult<()>;

    /// Called when an open file is closed.
    ///
    /// There will be one of these for each `open` call. After `release`, no more calls will be
    /// made with the given file handle.
    ///
    /// * `path`: path to the file.
    /// * `fh`: file handle returned from the `open` call.
    /// * `flags`: the flags passed when the file was opened.
    /// * `lock_owner`: if the filesystem supports locking (`setlk`, `getlk`), remove all locks
    ///   belonging to this lock owner.
    /// * `flush`: whether pending data must be flushed or not.
    async fn release(
        &self,
        req: RequestInfo,
        path: &AbsolutePath,
        fh: FileHandle,
        flags: OpenFlags,
        // TODO What to do with lock_owner in flush and release? Wrap into a custom type?
        lock_owner: u64,
        flush: bool,
    ) -> FsResult<()>;

    /// Write out any pending changes of a file.
    ///
    /// When this returns, data should be written to persistent storage.
    ///
    /// * `path`: path to the file.
    /// * `fh`: file handle returned from the `open` call.
    /// * `datasync`: if `false`, also write metadata, otherwise just write file data.
    async fn fsync(
        &self,
        req: RequestInfo,
        path: &AbsolutePath,
        fh: FileHandle,
        datasync: bool,
    ) -> FsResult<()>;

    /// Open a directory.
    ///
    /// Analogous to the `opend` call.
    ///
    /// * `path`: path to the directory.
    /// * `flags`: file access flags. Will contain `O_DIRECTORY` at least.
    ///
    /// Return a struct with file handle and flags. The file handle will be passed to any subsequent
    /// calls that operate on the directory, and can be any value you choose, though it should
    /// allow your filesystem to identify the directory opened even without any path info.
    /// // TODO Wrap flags into some custom type instead of using u32
    async fn opendir(
        &self,
        req: RequestInfo,
        path: &AbsolutePath,
        flags: u32,
    ) -> FsResult<OpendirResponse>;

    /// Get the entries of a directory.
    ///
    /// * `path`: path to the directory.
    /// * `fh`: file handle returned from the `opendir` call.
    ///
    /// Return all the entries of the directory.
    /// TODO Should we change the API to a callback based one, similar to how `read` works? With the callback being called for each entry? Could reduce amount of copies needed.
    async fn readdir(
        &self,
        req: RequestInfo,
        path: &AbsolutePath,
        fh: FileHandle,
    ) -> FsResult<Vec<DirEntry>>;

    /// Close an open directory.
    ///
    /// This will be called exactly once for each `opendir` call.
    ///
    /// * `path`: path to the directory.
    /// * `fh`: file handle returned from the `opendir` call.
    /// * `flags`: the file access flags passed to the `opendir` call.
    /// // TODO Wrap flags into some custom type instead of using u32
    async fn releasedir(
        &self,
        req: RequestInfo,
        path: &AbsolutePath,
        fh: FileHandle,
        flags: u32,
    ) -> FsResult<()>;

    /// Write out any pending changes to a directory.
    ///
    /// Analogous to the `fsync` call.
    async fn fsyncdir(
        &self,
        req: RequestInfo,
        path: &AbsolutePath,
        fh: FileHandle,
        datasync: bool,
    ) -> FsResult<()>;

    /// Get filesystem statistics.
    ///
    /// * `path`: path to some folder in the filesystem.
    ///
    /// See the `Statfs` struct for more details.
    async fn statfs(&self, req: RequestInfo, path: &AbsolutePath) -> FsResult<Statfs>;

    /// Set a file extended attribute.
    ///
    /// * `path`: path to the file.
    /// * `name`: attribute name.
    /// * `value`: the data to set the value to.
    /// * `flags`: can be either `XATTR_CREATE` or `XATTR_REPLACE`.
    /// * `position`: offset into the attribute value to write data.
    async fn setxattr(
        &self,
        req: RequestInfo,
        path: &AbsolutePath,
        // TODO Use a wrapper type for attribute names that ensures its invariants (e.g. allowed characters)
        name: &str,
        value: &[u8],
        // TODO flags/position should be wrapped into a custom types instead of using u32
        flags: u32,
        position: NumBytes,
    ) -> FsResult<()>;

    /// Get the size of a file extended attribute.
    ///
    /// * `path`: path to the file
    /// * `name`: attribute name.
    async fn getxattr_numbytes(
        &self,
        req: RequestInfo,
        path: &AbsolutePath,
        name: &str,
    ) -> FsResult<NumBytes>;

    /// Get the data stored in a file extended attribute.
    ///
    /// * `path`: path to the file
    /// * `name`: attribute name.
    /// * `size`: the maximum number of bytes to read.
    ///
    /// TODO Should we change the API to a callback based one, similar to how `read` works? Could reduce amount of copies needed
    async fn getxattr_data(
        &self,
        req: RequestInfo,
        path: &AbsolutePath,
        name: &str,
        size: NumBytes,
    ) -> FsResult<Vec<u8>>;

    /// List extended attributes for a file.
    ///
    /// * `path`: path to the file.
    /// * `size`: maximum number of bytes to return.
    ///
    /// Return the number of bytes that would be returned by a call to [Self::listxattr_data].
    /// See [Self::listxattr_data] for a definition of what it returns.
    async fn listxattr_numbytes(&self, req: RequestInfo, path: &AbsolutePath)
        -> FsResult<NumBytes>;

    /// List extended attributes for a file.
    ///
    /// * `path`: path to the file.
    /// * `size`: maximum number of bytes to return.
    ///
    /// Return all the null-terminated attribute names.
    /// // TODO Come up with a better way to handle this return, and its combination with listxattr_numbytes.
    async fn listxattr_data(
        &self,
        req: RequestInfo,
        path: &AbsolutePath,
        size: NumBytes,
    ) -> FsResult<Vec<u8>>;

    /// Remove an extended attribute for a file.
    ///
    /// * `path`: path to the file.
    /// * `name`: name of the attribute to remove.
    async fn removexattr(&self, req: RequestInfo, path: &AbsolutePath, name: &str) -> FsResult<()>;

    /// Check for access to a file.
    ///
    /// * `path`: path to the file.
    /// * `mask`: mode bits to check for access to.
    ///
    /// Return `Ok(())` if all requested permissions are allowed, otherwise return `Err(EACCES)`
    /// or other error code as appropriate (e.g. `ENOENT` if the file doesn't exist).
    /// TODO Wrap mask into a custom type instead of using u32
    async fn access(&self, req: RequestInfo, path: &AbsolutePath, mask: u32) -> FsResult<()>;

    /// Create and open a new file.
    ///
    /// * `path`: path of the file to create
    /// * `mode`: the mode to set on the new file.
    /// * `flags`: flags like would be passed to `open`.
    ///
    /// Return a `CreateResponse` (which contains the new file's attributes as well as a file handle
    /// -- see documentation on `open` for more info on that).
    async fn create(
        &self,
        req: RequestInfo,
        path: &AbsolutePath,
        mode: Mode,
        // TODO What with flags? Wrap into a custom type instead of using u32? Also, the fuse-mt function not only takes but also returns flags. Is this necessary? What are these flags?
        flags: i32,
    ) -> FsResult<CreateResponse>;
}
