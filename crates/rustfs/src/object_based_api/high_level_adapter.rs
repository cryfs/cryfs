use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use super::utils::MaybeInitializedFs;
use super::{Device, Dir, File, Node, OpenFile, Symlink};
use crate::common::{
    AbsolutePath, Callback, DirEntry, FileHandle, FsError, FsResult, Gid, HandleMap, Mode,
    NumBytes, OpenFlags, RequestInfo, Statfs, Uid,
};
use crate::high_level_api::{
    AsyncFilesystem, AttrResponse, CreateResponse, IntoFs, OpenResponse, OpendirResponse,
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    with_async_drop_2,
};

// TODO Make sure each function checks the preconditions on its parameters, e.g. paths must be absolute, here and elsewhere.

// TODO Set these TTLs to the fuse defaults
const TTL_GETATTR: Duration = Duration::from_secs(0);
const TTL_MKDIR: Duration = Duration::from_secs(0);
const TTL_SYMLINK: Duration = Duration::from_secs(0);
const TTL_CREATE: Duration = Duration::from_secs(0);

pub struct ObjectBasedFsAdapter<Fs: Device>
where
    // TODO Is this send+sync bound only needed because fuse_mt goes multi threaded or would it also be required for fuser?
    Fs: Send + Sync + 'static,
    Fs::OpenFile: Send + Sync,
{
    // TODO We only need the Arc<RwLock<...>> because of initialization. Is there a better way to do that?
    fs: Arc<RwLock<MaybeInitializedFs<Fs>>>,

    // TODO Can we improve concurrency by locking less in open_files and instead making OpenFileList concurrency safe somehow?
    open_files: tokio::sync::RwLock<AsyncDropGuard<HandleMap<FileHandle, Fs::OpenFile>>>,
}

impl<Fs: Device> ObjectBasedFsAdapter<Fs>
where
    // TODO Is this send+sync bound only needed because fuse_mt goes multi threaded or would it also be required for fuser?
    Fs: Send + Sync + 'static,
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

    // TODO Test this is triggered by each operation
    async fn trigger_on_operation(&self) -> FsResult<()> {
        // TODO Many operations need to lock fs too, locking here means we lock it twice. Optimize perf.
        let fs = self.fs.read().unwrap();
        let fs = fs.get();
        fs.on_operation().await?;
        Ok(())
    }
}

impl<Fs: Device> Debug for ObjectBasedFsAdapter<Fs>
where
    Fs: Device + Send + Sync + 'static,
    Fs::OpenFile: Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObjectBasedFsAdapter")
            .field("open_files", &self.open_files)
            .finish()
    }
}

#[async_trait(?Send)]
impl<Fs> AsyncFilesystem for ObjectBasedFsAdapter<Fs>
where
    // TODO Are these Send+Sync bounds only needed because fuse_mt goes multi threaded or would it also be required for fuser? And do we really need the 'static?
    Fs: Device + Send + Sync + 'static,
    Fs::OpenFile: Send + Sync,
{
    async fn init(&self, req: RequestInfo) -> FsResult<()> {
        log::info!("init");
        self.fs.write().unwrap().initialize(req.uid, req.gid);
        Ok(())
    }

    async fn destroy(&self) {
        log::info!("destroy");
        self.open_files.write().await.async_drop().await.unwrap();
        self.fs.write().unwrap().take().destroy().await;
        // Nothing.
    }

    async fn getattr(
        &self,
        _req: RequestInfo,
        path: &AbsolutePath,
        fh: Option<FileHandle>,
    ) -> FsResult<AttrResponse> {
        self.trigger_on_operation().await?;

        let attrs = if let Some(fh) = fh {
            // TODO No unwrap
            let open_file_list = self.open_files.read().await;
            let open_file = open_file_list.get(fh).ok_or_else(|| {
                log::error!("getattr: no open file with handle {}", u64::from(fh));
                FsError::InvalidFileDescriptor { fh: u64::from(fh) }
            })?;
            open_file.getattr().await?
        } else {
            let fs = self.fs.read().unwrap();
            let node = fs.get().lookup(path).await?;
            with_async_drop_2!(node, { node.getattr().await })?
        };
        Ok(AttrResponse {
            ttl: TTL_GETATTR,
            attrs: attrs,
        })
    }

    async fn chmod(
        &self,
        _req: RequestInfo,
        path: &AbsolutePath,
        fh: Option<FileHandle>,
        mode: Mode,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        // TODO Make sure file/symlink/dir flags are correctly set by this
        if let Some(fh) = fh {
            let open_file_list = self.open_files.read().await;
            let open_file = open_file_list.get(fh).ok_or_else(|| {
                log::error!("chmod: no open file with handle {}", u64::from(fh));
                FsError::InvalidFileDescriptor { fh: u64::from(fh) }
            })?;
            open_file
                .setattr(Some(mode), None, None, None, None, None, None)
                .await?;
        } else {
            let fs = self.fs.read().unwrap();
            let node = fs.get().lookup(path).await?;
            with_async_drop_2!(node, {
                node.setattr(Some(mode), None, None, None, None, None, None)
                    .await
            })?;
        };
        Ok(())
    }

    async fn chown(
        &self,
        _req: RequestInfo,
        path: &AbsolutePath,
        fh: Option<FileHandle>,
        uid: Option<Uid>,
        gid: Option<Gid>,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        if let Some(fh) = fh {
            let open_file_list = self.open_files.read().await;
            let open_file = open_file_list.get(fh).ok_or_else(|| {
                log::error!("chown: no open file with handle {}", u64::from(fh));
                FsError::InvalidFileDescriptor { fh: u64::from(fh) }
            })?;
            open_file
                .setattr(None, uid, gid, None, None, None, None)
                .await?;
        } else {
            let fs = self.fs.read().unwrap();
            let node = fs.get().lookup(path).await?;
            with_async_drop_2!(node, {
                node.setattr(None, uid, gid, None, None, None, None).await
            })?;
        }

        Ok(())
    }

    async fn truncate(
        &self,
        _req: RequestInfo,
        path: &AbsolutePath,
        fh: Option<FileHandle>,
        size: NumBytes,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        if let Some(fh) = fh {
            let open_file_list = self.open_files.read().await;
            let open_file = open_file_list.get(fh).ok_or_else(|| {
                log::error!("truncate: no open file with handle {}", u64::from(fh));
                FsError::InvalidFileDescriptor { fh: u64::from(fh) }
            })?;
            open_file
                .setattr(None, None, None, Some(size), None, None, None)
                .await?;
        } else {
            let fs = self.fs.read().unwrap();
            let node = fs.get().lookup(path).await?;
            with_async_drop_2!(node, {
                node.setattr(None, None, None, Some(size), None, None, None)
                    .await
            })?;
        };
        Ok(())
    }

    async fn utimens(
        &self,
        _req: RequestInfo,
        path: &AbsolutePath,
        fh: Option<FileHandle>,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        if let Some(fh) = fh {
            let open_file_list = self.open_files.read().await;
            let open_file = open_file_list.get(fh).ok_or_else(|| {
                log::error!("utimens: no open file with handle {}", u64::from(fh));
                FsError::InvalidFileDescriptor { fh: u64::from(fh) }
            })?;
            open_file
                .setattr(None, None, None, None, atime, mtime, None)
                .await?;
        } else {
            let fs = self.fs.read().unwrap();
            let node = fs.get().lookup(path).await?;
            with_async_drop_2!(node, {
                node.setattr(None, None, None, None, atime, mtime, None)
                    .await
            })?;
        };
        Ok(())
    }

    async fn utimens_macos(
        &self,
        _req: RequestInfo,
        _path: &AbsolutePath,
        _fh: Option<FileHandle>,
        _crtime: Option<SystemTime>,
        _chgtime: Option<SystemTime>,
        _bkuptime: Option<SystemTime>,
        _flags: Option<u32>,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        // TODO Implement this
        Err(FsError::NotImplemented)
    }

    async fn readlink(&self, _req: RequestInfo, path: &AbsolutePath) -> FsResult<String> {
        self.trigger_on_operation().await?;

        let fs = self.fs.read().unwrap();
        let link = fs.get().lookup(path).await?;
        with_async_drop_2!(link, {
            let link = link.as_symlink().await?;
            link.target().await
        })
    }

    async fn mknod(
        &self,
        _req: RequestInfo,
        _path: &AbsolutePath,
        _mode: Mode,
        _rdev: u32,
    ) -> FsResult<AttrResponse> {
        self.trigger_on_operation().await?;

        // TODO Do we want to implement this?
        Err(FsError::NotImplemented)
    }

    async fn mkdir(
        &self,
        req: RequestInfo,
        path: &AbsolutePath,
        mode: Mode,
    ) -> FsResult<AttrResponse> {
        self.trigger_on_operation().await?;

        let (parent, name) = path.split_last().ok_or_else(|| {
            assert!(path.is_root());
            // TODO Here and throughout, use a consistent logging and decide how to log (1) things that are wrong in the file system vs (2) operations that are successful if returning errors, e.g. getattr on a non-existing path
            log::error!("mkdir: called with root path");
            FsError::InvalidOperation
        })?;
        let fs = self.fs.read().unwrap();
        let parent_dir = fs.get().lookup(parent).await?;
        with_async_drop_2!(parent_dir, {
            let parent_dir = parent_dir.as_dir().await?;
            let (new_dir_attrs, _) = parent_dir
                .create_child_dir(&name, mode, req.uid, req.gid)
                .await?;
            Ok(AttrResponse {
                ttl: TTL_MKDIR,
                attrs: new_dir_attrs,
            })
        })
    }

    async fn unlink(&self, _req: RequestInfo, path: &AbsolutePath) -> FsResult<()> {
        self.trigger_on_operation().await?;

        let (parent, name) = path.split_last().ok_or_else(|| {
            assert!(path.is_root());
            log::error!("unlink: called with root path");
            FsError::InvalidOperation
        })?;
        let fs = self.fs.read().unwrap();
        let parent_dir = fs.get().lookup(parent).await?;
        with_async_drop_2!(parent_dir, {
            let parent_dir = parent_dir.as_dir().await?;
            parent_dir.remove_child_file_or_symlink(&name).await?;
            Ok(())
        })
    }

    async fn rmdir(&self, _req: RequestInfo, path: &AbsolutePath) -> FsResult<()> {
        self.trigger_on_operation().await?;

        let (parent, name) = path.split_last().ok_or_else(|| {
            assert!(path.is_root());
            log::error!("rmdir: called with root path");
            FsError::InvalidOperation
        })?;
        let fs = self.fs.read().unwrap();
        let parent_dir = fs.get().lookup(parent).await?;
        with_async_drop_2!(parent_dir, {
            let parent_dir = parent_dir.as_dir().await?;
            parent_dir.remove_child_dir(&name).await?;
            Ok(())
        })
    }

    async fn symlink(
        &self,
        req: RequestInfo,
        path: &AbsolutePath,
        // TODO Custom type for target that can be an absolute-or-relative path
        target: &str,
    ) -> FsResult<AttrResponse> {
        self.trigger_on_operation().await?;

        let (parent, name) = path.split_last().ok_or_else(|| {
            assert!(path.is_root());
            log::error!("symlink: called with root path");
            FsError::InvalidOperation
        })?;
        let fs = self.fs.read().unwrap();
        let parent_dir = fs.get().lookup(parent).await?;
        with_async_drop_2!(parent_dir, {
            let parent_dir = parent_dir.as_dir().await?;
            let (new_symlink_attrs, _symlink) = parent_dir
                .create_child_symlink(&name, target, req.uid, req.gid)
                .await?;
            Ok(AttrResponse {
                ttl: TTL_SYMLINK,
                attrs: new_symlink_attrs,
            })
        })
    }

    async fn rename(
        &self,
        _req: RequestInfo,
        oldpath: &AbsolutePath,
        newpath: &AbsolutePath,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        if oldpath.is_root() {
            log::error!("rename: tried to rename the root directory into '{newpath}'");
            return Err(FsError::InvalidOperation);
        }
        if newpath.is_root() {
            log::error!("rename: tried to rename '{oldpath}' into the root directory");
            return Err(FsError::InvalidOperation);
        };
        let fs = self.fs.read().unwrap();
        // TODO Should rename overwrite a potentially already existing target or not? Make sure we handle that the right way.
        fs.get().rename(oldpath, newpath).await?;
        Ok(())
    }

    async fn link(
        &self,
        _req: RequestInfo,
        _oldpath: &AbsolutePath,
        _newpath: &AbsolutePath,
    ) -> FsResult<AttrResponse> {
        self.trigger_on_operation().await?;

        // TODO Should we implement this?
        Err(FsError::NotImplemented)
    }

    async fn open(
        &self,
        _req: RequestInfo,
        path: &AbsolutePath,
        flags: OpenFlags,
    ) -> FsResult<OpenResponse> {
        self.trigger_on_operation().await?;

        let fs = self.fs.read().unwrap();
        let file = fs.get().lookup(path).await?;
        with_async_drop_2!(file, {
            let file = file.as_file().await?;
            let result = match file.open(flags).await {
                Err(err) => Err(err),
                Ok(open_file) => {
                    let fh = self.open_files.write().await.add(open_file);
                    Ok(OpenResponse {
                        fh: fh.handle,
                        // TODO Do we need to change flags or is it ok to just return the flags passed in? If it's ok, then why do we have to return them?
                        flags,
                    })
                }
            };
            result
        })
    }

    async fn read<R, C>(
        &self,
        _req: RequestInfo,
        _path: &AbsolutePath,
        fh: FileHandle,
        offset: NumBytes,
        size: NumBytes,
        callback: C,
    ) -> R
    where
        C: for<'a> Callback<FsResult<&'a [u8]>, R>,
    {
        match self.trigger_on_operation().await {
            Ok(()) => {}
            Err(err) => {
                return callback.call(Err(err));
            }
        }

        let open_file_list = self.open_files.read().await;
        let open_file = open_file_list.get(fh).ok_or_else(|| {
            log::error!("read: no open file with handle {}", u64::from(fh));
            FsError::InvalidFileDescriptor { fh: u64::from(fh) }
        });
        match open_file {
            Ok(open_file) => {
                let data = open_file.read(offset, size).await;
                match data {
                    Ok(data) => {
                        let result = callback.call(Ok(data.as_ref()));
                        result
                    }
                    Err(err) => callback.call(Err(err)),
                }
            }
            Err(err) => callback.call(Err(err)),
        }
    }

    async fn write(
        &self,
        _req: RequestInfo,
        _path: &AbsolutePath,
        fh: FileHandle,
        offset: NumBytes,
        data: Vec<u8>,
        // TODO What is the `flags` parameter for?
        _flags: u32,
    ) -> FsResult<NumBytes> {
        self.trigger_on_operation().await?;

        let data_len = data.len();
        let data = data.into();
        let open_file_list = self.open_files.read().await;
        let open_file = open_file_list.get(fh).ok_or_else(|| {
            log::error!("write: no open file with handle {}", u64::from(fh));
            FsError::InvalidFileDescriptor { fh: u64::from(fh) }
        })?;
        open_file.write(offset, data).await?;
        // TODO No unwrap
        Ok(NumBytes::from(u64::try_from(data_len).unwrap()))
    }

    async fn flush(
        &self,
        _req: RequestInfo,
        _path: &AbsolutePath,
        fh: FileHandle,
        _lock_owner: u64,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        let open_file_list = self.open_files.read().await;
        let open_file = open_file_list.get(fh).ok_or_else(|| {
            log::error!("flush: no open file with handle {}", u64::from(fh));
            FsError::InvalidFileDescriptor { fh: u64::from(fh) }
        })?;
        open_file.flush().await?;
        Ok(())
    }

    async fn release(
        &self,
        _req: RequestInfo,
        _path: &AbsolutePath,
        fh: FileHandle,
        _flags: OpenFlags,
        _lock_owner: u64,
        _flush: bool,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        // TODO No unwrap
        let mut removed = self.open_files.write().await.remove(fh);
        removed.async_drop().await?;
        Ok(())
    }

    async fn fsync(
        &self,
        _req: RequestInfo,
        _path: &AbsolutePath,
        fh: FileHandle,
        datasync: bool,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        let open_file_list = self.open_files.read().await;
        let open_file = open_file_list.get(fh).ok_or_else(|| {
            log::error!("fsync: no open file with handle {}", u64::from(fh));
            FsError::InvalidFileDescriptor { fh: u64::from(fh) }
        })?;
        open_file.fsync(datasync).await?;
        Ok(())
    }

    async fn opendir(
        &self,
        _req: RequestInfo,
        _path: &AbsolutePath,
        flags: u32,
    ) -> FsResult<OpendirResponse> {
        self.trigger_on_operation().await?;

        // TODO Do we need opendir? The path seems to be passed to readdir, but the fuse_mt comment
        // to opendir seems to suggest that readdir may have to recognize dirs with just the fh and no path?
        Ok(OpendirResponse {
            fh: FileHandle::from(0),
            flags,
        })
    }

    // TODO For some reason, there's a weird bug in readdir() with the fuse_mt backend:
    // $ cd mountdir
    // $ mkdir bla
    // $ cd bla
    // $ echo content > newfile
    // $ ls
    // [doesn't show `newfile`]
    // $ cd ..
    // $ cd bla
    // $ ls
    // [now shows `newfile`]
    // Not sure if this is a bug in fuse_mt (check if it applies to inmemory/passthrough too) or our adapter.
    // Not sure if this also happens for the fuser backend.
    async fn readdir(
        &self,
        _req: RequestInfo,
        path: &AbsolutePath,
        _fh: FileHandle,
    ) -> FsResult<Vec<DirEntry>> {
        self.trigger_on_operation().await?;

        let fs = self.fs.read().unwrap();
        let dir = fs.get().lookup(path).await?;
        with_async_drop_2!(dir, {
            let dir = dir.as_dir().await?;
            let entries = dir.entries().await?;
            Ok(entries)
        })
    }

    async fn releasedir(
        &self,
        _req: RequestInfo,
        _path: &AbsolutePath,
        _fh: FileHandle,
        _flags: u32,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        // TODO If we need opendir, then we also need releasedir, see TODO comment in opendir
        Ok(())
    }

    async fn fsyncdir(
        &self,
        _req: RequestInfo,
        _path: &AbsolutePath,
        _fh: FileHandle,
        _datasync: bool,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        Err(FsError::NotImplemented)
    }

    async fn statfs(&self, _req: RequestInfo, _path: &AbsolutePath) -> FsResult<Statfs> {
        self.trigger_on_operation().await?;

        self.fs.read().unwrap().get().statfs().await
    }

    async fn setxattr(
        &self,
        _req: RequestInfo,
        _path: &AbsolutePath,
        _name: &str,
        _value: &[u8],
        _flags: u32,
        _position: NumBytes,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        // TODO Should we implement this?
        Err(FsError::NotImplemented)
    }

    async fn getxattr_numbytes(
        &self,
        _req: RequestInfo,
        _path: &AbsolutePath,
        _name: &str,
    ) -> FsResult<NumBytes> {
        self.trigger_on_operation().await?;

        // TODO Should we implement this?
        Err(FsError::NotImplemented)
    }

    async fn getxattr_data(
        &self,
        _req: RequestInfo,
        _path: &AbsolutePath,
        _name: &str,
        _size: NumBytes,
    ) -> FsResult<Vec<u8>> {
        self.trigger_on_operation().await?;

        // TODO Should we implement this?
        Err(FsError::NotImplemented)
    }

    async fn listxattr_numbytes(
        &self,
        _req: RequestInfo,
        _path: &AbsolutePath,
    ) -> FsResult<NumBytes> {
        self.trigger_on_operation().await?;

        // TODO Should we implement this?
        Err(FsError::NotImplemented)
    }

    async fn listxattr_data(
        &self,
        _req: RequestInfo,
        _path: &AbsolutePath,
        _size: NumBytes,
    ) -> FsResult<Vec<u8>> {
        self.trigger_on_operation().await?;

        // TODO Should we implement this?
        Err(FsError::NotImplemented)
    }

    async fn removexattr(
        &self,
        _req: RequestInfo,
        _path: &AbsolutePath,
        _name: &str,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        Err(FsError::NotImplemented)
    }

    async fn access(&self, _req: RequestInfo, _path: &AbsolutePath, _mask: u32) -> FsResult<()> {
        self.trigger_on_operation().await?;

        // TODO Should we implement access?
        Ok(())
    }

    async fn create(
        &self,
        req: RequestInfo,
        path: &AbsolutePath,
        mode: Mode,
        flags: i32,
    ) -> FsResult<CreateResponse> {
        self.trigger_on_operation().await?;

        let (parent, name) = path.split_last().ok_or_else(|| {
            assert!(path.is_root());
            // TODO Here and throughout, use a consistent logging and decide how to log (1) things that are wrong in the file system vs (2) operations that are successful if returning errors, e.g. getattr on a non-existing path
            log::error!("create: called with root path");
            FsError::InvalidOperation
        })?;
        let fs = self.fs.read().unwrap();
        let parent_dir = fs.get().lookup(parent).await?;
        with_async_drop_2!(parent_dir, {
            let parent_dir = parent_dir.as_dir().await?;
            let (file_attrs, mut node, open_file) = parent_dir
                .create_and_open_file(&name, mode, req.uid, req.gid)
                .await?;
            node.async_drop().await?;
            let fh = self.open_files.write().await.add(open_file);
            Ok(CreateResponse {
                ttl: TTL_CREATE,
                attrs: file_attrs,
                fh: fh.handle,
                // TODO Do we need to change flags or is it ok to just return the flags passed in? If it's ok, then why do we have to return them?
                flags,
            })
        })
    }
}

impl<Fn, D> IntoFs<ObjectBasedFsAdapter<D>> for Fn
where
    Fn: FnOnce(Uid, Gid) -> D + Send + Sync + 'static,
    D: Device + Send + Sync + 'static,
    D::OpenFile: Send + Sync,
{
    fn into_fs(self) -> AsyncDropGuard<ObjectBasedFsAdapter<D>> {
        ObjectBasedFsAdapter::new(self)
    }
}

// TODO ObjectBasedFsAdapter doesn't need to be AsyncDrop
#[async_trait]
impl<Fs> AsyncDrop for ObjectBasedFsAdapter<Fs>
where
    Fs: Device + Send + Sync + 'static,
    Fs::OpenFile: Send + Sync,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        // TODO If the object was never used (e.g. destroy never called), we need to destroy members here.
        Ok(())
    }
}
