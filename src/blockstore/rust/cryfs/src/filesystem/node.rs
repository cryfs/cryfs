use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::SystemTime;

use crate::filesystem::fsblobstore::FsBlobStore;
use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_rustfs::{object_based_api::Node, FsError, FsResult, Gid, Mode, NodeAttrs, Uid};
use cryfs_utils::async_drop::AsyncDrop;

pub struct CryNode<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send,
{
    blobstore: Arc<FsBlobStore<B>>,
    blob_id: BlobId,
}

#[async_trait]
impl<B> Node for CryNode<B>
where
    // TODO Do we really need B: 'static ?
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

    async fn utimens(
        &self,
        last_access: Option<SystemTime>,
        last_modification: Option<SystemTime>,
    ) -> FsResult<()> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }
}
