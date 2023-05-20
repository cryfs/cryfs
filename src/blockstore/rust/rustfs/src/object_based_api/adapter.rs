use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use crate::common::{DirEntry, FsError, FsResult, Gid, Mode, NumBytes, OpenFlags, Statfs, Uid};

use crate::low_level_api::{
    AsyncFilesystem, AttrResponse, CreateResponse, FileHandle, IntoFs, OpenResponse,
    OpendirResponse, RequestInfo,
};

use super::{open_file_list::OpenFileList, Device, Dir, File, Node, OpenFile, Symlink};

// TODO Make sure each function checks the preconditions on its parameters, e.g. paths must be absolute

// TODO Set these TTLs to the fuse defaults
const TTL_GETATTR: Duration = Duration::from_secs(1);
const TTL_MKDIR: Duration = Duration::from_secs(1);
const TTL_SYMLINK: Duration = Duration::from_secs(1);
const TTL_CREATE: Duration = Duration::from_secs(1);

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

pub struct ObjectBasedFsAdapter<Fs: Device>
where
    // TODO Is this send+sync bound only needed because fuse_mt goes multi threaded or would it also be required for fuser?
    Fs::OpenFile: Send + Sync,
{
    // TODO We only need the Arc<RwLock<...>> because of initialization. Is there a better way to do that?
    fs: Arc<RwLock<MaybeInitializedFs<Fs>>>,

    // TODO Can we improve concurrency by locking less in open_files and instead making OpenFileList concurrency safe somehow?
    open_files: RwLock<OpenFileList<Fs::OpenFile>>,
}

impl<Fs: Device> ObjectBasedFsAdapter<Fs>
where
    // TODO Is this send+sync bound only needed because fuse_mt goes multi threaded or would it also be required for fuser?
    Fs::OpenFile: Send + Sync,
{
    pub fn new(fs: impl FnOnce(Uid, Gid) -> Fs + Send + Sync + 'static) -> Self {
        let open_files = Default::default();
        Self {
            fs: Arc::new(RwLock::new(MaybeInitializedFs::Uninitialized(Some(
                Box::new(fs),
            )))),
            open_files,
        }
    }
}

#[async_trait(?Send)]
impl<Fs> AsyncFilesystem for ObjectBasedFsAdapter<Fs>
where
    // TODO Are these Send+Sync bounds only needed because fuse_mt goes multi threaded or would it also be required for fuser?
    Fs: Device + Send + Sync,
    Fs::OpenFile: Send + Sync,
{
    async fn init(&self, req: RequestInfo) -> FsResult<()> {
        log::info!("init");
        self.fs.write().unwrap().initialize(req.uid, req.gid);
        Ok(())
    }

    async fn destroy(&self) {
        log::info!("destroy");
        // Nothing.
    }

    async fn getattr(
        &self,
        _req: RequestInfo,
        path: &Path,
        fh: Option<FileHandle>,
    ) -> FsResult<AttrResponse> {
        let attrs = if let Some(fh) = fh {
            // TODO No unwrap
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
        Ok(AttrResponse {
            ttl: TTL_GETATTR,
            attrs: attrs,
        })
    }

    async fn chmod(
        &self,
        _req: RequestInfo,
        path: &Path,
        fh: Option<FileHandle>,
        mode: Mode,
    ) -> FsResult<()> {
        // TODO Make sure file/symlink/dir flags are correctly set by this
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
    }

    async fn chown(
        &self,
        _req: RequestInfo,
        path: &Path,
        fh: Option<FileHandle>,
        uid: Option<Uid>,
        gid: Option<Gid>,
    ) -> FsResult<()> {
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
    }

    async fn truncate(
        &self,
        _req: RequestInfo,
        path: &Path,
        fh: Option<FileHandle>,
        size: NumBytes,
    ) -> FsResult<()> {
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
    }

    async fn utimens(
        &self,
        _req: RequestInfo,
        path: &Path,
        fh: Option<FileHandle>,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
    ) -> FsResult<()> {
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
    }

    async fn utimens_macos(
        &self,
        _req: RequestInfo,
        _path: &Path,
        _fh: Option<FileHandle>,
        _crtime: Option<SystemTime>,
        _chgtime: Option<SystemTime>,
        _bkuptime: Option<SystemTime>,
        _flags: Option<u32>,
    ) -> FsResult<()> {
        // TODO Implement this
        Err(FsError::NotImplemented)
    }

    async fn readlink(&self, _req: RequestInfo, path: &Path) -> FsResult<PathBuf> {
        let link = self.fs.read().unwrap().get().load_symlink(path).await?;
        link.target().await
    }

    async fn mknod(
        &self,
        _req: RequestInfo,
        _parent: &Path,
        _name: &str,
        _mode: Mode,
        _rdev: u32,
    ) -> FsResult<AttrResponse> {
        // TODO Do we want to implement this?
        Err(FsError::NotImplemented)
    }

    async fn mkdir(
        &self,
        req: RequestInfo,
        parent: &Path,
        name: &str,
        mode: Mode,
    ) -> FsResult<AttrResponse> {
        let uid = Uid::from(req.uid);
        let gid = Gid::from(req.gid);
        let mode = Mode::from(mode).add_dir_flag();
        // TODO Assert mode doesn't have file or symlink flags set
        let parent_dir = self.fs.read().unwrap().get().load_dir(parent).await?;
        let new_dir_attrs = parent_dir.create_child_dir(&name, mode, uid, gid).await?;
        Ok(AttrResponse {
            ttl: TTL_MKDIR,
            attrs: new_dir_attrs,
        })
    }

    async fn unlink(&self, _req: RequestInfo, parent: &Path, name: &str) -> FsResult<()> {
        let parent_dir = self.fs.read().unwrap().get().load_dir(parent).await?;
        parent_dir.remove_child_file_or_symlink(&name).await?;
        Ok(())
    }

    async fn rmdir(&self, _req: RequestInfo, parent: &Path, name: &str) -> FsResult<()> {
        let parent_dir = self.fs.read().unwrap().get().load_dir(parent).await?;
        parent_dir.remove_child_dir(&name).await?;
        Ok(())
    }

    async fn symlink(
        &self,
        req: RequestInfo,
        parent: &Path,
        name: &str,
        target: &Path,
    ) -> FsResult<AttrResponse> {
        let uid = Uid::from(req.uid);
        let gid = Gid::from(req.gid);
        let parent_dir = self.fs.read().unwrap().get().load_dir(parent).await?;
        let new_symlink_attrs = parent_dir
            .create_child_symlink(&name, target, uid, gid)
            .await?;
        Ok(AttrResponse {
            ttl: TTL_SYMLINK,
            attrs: new_symlink_attrs,
        })
    }

    async fn rename(
        &self,
        _req: RequestInfo,
        oldparent: &Path,
        oldname: &str,
        newparent: &Path,
        newname: &str,
    ) -> FsResult<()> {
        let old_parent_dir = self.fs.read().unwrap().get().load_dir(oldparent).await?;
        let new_path = newparent.join(&*newname);
        // TODO Should rename overwrite a potentially already existing target or not? Make sure we handle that the right way.
        old_parent_dir.rename_child(&oldname, &new_path).await?;
        Ok(())
    }

    async fn link(
        &self,
        _req: RequestInfo,
        _path: &Path,
        _newparent: &Path,
        _newname: &str,
    ) -> FsResult<AttrResponse> {
        // TODO Should we implement this?
        Err(FsError::NotImplemented)
    }

    async fn open(
        &self,
        _req: RequestInfo,
        path: &Path,
        flags: OpenFlags,
    ) -> FsResult<OpenResponse> {
        let file = self.fs.read().unwrap().get().load_file(path).await?;
        let open_file = file.open(flags).await?;
        let fh = self.open_files.write().unwrap().add(open_file);
        Ok(OpenResponse {
            fh: fh.into(),
            // TODO Do we need to change flags or is it ok to just return the flags passed in? If it's ok, then why do we have to return them?
            flags,
        })
    }

    async fn read<CallbackResult>(
        &self,
        _req: RequestInfo,
        _path: &Path,
        fh: FileHandle,
        offset: NumBytes,
        size: NumBytes,
        callback: impl for<'a> FnOnce(FsResult<&'a [u8]>) -> CallbackResult,
    ) -> CallbackResult {
        let offset = NumBytes::from(offset);
        let size = NumBytes::from(u64::from(size));
        let open_file_list = self.open_files.read().unwrap();
        let open_file = open_file_list.get(fh.into()).ok_or_else(|| {
            log::error!("read: no open file with handle {}", u64::from(fh));
            FsError::InvalidFileDescriptor { fh: u64::from(fh) }
        });
        match open_file {
            Ok(open_file) => {
                let data = open_file.read(offset, size).await;
                match data {
                    Ok(data) => {
                        let result = callback(Ok(data.as_ref()));
                        result
                    }
                    Err(err) => callback(Err(err)),
                }
            }
            Err(err) => callback(Err(err)),
        }
    }

    async fn write(
        &self,
        _req: RequestInfo,
        _path: &Path,
        fh: FileHandle,
        offset: NumBytes,
        data: Vec<u8>,
        // TODO What is the `flags` parameter for?
        _flags: u32,
    ) -> FsResult<NumBytes> {
        let data_len = data.len();
        let data = data.into();
        let open_file_list = self.open_files.read().unwrap();
        let open_file = open_file_list.get(fh.into()).ok_or_else(|| {
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
        _path: &Path,
        fh: FileHandle,
        _lock_owner: u64,
    ) -> FsResult<()> {
        let open_file_list = self.open_files.read().unwrap();
        let open_file = open_file_list.get(fh.into()).ok_or_else(|| {
            log::error!("flush: no open file with handle {}", u64::from(fh));
            FsError::InvalidFileDescriptor { fh: u64::from(fh) }
        })?;
        open_file.flush().await?;
        Ok(())
    }

    async fn release(
        &self,
        _req: RequestInfo,
        _path: &Path,
        fh: FileHandle,
        _flags: OpenFlags,
        _lock_owner: u64,
        _flush: bool,
    ) -> FsResult<()> {
        // TODO No unwrap
        self.open_files.write().unwrap().remove(fh.into());
        Ok(())
    }

    async fn fsync(
        &self,
        _req: RequestInfo,
        _path: &Path,
        fh: FileHandle,
        datasync: bool,
    ) -> FsResult<()> {
        let open_file_list = self.open_files.read().unwrap();
        let open_file = open_file_list.get(fh.into()).ok_or_else(|| {
            log::error!("fsync: no open file with handle {}", u64::from(fh));
            FsError::InvalidFileDescriptor { fh: u64::from(fh) }
        })?;
        open_file.fsync(datasync).await?;
        Ok(())
    }

    async fn opendir(
        &self,
        _req: RequestInfo,
        _path: &Path,
        flags: u32,
    ) -> FsResult<OpendirResponse> {
        // TODO Do we need opendir? The path seems to be passed to readdir, but the fuse_mt comment
        // to opendir seems to suggest that readdir may have to recognize dirs with just the fh and no path?
        Ok(OpendirResponse {
            fh: FileHandle::from(0),
            flags,
        })
    }

    async fn readdir(
        &self,
        _req: RequestInfo,
        path: &Path,
        _fh: FileHandle,
    ) -> FsResult<Vec<DirEntry>> {
        let dir = self.fs.read().unwrap().get().load_dir(path).await?;
        let entries = dir.entries().await?;
        Ok(entries)
    }

    async fn releasedir(
        &self,
        _req: RequestInfo,
        _path: &Path,
        _fh: FileHandle,
        _flags: u32,
    ) -> FsResult<()> {
        // TODO If we need opendir, then we also need releasedir, see TODO comment in opendir
        Ok(())
    }

    async fn fsyncdir(
        &self,
        _req: RequestInfo,
        _path: &Path,
        _fh: FileHandle,
        _datasync: bool,
    ) -> FsResult<()> {
        Err(FsError::NotImplemented)
    }

    async fn statfs(&self, _req: RequestInfo, _path: &Path) -> FsResult<Statfs> {
        self.fs.read().unwrap().get().statfs().await
    }

    async fn setxattr(
        &self,
        _req: RequestInfo,
        _path: &Path,
        _name: &str,
        _value: &[u8],
        _flags: u32,
        _position: u32,
    ) -> FsResult<()> {
        // TODO Should we implement this?
        Err(FsError::NotImplemented)
    }

    async fn getxattr_numbytes(
        &self,
        _req: RequestInfo,
        _path: &Path,
        _name: &str,
    ) -> FsResult<NumBytes> {
        // TODO Should we implement this?
        Err(FsError::NotImplemented)
    }

    async fn getxattr_data(
        &self,
        _req: RequestInfo,
        _path: &Path,
        _name: &str,
        _size: NumBytes,
    ) -> FsResult<Vec<u8>> {
        // TODO Should we implement this?
        Err(FsError::NotImplemented)
    }

    async fn listxattr_numbytes(&self, _req: RequestInfo, _path: &Path) -> FsResult<NumBytes> {
        // TODO Should we implement this?
        Err(FsError::NotImplemented)
    }

    async fn listxattr_data(
        &self,
        _req: RequestInfo,
        _path: &Path,
        _size: NumBytes,
    ) -> FsResult<Vec<u8>> {
        // TODO Should we implement this?
        Err(FsError::NotImplemented)
    }

    async fn removexattr(&self, _req: RequestInfo, _path: &Path, _name: &str) -> FsResult<()> {
        Err(FsError::NotImplemented)
    }

    async fn access(&self, _req: RequestInfo, _path: &Path, _mask: u32) -> FsResult<()> {
        // TODO Should we implement access?
        Ok(())
    }

    async fn create(
        &self,
        req: RequestInfo,
        parent: &Path,
        name: &str,
        mode: Mode,
        flags: i32,
    ) -> FsResult<CreateResponse> {
        let mode = mode.add_file_flag();
        // TODO Assert that dir/symlink flags aren't set
        let parent_dir = self.fs.read().unwrap().get().load_dir(parent).await?;
        let (file_attrs, open_file) = parent_dir
            .create_and_open_file(&name, mode, req.uid, req.gid)
            .await?;
        let fh = self.open_files.write().unwrap().add(open_file);
        Ok(CreateResponse {
            ttl: TTL_CREATE,
            attrs: file_attrs,
            fh,
            // TODO Do we need to change flags or is it ok to just return the flags passed in? If it's ok, then why do we have to return them?
            flags,
        })
    }
}

impl<Fn, D> IntoFs<ObjectBasedFsAdapter<D>> for Fn
where
    Fn: FnOnce(Uid, Gid) -> D + Send + Sync + 'static,
    D: Device + Send + Sync,
    D::OpenFile: Send + Sync,
{
    fn into_fs(self) -> ObjectBasedFsAdapter<D> {
        ObjectBasedFsAdapter::new(self)
    }
}
