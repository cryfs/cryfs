use async_trait::async_trait;
use std::fmt::Debug;
use std::time::SystemTime;

use cryfs_blobstore::BlobStore;
use cryfs_rustfs::{
    object_based_api::OpenFile, FsError, FsResult, Gid, Mode, NodeAttrs, NumBytes, Uid,
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard},
    data::Data,
};

use super::node_info::NodeInfo;
use crate::filesystem::fsblobstore::FsBlobStore;

// TODO Make sure we don't keep a lock on the file blob, or keep the lock in an Arc that is shared between all File, Node and OpenFile instances of the same file

pub struct CryOpenFile<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
{
    blobstore: AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
    node_info: NodeInfo,
}

impl<B> CryOpenFile<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
{
    pub fn new(
        blobstore: AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
        node_info: NodeInfo,
    ) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            blobstore,
            node_info,
        })
    }
}

impl<B> Debug for CryOpenFile<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CryOpenFile")
            .field("node_info", &self.node_info)
            .finish()
    }
}

#[async_trait]
impl<B> OpenFile for CryOpenFile<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
{
    async fn getattr(&self) -> FsResult<NodeAttrs> {
        self.node_info.getattr(&self.blobstore).await
    }

    async fn chmod(&self, mode: Mode) -> FsResult<()> {
        self.node_info.chmod(&self.blobstore, mode).await
    }

    async fn chown(&self, uid: Option<Uid>, gid: Option<Gid>) -> FsResult<()> {
        self.node_info.chown(&self.blobstore, uid, gid).await
    }

    async fn truncate(&self, new_size: NumBytes) -> FsResult<()> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }

    async fn utimens(
        &self,
        last_access: Option<SystemTime>,
        last_modification: Option<SystemTime>,
    ) -> FsResult<()> {
        self.node_info
            .utimens(&self.blobstore, last_access, last_modification)
            .await
    }

    async fn read(&self, offset: NumBytes, size: NumBytes) -> FsResult<Data> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }

    async fn write(&self, offset: NumBytes, data: Data) -> FsResult<()> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }

    async fn flush(&self) -> FsResult<()> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }

    async fn fsync(&self, datasync: bool) -> FsResult<()> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }
}

#[async_trait]
impl<B> AsyncDrop for CryOpenFile<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), FsError> {
        self.blobstore.async_drop().await.map_err(|err| {
            log::error!("Failed to drop blobstore: {err:?}");
            FsError::Custom {
                error_code: libc::EIO,
            }
        })
    }
}
