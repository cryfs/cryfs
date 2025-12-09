use anyhow::Result;
use cryfs_rustfs::DirEntryOrReference;
use std::sync::Mutex;
use std::{fmt::Debug, sync::Arc};

use super::FilesystemDriver;
use super::common::request_info;
use async_trait::async_trait;
use cryfs_blobstore::{BlobStoreOnBlocks, TrackingBlobStore};
use cryfs_blockstore::{
    DynBlockStore, HLSharedBlockStore, HLTrackingBlockStore, LockingBlockStore,
};
use cryfs_filesystem::filesystem::CryDevice;
use cryfs_rustfs::{
    AbsolutePath, AbsolutePathBuf, Callback, FileHandle, FsResult, Gid, Mode, NodeAttrs, NodeKind,
    NumBytes, OpenInFlags, PathComponent, Statfs, Uid, high_level_api::AsyncFilesystem as _,
    object_based_api::ObjectBasedFsAdapter,
};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};
use std::time::SystemTime;

type Device = CryDevice<
    AsyncDropArc<
        TrackingBlobStore<
            BlobStoreOnBlocks<
                HLSharedBlockStore<HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>>,
            >,
        >,
    >,
>;

/// A [FilesystemDriver] implementation using the high-level Api from [rustfs], i.e. [ObjectBasedFsAdapter].
pub struct FusemtFilesystemDriver {
    fs: AsyncDropGuard<ObjectBasedFsAdapter<Device>>,
}

impl Debug for FusemtFilesystemDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FusemtFilesystemDriver")
    }
}

impl FilesystemDriver for FusemtFilesystemDriver {
    type NodeHandle = AbsolutePathBuf;

    type FileHandle = FileHandle;

    async fn new(device: AsyncDropGuard<Device>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            fs: ObjectBasedFsAdapter::new(|_uid, _gid| device),
        })
    }

    async fn init(&self) -> FsResult<()> {
        self.fs.init(request_info()).await
    }

    async fn destroy(&self) {
        self.fs.destroy().await;
    }

    async fn reset_cache_after_setup(&self) {
        self.fs.reset_cache_after_setup().await;
    }

    async fn reset_cache_after_test(&self) {
        // ObjectBasedFsAdapter doesn't have a cache to reset
    }

    async fn lookup(
        &self,
        parent: Option<AbsolutePathBuf>,
        name: &PathComponent,
    ) -> FsResult<AbsolutePathBuf> {
        // Fuse-mt doesn't really have a lookup operation, we can directly combine the path
        Ok(parent.unwrap_or_else(AbsolutePathBuf::root).join(name))
    }

    async fn mkdir(
        &self,
        parent: Option<AbsolutePathBuf>,
        name: &PathComponent,
    ) -> FsResult<AbsolutePathBuf> {
        let path = parent.unwrap_or_else(AbsolutePathBuf::root).join(name);
        self.fs
            .mkdir(request_info(), &path, Mode::default().add_dir_flag())
            .await?;
        Ok(path)
    }

    async fn create_file(
        &self,
        parent: Option<AbsolutePathBuf>,
        name: &PathComponent,
    ) -> FsResult<AbsolutePathBuf> {
        let path = parent.unwrap_or_else(AbsolutePathBuf::root).join(name);
        let new_file = self
            .fs
            .create(request_info(), &path, Mode::default().add_file_flag(), OpenInFlags::ReadWrite)
            .await?;
        self.fs
            .release(
                request_info(),
                &path,
                new_file.fh,
                OpenInFlags::ReadWrite,
                0,
                false,
            )
            .await?;
        Ok(path)
    }

    async fn create_and_open_file(
        &self,
        parent: Option<Self::NodeHandle>,
        name: &PathComponent,
    ) -> FsResult<(Self::NodeHandle, FileHandle)> {
        let path = parent.unwrap_or_else(AbsolutePathBuf::root).join(name);
        let new_file = self
            .fs
            .create(request_info(), &path, Mode::default().add_file_flag(), OpenInFlags::ReadWrite)
            .await?;
        Ok((path, new_file.fh))
    }

    async fn create_symlink(
        &self,
        parent: Option<Self::NodeHandle>,
        name: &PathComponent,
        target: &AbsolutePath,
    ) -> FsResult<Self::NodeHandle> {
        let path = parent.unwrap_or_else(AbsolutePathBuf::root).join(name);
        self.fs.symlink(request_info(), &path, target).await?;
        Ok(path)
    }

    async fn getattr(&self, node: Option<AbsolutePathBuf>) -> FsResult<NodeAttrs> {
        self.fs
            .getattr(
                request_info(),
                &node.unwrap_or_else(AbsolutePathBuf::root),
                None,
            )
            .await
            .map(|attr_response| attr_response.attrs)
    }

    async fn fgetattr(&self, node: AbsolutePathBuf, open_file: &FileHandle) -> FsResult<NodeAttrs> {
        self.fs
            .getattr(request_info(), &node, Some(*open_file))
            .await
            .map(|attr_response| attr_response.attrs)
    }

    async fn chmod(&self, node: Option<Self::NodeHandle>, mode: Mode) -> FsResult<()> {
        let path = node.unwrap_or_else(AbsolutePathBuf::root);
        self.fs.chmod(request_info(), &path, None, mode).await
    }

    async fn fchmod(
        &self,
        node: Self::NodeHandle,
        open_file: &FileHandle,
        mode: Mode,
    ) -> FsResult<()> {
        self.fs
            .chmod(request_info(), &node, Some(*open_file), mode)
            .await
    }

    async fn chown(
        &self,
        node: Option<Self::NodeHandle>,
        uid: Option<Uid>,
        gid: Option<Gid>,
    ) -> FsResult<()> {
        let path = node.unwrap_or_else(AbsolutePathBuf::root);
        self.fs.chown(request_info(), &path, None, uid, gid).await
    }

    async fn fchown(
        &self,
        node: Self::NodeHandle,
        open_file: &FileHandle,
        uid: Option<Uid>,
        gid: Option<Gid>,
    ) -> FsResult<()> {
        self.fs
            .chown(request_info(), &node, Some(*open_file), uid, gid)
            .await
    }

    async fn truncate(&self, node: Option<Self::NodeHandle>, size: NumBytes) -> FsResult<()> {
        let path = node.unwrap_or_else(AbsolutePathBuf::root);
        self.fs.truncate(request_info(), &path, None, size).await
    }

    async fn ftruncate(
        &self,
        node: Self::NodeHandle,
        open_file: &FileHandle,
        size: NumBytes,
    ) -> FsResult<()> {
        self.fs
            .truncate(request_info(), &node, Some(*open_file), size)
            .await
    }

    async fn utimens(
        &self,
        node: Option<Self::NodeHandle>,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
    ) -> FsResult<()> {
        let path = node.unwrap_or_else(AbsolutePathBuf::root);
        self.fs
            .utimens(request_info(), &path, None, atime, mtime)
            .await
    }

    async fn futimens(
        &self,
        node: Self::NodeHandle,
        open_file: &FileHandle,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
    ) -> FsResult<()> {
        self.fs
            .utimens(request_info(), &node, Some(*open_file), atime, mtime)
            .await
    }

    async fn readlink(&self, node: Self::NodeHandle) -> FsResult<AbsolutePathBuf> {
        let target = self.fs.readlink(request_info(), &node).await?;
        Ok(AbsolutePathBuf::try_from_string(target).unwrap())
    }

    async fn open(&self, node: Self::NodeHandle) -> FsResult<FileHandle> {
        let open_file = self
            .fs
            .open(request_info(), &node, OpenInFlags::ReadWrite)
            .await?;
        Ok(open_file.fh)
    }

    async fn release(&self, node: Self::NodeHandle, open_file: FileHandle) -> FsResult<()> {
        // The fuse sequence for releasing a file in fuse is: first flush, then release
        self.fs.flush(request_info(), &node, open_file, 0).await?;
        self.fs
            .release(
                request_info(),
                &node,
                open_file,
                OpenInFlags::ReadWrite,
                0,
                false,
            )
            .await
    }

    async fn statfs(&self) -> FsResult<Statfs> {
        self.fs.statfs(request_info(), AbsolutePath::root()).await
    }

    async fn readdir(&self, node: Option<Self::NodeHandle>) -> FsResult<Vec<(String, NodeKind)>> {
        let node = node.unwrap_or_else(AbsolutePathBuf::root);
        let fh = self.fs.opendir(request_info(), &node, OpenInFlags::Read).await?.fh;
        let entries = self.fs.readdir(request_info(), &node, fh).await?;
        Ok(entries
            .into_iter()
            .map(|entry| match entry {
                DirEntryOrReference::Entry(entry) => (entry.name.to_string(), entry.kind),
                DirEntryOrReference::SelfReference => (".".to_string(), NodeKind::Dir),
                DirEntryOrReference::ParentReference => ("..".to_string(), NodeKind::Dir),
            })
            .collect())
    }

    async fn unlink(&self, parent: Option<Self::NodeHandle>, name: &PathComponent) -> FsResult<()> {
        let path = parent.unwrap_or_else(AbsolutePathBuf::root).join(name);
        self.fs.unlink(request_info(), &path).await
    }

    async fn rmdir(&self, parent: Option<Self::NodeHandle>, name: &PathComponent) -> FsResult<()> {
        let path = parent.unwrap_or_else(AbsolutePathBuf::root).join(name);
        self.fs.rmdir(request_info(), &path).await
    }

    async fn read(
        &self,
        node: Self::NodeHandle,
        open_file: &mut FileHandle,
        offset: NumBytes,
        size: NumBytes,
    ) -> FsResult<Vec<u8>> {
        let data = Arc::new(Mutex::new(None));
        self.fs
            .read(
                request_info(),
                &node,
                *open_file,
                offset,
                size,
                ReadCallbackImpl {
                    data: Arc::clone(&data),
                },
            )
            .await;
        Arc::into_inner(data)
            .unwrap()
            .into_inner()
            .unwrap()
            .unwrap()
    }

    async fn write(
        &self,
        node: Self::NodeHandle,
        open_file: &mut FileHandle,
        offset: NumBytes,
        data: Vec<u8>,
    ) -> FsResult<()> {
        let len = NumBytes::from(data.len() as u64);
        let written = self
            .fs
            .write(request_info(), &node, *open_file, offset, data, 0)
            .await?;
        assert_eq!(written, len);
        Ok(())
    }

    async fn rename(
        &self,
        old_parent: Option<Self::NodeHandle>,
        old_name: &PathComponent,
        new_parent: Option<Self::NodeHandle>,
        new_name: &PathComponent,
    ) -> FsResult<()> {
        let old_path = old_parent
            .unwrap_or_else(AbsolutePathBuf::root)
            .join(old_name);
        let new_path = new_parent
            .unwrap_or_else(AbsolutePathBuf::root)
            .join(new_name);
        self.fs.rename(request_info(), &old_path, &new_path).await
    }

    async fn fsync(
        &self,
        node: Self::NodeHandle,
        open_file: &mut FileHandle,
        datasync: bool,
    ) -> FsResult<()> {
        self.fs
            .fsync(request_info(), &node, *open_file, datasync)
            .await
    }
}

#[async_trait]
impl AsyncDrop for FusemtFilesystemDriver {
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<()> {
        self.fs.async_drop().await?;
        Ok(())
    }
}

#[derive(Default)]
struct ReadCallbackImpl {
    data: Arc<Mutex<Option<FsResult<Vec<u8>>>>>,
}

impl<'a> Callback<FsResult<&'a [u8]>, ()> for ReadCallbackImpl {
    fn call(self, result: FsResult<&'a [u8]>) {
        *self.data.lock().unwrap() = Some(result.map(|data| data.to_vec()));
    }
}
