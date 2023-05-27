use async_trait::async_trait;
use std::fmt::Debug;
use std::time::SystemTime;

use super::node::CryNode;
use cryfs_blobstore::BlobStore;
use cryfs_rustfs::{
    object_based_api::OpenFile, FsError, FsResult, Gid, Mode, NodeAttrs, NumBytes, Uid,
};
use cryfs_utils::async_drop::AsyncDrop;
use cryfs_utils::data::Data;

// TODO Make sure we don't keep a lock on the file blob, or keep the lock in an Arc that is shared between all File, Node and OpenFile instances of the same file

pub struct CryOpenFile<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send,
{
    node: CryNode<B>,
}

#[async_trait]
impl<B> OpenFile for CryOpenFile<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send,
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
