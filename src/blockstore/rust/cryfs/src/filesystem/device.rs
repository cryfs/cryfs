use async_trait::async_trait;
use std::fmt::Debug;
use std::path::Path;
use std::sync::Arc;

use cryfs_blobstore::BlobStore;
use cryfs_rustfs::{object_based_api::Device, FsError, FsResult, Statfs};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};

use super::{
    dir::CryDir, file::CryFile, node::CryNode, open_file::CryOpenFile, symlink::CrySymlink,
};
use crate::filesystem::fsblobstore::FsBlobStore;

pub struct CryDevice<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send,
{
    blobstore: AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
}

impl<B> CryDevice<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send,
{
    pub fn new(blobstore: AsyncDropGuard<B>) -> Self
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send,
    {
        Self {
            blobstore: AsyncDropArc::new(FsBlobStore::new(blobstore)),
        }
    }
}

#[async_trait]
impl<B> Device for CryDevice<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send,
{
    type Node = CryNode<B>;
    type Dir = CryDir<B>;
    type Symlink = CrySymlink<B>;
    type File = CryFile<B>;
    type OpenFile = CryOpenFile<B>;

    async fn load_node(&self, path: &Path) -> FsResult<Self::Node> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }

    async fn load_dir(&self, path: &Path) -> FsResult<Self::Dir> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }

    async fn load_symlink(&self, path: &Path) -> FsResult<Self::Symlink> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }

    async fn load_file(&self, path: &Path) -> FsResult<Self::File> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }

    async fn statfs(&self) -> FsResult<Statfs> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }
}
