use async_trait::async_trait;
use std::fmt::Debug;
use std::time::SystemTime;
use tokio::sync::OnceCell;

use super::fsblobstore::FsBlob;
use super::node_info::NodeInfo;
use crate::filesystem::fsblobstore::{BlobType, FsBlobStore};
use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_rustfs::{object_based_api::Node, FsResult, Gid, Mode, NodeAttrs, PathComponentBuf, Uid};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};

pub struct CryNode<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    blobstore: &'a AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
    node_info: NodeInfo,
}

impl<'a, B> CryNode<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    pub async fn load_blob(&self) -> FsResult<AsyncDropGuard<FsBlob<'a, B>>> {
        self.node_info.load_blob(&self.blobstore).await
    }

    pub async fn blob_id(&self) -> FsResult<BlobId> {
        Ok(self.node_info.blob_details(&self.blobstore).await?.blob_id)
    }

    pub async fn node_type(&self) -> FsResult<BlobType> {
        Ok(self
            .node_info
            .blob_details(&self.blobstore)
            .await?
            .blob_type)
    }
}

impl<'a, B> CryNode<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    pub fn new(
        blobstore: &'a AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
        parent_blob_id: BlobId,
        name: PathComponentBuf,
    ) -> Self {
        Self {
            blobstore,
            node_info: NodeInfo::IsNotRootDir {
                parent_blob_id,
                name,
                blob_details: OnceCell::default(),
            },
        }
    }

    pub fn new_rootdir(
        blobstore: &'a AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
        root_blob_id: BlobId,
    ) -> Self {
        Self {
            blobstore,
            node_info: NodeInfo::IsRootDir { root_blob_id },
        }
    }

    pub(super) fn blobstore(&self) -> &'a AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>> {
        self.blobstore
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
        self.node_info.getattr(&self.blobstore).await
    }

    async fn chmod(&self, mode: Mode) -> FsResult<()> {
        self.node_info.chmod(&self.blobstore, mode).await
    }

    async fn chown(&self, uid: Option<Uid>, gid: Option<Gid>) -> FsResult<()> {
        self.node_info.chown(&self.blobstore, uid, gid).await
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
}
