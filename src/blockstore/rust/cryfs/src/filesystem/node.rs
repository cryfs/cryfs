use anyhow::Result;
use async_trait::async_trait;
use std::fmt::Debug;
use std::time::SystemTime;

use super::fsblobstore::FsBlob;
use crate::filesystem::fsblobstore::{BlobType, FsBlobStore};
use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_rustfs::{object_based_api::Node, FsError, FsResult, Gid, Mode, NodeAttrs, Uid};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};

pub struct CryNode<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    blobstore: &'a AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
    blob_id: BlobId,
    blob_type: BlobType,
}

impl<'a, B> CryNode<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    pub fn new(
        blobstore: &'a AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
        blob_type: BlobType,
        blob_id: BlobId,
    ) -> Self {
        Self {
            blobstore,
            blob_id,
            blob_type,
        }
    }

    pub fn node_type(&self) -> BlobType {
        self.blob_type
    }

    pub(super) fn blobstore(&self) -> &'a AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>> {
        self.blobstore
    }

    pub(super) fn blob_id(&self) -> &BlobId {
        &self.blob_id
    }

    pub(super) async fn load_blob(&self) -> Result<AsyncDropGuard<FsBlob<'a, B>>, FsError> {
        self.blobstore
            .load(&self.blob_id)
            .await
            .map_err(|err| {
                log::error!("Error loading blob {:?}: {:?}", &self.blob_id, err);
                FsError::UnknownError
            })?
            .ok_or_else(|| {
                log::error!("Blob {:?} not found", &self.blob_id);
                FsError::CorruptedFilesystem {
                    message: format!("Didn't find blob {:?}", self.blob_id),
                }
            })
    }
}

#[async_trait]
impl<'a, B> Node for CryNode<'a, B>
where
    // TODO Do we really need B: 'static ?
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
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
