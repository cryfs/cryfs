use async_trait::async_trait;
use std::fmt::Debug;

use super::node::CryNode;
use cryfs_blobstore::BlobStore;
use cryfs_rustfs::{object_based_api::Symlink, FsError, FsResult};
use cryfs_utils::async_drop::AsyncDrop;

pub struct CrySymlink<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    // TODO Do we need to store node here or can we just store the target directly?
    node: CryNode<'a, B>,
}

impl<'a, B> CrySymlink<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    pub fn new(node: CryNode<'a, B>) -> Self {
        Self { node }
    }
}

#[async_trait]
impl<'a, B> Symlink for CrySymlink<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    async fn target(&self) -> FsResult<String> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }
}
