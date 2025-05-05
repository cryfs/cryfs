use anyhow::Result;
use std::fmt::Debug;

use super::FilesystemDriver;
use crate::fixture::request_info;
use async_trait::async_trait;
use cryfs_blobstore::{BlobStoreOnBlocks, TrackingBlobStore};
use cryfs_blockstore::{
    DynBlockStore, HLSharedBlockStore, HLTrackingBlockStore, LockingBlockStore,
};
use cryfs_filesystem::filesystem::CryDevice;
use cryfs_rustfs::{
    AbsolutePath, AbsolutePathBuf, FsResult, Mode, NodeAttrs, OpenFlags, PathComponent,
    high_level_api::AsyncFilesystem as _, object_based_api::ObjectBasedFsAdapter,
};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};

pub struct FusemtFilesystemDriver {
    fs: AsyncDropGuard<
        ObjectBasedFsAdapter<
            CryDevice<
                AsyncDropArc<
                    TrackingBlobStore<
                        BlobStoreOnBlocks<
                            HLSharedBlockStore<
                                HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>,
                            >,
                        >,
                    >,
                >,
            >,
        >,
    >,
}

impl Debug for FusemtFilesystemDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FusemtFilesystemDriver")
    }
}

impl FilesystemDriver for FusemtFilesystemDriver {
    type NodeHandle = AbsolutePathBuf;

    async fn new(
        device: AsyncDropGuard<
            CryDevice<
                AsyncDropArc<
                    TrackingBlobStore<
                        BlobStoreOnBlocks<
                            HLSharedBlockStore<
                                HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>,
                            >,
                        >,
                    >,
                >,
            >,
        >,
    ) -> AsyncDropGuard<Self> {
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
            .create(request_info(), &path, Mode::default().add_file_flag(), 0)
            .await?;
        self.fs
            .release(
                request_info(),
                &path,
                new_file.fh,
                OpenFlags::Read,
                0,
                false,
            )
            .await?;
        Ok(path)
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
}

#[async_trait]
impl AsyncDrop for FusemtFilesystemDriver {
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<()> {
        self.fs.async_drop().await?;
        Ok(())
    }
}
