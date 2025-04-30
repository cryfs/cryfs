use cryfs_blobstore::BlobStoreOnBlocks;
use cryfs_blockstore::{
    DynBlockStore, HLSharedBlockStore, HLTrackingBlockStore, LockingBlockStore,
};
use cryfs_filesystem::filesystem::CryDevice;
use cryfs_rustfs::{
    AbsolutePath, FsError, FsResult, Mode,
    high_level_api::AsyncFilesystem,
    low_level_api::AsyncFilesystemLL,
    object_based_api::{FUSE_ROOT_ID, ObjectBasedFsAdapter, ObjectBasedFsAdapterLL},
};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};
use std::fmt::Debug;

use crate::fixture::request_info;

/// An interface abstracting over [AsyncFilesystem] and [AsyncFilesystemLL], offering common file system operations.
pub trait FilesystemTestExt: AsyncDrop + Debug {
    async fn new(
        device: AsyncDropGuard<
            CryDevice<
                AsyncDropArc<
                    BlobStoreOnBlocks<
                        HLSharedBlockStore<HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>>,
                    >,
                >,
            >,
        >,
    ) -> AsyncDropGuard<Self>
    where
        Self: Sized;

    async fn init(&self) -> Result<(), FsError>;
    async fn destroy(&self);
    async fn mkdir(&self, path: &AbsolutePath) -> FsResult<()>;
}

impl FilesystemTestExt
    for ObjectBasedFsAdapterLL<
        CryDevice<
            AsyncDropArc<
                BlobStoreOnBlocks<
                    HLSharedBlockStore<HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>>,
                >,
            >,
        >,
    >
{
    async fn new(
        device: AsyncDropGuard<
            CryDevice<
                AsyncDropArc<
                    BlobStoreOnBlocks<
                        HLSharedBlockStore<HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>>,
                    >,
                >,
            >,
        >,
    ) -> AsyncDropGuard<Self> {
        ObjectBasedFsAdapterLL::new(|_uid, _gid| device)
    }

    async fn init(&self) -> FsResult<()> {
        AsyncFilesystemLL::init(self, &request_info()).await
    }

    async fn destroy(&self) {
        AsyncFilesystemLL::destroy(self).await;
    }

    async fn mkdir(&self, path: &AbsolutePath) -> FsResult<()> {
        let (parent_path, name) = path.split_last().unwrap();
        let mut inos = vec![FUSE_ROOT_ID];
        for component in parent_path.iter() {
            let parent_ino = *inos.last().unwrap();
            let child_ino = AsyncFilesystemLL::lookup(self, &request_info(), parent_ino, component)
                .await?
                .ino
                .handle;
            inos.push(child_ino);
        }
        let parent_ino = *inos.last().unwrap();
        AsyncFilesystemLL::mkdir(
            self,
            &request_info(),
            parent_ino,
            name,
            Mode::default().add_dir_flag(),
            0,
        )
        .await?;
        for ino in inos.iter().skip(1).rev() {
            AsyncFilesystemLL::forget(self, &request_info(), *ino, 1).await?;
        }
        Ok(())
    }
}

impl FilesystemTestExt
    for ObjectBasedFsAdapter<
        CryDevice<
            AsyncDropArc<
                BlobStoreOnBlocks<
                    HLSharedBlockStore<HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>>,
                >,
            >,
        >,
    >
{
    async fn new(
        device: AsyncDropGuard<
            CryDevice<
                AsyncDropArc<
                    BlobStoreOnBlocks<
                        HLSharedBlockStore<HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>>,
                    >,
                >,
            >,
        >,
    ) -> AsyncDropGuard<Self> {
        ObjectBasedFsAdapter::new(|_uid, _gid| device)
    }

    async fn init(&self) -> FsResult<()> {
        AsyncFilesystem::init(self, request_info()).await
    }

    async fn destroy(&self) {
        AsyncFilesystem::destroy(self).await;
    }

    async fn mkdir(&self, path: &AbsolutePath) -> FsResult<()> {
        AsyncFilesystem::mkdir(self, request_info(), path, Mode::default().add_dir_flag()).await?;
        Ok(())
    }
}
