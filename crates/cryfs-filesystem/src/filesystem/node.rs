use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::SystemTime;

use super::CryDevice;
use super::node_info::NodeInfo;
use super::{dir::CryDir, file::CryFile, symlink::CrySymlink};
use crate::filesystem::concurrentfsblobstore::{ConcurrentFsBlob, ConcurrentFsBlobStore};
use crate::filesystem::fsblobstore::BlobType;
use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_rustfs::{
    FsError, FsResult, Gid, Mode, NodeAttrs, NumBytes, Uid, object_based_api::Node,
};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};

pub struct CryNode<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    blobstore: AsyncDropGuard<AsyncDropArc<ConcurrentFsBlobStore<B>>>,

    // node_info is an `Arc` so that when we call [Self::as_dir], [Self::as_file] or [Self::as_symlink]
    // and those instances change the `NodeInfo` (e.g. load its cache), that loaded cache transfers to
    // the [CryNode] instance as well. This is important because [cryfs_rustfs] keeps the [CryNode]
    // instance in its `inode_table` and potentially reuses it.
    node_info: AsyncDropGuard<AsyncDropArc<NodeInfo>>,
}

impl<B> CryNode<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    pub async fn load_blob(&self) -> FsResult<AsyncDropGuard<ConcurrentFsBlob<B>>> {
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

impl<B> CryNode<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    pub fn new(
        blobstore: AsyncDropGuard<AsyncDropArc<ConcurrentFsBlobStore<B>>>,
        node_info: AsyncDropGuard<NodeInfo>,
    ) -> AsyncDropGuard<Self> {
        Self::new_internal(blobstore, AsyncDropArc::new(node_info))
    }

    pub(super) fn new_internal(
        blobstore: AsyncDropGuard<AsyncDropArc<ConcurrentFsBlobStore<B>>>,
        node_info: AsyncDropGuard<AsyncDropArc<NodeInfo>>,
    ) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            blobstore,
            node_info,
        })
    }
}

#[async_trait]
impl<B> Node for CryNode<B>
where
    // TODO Do we really need B: 'static ?
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    type Device = CryDevice<B>;

    async fn as_dir<'a>(&'a self) -> FsResult<AsyncDropGuard<CryDir<'a, B>>> {
        if self.node_info.node_type(&self.blobstore).await? == BlobType::Dir {
            Ok(CryDir::new(
                &self.blobstore,
                AsyncDropArc::clone(&self.node_info),
            ))
        } else {
            Err(FsError::NodeIsNotADirectory)
        }
    }

    async fn as_symlink<'a>(&'a self) -> FsResult<AsyncDropGuard<CrySymlink<'a, B>>> {
        if self.node_info.node_type(&self.blobstore).await? == BlobType::Symlink {
            Ok(CrySymlink::new(
                &self.blobstore,
                AsyncDropArc::clone(&self.node_info),
            ))
        } else {
            Err(FsError::NodeIsNotASymlink)
        }
    }

    async fn as_file<'a>(&'a self) -> FsResult<AsyncDropGuard<CryFile<'a, B>>> {
        match self.node_info.node_type(&self.blobstore).await? {
            BlobType::File => Ok(CryFile::new(
                &self.blobstore,
                AsyncDropArc::clone(&self.node_info),
            )),
            BlobType::Symlink => {
                // TODO What's the right error here?
                Err(FsError::UnknownError)
            }
            BlobType::Dir => Err(FsError::NodeIsADirectory),
        }
    }

    async fn getattr(&self) -> FsResult<NodeAttrs> {
        self.node_info.getattr(&self.blobstore).await
    }

    async fn setattr(
        &self,
        mode: Option<Mode>,
        uid: Option<Uid>,
        gid: Option<Gid>,
        size: Option<NumBytes>,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
        ctime: Option<SystemTime>,
    ) -> FsResult<NodeAttrs> {
        self.node_info
            .setattr(&self.blobstore, mode, uid, gid, size, atime, mtime, ctime)
            .await
    }
}

impl<B> Debug for CryNode<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CryNode")
            .field("node_info", &self.node_info)
            .finish()
    }
}

#[async_trait]
impl<B> AsyncDrop for CryNode<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), FsError> {
        self.node_info.async_drop().await?;
        self.blobstore.async_drop().await
    }
}
