use fuse_mt::{
    CallbackResult, FileAttr, FilesystemMT, RequestInfo, ResultCreate, ResultData, ResultEmpty,
    ResultEntry, ResultOpen, ResultReaddir, ResultSlice, ResultStatfs, ResultWrite, ResultXattr,
};
use std::ffi::OsStr;
use std::future::Future;
use std::path::Path;
use std::time::{Duration, SystemTime};

use crate::interface::{Device, Dir, DirEntry, FsError, FsResult, Node, NodeAttrs};
use crate::utils::{Mode, NodeKind};

pub struct FsAdapter<Fs: Device> {
    fs: Fs,
    runtime: tokio::runtime::Runtime,
}

impl<Fs: Device> FsAdapter<Fs> {
    pub fn new(fs: Fs) -> Self {
        // TODO Runtime settings
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .thread_name("rustfs")
            .build()
            .unwrap();
        Self { fs, runtime }
    }

    fn run_async<R, F>(&self, log_msg: &str, func: impl FnOnce() -> F) -> Result<R, libc::c_int>
    where
        F: Future<Output = FsResult<R>>,
    {
        self.runtime.block_on(async move {
            log::info!("{}...", log_msg);
            let result = func().await;
            match result {
                Ok(ok) => {
                    log::info!("{}...done", log_msg);
                    Ok(ok)
                }
                Err(err) => {
                    log::error!("{}...failed: {}", log_msg, err);
                    Err(err.system_error_code())
                }
            }
        })
    }
}

impl<Fs: Device> FilesystemMT for FsAdapter<Fs> {
    /// Called on mount, before any other function.
    fn init(&self, _req: RequestInfo) -> ResultEmpty {
        log::info!("init");
        Ok(())
    }

    /// Called on filesystem unmount.
    fn destroy(&self) {
        log::info!("destroy");
        // Nothing.
    }

    /// Get the attributes of a filesystem entry.
    ///
    /// * `fh`: a file handle if this is called on an open file.
    fn getattr(&self, _req: RequestInfo, path: &Path, fh: Option<u64>) -> ResultEntry {
        self.run_async(&format!("getattr {path:?}"), move || async move {
            if fh.is_some() {
                todo!();
            }
            let node = self.fs.load_node(path).await?;
            // TODO No unwrap
            let attrs = node.getattr().await?;
            // TODO What is the ttl here?
            let ttl = Duration::ZERO;
            Ok((ttl, convert_file_attrs(attrs)))
        })
    }

    // The following operations in the FUSE C API are all one kernel call: setattr
    // We split them out to match the C API's behavior.

    /// Change the mode of a filesystem entry.
    ///
    /// * `fh`: a file handle if this is called on an open file.
    /// * `mode`: the mode to change the file to.
    fn chmod(&self, _req: RequestInfo, path: &Path, _fh: Option<u64>, mode: u32) -> ResultEmpty {
        log::warn!("chmod({path:?}, mode={mode})...unimplemented");
        Err(libc::ENOSYS)
    }

    /// Change the owner UID and/or group GID of a filesystem entry.
    ///
    /// * `fh`: a file handle if this is called on an open file.
    /// * `uid`: user ID to change the file's owner to. If `None`, leave the UID unchanged.
    /// * `gid`: group ID to change the file's group to. If `None`, leave the GID unchanged.
    fn chown(
        &self,
        _req: RequestInfo,
        path: &Path,
        _fh: Option<u64>,
        uid: Option<u32>,
        gid: Option<u32>,
    ) -> ResultEmpty {
        log::warn!("chown({path:?}, uid={uid:?}, gid={gid:?})...unimplemented");
        Err(libc::ENOSYS)
    }

    /// Set the length of a file.
    ///
    /// * `fh`: a file handle if this is called on an open file.
    /// * `size`: size in bytes to set as the file's length.
    fn truncate(&self, _req: RequestInfo, path: &Path, _fh: Option<u64>, size: u64) -> ResultEmpty {
        log::warn!("truncate({path:?}, size={size})...unimplemented");
        Err(libc::ENOSYS)
    }

    /// Set timestamps of a filesystem entry.
    ///
    /// * `fh`: a file handle if this is called on an open file.
    /// * `atime`: the time of last access.
    /// * `mtime`: the time of last modification.
    fn utimens(
        &self,
        _req: RequestInfo,
        path: &Path,
        _fh: Option<u64>,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
    ) -> ResultEmpty {
        log::warn!("utimens({path:?}, atime={atime:?}, mtime={mtime:?})...unimplemented");
        Err(libc::ENOSYS)
    }

    /// Set timestamps of a filesystem entry (with extra options only used on MacOS).
    #[allow(clippy::too_many_arguments)]
    fn utimens_macos(
        &self,
        _req: RequestInfo,
        path: &Path,
        _fh: Option<u64>,
        crtime: Option<SystemTime>,
        chgtime: Option<SystemTime>,
        bkuptime: Option<SystemTime>,
        flags: Option<u32>,
    ) -> ResultEmpty {
        log::warn!("utimens_macos({path:?}, crtime={crtime:?}, chgtime={chgtime:?}, bkuptime={bkuptime:?}, flags={flags:?})...unimplemented");
        Err(libc::ENOSYS)
    }

    // END OF SETATTR FUNCTIONS

    /// Read a symbolic link.
    fn readlink(&self, _req: RequestInfo, path: &Path) -> ResultData {
        log::warn!("readlink({path:?})...unimplemented");
        Err(libc::ENOSYS)
    }

    /// Create a special file.
    ///
    /// * `parent`: path to the directory to make the entry under.
    /// * `name`: name of the entry.
    /// * `mode`: mode for the new entry.
    /// * `rdev`: if mode has the bits `S_IFCHR` or `S_IFBLK` set, this is the major and minor numbers for the device file. Otherwise it should be ignored.
    fn mknod(
        &self,
        _req: RequestInfo,
        parent: &Path,
        name: &OsStr,
        mode: u32,
        rdev: u32,
    ) -> ResultEntry {
        log::warn!("mknod({parent:?}, name={name:?}, mode={mode}, rdev={rdev})...unimplemented");
        Err(libc::ENOSYS)
    }

    /// Create a directory.
    ///
    /// * `parent`: path to the directory to make the directory under.
    /// * `name`: name of the directory.
    /// * `mode`: permissions for the new directory.
    fn mkdir(&self, _req: RequestInfo, parent: &Path, name: &OsStr, mode: u32) -> ResultEntry {
        log::warn!("mkdir({parent:?}, name={name:?}, mode={mode})...unimplemented");
        Err(libc::ENOSYS)
    }

    /// Remove a file.
    ///
    /// * `parent`: path to the directory containing the file to delete.
    /// * `name`: name of the file to delete.
    fn unlink(&self, _req: RequestInfo, parent: &Path, name: &OsStr) -> ResultEmpty {
        log::warn!("unlink({parent:?}, name={name:?})...unimplemented");
        Err(libc::ENOSYS)
    }

    /// Remove a directory.
    ///
    /// * `parent`: path to the directory containing the directory to delete.
    /// * `name`: name of the directory to delete.
    fn rmdir(&self, _req: RequestInfo, parent: &Path, name: &OsStr) -> ResultEmpty {
        log::warn!("rmdir({parent:?}, name={name:?})...unimplemented");
        Err(libc::ENOSYS)
    }

    /// Create a symbolic link.
    ///
    /// * `parent`: path to the directory to make the link in.
    /// * `name`: name of the symbolic link.
    /// * `target`: path (may be relative or absolute) to the target of the link.
    fn symlink(
        &self,
        _req: RequestInfo,
        parent: &Path,
        name: &OsStr,
        target: &Path,
    ) -> ResultEntry {
        log::warn!("symlink({parent:?}, name={name:?}, target={target:?})...unimplemented");
        Err(libc::ENOSYS)
    }

    /// Rename a filesystem entry.
    ///
    /// * `parent`: path to the directory containing the existing entry.
    /// * `name`: name of the existing entry.
    /// * `newparent`: path to the directory it should be renamed into (may be the same as `parent`).
    /// * `newname`: name of the new entry.
    fn rename(
        &self,
        _req: RequestInfo,
        parent: &Path,
        name: &OsStr,
        newparent: &Path,
        newname: &OsStr,
    ) -> ResultEmpty {
        log::warn!(
            "rename({parent:?}, name={name:?}, newparent={newparent:?}, newname={newname:?})...unimplemented"
        );
        Err(libc::ENOSYS)
    }

    /// Create a hard link.
    ///
    /// * `path`: path to an existing file.
    /// * `newparent`: path to the directory for the new link.
    /// * `newname`: name for the new link.
    fn link(
        &self,
        _req: RequestInfo,
        path: &Path,
        newparent: &Path,
        newname: &OsStr,
    ) -> ResultEntry {
        log::warn!("link({path:?}, newparent={newparent:?}, newname={newname:?})...unimplemented");
        Err(libc::ENOSYS)
    }

    /// Open a file.
    ///
    /// * `path`: path to the file.
    /// * `flags`: one of `O_RDONLY`, `O_WRONLY`, or `O_RDWR`, plus maybe additional flags.
    ///
    /// Return a tuple of (file handle, flags). The file handle will be passed to any subsequent
    /// calls that operate on the file, and can be any value you choose, though it should allow
    /// your filesystem to identify the file opened even without any path info.
    fn open(&self, _req: RequestInfo, path: &Path, flags: u32) -> ResultOpen {
        log::warn!("open({path:?}, flags={flags})...unimplemented");
        Err(libc::ENOSYS)
    }

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
    fn read(
        &self,
        _req: RequestInfo,
        path: &Path,
        fh: u64,
        offset: u64,
        size: u32,
        callback: impl FnOnce(ResultSlice<'_>) -> CallbackResult,
    ) -> CallbackResult {
        log::warn!("read({path:?}, fh={fh}, offset={offset}, size={size})...unimplemented");
        callback(Err(libc::ENOSYS))
    }

    /// Write to a file.
    ///
    /// * `path`: path to the file.
    /// * `fh`: file handle returned from the `open` call.
    /// * `offset`: offset into the file to start writing.
    /// * `data`: the data to write
    /// * `flags`:
    ///
    /// Return the number of bytes written.
    fn write(
        &self,
        _req: RequestInfo,
        path: &Path,
        fh: u64,
        offset: u64,
        data: Vec<u8>,
        flags: u32,
    ) -> ResultWrite {
        log::warn!("write({path:?}, fh={fh}, offset={offset}, data={data:?}, flags={flags})...unimplemented");
        Err(libc::ENOSYS)
    }

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
    fn flush(&self, _req: RequestInfo, path: &Path, fh: u64, lock_owner: u64) -> ResultEmpty {
        log::warn!("flush({path:?}, fh={fh}, lock_owner={lock_owner})...unimplemented");
        Err(libc::ENOSYS)
    }

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
    fn release(
        &self,
        _req: RequestInfo,
        path: &Path,
        fh: u64,
        flags: u32,
        lock_owner: u64,
        flush: bool,
    ) -> ResultEmpty {
        log::warn!(
            "release({path:?}, fh={fh}, flags={flags}, lock_owner={lock_owner}, flush={flush})...unimplemented",
        );
        Err(libc::ENOSYS)
    }

    /// Write out any pending changes of a file.
    ///
    /// When this returns, data should be written to persistent storage.
    ///
    /// * `path`: path to the file.
    /// * `fh`: file handle returned from the `open` call.
    /// * `datasync`: if `false`, also write metadata, otherwise just write file data.
    fn fsync(&self, _req: RequestInfo, path: &Path, fh: u64, datasync: bool) -> ResultEmpty {
        log::warn!("fsync({path:?}, fh={fh}, datasync={datasync})...unimplemented");
        Err(libc::ENOSYS)
    }

    /// Open a directory.
    ///
    /// Analogous to the `opend` call.
    ///
    /// * `path`: path to the directory.
    /// * `flags`: file access flags. Will contain `O_DIRECTORY` at least.
    ///
    /// Return a tuple of (file handle, flags). The file handle will be passed to any subsequent
    /// calls that operate on the directory, and can be any value you choose, though it should
    /// allow your filesystem to identify the directory opened even without any path info.
    fn opendir(&self, _req: RequestInfo, path: &Path, flags: u32) -> ResultOpen {
        self.run_async(
            &format!("opendir({path:?}, flags={flags})"),
            move || async move {
                // TODO Do we need opendir? The path seems to be passed to readdir, but the fuse_mt comment
                // to opendir seems to suggest that readdir may have to recognize dirs with just the fh and no path?
                Ok((0, flags))
            },
        )
    }

    /// Get the entries of a directory.
    ///
    /// * `path`: path to the directory.
    /// * `fh`: file handle returned from the `opendir` call.
    ///
    /// Return all the entries of the directory.
    fn readdir(&self, _req: RequestInfo, path: &Path, fh: u64) -> ResultReaddir {
        self.run_async(&format!("readdir({path:?}, fh={fh})"), move || async move {
            let dir = self.fs.load_dir(path).await?;
            // TODO No unwrap
            let entries = dir.entries().await?;
            let entries = convert_dir_entries(entries);
            Ok(entries)
        })
    }

    /// Close an open directory.
    ///
    /// This will be called exactly once for each `opendir` call.
    ///
    /// * `path`: path to the directory.
    /// * `fh`: file handle returned from the `opendir` call.
    /// * `flags`: the file access flags passed to the `opendir` call.
    fn releasedir(&self, _req: RequestInfo, path: &Path, fh: u64, flags: u32) -> ResultEmpty {
        self.run_async(
            &format!("releasedir({path:?}, fh={fh}, flags={flags})"),
            move || async move {
                // TODO If we need opendir, then we also need releasedir, see TODO comment in opendir
                Ok(())
            },
        )
    }

    /// Write out any pending changes to a directory.
    ///
    /// Analogous to the `fsync` call.
    fn fsyncdir(&self, _req: RequestInfo, path: &Path, fh: u64, datasync: bool) -> ResultEmpty {
        log::warn!("fsyncdir({path:?}, fh={fh}, datasync={datasync})...unimplemented");
        Err(libc::ENOSYS)
    }

    /// Get filesystem statistics.
    ///
    /// * `path`: path to some folder in the filesystem.
    ///
    /// See the `Statfs` struct for more details.
    fn statfs(&self, _req: RequestInfo, path: &Path) -> ResultStatfs {
        log::warn!("statfs({path:?})...unimplemented");
        Err(libc::ENOSYS)
    }

    /// Set a file extended attribute.
    ///
    /// * `path`: path to the file.
    /// * `name`: attribute name.
    /// * `value`: the data to set the value to.
    /// * `flags`: can be either `XATTR_CREATE` or `XATTR_REPLACE`.
    /// * `position`: offset into the attribute value to write data.
    fn setxattr(
        &self,
        _req: RequestInfo,
        path: &Path,
        name: &OsStr,
        value: &[u8],
        flags: u32,
        position: u32,
    ) -> ResultEmpty {
        log::warn!(
            "setxattr({path:?}, name={name:?}, value={value:?}, flags={flags}, position={position})...unimplemented",
        );
        Err(libc::ENOSYS)
    }

    /// Get a file extended attribute.
    ///
    /// * `path`: path to the file
    /// * `name`: attribute name.
    /// * `size`: the maximum number of bytes to read.
    ///
    /// If `size` is 0, return `Xattr::Size(n)` where `n` is the size of the attribute data.
    /// Otherwise, return `Xattr::Data(data)` with the requested data.
    fn getxattr(&self, _req: RequestInfo, path: &Path, name: &OsStr, size: u32) -> ResultXattr {
        log::warn!("getxattr({path:?}, name={name:?}, size={size})...unimplemented");
        Err(libc::ENOSYS)
    }

    /// List extended attributes for a file.
    ///
    /// * `path`: path to the file.
    /// * `size`: maximum number of bytes to return.
    ///
    /// If `size` is 0, return `Xattr::Size(n)` where `n` is the size required for the list of
    /// attribute names.
    /// Otherwise, return `Xattr::Data(data)` where `data` is all the null-terminated attribute
    /// names.
    fn listxattr(&self, _req: RequestInfo, path: &Path, size: u32) -> ResultXattr {
        log::warn!("listxattr({path:?}, size={size})...unimplemented");
        Err(libc::ENOSYS)
    }

    /// Remove an extended attribute for a file.
    ///
    /// * `path`: path to the file.
    /// * `name`: name of the attribute to remove.
    fn removexattr(&self, _req: RequestInfo, path: &Path, name: &OsStr) -> ResultEmpty {
        log::warn!("removexattr({path:?}, name={name:?})...unimplemented");
        Err(libc::ENOSYS)
    }

    /// Check for access to a file.
    ///
    /// * `path`: path to the file.
    /// * `mask`: mode bits to check for access to.
    ///
    /// Return `Ok(())` if all requested permissions are allowed, otherwise return `Err(EACCES)`
    /// or other error code as appropriate (e.g. `ENOENT` if the file doesn't exist).
    fn access(&self, _req: RequestInfo, path: &Path, mask: u32) -> ResultEmpty {
        self.run_async(
            &format!("access({path:?}, mask={mask})"),
            move || async move {
                // TODO Should we implement access?
                Ok(())
            },
        )
    }

    /// Create and open a new file.
    ///
    /// * `parent`: path to the directory to create the file in.
    /// * `name`: name of the file to be created.
    /// * `mode`: the mode to set on the new file.
    /// * `flags`: flags like would be passed to `open`.
    ///
    /// Return a `CreatedEntry` (which contains the new file's attributes as well as a file handle
    /// -- see documentation on `open` for more info on that).
    fn create(
        &self,
        _req: RequestInfo,
        parent: &Path,
        name: &OsStr,
        mode: u32,
        flags: u32,
    ) -> ResultCreate {
        log::warn!("create({parent:?}, name={name:?}, mode={mode}, flags={flags})...unimplemented",);
        Err(libc::ENOSYS)
    }
}

fn convert_file_attrs(attrs: NodeAttrs) -> FileAttr {
    FileAttr {
        size: attrs.num_bytes.into(),
        blocks: attrs.blocks,
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
    }
}

fn convert_node_kind(kind: NodeKind) -> fuse_mt::FileType {
    match kind {
        NodeKind::File => fuse_mt::FileType::RegularFile,
        NodeKind::Dir => fuse_mt::FileType::Directory,
        NodeKind::Symlink => fuse_mt::FileType::Symlink,
    }
}

fn convert_permission_bits(mode: Mode) -> u16 {
    let mode_bits: u32 = mode.into();
    // TODO Is 0o777 the right mask or do we need 0o7777?
    let perm_bits = mode_bits & 0o777;
    perm_bits as u16
}

fn convert_dir_entries(entries: Vec<DirEntry>) -> Vec<fuse_mt::DirectoryEntry> {
    entries
        .into_iter()
        .map(|entry| fuse_mt::DirectoryEntry {
            name: entry.name.into(), // TODO Is into() the best way to convert from String to OsString?
            kind: convert_node_kind(entry.kind),
        })
        .collect()
}
