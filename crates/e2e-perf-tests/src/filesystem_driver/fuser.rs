use anyhow::Result;
use async_trait::async_trait;
use std::fmt::Debug;

use super::FilesystemDriver;
use crate::fixture::request_info;
use cryfs_blobstore::{BlobStoreOnBlocks, TrackingBlobStore};
use cryfs_blockstore::{
    DynBlockStore, HLSharedBlockStore, HLTrackingBlockStore, LockingBlockStore,
};
use cryfs_filesystem::filesystem::CryDevice;
use cryfs_rustfs::{
    FsResult, InodeNumber, Mode, PathComponent,
    low_level_api::AsyncFilesystemLL as _,
    object_based_api::{FUSE_ROOT_ID, ObjectBasedFsAdapterLL},
};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};

pub struct FuserFilesystemDriver {
    fs: AsyncDropGuard<
        ObjectBasedFsAdapterLL<
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

impl Debug for FuserFilesystemDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FuserFilesystemDriver")
    }
}

impl FilesystemDriver for FuserFilesystemDriver {
    type NodeHandle = InodeNumber;

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
        AsyncDropGuard::new(FuserFilesystemDriver {
            fs: ObjectBasedFsAdapterLL::new(|_uid, _gid| device),
        })
    }

    async fn init(&self) -> FsResult<()> {
        self.fs.init(&request_info()).await
    }

    async fn destroy(&self) {
        self.fs.destroy().await;
    }

    async fn mkdir(
        &self,
        parent: Option<InodeNumber>,
        name: &PathComponent,
    ) -> FsResult<InodeNumber> {
        Ok(self
            .fs
            .mkdir(
                &request_info(),
                parent.unwrap_or(FUSE_ROOT_ID),
                name,
                Mode::default().add_dir_flag(),
                0,
            )
            .await?
            .ino
            .handle)
    }

    async fn create_and_open_file(
        &self,
        parent: Option<Self::NodeHandle>,
        name: &PathComponent,
    ) -> FsResult<InodeNumber> {
        Ok(self
            .fs
            .create(
                &request_info(),
                parent.unwrap_or(FUSE_ROOT_ID),
                name,
                Mode::default().add_file_flag(),
                0,
                0,
            )
            .await?
            .ino
            .handle)
    }
}

#[async_trait]
impl AsyncDrop for FuserFilesystemDriver {
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<()> {
        self.fs.async_drop().await?;
        Ok(())
    }
}
