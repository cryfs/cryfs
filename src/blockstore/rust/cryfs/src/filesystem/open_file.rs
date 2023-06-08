use async_trait::async_trait;
use std::fmt::Debug;
use std::time::SystemTime;

use crate::filesystem::fsblobstore::FsBlobStore;
use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_rustfs::{
    object_based_api::OpenFile, FsError, FsResult, Gid, Mode, NodeAttrs, NumBytes, Uid,
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard},
    data::Data,
};

// TODO Make sure we don't keep a lock on the file blob, or keep the lock in an Arc that is shared between all File, Node and OpenFile instances of the same file

pub struct CryOpenFile<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
{
    // TODO Deduplicate with CryNode
    blobstore: AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
    blob_id: BlobId,
}

impl<B> CryOpenFile<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
{
    pub fn new(
        blobstore: AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
        blob_id: BlobId,
    ) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self { blobstore, blob_id })
    }
}

impl<B> Debug for CryOpenFile<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CryOpenFile")
            .field("blob_id", &self.blob_id)
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
        // TODO Implement
        Err(FsError::NotImplemented)
    }

    async fn chmod(&self, mode: Mode) -> FsResult<()> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }

    async fn chown(&self, uid: Option<Uid>, gid: Option<Gid>) -> FsResult<()> {
        // TODO Implement
        Err(FsError::NotImplemented)
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
        // TODO Implement
        Err(FsError::NotImplemented)
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
