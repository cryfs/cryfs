use fuse_mt::{
    CallbackResult, CreatedEntry, FileAttr, FilesystemMT, RequestInfo, ResultCreate, ResultData,
    ResultEmpty, ResultEntry, ResultOpen, ResultReaddir, ResultSlice, ResultStatfs, ResultWrite,
    ResultXattr,
};
use std::borrow::Cow;
use std::ffi::OsStr;
use std::future::Future;
use std::os::unix::ffi::OsStringExt;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use crate::interface::{
    Device, Dir, DirEntry, File, FsError, FsResult, Node, NodeAttrs, OpenFile, Statfs, Symlink,
};
use crate::open_file_list::OpenFileList;
use crate::utils::{Gid, Mode, NodeKind, NumBytes, OpenFlags, Uid};

// TODO Make sure each function checks the preconditions on its parameters, e.g. paths must be absolute
// TODO Check which of the logging statements parameters actually need :? formatting
// TODO Decide for logging whether we want parameters in parentheses or not, currently it's inconsistent
// TODO Go through fuse documentation and syscall manpages to check for behavior and possible error codes
// TODO We don't need the multithreading from fuse_mt, it's probably better to use fuser instead.
// TODO This adapter currently adapts between multiple things. fuse_mt -> async interface -> rust_fs interface. Can we split that by having one adapter that only goes to an async version of fuse_mt/fuser and a second one that goes to rust_fs?
// TODO Which operations are supposed to follow symlinks, which ones aren't? Make sure we handle that correctly. Does fuse automatically deref symlinks before calling us?

enum MaybeInitializedFs<Fs: Device> {
    Uninitialized(Option<Box<dyn FnOnce(Uid, Gid) -> Fs + Send + Sync>>),
    Initialized(Fs),
}

impl<Fs: Device> MaybeInitializedFs<Fs> {
    pub fn initialize(&mut self, uid: Uid, gid: Gid) {
        match self {
            MaybeInitializedFs::Uninitialized(construct_fs) => {
                let construct_fs = construct_fs
                    .take()
                    .expect("MaybeInitializedFs::initialize() called twice");
                let fs = construct_fs(uid, gid);
                *self = MaybeInitializedFs::Initialized(fs);
            }
            MaybeInitializedFs::Initialized(_) => {
                panic!("MaybeInitializedFs::initialize() called twice");
            }
        }
    }

    pub fn get(&self) -> &Fs {
        match self {
            MaybeInitializedFs::Uninitialized(_) => {
                panic!("MaybeInitializedFs::get() called before initialize()");
            }
            MaybeInitializedFs::Initialized(fs) => fs,
        }
    }
}

pub struct FsAdapter<Fs: Device>
where
    // TODO Is this send+sync bound only needed because fuse_mt goes multi threaded or would it also be required for fuser?
    Fs::OpenFile: Send + Sync,
{
    // TODO We only need the Arc<RwLock<...>> because of initialization. Is there a better way to do that?
    fs: Arc<RwLock<MaybeInitializedFs<Fs>>>,

    runtime: tokio::runtime::Runtime,

    // TODO Can we improve concurrency by locking less in open_files and instead making OpenFileList concurrency safe somehow?
    open_files: RwLock<OpenFileList<Fs::OpenFile>>,
}

impl<Fs: Device> FsAdapter<Fs>
where
    // TODO Is this send+sync bound only needed because fuse_mt goes multi threaded or would it also be required for fuser?
    Fs::OpenFile: Send + Sync,
{
    pub fn new(fs: impl FnOnce(Uid, Gid) -> Fs + Send + Sync + 'static) -> Self {
        // TODO Runtime settings
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .thread_name("rustfs")
            .build()
            .unwrap();
        let open_files = Default::default();
        Self {
            fs: Arc::new(RwLock::new(MaybeInitializedFs::Uninitialized(Some(
                Box::new(fs),
            )))),
            runtime,
            open_files,
        }
    }

    fn run_async<R, F>(&self, log_msg: &str, func: impl FnOnce() -> F) -> Result<R, libc::c_int>
    where
        F: Future<Output = FsResult<R>>,
    {
        // TODO Is it ok to call block_on concurrently for multiple fs operations? Probably not.
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

impl<Fs: Device> FilesystemMT for FsAdapter<Fs>
where
    // TODO Is this send+sync bound only needed because fuse_mt goes multi threaded or would it also be required for fuser?
    Fs::OpenFile: Send + Sync,
{
    /// Called on mount, before any other function.
    fn init(&self, req: RequestInfo) -> ResultEmpty {
        log::info!("init");
        let uid = Uid::from(req.uid);
        let gid = Gid::from(req.gid);
        self.fs.write().unwrap().initialize(uid, gid);
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
            let attrs = if let Some(fh) = fh {
                let open_file_list = self.open_files.read().unwrap();
                let open_file = open_file_list.get(fh.into()).ok_or_else(|| {
                    log::error!("getattr: no open file with handle {}", u64::from(fh));
                    FsError::InvalidFileDescriptor { fh: u64::from(fh) }
                })?;
                open_file.getattr().await?
            } else {
                let node = self.fs.read().unwrap().get().load_node(path).await?;
                node.getattr().await?
            };
            // TODO What is the ttl here?
            let ttl = Duration::ZERO;
            Ok((ttl, convert_node_attrs(attrs)))
        })
    }

    // The following operations in the FUSE C API are all one kernel call: setattr
    // We split them out to match the C API's behavior.

    /// Change the mode of a filesystem entry.
    ///
    /// * `fh`: a file handle if this is called on an open file.
    /// * `mode`: the mode to change the file to.
    fn chmod(&self, _req: RequestInfo, path: &Path, fh: Option<u64>, mode: u32) -> ResultEmpty {
        self.run_async(
            &format!("chmod({path:?}, mode={mode})"),
            move || async move {
                let mode = Mode::from(mode);
                if let Some(fh) = fh {
                    let open_file_list = self.open_files.read().unwrap();
                    let open_file = open_file_list.get(fh.into()).ok_or_else(|| {
                        log::error!("chmod: no open file with handle {}", u64::from(fh));
                        FsError::InvalidFileDescriptor { fh: u64::from(fh) }
                    })?;
                    open_file.chmod(mode).await?
                } else {
                    let node = self.fs.read().unwrap().get().load_node(path).await?;
                    node.chmod(mode).await?;
                };
                Ok(())
            },
        )
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
        fh: Option<u64>,
        uid: Option<u32>,
        gid: Option<u32>,
    ) -> ResultEmpty {
        self.run_async(
            &format!("chown({path:?}, uid={uid:?}, gid={gid:?})"),
            move || async move {
                let uid = uid.map(Uid::from);
                let gid = gid.map(Gid::from);

                if let Some(fh) = fh {
                    let open_file_list = self.open_files.read().unwrap();
                    let open_file = open_file_list.get(fh.into()).ok_or_else(|| {
                        log::error!("chown: no open file with handle {}", u64::from(fh));
                        FsError::InvalidFileDescriptor { fh: u64::from(fh) }
                    })?;
                    open_file.chown(uid, gid).await?
                } else {
                    let node = self.fs.read().unwrap().get().load_node(path).await?;
                    node.chown(uid, gid).await?;
                }

                Ok(())
            },
        )
    }

    /// Set the length of a file.
    ///
    /// * `fh`: a file handle if this is called on an open file.
    /// * `size`: size in bytes to set as the file's length.
    fn truncate(&self, _req: RequestInfo, path: &Path, fh: Option<u64>, size: u64) -> ResultEmpty {
        let size = NumBytes::from(size);
        self.run_async(&format!("getattr {path:?}"), move || async move {
            if let Some(fh) = fh {
                let open_file_list = self.open_files.read().unwrap();
                let open_file = open_file_list.get(fh.into()).ok_or_else(|| {
                    log::error!("truncate: no open file with handle {}", u64::from(fh));
                    FsError::InvalidFileDescriptor { fh: u64::from(fh) }
                })?;
                open_file.truncate(size).await?
            } else {
                let file = self.fs.read().unwrap().get().load_file(path).await?;
                file.truncate(size).await?
            };
            Ok(())
        })
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
        fh: Option<u64>,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
    ) -> ResultEmpty {
        self.run_async(
            &format!("utimens({path:?}, atime={atime:?}, mtime={mtime:?})"),
            move || async move {
                if let Some(fh) = fh {
                    let open_file_list = self.open_files.read().unwrap();
                    let open_file = open_file_list.get(fh.into()).ok_or_else(|| {
                        log::error!("utimens: no open file with handle {}", u64::from(fh));
                        FsError::InvalidFileDescriptor { fh: u64::from(fh) }
                    })?;
                    open_file.utimens(atime, mtime).await?
                } else {
                    let node = self.fs.read().unwrap().get().load_node(path).await?;
                    node.utimens(atime, mtime).await?
                };
                Ok(())
            },
        )
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
        self.run_async(&format!("readlink({path:?})"), move || async move {
            let link = self.fs.read().unwrap().get().load_symlink(path).await?;
            // TODO is OsStr the best way to convert our path to the return value?
            Ok(link.target().await?.as_os_str().to_owned().into_vec())
        })
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
    fn mkdir(&self, req: RequestInfo, parent: &Path, name: &OsStr, mode: u32) -> ResultEntry {
        self.run_async(
            &format!("mkdir({parent:?}, name={name:?}, mode={mode})"),
            move || async move {
                let name = parse_node_name(name);
                let uid = Uid::from(req.uid);
                let gid = Gid::from(req.gid);
                let mode = Mode::from(mode);
                let parent_dir = self.fs.read().unwrap().get().load_dir(parent).await?;
                let new_dir_attrs = parent_dir.create_child_dir(&name, mode, uid, gid).await?;
                // TODO What is the ttl here?
                let ttl = Duration::ZERO;
                Ok((ttl, convert_node_attrs(new_dir_attrs)))
            },
        )
    }

    /// Remove a file.
    ///
    /// * `parent`: path to the directory containing the file to delete.
    /// * `name`: name of the file to delete.
    fn unlink(&self, _req: RequestInfo, parent: &Path, name: &OsStr) -> ResultEmpty {
        self.run_async(
            &format!("unlink({parent:?}, name={name:?})"),
            move || async move {
                let name = parse_node_name(name);
                let parent_dir = self.fs.read().unwrap().get().load_dir(parent).await?;
                parent_dir.remove_child_file_or_symlink(&name).await?;
                Ok(())
            },
        )
    }

    /// Remove a directory.
    ///
    /// * `parent`: path to the directory containing the directory to delete.
    /// * `name`: name of the directory to delete.
    fn rmdir(&self, _req: RequestInfo, parent: &Path, name: &OsStr) -> ResultEmpty {
        self.run_async(
            &format!("rmdir({parent:?}, name={name:?})"),
            move || async move {
                let name = parse_node_name(name);
                let parent_dir = self.fs.read().unwrap().get().load_dir(parent).await?;
                parent_dir.remove_child_dir(&name).await?;
                Ok(())
            },
        )
    }

    /// Create a symbolic link.
    ///
    /// * `parent`: path to the directory to make the link in.
    /// * `name`: name of the symbolic link.
    /// * `target`: path (may be relative or absolute) to the target of the link.
    fn symlink(&self, req: RequestInfo, parent: &Path, name: &OsStr, target: &Path) -> ResultEntry {
        self.run_async(
            &format!("symlink({parent:?}, parent={parent:?} name={name:?}, target={target:?})"),
            move || async move {
                let name = parse_node_name(name);
                let uid = Uid::from(req.uid);
                let gid = Gid::from(req.gid);
                let parent_dir = self.fs.read().unwrap().get().load_dir(parent).await?;
                let new_symlink_attrs = parent_dir
                    .create_child_symlink(&name, target, uid, gid)
                    .await?;
                // TODO What is the ttl here?
                let ttl = Duration::ZERO;
                Ok((ttl, convert_node_attrs(new_symlink_attrs)))
            },
        )
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
        oldparent: &Path,
        oldname: &OsStr,
        newparent: &Path,
        newname: &OsStr,
    ) -> ResultEmpty {
        self.run_async(
            &format!(
                "rename(oldparent={oldparent:?}, oldname={oldname:?}, newparent={newparent:?}, newname={newname:?})"
            ),
            move || async move {
                let oldname = parse_node_name(oldname);
                let newname = parse_node_name(newname);
                let old_parent_dir = self.fs.read().unwrap().get().load_dir(oldparent).await?;
                let new_path = newparent.join(&*newname);
                // TODO Should rename overwrite a potentially already existing target or not? Make sure we handle that the right way.
                old_parent_dir
                    .rename_child(&oldname, &new_path)
                    .await?;
                Ok(())
            },
        )
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
        // TODO flags should be i32 and is in fuser, but fuse_mt accidentally converts it to u32. Undo that.
        let flags = flags as i32;
        self.run_async(
            &format!("open({path:?}, flags={flags})"),
            move || async move {
                let file = self.fs.read().unwrap().get().load_file(path).await?;
                let open_file = file.open(parse_openflags(flags)).await?;
                let fh = self.open_files.write().unwrap().add(open_file);
                // TODO Do we need to change flags or is it ok to just return the flags passed in? If it's ok, then why do we have to return them?
                Ok((fh.into(), flags as u32))
            },
        )
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
        self.run_async(
            &format!("read({path:?}, fh={fh}, offset={offset}, size={size})"),
            move || async move {
                let offset = NumBytes::from(offset);
                let size = NumBytes::from(u64::from(size));
                let open_file_list = self.open_files.read().unwrap();
                let open_file = open_file_list.get(fh.into()).ok_or_else(|| {
                    log::error!("read: no open file with handle {}", u64::from(fh));
                    FsError::InvalidFileDescriptor { fh: u64::from(fh) }
                });
                let callback_result = match open_file {
                    Ok(open_file) => {
                        let data = open_file
                            .read(offset, size).await;
                        match data {
                            Ok(data) => callback(Ok(data.as_ref())),
                            Err(err) => callback(Err(err.system_error_code())),
                        }
                    }
                    Err(err) => callback(Err(err.system_error_code())),
                };
                Ok(callback_result)
            },
        // TODO Having to .expect() here is weird. Should we instead write a variant of `run_async` that doesn't map errors and just returns T? That would be simpler.
        ).expect("We're not throwing any errors in the async block, so this should never fail. Errors are instead being passed to the callback function.")
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
        // TODO What is the `flags` parameter for?
        self.run_async(
            &format!("write({path:?}, fh={fh}, offset={offset}, data={data:?}, flags={flags})"),
            move || async move {
                let data_len = data.len();
                let data = data.into();
                let offset = NumBytes::from(offset);
                let open_file_list = self.open_files.read().unwrap();
                let open_file = open_file_list.get(fh.into()).ok_or_else(|| {
                    log::error!("write: no open file with handle {}", u64::from(fh));
                    FsError::InvalidFileDescriptor { fh: u64::from(fh) }
                })?;
                open_file.write(offset, data).await?;
                Ok(u32::try_from(data_len).unwrap())
            },
        )
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
    fn flush(&self, _req: RequestInfo, path: &Path, fh: u64, _lock_owner: u64) -> ResultEmpty {
        self.run_async(&format!("flush({path:?}, fh={fh})"), move || async move {
            let open_file_list = self.open_files.read().unwrap();
            let open_file = open_file_list.get(fh.into()).ok_or_else(|| {
                log::error!("flush: no open file with handle {}", u64::from(fh));
                FsError::InvalidFileDescriptor { fh: u64::from(fh) }
            })?;
            open_file.flush().await?;
            Ok(())
        })
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
        self.run_async(
            &format!(
                "release({path:?}, fh={fh}, flags={flags}, lock_owner={lock_owner}, flush={flush})"
            ),
            move || async move {
                self.open_files.write().unwrap().remove(fh.into());
                Ok(())
            },
        )
    }

    /// Write out any pending changes of a file.
    ///
    /// When this returns, data should be written to persistent storage.
    ///
    /// * `path`: path to the file.
    /// * `fh`: file handle returned from the `open` call.
    /// * `datasync`: if `false`, also write metadata, otherwise just write file data.
    fn fsync(&self, _req: RequestInfo, path: &Path, fh: u64, datasync: bool) -> ResultEmpty {
        self.run_async(
            &format!("fsync({path:?}, fh={fh}, datasync={datasync})"),
            move || async move {
                let open_file_list = self.open_files.read().unwrap();
                let open_file = open_file_list.get(fh.into()).ok_or_else(|| {
                    log::error!("fsync: no open file with handle {}", u64::from(fh));
                    FsError::InvalidFileDescriptor { fh: u64::from(fh) }
                })?;
                open_file.fsync(datasync).await?;
                Ok(())
            },
        )
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
            let dir = self.fs.read().unwrap().get().load_dir(path).await?;
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
        log::warn!("statfs({path:?})...");
        self.run_async(&format!("statfs({path:?})"), move || async move {
            let stat = self.fs.read().unwrap().get().statfs().await?;
            Ok(convert_statfs(stat))
        })
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
        req: RequestInfo,
        parent: &Path,
        name: &OsStr,
        mode: u32,
        flags: u32,
    ) -> ResultCreate {
        // TODO flags should be i32 and is in fuser, but fuse_mt accidentally converts it to u32. Undo that.
        let flags = flags as i32;
        self.run_async(
            &format!("create({parent:?}, name={name:?}, mode={mode}, flags={flags})"),
            move || async move {
                let name = parse_node_name(name);
                let uid = Uid::from(req.uid);
                let gid = Gid::from(req.gid);
                let mode = Mode::from(mode);
                let parent_dir = self.fs.read().unwrap().get().load_dir(parent).await?;
                let (file_attrs, open_file) = parent_dir
                    .create_and_open_file(&name, mode, uid, gid)
                    .await?;
                let fh = self.open_files.write().unwrap().add(open_file);
                Ok(CreatedEntry {
                    // TODO What is ttl here?
                    ttl: Duration::ZERO,
                    attr: convert_node_attrs(file_attrs),
                    fh: fh.into(),
                    // TODO Do we need to change flags or is it ok to just return the flags passed in? If it's ok, then why do we have to return them?
                    flags: flags as u32,
                })
            },
        )
    }
}

fn convert_node_attrs(attrs: NodeAttrs) -> FileAttr {
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

fn parse_node_name(name: &OsStr) -> Cow<'_, str> {
    let name = name.to_string_lossy(); // TODO Is to_string_lossy the best way to convert from OsString to String?
    assert!(!name.contains('/'), "name must not contain '/': {name:?}");
    name
}

fn parse_openflags(flags: i32) -> OpenFlags {
    // TODO Is this the right way to parse openflags? Are there other flags than just Read+Write?
    //      https://docs.rs/fuser/latest/fuser/trait.Filesystem.html#method.open seems to suggest so.
    match flags & libc::O_ACCMODE {
        libc::O_RDONLY => OpenFlags::Read,
        libc::O_WRONLY => OpenFlags::Write,
        libc::O_RDWR => OpenFlags::ReadWrite,
        _ => panic!("invalid flags: {flags}"),
    }
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
