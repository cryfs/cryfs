use async_trait::async_trait;
use futures::{
    join,
    stream::{FuturesOrdered, StreamExt},
};
use std::fmt::Debug;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

use super::utils::{MaybeInitializedFs, OpenFileList};
use super::{Device, Dir, File, Node, OpenFile, Symlink};
#[cfg(target_os = "macos")]
use crate::low_level_api::ReplyXTimes;
use crate::low_level_api::{
    AsyncFilesystemLL, IntoFsLL, ReplyAttr, ReplyBmap, ReplyCreate, ReplyDirectoryAddResult,
    ReplyEntry, ReplyIoctl, ReplyLock, ReplyLseek, ReplyOpen, ReplyWrite,
};
use crate::{NodeKind, PathComponentBuf, common::DirEntry};
use crate::{
    common::{
        Callback, FileHandle, FsError, FsResult, Gid, HandleMap, HandleWithGeneration, InodeNumber,
        Mode, NumBytes, OpenFlags, PathComponent, RequestInfo, Statfs, Uid,
    },
    low_level_api::{ReplyDirectory, ReplyDirectoryPlus},
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard, flatten_async_drop},
    with_async_drop_2,
};

pub const FUSE_ROOT_ID: InodeNumber = InodeNumber::from_const(fuser::FUSE_ROOT_ID);

// TODO What are good TTLs here?
const TTL_LOOKUP: Duration = Duration::from_secs(1);
const TTL_GETATTR: Duration = Duration::from_secs(1);
const TTL_CREATE: Duration = Duration::from_secs(1);
const TTL_SYMLINK: Duration = Duration::from_secs(1);

mod inode_info {
    use super::*;

    #[derive(Debug)]
    pub struct InodeInfo<Fs>
    where
        Fs: Device + Debug,
    {
        node: AsyncDropGuard<AsyncDropArc<Fs::Node>>,
        parent_inode: InodeNumber,
    }

    impl<Fs> InodeInfo<Fs>
    where
        Fs: Device + Debug,
    {
        pub fn new(
            node: AsyncDropGuard<AsyncDropArc<Fs::Node>>,
            parent_inode: InodeNumber,
        ) -> AsyncDropGuard<Self> {
            AsyncDropGuard::new(Self { node, parent_inode })
        }

        pub fn parent_inode(&self) -> InodeNumber {
            self.parent_inode
        }

        pub fn node(&self) -> AsyncDropGuard<AsyncDropArc<Fs::Node>> {
            AsyncDropArc::clone(&self.node)
        }

        #[cfg(feature = "testutils")]
        pub async fn fsync(&self) -> FsResult<()> {
            self.node.fsync(false).await
        }
    }

    #[async_trait]
    impl<Fs> AsyncDrop for InodeInfo<Fs>
    where
        Fs: Device + Debug,
    {
        type Error = FsError;
        async fn async_drop_impl(&mut self) -> FsResult<()> {
            self.node.async_drop().await?;
            Ok(())
        }
    }
}
use inode_info::InodeInfo;

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

    // TODO Do we need Arc for inodes?
    // TODO InodeInfo here holds a reference to the ConcurrentFsBlob, which blocks the blob from being removed. This would be a deadlock in unlink/rmdir if we store a reference to the self blob in NodeInfo.
    //      Right now, we only store a reference to the parent blob and that's fine because child inodes are forgotten before the parent can be removed.
    inodes: Arc<RwLock<AsyncDropGuard<HandleMap<InodeNumber, InodeInfo<Fs>>>>>,

    open_files: AsyncDropGuard<OpenFileList<Fs::OpenFile>>,
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
        let mut inodes = HandleMap::new();

        Self::block_root_handle(&mut inodes);

        AsyncDropGuard::new(Self {
            fs: Arc::new(RwLock::new(MaybeInitializedFs::new_uninitialized(
                Box::new(fs),
            ))),
            inodes: Arc::new(RwLock::new(inodes)),
            open_files: OpenFileList::new(),
        })
    }

    fn block_root_handle(inodes: &mut HandleMap<InodeNumber, InodeInfo<Fs>>) {
        // We need to block zero because fuse seems to dislike it.
        inodes.block_handle(InodeNumber::from(0));
        // FUSE_ROOT_ID represents the root directory. We can't use it for other inodes.
        if fuser::FUSE_ROOT_ID != 0 {
            inodes.block_handle(FUSE_ROOT_ID);
        }
    }

    #[cfg(feature = "testutils")]
    pub async fn reset_cache_after_setup(&self) {
        // clear inodes
        let mut inodes = self.inodes.write().await;
        for (_handle, mut object) in inodes.drain() {
            object.fsync().await.unwrap();
            object.async_drop().await.unwrap();
        }
        Self::block_root_handle(&mut inodes);

        // flush open files
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

    #[cfg(feature = "testutils")]
    pub async fn reset_cache_after_test(&self) {
        let mut inodes = self.inodes.write().await;
        for (_handle, mut object) in inodes.drain() {
            object.async_drop().await.unwrap();
        }
        Self::block_root_handle(&mut inodes);
    }

    #[cfg(feature = "testutils")]
    pub async fn flush_cache(&self) {
        let inodes = self.inodes.write().await;
        for (_handle, object) in inodes.iter() {
            object.fsync().await.unwrap();
        }
        self._flush_open_files().await;
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
    ) -> FsResult<(AsyncDropGuard<AsyncDropArc<Fs::Node>>, Option<InodeNumber>)> {
        // TODO Once async closures are stable, we can - instead of returning an AsyncDropArc - take a callback parameter and pass &Fs::Node to it.
        //      That would simplify all the call sites (e.g. don't require them to call async_drop on the returned value anymore).
        //      See https://stackoverflow.com/questions/76625378/async-closure-holding-reference-over-await-point
        if ino == FUSE_ROOT_ID {
            let fs = self.fs.read().await;
            let fs = fs.get();
            let node = Dir::into_node(fs.rootdir().await?);
            Ok((AsyncDropArc::new(node), None))
        } else {
            let inodes = self.inodes.read().await;
            let inode = inodes.get(ino).expect("Error: Inode number unassigned");
            Ok((inode.node(), Some(inode.parent_inode())))
        }
    }

    async fn get_inode(
        &self,
        ino: InodeNumber,
    ) -> FsResult<AsyncDropGuard<AsyncDropArc<Fs::Node>>> {
        Ok(self.get_inode_and_parent_ino(ino).await?.0)
    }

    async fn add_inode(
        &self,
        parent_ino: InodeNumber,
        node: AsyncDropGuard<Fs::Node>,
        name: &PathComponent,
    ) -> HandleWithGeneration<InodeNumber> {
        let child_ino = self
            .inodes
            .write()
            .await
            .add(InodeInfo::new(AsyncDropArc::new(node), parent_ino));
        log::info!("New inode {child_ino:?}: parent={parent_ino:?}, name={name}");
        child_ino
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
        self.fs.write().await.initialize(req.uid, req.gid);
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

        // TODO Will lookup() be called multiple times with the same parent+name, before the previous one is forgotten, and is it ok to give the second call a different inode while the first call is still ongoing?
        let parent_node = self.get_inode(parent_ino).await?;
        let mut child = with_async_drop_2!(parent_node, {
            let parent_node_dir = parent_node
                .as_dir()
                .await
                .expect("Error: Inode number is not a directory");
            with_async_drop_2!(parent_node_dir, {
                // TODO Can we avoid the async_drop here by using something like parent_node_dir.into_lookup_child() ?
                parent_node_dir.lookup_child(&name).await
            })
        })?;

        // TODO async_drop parent_node concurrently with the child.getattr() call below.

        match child.getattr().await {
            Ok(attr) => {
                let ino = self.add_inode(parent_ino, child, name).await;
                Ok(ReplyEntry {
                    ttl: TTL_LOOKUP,
                    ino,
                    attr,
                })
            }
            Err(err) => {
                child.async_drop().await?;
                Err(err)
            }
        }
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
        // But we don't reuse inode numbers so nlookup should always be 1.
        assert_eq!(
            1, nlookup,
            "We don't reuse inode numbers so nlookup should always be 1"
        );

        let mut entry = self.inodes.write().await.remove(ino);
        entry.async_drop().await?;
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
        crtime: Option<SystemTime>,
        chgtime: Option<SystemTime>,
        bkuptime: Option<SystemTime>,
        flags: Option<u32>,
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

        // TODO What to do with crtime, chgtime, bkuptime, flags?
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
        req: &RequestInfo,
        parent_ino: InodeNumber,
        name: &PathComponent,
        mode: Mode,
        umask: u32,
        rdev: u32,
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
                Ok((attrs, Dir::into_node(child)))
            })
        })?;
        // Fuser counts mkdir/create/symlink as a lookup and will call forget on the inode we allocate here.
        let ino = self.add_inode(parent_ino, child, name).await;
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
                parent_dir.remove_child_file_or_symlink(&name).await
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
            with_async_drop_2!(parent_dir, { parent_dir.remove_child_dir(&name).await })?;
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
                Ok((attrs, Symlink::into_node(child)))
            })
        })?;
        // Fuser counts mkdir/create/symlink as a lookup and will call forget on the inode we allocate here.
        // TODO Check this is actually the case for symlink, I only checked mkdir so far
        let ino = self.add_inode(parent_ino, child, name).await;
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
                    parent_dir.rename_child(&oldname, &newname).await
                })?;
                Ok(())
            })
        } else {
            let (oldparent, newparent) =
                join!(self.get_inode(oldparent_ino), self.get_inode(newparent_ino));
            let (mut oldparent, mut newparent) = flatten_async_drop(oldparent, newparent).await?;
            let result = (async || {
                let oldparent_dir = oldparent.as_dir().await?;
                with_async_drop_2!(oldparent_dir, {
                    let newparent_dir = newparent.as_dir().await?;
                    oldparent_dir
                        .move_child_to(oldname, newparent_dir, newname)
                        .await
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
        req: &RequestInfo,
        ino: InodeNumber,
        newparent_ino: InodeNumber,
        newname: &PathComponent,
    ) -> FsResult<ReplyEntry> {
        self.trigger_on_operation().await?;

        // TODO
        Err(FsError::NotImplemented)
    }

    async fn open(
        &self,
        _req: &RequestInfo,
        ino: InodeNumber,
        flags: OpenFlags,
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
                // TODO What flags to return here? Just same as the argument?
                flags,
            })
        })
    }

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
        _flags: i32,
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
        _ino: InodeNumber,
        _flags: i32,
    ) -> FsResult<ReplyOpen> {
        self.trigger_on_operation().await?;

        // We don't need opendir/releasedir because readdir works directly on the inode.
        Ok(ReplyOpen {
            fh: FileHandle::from(0),
            flags: OpenFlags::ReadWrite,
        })
    }

    async fn readdir<R: ReplyDirectory + Send + 'static>(
        &self,
        _req: &RequestInfo,
        ino: InodeNumber,
        _fh: FileHandle,
        offset: u64,
        reply: &mut R,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        // TODO Allow readdir on fh instead of ino?
        let (node, parent_ino) = self.get_inode_and_parent_ino(ino).await?;

        with_async_drop_2!(node, {
            let parent_ino = parent_ino.unwrap_or(FUSE_ROOT_ID); // root is parent of itself
            let dir = node.as_dir().await?;
            with_async_drop_2!(dir, {
                // TODO Instead of loading all entries and then possibly only returning some because the buffer gets full, try to only load the necessary ones? But concurrently somehow?
                let entries = dir.entries().await?;
                let offset = usize::try_from(offset).unwrap(); // TODO No unwrap

                if offset == 0 {
                    match reply.add_self_reference(ino, 0) {
                        ReplyDirectoryAddResult::Full => {
                            return Ok(());
                        }
                        ReplyDirectoryAddResult::NotFull => {
                            // continue
                        }
                    }
                }

                if offset <= 1 {
                    match reply.add_parent_reference(parent_ino, 1) {
                        ReplyDirectoryAddResult::Full => {
                            return Ok(());
                        }
                        ReplyDirectoryAddResult::NotFull => {
                            // continue
                        }
                    }
                }

                // TODO Add tests for the offset calculations including '.' and '..' here
                let entries = entries
                    .into_iter()
                    .enumerate()
                    .skip(offset.saturating_sub(2)) // skip 2 less because those offset indices are for '.' and '..'
                    .map(async |(offset, entry)| {
                        let offset = offset + 2; // offset + 2 because of '.' and '..'
                        let child = dir.lookup_child(&entry.name).await.unwrap(); // TODO No unwrap

                        // TODO Check that readdir is actually supposed to register the inode and that [Self::forget] will be called for this inode
                        //      Note also that fuse-mt actually doesn't register the inode here and a comment there claims that fuse just ignores it, see https://github.com/wfraser/fuse-mt/blob/881d7320b4c73c0bfbcbca48a5faab2a26f3e9e8/src/fusemt.rs#L619
                        //      fuse documentation says it shouldn't lookup: https://libfuse.github.io/doxygen/structfuse__lowlevel__ops.html#af1ef8e59e0cb0b02dc0e406898aeaa51
                        //      (but readdirplus should? see https://github.com/libfuse/libfuse/blob/7b9e7eeec6c43a62ab1e02dfb6542e6bfb7f72dc/include/fuse_lowlevel.h#L1209 )
                        //      I think for readdir, the correct behavior might be: return ino if in cache, otherwise return -1. Or just always return -1. See the `readdir_ino` config of libfuse.
                        let child_ino = self.add_inode(ino, child, &entry.name).await;
                        (offset, child_ino.handle, entry)
                    });
                // TODO Does this need to be FuturesOrdered or can we reply them without order?
                let mut entries: FuturesOrdered<_> = entries.collect();
                while let Some((offset, ino, entry)) = entries.next().await {
                    // offset+1 because fuser actually expects us to return the offset of the **next** entry
                    let offset = i64::try_from(offset).unwrap() + 1; // TODO No unwrap
                    let buffer_is_full = reply.add(ino.into(), offset, entry.kind, &entry.name);
                    match buffer_is_full {
                        ReplyDirectoryAddResult::Full => {
                            // TODO Can we cancel the stream if the buffer is full?
                            // TODO Test the scenario where a directory has lots of entries, the buffer gets full and fuser calls readdir() multiple times
                            break;
                        }
                        ReplyDirectoryAddResult::NotFull => {
                            // continue
                        }
                    }
                }

                Ok(())
            })
        })
    }

    async fn readdirplus<R: ReplyDirectoryPlus + Send + 'static>(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        offset: u64,
        reply: &mut R,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        // TODO
        Err(FsError::NotImplemented)
    }

    async fn releasedir(
        &self,
        _req: &RequestInfo,
        _ino: InodeNumber,
        _fh: FileHandle,
        _flags: i32,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        // We don't need opendir/releasedir because readdir works directly on the inode.
        Ok(())
    }

    async fn fsyncdir(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        datasync: bool,
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
        req: &RequestInfo,
        ino: InodeNumber,
        name: &PathComponent,
        value: &[u8],
        flags: i32,
        position: NumBytes,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        // TODO
        Err(FsError::NotImplemented)
    }

    async fn getxattr_numbytes(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        name: &PathComponent,
    ) -> FsResult<NumBytes> {
        self.trigger_on_operation().await?;

        // TODO
        return Err(FsError::NotImplemented);
    }

    async fn getxattr_data(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        name: &PathComponent,
        max_bytes_to_read: NumBytes,
    ) -> FsResult<Vec<u8>> {
        self.trigger_on_operation().await?;

        // TODO
        return Err(FsError::NotImplemented);
    }

    async fn listxattr_numbytes(&self, req: &RequestInfo, ino: InodeNumber) -> FsResult<NumBytes> {
        self.trigger_on_operation().await?;

        // TODO
        return Err(FsError::NotImplemented);
    }

    async fn listxattr_data(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        max_bytes_to_read: NumBytes,
    ) -> FsResult<Vec<u8>> {
        self.trigger_on_operation().await?;

        // TODO
        return Err(FsError::NotImplemented);
    }

    async fn removexattr(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        name: &PathComponent,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        // TODO
        Err(FsError::NotImplemented)
    }

    async fn access(&self, req: &RequestInfo, ino: InodeNumber, mask: i32) -> FsResult<()> {
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
        flags: i32,
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
                    .create_and_open_file(&name, mode, req.uid, req.gid)
                    .await
            })?;

            // Fuser counts mkdir/create/symlink as a lookup and will call forget on the inode we allocate here.
            // TODO Check this is actually the case for create, I only checked mkdir so far
            //      Note also that fuse-mt actually doesn't register the inode here and a comment there claims that fuse just ignores it, see https://github.com/wfraser/fuse-mt/blob/881d7320b4c73c0bfbcbca48a5faab2a26f3e9e8/src/fusemt.rs#L619
            let child_ino = self.add_inode(parent_ino, child_node, name).await;

            let fh = self.open_files.add(open_file);
            Ok(ReplyCreate {
                ttl: TTL_CREATE,
                attr,
                ino: child_ino,
                fh: fh.handle,
                // TODO Do we need to change flags or is it ok to just return the flags passed in? If it's ok, then why do we have to return them?
                flags: u32::try_from(flags).unwrap(), // TODO No unwrap?
            })
        })
    }

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
    ) -> FsResult<ReplyLock> {
        self.trigger_on_operation().await?;

        // TODO
        Err(FsError::NotImplemented)
    }

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
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        // TODO
        Err(FsError::NotImplemented)
    }

    async fn bmap(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        blocksize: NumBytes,
        idx: u64,
    ) -> FsResult<ReplyBmap> {
        self.trigger_on_operation().await?;

        // TODO
        Err(FsError::NotImplemented)
    }

    /// control device
    async fn ioctl(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        flags: u32,
        cmd: u32,
        in_data: &[u8],
        out_size: u32,
    ) -> FsResult<ReplyIoctl> {
        self.trigger_on_operation().await?;

        // TODO
        Err(FsError::NotImplemented)
    }

    async fn fallocate(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        offset: NumBytes,
        length: NumBytes,
        mode: Mode,
    ) -> FsResult<()> {
        self.trigger_on_operation().await?;

        // TODO
        Err(FsError::NotImplemented)
    }

    async fn lseek(
        &self,
        req: &RequestInfo,
        ino: InodeNumber,
        fh: FileHandle,
        offset: NumBytes,
        whence: i32,
    ) -> FsResult<ReplyLseek> {
        self.trigger_on_operation().await?;

        // TODO
        Err(FsError::NotImplemented)
    }

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

impl<Fn, Fs> IntoFsLL<ObjectBasedFsAdapterLL<Fs>> for Fn
where
    Fn: FnOnce(Uid, Gid) -> AsyncDropGuard<Fs> + Send + Sync + 'static,
    Fs: Device + AsyncDrop + Send + Sync + Debug + 'static,
    for<'a> Fs::File<'a>: Send,
    Fs::OpenFile: Send + Sync,
{
    fn into_fs(self) -> AsyncDropGuard<ObjectBasedFsAdapterLL<Fs>> {
        ObjectBasedFsAdapterLL::new(self)
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
        self.inodes.write().await.async_drop().await.unwrap();
        self.fs.write().await.async_drop().await.unwrap();
        Ok(())
    }
}
