use async_trait::async_trait;
use futures::join;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

use super::utils::{MaybeInitializedFs, OpenFileList};
use super::{Device, Dir, File, Node, OpenFile, Symlink};
#[cfg(target_os = "macos")]
use crate::low_level_api::ReplyXTimes;
use crate::object_based_api::utils::{DUMMY_INO, MakeOrphanError, MoveInodeError};
use crate::{
    DirEntry,
    common::{
        Callback, FileHandle, FsError, FsResult, Gid, InodeNumber, Mode, NumBytes, OpenInFlags,
        OpenOutFlags, RequestInfo, Statfs, Uid,
    },
    low_level_api::{
        AsyncFilesystemLL, ReplyAttr, ReplyBmap, ReplyCreate, ReplyDirectory,
        ReplyDirectoryAddResult, ReplyDirectoryPlus, ReplyEntry, ReplyIoctl, ReplyLock, ReplyLseek,
        ReplyOpen, ReplyWrite,
    },
    object_based_api::utils::{DirCache, InodeList, OpenDirHandle},
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard, flatten_async_drop},
    path::PathComponent,
    with_async_drop_2,
};

// TODO What are good TTLs here?
const TTL_LOOKUP: Duration = Duration::from_secs(1);
const TTL_GETATTR: Duration = Duration::from_secs(1);
const TTL_CREATE: Duration = Duration::from_secs(1);
const TTL_SYMLINK: Duration = Duration::from_secs(1);

// TODO Can we share more code with [super::high_level_adapter::ObjectBasedFsAdapter]?
pub struct ObjectBasedFsAdapterLL<Fs>
where
    // TODO Is this send+sync bound only needed because fuse_mt goes multi threaded or would it also be required for fuser?
    Fs: Device + AsyncDrop + Send + Sync + Debug + 'static,
    for<'a> Fs::File<'a>: Send,
    Fs::OpenFile: Send + Sync,
{
    // TODO We only need the Arc<RwLock<...>> because of initialization. Is there a better way to do that?
    fs: Arc<RwLock<AsyncDropGuard<MaybeInitializedFs<Fs>>>>,

    inodes: AsyncDropGuard<InodeList<Fs>>,

    open_files: AsyncDropGuard<OpenFileList<Fs::OpenFile>>,
    open_dirs: AsyncDropGuard<DirCache>,
}

impl<Fs> ObjectBasedFsAdapterLL<Fs>
where
    // TODO Is this send+sync bound only needed because fuse_mt goes multi threaded or would it also be required for fuser?
    Fs: Device + AsyncDrop + Send + Sync + Debug + 'static,
    for<'a> Fs::File<'a>: Send,
    Fs::OpenFile: Send + Sync,
{
    pub fn new(
        fs: impl FnOnce(Uid, Gid) -> AsyncDropGuard<Fs> + Send + Sync + 'static,
    ) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            fs: Arc::new(RwLock::new(MaybeInitializedFs::new_uninitialized(
                Box::new(fs),
            ))),
            inodes: InodeList::new(),
            open_files: OpenFileList::new(),
            open_dirs: DirCache::new(),
        })
    }

    #[cfg(feature = "testutils")]
    pub async fn reset_cache_after_test(&self) {
        self.inodes.clear_all_slow().await.unwrap();
    }

    #[cfg(feature = "testutils")]
    pub async fn flush_cache(&self) {
        self.inodes.fsync_all().await.unwrap();
        self._flush_open_files().await;
    }

    #[cfg(feature = "testutils")]
    async fn _flush_open_files(&self) {
        use crate::object_based_api::utils::ForEachCallback;
        struct OpenFileFsyncCallback<OF> {
            _phantom: std::marker::PhantomData<OF>,
        }
        impl<OF> ForEachCallback<OF> for OpenFileFsyncCallback<OF>
        where
            OF: OpenFile + Send + Sync,
        {
            async fn call(&self, file: &OF) -> Result<(), FsError> {
                file.fsync(false).await
            }
        }
        self.open_files
            .for_each(OpenFileFsyncCallback {
                _phantom: std::marker::PhantomData,
            })
            .await
            .unwrap();
    }

    // TODO Test this is triggered by each operation
    async fn trigger_on_operation(&self) -> FsResult<()> {
        // TODO Many operations need to lock fs too, locking here means we lock it twice. Optimize perf.
        let fs = self.fs.read().await;
        let fs = fs.get();
        fs.on_operation().await?;
        Ok(())
    }

    // TODO Does this need to return an Arc::clone of the inode or can we just return a reference?
    /// This function allows file system operations to abstract over whether a requested inode number is the root node or whether it is looked up from the inode table `inodes`.
    async fn get_inode_and_parent_ino(
        &self,
        ino: InodeNumber,
    ) -> FsResult<(AsyncDropGuard<AsyncDropArc<Fs::Node>>, InodeNumber)> {
        // TODO Once async closures are stable, we can - instead of returning an AsyncDropArc - take a callback parameter and pass &Fs::Node to it.
        //      That would simplify all the call sites (e.g. don't require them to call async_drop on the returned value anymore).
        //      See https://stackoverflow.com/questions/76625378/async-closure-holding-reference-over-await-point
        self.inodes.get_node_and_parent_ino(ino).await
    }

    async fn get_inode(
        &self,
        ino: InodeNumber,
    ) -> FsResult<AsyncDropGuard<AsyncDropArc<Fs::Node>>> {
        self.inodes.get_node(ino).await
    }

    async fn _orphan_inode(&self, parent_ino: InodeNumber, name: &PathComponent) {
        match self.inodes.make_into_orphan(parent_ino, name).await {
            Ok(()) => {
                // everything ok
            }
            Err(MakeOrphanError::ParentNotFound) => {
                panic!(
                    "Tried to orphan inode with name {:?} under parent inode {:?}, and the operation (unlink/rmdir) seems to have been successful, but when trying to orphan, the parent inode was not found",
                    name, parent_ino
                );
            }
            Err(MakeOrphanError::ChildNotFound) => {
                // This can happen if the unlink'ed/rmdir'ed file/directory was never looked up before, so it is not in the inode list.
            }
        }
    }

    async fn _move_inode(
        &self,
        old_parent_ino: InodeNumber,
        old_name: &PathComponent,
        new_parent_ino: InodeNumber,
        new_name: &PathComponent,
    ) -> FsResult<()> {
        match self
            .inodes
            .move_inode(
                old_parent_ino,
                old_name,
                new_parent_ino,
                new_name.to_owned(),
            )
            .await
        {
            Ok(()) => {
                // everything ok
                Ok(())
            }
            Err(MoveInodeError::OldParentNotFound) => {
                panic!(
                    "Tried to move inode with name {:?} under parent inode {:?} to name {:?} under parent inode {:?}, but the operation (rename) seems to have been successful, but when trying to update the inode list, the old parent inode was not found",
                    old_name, old_parent_ino, new_name, new_parent_ino
                );
            }
            Err(MoveInodeError::NewParentNotFound) => {
                panic!(
                    "Tried to move inode with name {:?} under parent inode {:?} to name {:?} under parent inode {:?}, but the operation (rename) seems to have been successful, but when trying to update the inode list, the new parent inode was not found",
                    old_name, old_parent_ino, new_name, new_parent_ino
                );
            }
            Err(MoveInodeError::ChildNotFound) => {
                // The moved inode wasn't loaded, everything is ok. No need to adjust anything.
                Ok(())
            }
            Err(MoveInodeError::ErrorWhileDroppingNode(err)) => Err(err),
        }
    }
}

#[async_trait]
impl<Fs> AsyncFilesystemLL for ObjectBasedFsAdapterLL<Fs>
where
    // TODO Do we need those Send + Sync + 'static bounds?
    Fs: Device + AsyncDrop + Send + Sync + Debug + 'static,
    for<'a> Fs::File<'a>: Send,
    Fs::OpenFile: Send + Sync,
{
    async fn init(&self, req: &RequestInfo) -> FsResult<()> {
        log::info!("init");
        let mut fs = self.fs.write().await;
        fs.initialize(req.uid, req.gid);
        let rootdir = Dir::into_node(fs.get().rootdir().await?);
        self.inodes.insert_rootdir(rootdir).await;
        Ok(())
    }

    async fn destroy(&self) {
        log::info!("destroy");
        // Nothing.
    }

    async fn lookup(
        &self,
        _req: &RequestInfo,
        parent_ino: InodeNumber,
        name: &PathComponent,
    ) -> FsResult<ReplyEntry> {
        self.trigger_on_operation().await?;

        // TODO CryDir.lookup_child looks up name in the dir entry list from the parent dir node to get the blob id.
        //      child.getattr looks it up again, to get the node attrs. Both need to first lock the parent dir
        //      and then look up the same entry. Can we optimize that?

        let name_clone = name.to_owned();
        let load_child = move |parent_node: &AsyncDropGuard<AsyncDropArc<Fs::Node>>| {
            let parent_node = AsyncDropArc::clone(parent_node); // TODO Why is this necessary?
            async move {
                with_async_drop_2!(parent_node, {
                    let parent_node_dir = parent_node
                        .as_dir()
                        .await
                        .expect("Error: Inode number is not a directory");
                    with_async_drop_2!(parent_node_dir, {
                        // TODO Can we avoid the async_drop here by using something like parent_node_dir.into_lookup_child() ?
                        parent_node_dir.lookup_child(&name_clone).await
                    })
                })
            }
        };

        let (ino, child) = self
            .inodes
            .add_or_increment_refcount(parent_ino, name.to_owned(), load_child)
            .await?;

        with_async_drop_2!(child, {
            child.getattr().await.map(|attr| ReplyEntry {
                ttl: TTL_LOOKUP,
                ino,
                attr,
            })
        })
    }

    async fn forget(&self, _req: &RequestInfo, ino: InodeNumber, nlookup: u64) -> FsResult<()> {
        self.trigger_on_operation().await?;

        // From the fuser documentation:
        // ```
        // The nlookup parameter indicates the number of lookups previously performed on
        // this inode. If the filesystem implements inode lifetimes, it is recommended that
        // inodes acquire a single reference on each lookup, and lose nlookup references on
        // each forget. The filesystem may ignore forget calls, if the inodes don't need to
        // have a limited lifetime. On unmount it is not guaranteed, that all referenced
        // inodes will receive a forget message.
        // ```

        self.inodes.forget(ino, nlookup).await?;
        Ok(())
    }

    async fn getattr(
        &self,
        _req: &RequestInfo,
        ino: InodeNumber,
        fh: Option<FileHandle>,
    ) -> FsResult<ReplyAttr> {
        self.trigger_on_operation().await?;

        let attr = if let Some(fh) = fh {
            self.open_files
                .get(fh, async |open_file| open_file.getattr().await)
                .await?
        } else {
            let node = self.get_inode(ino).await?;
            with_async_drop_2!(node, { node.getattr().await })?
        };

        Ok(ReplyAttr {
            ttl: TTL_GETATTR,
            attr,
            ino,
        })
    }

    async fn setattr(
        &self,
        _req: &RequestInfo,
        ino: InodeNumber,
        mode: Option<Mode>,
        uid: Option<Uid>,
        gid: Option<Gid>,
        size: Option<NumBytes>,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
        ctime: Option<SystemTime>,
        fh: Option<FileHandle>,
        // TODO Add support for crtime, chgtme, bkuptime, flags. They're only relevant for macos.
        _crtime: Option<SystemTime>,
        _chgtime: Option<SystemTime>,
        _bkuptime: Option<SystemTime>,
        _flags: Option<u32>,
    ) -> FsResult<ReplyAttr> {
        self.trigger_on_operation().await?;

        let attr = if let Some(fh) = fh {
            self.open_files
                .get(fh, async |open_file| {
                    open_file
                        .setattr(mode, uid, gid, size, atime, mtime, ctime)
                        .await
                })
                .await?
        } else {
            let node = self.get_inode(ino).await?;
            with_async_drop_2!(node, {
                node.setattr(mode, uid, gid, size, atime, mtime, ctime)
                    .await
            })?
        };

        Ok(ReplyAttr {
            ttl: TTL_GETATTR,
            attr,
            ino,
        })
    }

    async fn readlink<R, C>(&self, _req: &RequestInfo, ino: InodeNumber, callback: C) -> R
    where
        R: 'static,
        C: Send + 'static + for<'a> Callback<FsResult<&'a str>, R>,
    {
        match self.trigger_on_operation().await {
            Ok(()) => (),
            Err(err) => {
                return callback.call(Err(err));
            }
        }

        let mut inode = match self.get_inode(ino).await {
            Ok(inode) => inode,
            Err(err) => return callback.call(Err(err)),
        };
        let target = match inode.as_symlink().await {
            Ok(inode_symlink) => with_async_drop_2!(inode_symlink, {
                let target = inode_symlink.target();
                match target.await {
                    Ok(target) => Ok(target),
                    Err(err) => Err(err),
                }
            }),
            Err(err) => Err(err),
        };
        let target = match target {
            Ok(ref target) => Ok(target.as_str()),
            Err(err) => Err(err),
        };
        let target = match inode.async_drop().await {
            Ok(()) => target,
            Err(err) => Err(err),
        };
        callback.call(target)
    }

    async fn mknod(
        &self,
        _req: &RequestInfo,
        _parent_ino: InodeNumber,
        _name: &PathComponent,
        _mode: Mode,
        _umask: u32,
        _rdev: u32,
    ) -> FsResult<ReplyEntry> {
        self.trigger_on_operation().await?;

        // TODO
        Err(FsError::NotImplemented)
    }

    async fn mkdir(
        &self,
        req: &RequestInfo,
        parent_ino: InodeNumber,
        name: &PathComponent,
        mode: Mode,
        _umask: u32,
    ) -> FsResult<ReplyEntry> {
        self.trigger_on_operation().await?;

        // In my tests with fuser 0.12.0, umask is already auto-applied to mode and the `umask` argument is always `0`.
        // TODO see https://github.com/cberner/fuser/issues/256
        let parent = self.get_inode(parent_ino).await?;
        let (attr, child) = with_async_drop_2!(parent, {
            let parent_dir = parent.as_dir().await?;

            // TODO Can we avoid the async_drop here by using something like dir.into_create_child_dir() ?
            with_async_drop_2!(parent_dir, {
                let (attrs, child) = parent_dir
                    .create_child_dir(&name, mode, req.uid, req.gid)
                    .await?;
                Ok::<_, FsError>((attrs, Dir::into_node(child)))
            })
        })?;
        // Fuser counts mkdir/create/symlink as a lookup and will call forget on the inode we allocate here.
        let ino = self
            .inodes
            .add(parent_ino, child, name.to_owned())
            .await
            .expect("Parent inode vanished while executing");
        Ok(ReplyEntry {
            ttl: TTL_GETATTR,
            attr,
            ino: ino,
        })
    }

    async fn unlink(
        &self,
        _req: &RequestInfo,
        parent_ino: InodeNumber,
        name: &PathComponent,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        let parent = self.get_inode(parent_ino).await?;
        with_async_drop_2!(parent, {
            let parent_dir = parent.as_dir().await?;
            with_async_drop_2!(parent_dir, {
                parent_dir.remove_child_file_or_symlink(&name).await?;
                self._orphan_inode(parent_ino, name).await;
                Ok::<_, FsError>(())
            })?;
            Ok(())
        })
    }

    async fn rmdir(
        &self,
        _req: &RequestInfo,
        parent_ino: InodeNumber,
        name: &PathComponent,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        let parent = self.get_inode(parent_ino).await?;
        with_async_drop_2!(parent, {
            let parent_dir = parent.as_dir().await?;
            with_async_drop_2!(parent_dir, {
                parent_dir.remove_child_dir(&name).await?;
                self._orphan_inode(parent_ino, name).await;
                Ok::<_, FsError>(())
            })?;
            Ok(())
        })
    }

    async fn symlink(
        &self,
        req: &RequestInfo,
        parent_ino: InodeNumber,
        name: &PathComponent,
        link: &str,
    ) -> FsResult<ReplyEntry> {
        self.trigger_on_operation().await?;

        // TODO Here (and maybe also in mkdir / create_file), existing nodes should be overwritten

        let parent = self.get_inode(parent_ino).await?;
        let (attrs, child) = with_async_drop_2!(parent, {
            let parent_dir = parent.as_dir().await?;
            // TODO Can we avoid the async_drop here by using something like dir.into_create_child_symlink()?
            with_async_drop_2!(parent_dir, {
                let (attrs, child) = parent_dir
                    .create_child_symlink(&name, link, req.uid, req.gid)
                    .await?;
                Ok::<_, FsError>((attrs, Symlink::into_node(child)))
            })
        })?;
        // Fuser counts mkdir/create/symlink as a lookup and will call forget on the inode we allocate here.
        let ino = self
            .inodes
            .add(parent_ino, child, name.to_owned())
            .await
            .expect("Parent inode vanished while executing");
        Ok(ReplyEntry {
            ttl: TTL_SYMLINK,
            attr: attrs,
            ino,
        })
    }

    async fn rename(
        &self,
        _req: &RequestInfo,
        oldparent_ino: InodeNumber,
        oldname: &PathComponent,
        newparent_ino: InodeNumber,
        newname: &PathComponent,
        _flags: u32,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        // TODO Honor flags
        // TODO Check that oldparent+oldname/newparent+newname aren't ancestors of each other, or at least write a test that fuse already blocks that
        if oldparent_ino == newparent_ino {
            let shared_parent = self.get_inode(oldparent_ino).await?;
            with_async_drop_2!(shared_parent, {
                let parent_dir = shared_parent.as_dir().await?;
                with_async_drop_2!(parent_dir, {
                    parent_dir.rename_child(&oldname, &newname).await?;
                    self._move_inode(oldparent_ino, oldname, newparent_ino, newname)
                        .await?;
                    Ok::<_, FsError>(())
                })?;
                Ok(())
            })
        } else {
            let (oldparent, newparent) =
                join!(self.get_inode(oldparent_ino), self.get_inode(newparent_ino));
            let (mut oldparent, mut newparent) =
                flatten_async_drop::<FsError, _, _, _, _>(oldparent, newparent).await?;
            let result = (async || {
                let oldparent_dir = oldparent.as_dir().await?;
                with_async_drop_2!(oldparent_dir, {
                    let newparent_dir = newparent.as_dir().await?;
                    oldparent_dir
                        .move_child_to(oldname, newparent_dir, newname)
                        .await?;
                    self._move_inode(oldparent_ino, oldname, newparent_ino, newname)
                        .await?;
                    Ok::<_, FsError>(())
                })
            })()
            .await;
            // TODO Drop concurrently and drop latter even if first one fails
            oldparent.async_drop().await?;
            newparent.async_drop().await?;
            result
        }
    }

    async fn link(
        &self,
        _req: &RequestInfo,
        _ino: InodeNumber,
        _newparent_ino: InodeNumber,
        _newname: &PathComponent,
    ) -> FsResult<ReplyEntry> {
        self.trigger_on_operation().await?;

        // TODO
        Err(FsError::NotImplemented)
    }

    async fn open(
        &self,
        _req: &RequestInfo,
        ino: InodeNumber,
        flags: OpenInFlags,
    ) -> FsResult<ReplyOpen> {
        self.trigger_on_operation().await?;

        let inode = self.get_inode(ino).await?;
        with_async_drop_2!(inode, {
            let file = inode.as_file().await?;
            let open_file = File::into_open(file, flags);
            let open_file = open_file.await?;
            let fh = self.open_files.add(open_file);
            Ok(ReplyOpen {
                fh: fh.handle,
                flags: OpenOutFlags {},
            })
        })
    }

    async fn read<R, C>(
        &self,
        _req: &RequestInfo,
        _ino: InodeNumber,
        fh: FileHandle,
        offset: NumBytes,
        size: NumBytes,
        _flags: i32, // TODO Handle flags, see https://docs.rs/fuser/latest/fuser/trait.Filesystem.html#method.read
        _lock_owner: Option<u64>, // TODO What to do with lock_owner?
        callback: C,
    ) -> R
    where
        R: 'static,
        C: Send + for<'a> Callback<FsResult<&'a [u8]>, R>,
    {
        match self.trigger_on_operation().await {
            Ok(()) => (),
            Err(err) => {
                return callback.call(Err(err));
            }
        }

        let data = self
            .open_files
            .get(fh, async |open_file| open_file.read(offset, size).await)
            .await;
        let data = match data {
            Ok(ref data) => Ok(data.as_ref()),
            Err(err) => Err(err),
        };
        callback.call(data)
    }

    async fn write(
        &self,
        _req: &RequestInfo,
        _ino: InodeNumber,
        fh: FileHandle,
        offset: NumBytes,
        data: Vec<u8>,
        _write_flags: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
    ) -> FsResult<ReplyWrite> {
        self.trigger_on_operation().await?;

        let len = data.len();

        // TODO What to do with WriteFlags, flags, lock_owner?
        self.open_files
            .get(fh, async |open_file| {
                open_file.write(offset, data.into()).await
            })
            .await?;

        Ok(ReplyWrite {
            written: NumBytes::from(u64::try_from(len).unwrap()), // TODO No unwrap
        })
    }

    async fn flush(
        &self,
        _req: &RequestInfo,
        _ino: InodeNumber,
        fh: FileHandle,
        _lock_owner: u64,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        // TODO What to do about lock_owner?
        self.open_files
            .get(fh, async |open_file| open_file.flush().await)
            .await
    }

    async fn release(
        &self,
        _req: &RequestInfo,
        _ino: InodeNumber,
        fh: FileHandle,
        _flags: OpenInFlags,
        _lock_owner: Option<u64>,
        flush: bool,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        // TODO Would it make sense to have `fh` always be equal to `ino`? Might simplify some things. Also, we could add an `assert_eq!(ino, fh)` here.

        // TODO What to do with flags, lock_owner?
        let open_file = self.open_files.remove(fh);
        with_async_drop_2!(open_file, {
            if flush {
                // TODO Is this actually what the `flush` parameter should do?
                open_file.flush().await?;
            }
            Ok(())
        })
    }

    async fn fsync(
        &self,
        _req: &RequestInfo,
        _ino: InodeNumber,
        fh: FileHandle,
        datasync: bool,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        // TODO What to do about lock_owner?
        self.open_files
            .get(fh, async |open_file| open_file.fsync(datasync).await)
            .await
    }

    async fn opendir(
        &self,
        _req: &RequestInfo,
        ino: InodeNumber,
        _flags: OpenInFlags, // TODO What to do with flags?
    ) -> FsResult<ReplyOpen> {
        self.trigger_on_operation().await?;

        let fh = self.open_dirs.add(ino);

        Ok(ReplyOpen {
            fh: fh.handle.into(),
            flags: OpenOutFlags {},
        })
    }

    async fn readdir<R: ReplyDirectory + Send + 'static>(
        &self,
        _req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        offset: u64,
        reply: &mut R,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        let dir_cache_entry = self.open_dirs.get(OpenDirHandle::from(fh)).ok_or_else(|| {
            log::error!("Tried to access a file descriptor for a directory that isn't opened");
            FsError::InvalidFileDescriptor { fh: fh.into() }
        })?;
        if dir_cache_entry.dir_ino() != ino {
            log::error!(
                "Tried to access a directory with inode {ino:?} using a file descriptor for inode {:?}",
                dir_cache_entry.dir_ino()
            );
            return Err(FsError::InvalidFileDescriptor { fh: fh.into() });
        }

        let (node, parent_ino) = self.get_inode_and_parent_ino(ino).await?;

        with_async_drop_2!(node, {
            let dir = node.as_dir().await?;
            with_async_drop_2!(dir, {
                let offset = usize::try_from(offset).unwrap(); // TODO No unwrap

                if offset == 0 {
                    match reply.add_self_reference(ino, 1) {
                        ReplyDirectoryAddResult::Full => {
                            return Ok(());
                        }
                        ReplyDirectoryAddResult::NotFull => {
                            // continue
                        }
                    }
                }

                if offset <= 1 {
                    match reply.add_parent_reference(parent_ino, 2) {
                        ReplyDirectoryAddResult::Full => {
                            return Ok(());
                        }
                        ReplyDirectoryAddResult::NotFull => {
                            // continue
                        }
                    }
                }

                // TODO Instead of loading all entries and then possibly only returning some because the buffer gets full, try to only load the necessary ones? But concurrently somehow?
                let load_dir_entries = async || dir.entries().await;

                let handle_dir_entries = move |entries: &[DirEntry]| {
                    // TODO Add tests for the offset calculations including '.' and '..' here
                    let entries = entries.iter().enumerate().skip(offset.saturating_sub(2)); // skip 2 less because those offset indices are for '.' and '..'
                    for (offset, entry) in entries {
                        // Readdir is not supposed to lookup or register inodes. The fuse API seems to just ignore it. There will never be forget() calls for these.
                        // See:
                        //  * https://github.com/wfraser/fuse-mt/blob/881d7320b4c73c0bfbcbca48a5faab2a26f3e9e8/src/fusemt.rs#L619
                        //  * https://libfuse.github.io/doxygen/structfuse__lowlevel__ops.html#af1ef8e59e0cb0b02dc0e406898aeaa51
                        // But note that readdirplus is supposed to register inodes.
                        //  * https://github.com/libfuse/libfuse/blob/7b9e7eeec6c43a62ab1e02dfb6542e6bfb7f72dc/include/fuse_lowlevel.h#L1209
                        let ino = DUMMY_INO;

                        // offset +2 because of '.' and '..' and +1 because fuser actually expects us to return the offset of the **next** entry
                        let offset = offset + 3;

                        let offset = i64::try_from(offset).unwrap(); // TODO No unwrap
                        let buffer_is_full = reply.add(ino.into(), offset, entry.kind, &entry.name);
                        match buffer_is_full {
                            ReplyDirectoryAddResult::Full => {
                                // TODO Test the scenario where a directory has lots of entries, the buffer gets full and fuser calls readdir() multiple times
                                //      In that test, ensure that we only load the directory entries once from the underlying FS and then respond from the cache.
                                break;
                            }
                            ReplyDirectoryAddResult::NotFull => {
                                // continue
                            }
                        }
                    }
                    Ok(())
                };

                dir_cache_entry
                    .get_or_query_entries(load_dir_entries, handle_dir_entries)
                    .await
            })
        })
    }

    async fn readdirplus<R: ReplyDirectoryPlus + Send + 'static>(
        &self,
        _req: &RequestInfo,
        _ino: InodeNumber,
        _fh: FileHandle,
        _offset: u64,
        _reply: &mut R,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        // TODO
        Err(FsError::NotImplemented)
    }

    async fn releasedir(
        &self,
        _req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        _flags: OpenInFlags,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        let removed = self.open_dirs.remove(OpenDirHandle::from(fh));
        let removed_ino = removed.dir_ino();
        assert_eq!(
            ino, removed_ino,
            "Releasedir handle does not match inode number. Expected: {ino:?}, dir cache: {removed_ino:?}",
        );

        Ok(())
    }

    async fn fsyncdir(
        &self,
        _req: &RequestInfo,
        _ino: InodeNumber,
        _fh: FileHandle,
        _datasync: bool,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        // TODO
        Err(FsError::NotImplemented)
    }

    async fn statfs(&self, _req: &RequestInfo, _ino: InodeNumber) -> FsResult<Statfs> {
        self.trigger_on_operation().await?;

        self.fs.read().await.get().statfs().await
    }

    async fn setxattr(
        &self,
        _req: &RequestInfo,
        _ino: InodeNumber,
        _name: &PathComponent,
        _value: &[u8],
        _flags: i32,
        _position: NumBytes,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        // TODO
        Err(FsError::NotImplemented)
    }

    async fn getxattr_numbytes(
        &self,
        _req: &RequestInfo,
        _ino: InodeNumber,
        _name: &PathComponent,
    ) -> FsResult<NumBytes> {
        self.trigger_on_operation().await?;

        // TODO
        return Err(FsError::NotImplemented);
    }

    async fn getxattr_data(
        &self,
        _req: &RequestInfo,
        _ino: InodeNumber,
        _name: &PathComponent,
        _max_bytes_to_read: NumBytes,
    ) -> FsResult<Vec<u8>> {
        self.trigger_on_operation().await?;

        // TODO
        return Err(FsError::NotImplemented);
    }

    async fn listxattr_numbytes(
        &self,
        _req: &RequestInfo,
        _ino: InodeNumber,
    ) -> FsResult<NumBytes> {
        self.trigger_on_operation().await?;

        // TODO
        return Err(FsError::NotImplemented);
    }

    async fn listxattr_data(
        &self,
        _req: &RequestInfo,
        _ino: InodeNumber,
        _max_bytes_to_read: NumBytes,
    ) -> FsResult<Vec<u8>> {
        self.trigger_on_operation().await?;

        // TODO
        return Err(FsError::NotImplemented);
    }

    async fn removexattr(
        &self,
        _req: &RequestInfo,
        _ino: InodeNumber,
        _name: &PathComponent,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        // TODO
        Err(FsError::NotImplemented)
    }

    async fn access(&self, _req: &RequestInfo, _ino: InodeNumber, _mask: i32) -> FsResult<()> {
        self.trigger_on_operation().await?;

        // TODO Should we implement access?
        Ok(())
    }

    async fn create(
        &self,
        req: &RequestInfo,
        parent_ino: InodeNumber,
        name: &PathComponent,
        mode: Mode,
        _umask: u32,
        flags: OpenInFlags,
    ) -> FsResult<ReplyCreate> {
        self.trigger_on_operation().await?;

        // In my tests with fuser 0.12.0, umask is already auto-applied to mode and the `umask` argument is always `0`.
        // TODO see https://github.com/cberner/fuser/issues/256
        let parent = self.get_inode(parent_ino).await?;
        with_async_drop_2!(parent, {
            let parent_dir = parent.as_dir().await?;
            // TODO Can we avoid the async_drop here by using something like dir.into_create_and_open_file() ?
            let (attr, child_node, open_file) = with_async_drop_2!(parent_dir, {
                parent_dir
                    .create_and_open_file(&name, mode, req.uid, req.gid, flags)
                    .await
            })?;

            // Fuser counts mkdir/create/symlink as a lookup and will call forget on the inode we allocate here.
            let child_ino = self
                .inodes
                .add(parent_ino, child_node, name.to_owned())
                .await
                .expect("Parent inode vanished while executing");

            let fh = self.open_files.add(open_file);
            Ok(ReplyCreate {
                ttl: TTL_CREATE,
                attr,
                ino: child_ino,
                fh: fh.handle,
                flags: OpenOutFlags {},
            })
        })
    }

    async fn getlk(
        &self,
        _req: &RequestInfo,
        _ino: InodeNumber,
        _fh: FileHandle,
        _lock_owner: u64,
        _start: u64,
        _end: u64,
        _typ: i32,
        _pid: u32,
    ) -> FsResult<ReplyLock> {
        self.trigger_on_operation().await?;

        // TODO
        Err(FsError::NotImplemented)
    }

    async fn setlk(
        &self,
        _req: &RequestInfo,
        _ino: InodeNumber,
        _fh: FileHandle,
        _lock_owner: u64,
        _start: u64,
        _end: u64,
        _typ: i32,
        _pid: u32,
        _sleep: bool,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        // TODO
        Err(FsError::NotImplemented)
    }

    async fn bmap(
        &self,
        _req: &RequestInfo,
        _ino: InodeNumber,
        _blocksize: NumBytes,
        _idx: u64,
    ) -> FsResult<ReplyBmap> {
        self.trigger_on_operation().await?;

        // TODO
        Err(FsError::NotImplemented)
    }

    /// control device
    async fn ioctl(
        &self,
        _req: &RequestInfo,
        _ino: InodeNumber,
        _fh: FileHandle,
        _flags: u32,
        _cmd: u32,
        _in_data: &[u8],
        _out_size: u32,
    ) -> FsResult<ReplyIoctl> {
        self.trigger_on_operation().await?;

        // TODO
        Err(FsError::NotImplemented)
    }

    async fn fallocate(
        &self,
        _req: &RequestInfo,
        _ino: InodeNumber,
        _fh: FileHandle,
        _offset: NumBytes,
        _length: NumBytes,
        _mode: Mode,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        // TODO
        Err(FsError::NotImplemented)
    }

    async fn lseek(
        &self,
        _req: &RequestInfo,
        _ino: InodeNumber,
        _fh: FileHandle,
        _offset: NumBytes,
        _whence: i32,
    ) -> FsResult<ReplyLseek> {
        self.trigger_on_operation().await?;

        // TODO
        Err(FsError::NotImplemented)
    }

    async fn copy_file_range(
        &self,
        _req: &RequestInfo,
        _ino_in: InodeNumber,
        _fh_in: FileHandle,
        _offset_in: NumBytes,
        _ino_out: InodeNumber,
        _fh_out: FileHandle,
        _offset_out: NumBytes,
        _len: NumBytes,
        _flags: u32,
    ) -> FsResult<ReplyWrite> {
        self.trigger_on_operation().await?;

        // TODO
        Err(FsError::NotImplemented)
    }

    #[cfg(target_os = "macos")]
    async fn setvolname(&self, req: &RequestInfo, name: &str) -> FsResult<()> {
        self.trigger_on_operation().await?;

        // TODO
        Err(FsError::NotImplemented)
    }

    #[cfg(target_os = "macos")]
    async fn exchange(
        &self,
        req: &RequestInfo,
        parent_ino: InodeNumber,
        name: &PathComponent,
        newparent_ino: InodeNumber,
        newname: &PathComponent,
        options: u64,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        // TODO
        Err(FsError::NotImplemented)
    }

    #[cfg(target_os = "macos")]
    async fn getxtimes(&self, req: &RequestInfo, ino: InodeNumber) -> FsResult<ReplyXTimes> {
        self.trigger_on_operation().await?;

        // TODO
        Err(FsError::NotImplemented)
    }
}

impl<Fs: Device> Debug for ObjectBasedFsAdapterLL<Fs>
where
    Fs: Device + AsyncDrop + Send + Sync + Debug + 'static,
    for<'a> Fs::File<'a>: Send,
    Fs::OpenFile: Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObjectBasedFsAdapterLL")
            .field("open_files", &self.open_files)
            .finish()
    }
}

#[async_trait]
impl<Fs> AsyncDrop for ObjectBasedFsAdapterLL<Fs>
where
    Fs: Device + AsyncDrop + Send + Sync + Debug + 'static,
    for<'a> Fs::File<'a>: Send,
    Fs::OpenFile: Send + Sync,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        // TODO Can we add a check here that inodes are empty? To ensure we've handled them correctly?
        //      Or is it actually allowed fuse behavior to keep files open and/or inodes active on shutdown?
        self.open_files.async_drop().await.unwrap();
        self.open_dirs.async_drop().await.unwrap();
        self.inodes.async_drop().await.unwrap();
        self.fs.write().await.async_drop().await.unwrap();
        Ok(())
    }
}
