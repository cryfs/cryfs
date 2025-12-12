use cryfs_utils::safe_panic;
use fuse_mt::{
    CallbackResult, CreatedEntry, FileAttr, FilesystemMT, RequestInfo, ResultCreate, ResultData,
    ResultEmpty, ResultEntry, ResultOpen, ResultReaddir, ResultSlice, ResultStatfs, ResultWrite,
    ResultXattr, Xattr,
};
use std::ffi::OsStr;
use std::fmt::Debug;
use std::num::NonZeroU64;
use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{RwLockReadGuard, RwLockWriteGuard};

use crate::common::{
    Callback, DirEntryOrReference, FileHandle, FsError, FsResult, Gid, Mode, NodeAttrs, NodeKind,
    NumBytes, OpenInFlags, OpenOutFlags, Statfs, Uid,
};
use crate::high_level_api::AsyncFilesystem;
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    path::{AbsolutePath, AbsolutePathBuf, PathComponent},
};

// (all these TODOs apply to here and to the fuser backend)
// TODO Make sure each function checks the preconditions on its parameters, e.g. paths must be absolute
// TODO Check which of the logging statements parameters actually need :? formatting
// TODO Decide for logging whether we want parameters in parentheses or not, currently it's inconsistent
// TODO Go through fuse documentation and syscall manpages to check for behavior and possible error codes, make sure we handle all of them
// TODO We don't need the multithreading from fuse_mt, it's probably better to use fuser instead.
// TODO Which operations are supposed to follow symlinks, which ones aren't? Make sure we handle that correctly. Does fuse automatically deref symlinks before calling us?
// TODO https://www.cs.hmc.edu/~geoff/classes/hmc.cs135.201001/homework/fuse/fuse_doc.html#function-purposes :
//  - "Set flag_nullpath_ok nonzero if your code can accept a NULL path argument (because it gets file information from fi->fh) for the following operations: fgetattr, flush, fsync, fsyncdir, ftruncate, lock, read, readdir, release, releasedir, and write. This will allow FUSE to run more efficiently."
//  - Check function documentation and corner cases are as I expect them to be

pub struct BackendAdapter<Fs>
where
    Fs: AsyncFilesystem + AsyncDrop<Error = FsError> + Debug + Send + Sync + 'static,
{
    // TODO RwLock is only needed for async drop. Can we remove it?
    fs: Arc<tokio::sync::RwLock<AsyncDropGuard<Fs>>>,

    runtime: tokio::runtime::Handle,
}

impl<Fs> Debug for BackendAdapter<Fs>
where
    Fs: AsyncFilesystem + AsyncDrop<Error = FsError> + Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BackendAdapter")
            .field("fs", &self.fs)
            .finish()
    }
}

impl<Fs> BackendAdapter<Fs>
where
    Fs: AsyncFilesystem + AsyncDrop<Error = FsError> + Debug + Send + Sync + 'static,
{
    pub fn new(fs: AsyncDropGuard<Fs>, runtime: tokio::runtime::Handle) -> Self {
        Self {
            fs: Arc::new(tokio::sync::RwLock::new(fs)),
            runtime,
        }
    }

    pub(super) fn internal_arc(&self) -> Arc<tokio::sync::RwLock<AsyncDropGuard<Fs>>> {
        Arc::clone(&self.fs)
    }

    fn run_async<R: Debug>(
        &self,
        log_msg: &str,
        func: impl AsyncFnOnce() -> FsResult<R>,
    ) -> Result<R, libc::c_int> {
        // TODO Is it ok to call block_on concurrently for multiple fs operations? Probably not.
        self.runtime.block_on(async move {
            log::info!("{}...", log_msg);
            let result = func().await;
            match result {
                Ok(ok) => {
                    log::info!("{log_msg}...success: {ok:?}");
                    Ok(ok)
                }
                Err(err) => {
                    log::info!("{log_msg}...failed: {err:?}");
                    Err(err.system_error_code())
                }
            }
        })
    }

    async fn fs(&self) -> FsResult<RwLockReadGuard<'_, AsyncDropGuard<Fs>>> {
        let fs = self.fs.read().await;
        if fs.is_dropped() {
            // Gracefully handle if [Self::destroy] was already called. This can happen in corner cases where
            // a file held open and closed after the file system is already unmounted.
            // We can't really handle it well or honor those operations,
            // but at least we can avoid a panic.
            log::error!(
                "Received a file system operation after destroy() terminated the file system"
            );
            return Err(FsError::FilesystemDestroyed);
        }
        Ok(fs)
    }

    async fn fs_mut(&self) -> FsResult<RwLockWriteGuard<'_, AsyncDropGuard<Fs>>> {
        let fs = self.fs.write().await;
        if fs.is_dropped() {
            // Gracefully handle if [Self::destroy] was already called. This can happen in corner cases where
            // a file held open and closed after the file system is already unmounted.
            // We can't really handle it well or honor those operations,
            // but at least we can avoid a panic.
            log::error!(
                "Received a file system operation after destroy() terminated the file system"
            );
            return Err(FsError::FilesystemDestroyed);
        }
        Ok(fs)
    }
}

impl<Fs> FilesystemMT for BackendAdapter<Fs>
where
    Fs: AsyncFilesystem + AsyncDrop<Error = FsError> + Debug + Send + Sync + 'static,
{
    fn init(&self, req: RequestInfo) -> ResultEmpty {
        self.run_async(&format!("init"), async move || {
            let fs = self.fs().await?;
            fs.init(req.into()).await?;
            Ok(())
        })
    }

    fn destroy(&self) {
        self.run_async(&format!("destroy"), async move || {
            let mut fs = self.fs_mut().await?;
            fs.destroy().await;
            fs.async_drop().await?;
            Ok(())
        })
        .expect("failed to drop file system");

        // TODO Is there a way to do the above without a call to expect()?
    }

    fn getattr(&self, req: RequestInfo, path: &Path, fh: Option<u64>) -> ResultEntry {
        self.run_async(&format!("getattr ({path:?}, fh={fh:?})"), async move || {
            let path = parse_absolute_path(path)?;
            let response = self
                .fs()
                .await?
                .getattr(req.into(), path, fh.into_fh()?)
                .await?;
            Ok((response.ttl, convert_node_attrs(response.attrs)))
        })
    }

    fn chmod(&self, req: RequestInfo, path: &Path, fh: Option<u64>, mode: u32) -> ResultEmpty {
        self.run_async(
            &format!("chmod({path:?}, fh={fh:?}, mode={mode})"),
            async move || {
                let path = parse_absolute_path(path)?;
                self.fs()
                    .await?
                    .chmod(req.into(), path, fh.into_fh()?, Mode::from(mode))
                    .await
            },
        )
    }

    fn chown(
        &self,
        req: RequestInfo,
        path: &Path,
        fh: Option<u64>,
        uid: Option<u32>,
        gid: Option<u32>,
    ) -> ResultEmpty {
        self.run_async(
            &format!("chown({path:?}, fh={fh:?}, uid={uid:?}, gid={gid:?})"),
            async move || {
                let path = parse_absolute_path(path)?;
                self.fs()
                    .await?
                    .chown(
                        req.into(),
                        path,
                        fh.into_fh()?,
                        uid.into_uid(),
                        gid.into_gid(),
                    )
                    .await
            },
        )
    }

    fn truncate(&self, req: RequestInfo, path: &Path, fh: Option<u64>, size: u64) -> ResultEmpty {
        self.run_async(
            &format!("truncate({path:?}, fh={fh:?}, size={size})"),
            async move || {
                let path = parse_absolute_path(path)?;
                self.fs()
                    .await?
                    .truncate(req.into(), path, fh.into_fh()?, NumBytes::from(size))
                    .await
            },
        )
    }

    fn utimens(
        &self,
        req: RequestInfo,
        path: &Path,
        fh: Option<u64>,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
    ) -> ResultEmpty {
        self.run_async(
            &format!("utimens({path:?}, fh={fh:?}, atime={atime:?}, mtime={mtime:?})"),
            async move || {
                let path = parse_absolute_path(path)?;
                self.fs()
                    .await?
                    .utimens(req.into(), path, fh.into_fh()?, atime, mtime)
                    .await
            },
        )
    }

    /// Set timestamps of a filesystem entry (with extra options only used on MacOS).
    fn utimens_macos(
        &self,
        req: RequestInfo,
        path: &Path,
        fh: Option<u64>,
        crtime: Option<SystemTime>,
        chgtime: Option<SystemTime>,
        bkuptime: Option<SystemTime>,
        flags: Option<u32>,
    ) -> ResultEmpty {
        self.run_async(&format!("utimens({path:?}, fh={fh:?}, crtime={crtime:?}, chgtime={chgtime:?}, bkuptime={bkuptime:?}, flags={flags:?})"), async move ||{
            let path = parse_absolute_path(path)?;
            self.fs().await?.utimens_macos(req.into(), path, fh.into_fh()?, crtime, chgtime, bkuptime, flags).await
        })
    }

    fn readlink(&self, req: RequestInfo, path: &Path) -> ResultData {
        self.run_async(&format!("readlink({path:?})"), async move || {
            let path = parse_absolute_path(path)?;
            let target = self.fs().await?.readlink(req.into(), path).await?;
            Ok(target.into_bytes())
        })
    }

    fn mknod(
        &self,
        req: RequestInfo,
        parent: &Path,
        name: &OsStr,
        mode: u32,
        rdev: u32,
    ) -> ResultEntry {
        self.run_async(
            &format!("mknod({parent:?}, name={name:?}, mode={mode}, rdev={rdev})"),
            async move || {
                let path = parse_absolute_path_with_last_component(parent, name)?;
                let response = self
                    .fs()
                    .await?
                    .mknod(req.into(), &path, Mode::from(mode), rdev)
                    .await?;
                Ok((response.ttl, convert_node_attrs(response.attrs)))
            },
        )
    }

    fn mkdir(&self, req: RequestInfo, parent: &Path, name: &OsStr, mode: u32) -> ResultEntry {
        // TODO A comment in our C++ code base said that DokanY seems to call mkdir('/') and had code ignoring that. Do we still need that?

        let mode = Mode::from(mode).add_dir_flag();
        // TODO Assert that file/symlink flags aren't set
        self.run_async(
            &format!("mkdir({parent:?}, name={name:?}, mode={mode})"),
            async move || {
                let path = parse_absolute_path_with_last_component(parent, name)?;
                let response = self.fs().await?.mkdir(req.into(), &path, mode).await?;
                Ok((response.ttl, convert_node_attrs(response.attrs)))
            },
        )
    }

    fn unlink(&self, req: RequestInfo, parent: &Path, name: &OsStr) -> ResultEmpty {
        self.run_async(
            &format!("unlink({parent:?}, name={name:?})"),
            async move || {
                let path = parse_absolute_path_with_last_component(parent, name)?;
                self.fs().await?.unlink(req.into(), &path).await
            },
        )
    }

    fn rmdir(&self, req: RequestInfo, parent: &Path, name: &OsStr) -> ResultEmpty {
        self.run_async(
            &format!("rmdir({parent:?}, name={name:?})"),
            async move || {
                let path = parse_absolute_path_with_last_component(parent, name)?;
                self.fs().await?.rmdir(req.into(), &path).await
            },
        )
    }

    fn symlink(&self, req: RequestInfo, parent: &Path, name: &OsStr, target: &Path) -> ResultEntry {
        self.run_async(
            &format!("symlink({parent:?}, parent={parent:?} name={name:?}, target={target:?})"),
            async move || {
                let path = parse_absolute_path_with_last_component(parent, name)?;
                // TODO Use custom path type for target than can represent absolute-or-relative paths and enforces its invariants,
                //      similar to how we have an `AbsolutePath` type. Then we won't need these manual checks here anymore.
                let target = target.to_str().ok_or_else(|| {
                    log::warn!("Symlink target is not utf-8");
                    FsError::InvalidPath
                })?;
                let response = self.fs().await?.symlink(req.into(), &path, target).await?;
                Ok((response.ttl, convert_node_attrs(response.attrs)))
            },
        )
    }

    fn rename(
        &self,
        req: RequestInfo,
        oldparent: &Path,
        oldname: &OsStr,
        newparent: &Path,
        newname: &OsStr,
    ) -> ResultEmpty {
        self.run_async(
            &format!(
                "rename(oldparent={oldparent:?}, oldname={oldname:?}, newparent={newparent:?}, newname={newname:?})"
            ),
            async move || {
                let oldpath = parse_absolute_path_with_last_component(oldparent, oldname)?;
                let newpath = parse_absolute_path_with_last_component(newparent, newname)?;
                self.fs().await?.rename(
                    req.into(),
                    &oldpath,
                    &newpath,
                ).await
            },
        )
    }

    fn link(
        &self,
        req: RequestInfo,
        oldpath: &Path,
        newparent: &Path,
        newname: &OsStr,
    ) -> ResultEntry {
        self.run_async(
            &format!("link(oldpath={oldpath:?}, newparent={newparent:?}, newname={newname:?})"),
            async move || {
                let oldpath = parse_absolute_path(oldpath)?;
                let newpath = parse_absolute_path_with_last_component(newparent, newname)?;
                let response = self
                    .fs()
                    .await?
                    .link(req.into(), &oldpath, &newpath)
                    .await?;
                Ok((response.ttl, convert_node_attrs(response.attrs)))
            },
        )
    }

    fn open(&self, req: RequestInfo, path: &Path, flags: u32) -> ResultOpen {
        self.run_async(&format!("open({path:?}, flags={flags})"), async move || {
            let path = parse_absolute_path(path)?;
            let response = self
                .fs()
                .await?
                .open(req.into(), path, parse_open_in_flags(flags))
                .await?;
            let flags = convert_open_out_flags(response.flags.into());
            Ok((NonZeroU64::from(response.fh).get(), flags))
        })
    }

    fn read(
        &self,
        req: RequestInfo,
        path: &Path,
        fh: u64,
        offset: u64,
        size: u32,
        callback: impl FnOnce(ResultSlice<'_>) -> CallbackResult,
    ) -> CallbackResult {
        // TODO Is it ok to call block_on concurrently for multiple fs operations? Probably not.
        self.runtime.block_on(async move {
            let log_msg = format!("read({path:?}, fh={fh}, offset={offset}, size={size})");
            log::info!("{log_msg}...");
            match parse_file_handle(fh) {
                Err(err) => callback(Err(err.system_error_code())),
                Ok(fh) => match parse_absolute_path(path) {
                    Err(err) => callback(Err(err.system_error_code())),
                    Ok(path) => {
                        let mut result = None;
                        match self.fs().await {
                            Err(err) => callback(Err(err.system_error_code())),
                            Ok(fs) => {
                                fs.read(
                                    req.into(),
                                    path,
                                    fh,
                                    NumBytes::from(offset),
                                    NumBytes::from(u64::from(size)),
                                    DataCallback::new(log_msg, callback, &mut result),
                                )
                                .await;
                                result.expect("callback not called")
                            }
                        }
                    }
                },
            }
        })
    }

    fn write(
        &self,
        req: RequestInfo,
        path: &Path,
        fh: u64,
        offset: u64,
        data: Vec<u8>,
        flags: u32,
    ) -> ResultWrite {
        self.run_async(
            &format!(
                "write({path:?}, fh={fh}, offset={offset}, data=[{data_len} bytes], flags={flags})",
                data_len = data.len(),
            ),
            async move || {
                let fh = parse_file_handle(fh)?;
                let path = parse_absolute_path(path)?;
                let response = self
                    .fs()
                    .await?
                    .write(req.into(), path, fh, NumBytes::from(offset), data, flags)
                    .await?;
                // TODO No unwrap
                Ok(u32::try_from(u64::from(response)).unwrap())
            },
        )
    }

    fn flush(&self, req: RequestInfo, path: &Path, fh: u64, lock_owner: u64) -> ResultEmpty {
        self.run_async(
            &format!("flush({path:?}, fh={fh}, lock_owner={lock_owner})"),
            async move || {
                let fh = parse_file_handle(fh)?;
                let path = parse_absolute_path(path)?;
                self.fs()
                    .await?
                    .flush(req.into(), path, fh, lock_owner)
                    .await
            },
        )
    }

    fn release(
        &self,
        req: RequestInfo,
        path: &Path,
        fh: u64,
        flags: u32,
        lock_owner: u64,
        flush: bool,
    ) -> ResultEmpty {
        self.run_async(
            &format!(
                "release({path:?}, fh={fh}, flags={flags}, lock_owner={lock_owner}, flush={flush})"
            ),
            async move || {
                let fh = parse_file_handle(fh)?;
                let path = parse_absolute_path(path)?;
                self.fs()
                    .await?
                    .release(
                        req.into(),
                        path,
                        fh,
                        parse_open_in_flags(flags),
                        lock_owner,
                        flush,
                    )
                    .await
            },
        )
    }

    fn fsync(&self, req: RequestInfo, path: &Path, fh: u64, datasync: bool) -> ResultEmpty {
        self.run_async(
            &format!("fsync({path:?}, fh={fh}, datasync={datasync})"),
            async move || {
                let fh = parse_file_handle(fh)?;
                let path = parse_absolute_path(path)?;
                self.fs().await?.fsync(req.into(), path, fh, datasync).await
            },
        )
    }

    fn opendir(&self, req: RequestInfo, path: &Path, flags: u32) -> ResultOpen {
        let flags = parse_open_in_flags(flags);
        self.run_async(
            &format!("opendir({path:?}, flags={flags})"),
            async move || {
                let path = parse_absolute_path(path)?;
                let response = self.fs().await?.opendir(req.into(), path, flags).await?;
                Ok((
                    NonZeroU64::from(response.fh).get(),
                    convert_open_out_flags(response.flags),
                ))
            },
        )
    }

    fn readdir(&self, req: RequestInfo, path: &Path, fh: u64) -> ResultReaddir {
        self.run_async(&format!("readdir({path:?}, fh={fh})"), async move || {
            let fh = parse_file_handle(fh)?;
            let path = parse_absolute_path(path)?;
            let fs = self.fs().await?;
            let entries = fs.readdir(req.into(), path, fh).await?;
            let entries = convert_dir_entries(entries).collect::<Vec<_>>();
            Ok(entries)
        })
    }

    fn releasedir(&self, req: RequestInfo, path: &Path, fh: u64, flags: u32) -> ResultEmpty {
        self.run_async(
            &format!("releasedir({path:?}, fh={fh}, flags={flags})"),
            async move || {
                let fh = parse_file_handle(fh)?;
                let path = parse_absolute_path(path)?;
                self.fs()
                    .await?
                    .releasedir(req.into(), path, fh, parse_open_in_flags(flags))
                    .await
            },
        )
    }

    fn fsyncdir(&self, req: RequestInfo, path: &Path, fh: u64, datasync: bool) -> ResultEmpty {
        self.run_async(
            &format!("fsyncdir({path:?}, fh={fh}, datasync={datasync})"),
            async move || {
                let fh = parse_file_handle(fh)?;
                let path = parse_absolute_path(path)?;
                self.fs()
                    .await?
                    .fsyncdir(req.into(), path, fh, datasync)
                    .await
            },
        )
    }

    fn statfs(&self, req: RequestInfo, path: &Path) -> ResultStatfs {
        self.run_async(&format!("statfs({path:?})"), async move || {
            let path = parse_absolute_path(path)?;
            let response = self.fs().await?.statfs(req.into(), path).await?;
            Ok(convert_statfs(response))
        })
    }

    fn setxattr(
        &self,
        req: RequestInfo,
        path: &Path,
        name: &OsStr,
        value: &[u8],
        flags: u32,
        position: u32,
    ) -> ResultEmpty {
        self.run_async(
            &format!(
                "setxattr({path:?}, name={name:?}, value=[{value_len} bytes], flags={flags}, position={position})",
                value_len = value.len(),
            ),
            async move || {
                let path = parse_absolute_path(path)?;
                let name = parse_xattr_name(name)?;
                self.fs().await?.setxattr(
                    req.into(),
                    path,
                    name,
                    value,
                    flags,
                    NumBytes::from(u64::from(position)),
                ).await
            },
        )
    }

    fn getxattr(&self, req: RequestInfo, path: &Path, name: &OsStr, size: u32) -> ResultXattr {
        self.run_async(
            &format!("getxattr({path:?}, name={name:?}, size={size})"),
            async move || {
                let req = req.into();
                let path = parse_absolute_path(path)?;
                let name = parse_xattr_name(name)?;
                // fuse_mt wants us to return Xattr::Size if the `size` parameter is zero, and the data otherwise.
                if 0 == size {
                    let response = self.fs().await?.getxattr_numbytes(req, path, &name).await?;
                    // TODO No unwrap
                    Ok(Xattr::Size(u32::try_from(u64::from(response)).unwrap()))
                } else {
                    let response = self
                        .fs()
                        .await?
                        .getxattr_data(req, path, &name, NumBytes::from(u64::from(size)))
                        .await?;
                    Ok(Xattr::Data(response))
                }
            },
        )
    }

    fn listxattr(&self, req: RequestInfo, path: &Path, size: u32) -> ResultXattr {
        self.run_async(
            &format!("getxattr({path:?}, size={size})"),
            async move || {
                let req = req.into();
                let path = parse_absolute_path(path)?;
                // fuse_mt wants us to return Xattr::Size if the `size` parameter is zero, and the data otherwise.
                if 0 == size {
                    let response = self.fs().await?.listxattr_numbytes(req, path).await?;
                    // TODO No unwrap
                    Ok(Xattr::Size(u32::try_from(u64::from(response)).unwrap()))
                } else {
                    let response = self
                        .fs()
                        .await?
                        .listxattr_data(req, path, NumBytes::from(u64::from(size)))
                        .await?;
                    Ok(Xattr::Data(response))
                }
            },
        )
    }

    fn removexattr(&self, req: RequestInfo, path: &Path, name: &OsStr) -> ResultEmpty {
        self.run_async(
            &format!("removexattr({path:?}, name={name:?})"),
            async move || {
                let path = parse_absolute_path(path)?;
                let name = parse_xattr_name(name)?;
                self.fs().await?.removexattr(req.into(), path, name).await
            },
        )
    }

    fn access(&self, req: RequestInfo, path: &Path, mask: u32) -> ResultEmpty {
        self.run_async(&format!("access({path:?}, mask={mask})"), async move || {
            let path = parse_absolute_path(path)?;
            self.fs().await?.access(req.into(), path, mask).await
        })
    }

    fn create(
        &self,
        req: RequestInfo,
        parent: &Path,
        name: &OsStr,
        mode: u32,
        flags: u32,
    ) -> ResultCreate {
        let flags = parse_open_in_flags(flags);
        let mode = Mode::from(mode).add_file_flag();
        // TODO Assert that dir/symlink flags aren't set
        self.run_async(
            &format!("create({parent:?}, name={name:?}, mode={mode:?}, flags={flags})"),
            async move || {
                let path = parse_absolute_path_with_last_component(parent, name)?;
                let response = self
                    .fs()
                    .await?
                    .create(req.into(), &path, mode, flags)
                    .await?;
                let flags = convert_open_out_flags(response.flags);
                Ok(CreatedEntry {
                    ttl: response.ttl,
                    attr: convert_node_attrs(response.attrs),
                    fh: NonZeroU64::from(response.fh).get(),
                    flags,
                })
            },
        )
    }
}

fn convert_node_attrs(attrs: NodeAttrs) -> FileAttr {
    let size: u64 = attrs.num_bytes.into();
    FileAttr {
        size,
        blocks: attrs.num_blocks.unwrap_or(size.div_ceil(512)),
        atime: attrs.atime,
        mtime: attrs.mtime,
        ctime: attrs.ctime,
        crtime: attrs.ctime, // TODO actually store and compute crtime
        kind: convert_node_kind(attrs.mode.node_kind()),
        perm: convert_permission_bits(attrs.mode),
        nlink: attrs.nlink,
        uid: attrs.uid.into(),
        gid: attrs.gid.into(),
        // Device ID (if special file)
        rdev: 0, // TODO What to do about this?
        // Flags (macOS only; see chflags(2))
        flags: 0, // TODO What to do about this?
    }
}

impl<Fs> Drop for BackendAdapter<Fs>
where
    Fs: AsyncFilesystem + AsyncDrop<Error = FsError> + Debug + Send + Sync + 'static,
{
    fn drop(&mut self) {
        if !self.fs.blocking_read().is_dropped() {
            safe_panic!("BackendAdapter dropped without calling destroy() first");
        }
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

fn convert_dir_entries(
    entries: impl Iterator<Item = DirEntryOrReference>,
) -> impl Iterator<Item = fuse_mt::DirectoryEntry> {
    entries.map(|entry| match entry {
        DirEntryOrReference::Entry(entry) => fuse_mt::DirectoryEntry {
            name: std::ffi::OsString::from(String::from(entry.name)),
            kind: convert_node_kind(entry.kind),
        },
        DirEntryOrReference::SelfReference => fuse_mt::DirectoryEntry {
            name: std::ffi::OsString::from(".".to_string()),
            kind: fuse_mt::FileType::Directory,
        },
        DirEntryOrReference::ParentReference => fuse_mt::DirectoryEntry {
            name: std::ffi::OsString::from("..".to_string()),
            kind: fuse_mt::FileType::Directory,
        },
    })
}

fn parse_absolute_path(path: &Path) -> FsResult<&AbsolutePath> {
    path.try_into().map_err(|err| {
        log::warn!("Invalid path '{path:?}': {err}");
        FsError::InvalidPath
    })
}

fn parse_path_component(component: &OsStr) -> FsResult<&PathComponent> {
    component.try_into().map_err(|err| {
        log::warn!("Invalid path component '{component:?}': {err}");
        FsError::InvalidPath
    })
}

fn parse_absolute_path_with_last_component(
    parent: &Path,
    last_component: &OsStr,
) -> FsResult<AbsolutePathBuf> {
    let parent = parse_absolute_path(parent)?.to_owned();
    Ok(parent.push(parse_path_component(last_component)?))
}

fn parse_xattr_name(name: &OsStr) -> FsResult<&str> {
    // TODO We should probably introduce a custom wrapper type for XattrName, similar to how we have a PathComponent type, and enforce invariants there.
    name.to_str().ok_or_else(|| {
        log::warn!("xattr name is not valid UTF-8");
        // TODO Better error return type
        FsError::UnknownError
    })
}

fn parse_open_in_flags(flags: u32) -> OpenInFlags {
    // TODO flags should be i32 and is in fuser, but fuse_mt accidentally converts it to u32. Undo that.
    let flags = flags as i32;
    // TODO Is this the right way to parse openflags? Are there other flags than just Read+Write?
    //      https://docs.rs/fuser/latest/fuser/trait.Filesystem.html#method.open seems to suggest so.
    // TODO This is duplicate between fuser and fuse_mt
    match flags & libc::O_ACCMODE {
        libc::O_RDONLY => OpenInFlags::Read,
        libc::O_WRONLY => OpenInFlags::Write,
        libc::O_RDWR => OpenInFlags::ReadWrite,
        _ => panic!("invalid flags: {flags}"),
    }
}

fn parse_file_handle(fh: u64) -> FsResult<FileHandle> {
    FileHandle::try_from(fh).ok_or_else(|| {
        log::error!("Kernel gave us zero as a file handle");
        FsError::InvalidOperation
    })
}

fn convert_open_out_flags(flags: OpenOutFlags) -> u32 {
    // TODO This is duplicate between fuser and fuse_mt
    // TODO Not implemented yet
    let OpenOutFlags {} = flags;
    0
}

fn convert_statfs(statfs: Statfs) -> fuse_mt::Statfs {
    fuse_mt::Statfs {
        blocks: statfs.num_total_blocks,
        bfree: statfs.num_free_blocks,
        bavail: statfs.num_available_blocks,
        files: statfs.num_total_inodes,
        ffree: statfs.num_free_inodes,
        bsize: statfs.blocksize,
        namelen: statfs.max_filename_length,
        // TODO What is fragment size? Should it be different to blocksize?
        frsize: statfs.blocksize,
    }
}

impl From<fuse_mt::RequestInfo> for crate::common::RequestInfo {
    fn from(value: fuse_mt::RequestInfo) -> Self {
        Self {
            unique: value.unique,
            uid: Uid::from(value.uid),
            gid: Gid::from(value.gid),
            pid: value.pid,
        }
    }
}

trait IntoOptionFileHandle {
    fn into_fh(self) -> FsResult<Option<FileHandle>>;
}
impl IntoOptionFileHandle for Option<u64> {
    fn into_fh(self) -> FsResult<Option<FileHandle>> {
        self.map(|v| {
            NonZeroU64::new(v).map(FileHandle::from).ok_or_else(|| {
                log::error!("Kernel gave us zero as a file handle");
                FsError::InvalidOperation
            })
        })
        .transpose()
    }
}

trait IntoOptionUid {
    fn into_uid(self) -> Option<Uid>;
}
impl IntoOptionUid for Option<u32> {
    fn into_uid(self) -> Option<Uid> {
        self.map(Uid::from)
    }
}

trait IntoOptionGid {
    fn into_gid(self) -> Option<Gid>;
}
impl IntoOptionGid for Option<u32> {
    fn into_gid(self) -> Option<Gid> {
        self.map(Gid::from)
    }
}

struct DataCallback<'r, F>
where
    F: FnOnce(ResultSlice<'_>) -> CallbackResult,
{
    log_msg: String,
    callback: F,
    result: &'r mut Option<CallbackResult>,
}

impl<'r, F> DataCallback<'r, F>
where
    F: FnOnce(ResultSlice<'_>) -> CallbackResult,
{
    pub fn new(log_msg: String, callback: F, result: &'r mut Option<CallbackResult>) -> Self {
        Self {
            log_msg,
            callback,
            result,
        }
    }
}

impl<'a, 'r, F> Callback<FsResult<&'a [u8]>, ()> for DataCallback<'r, F>
where
    F: FnOnce(ResultSlice<'_>) -> CallbackResult,
{
    fn call(self, result: FsResult<&'a [u8]>) {
        match result {
            Ok(slice) => {
                *self.result = Some((self.callback)(Ok(slice)));
                log::info!("{}...success: [{} bytes]", self.log_msg, slice.len());
            }
            Err(err) => {
                *self.result = Some((self.callback)(Err(err.system_error_code())));
                log::info!("{}...failed: {err:?}", self.log_msg);
            }
        }
    }
}
