use async_trait::async_trait;
use std::fmt::Debug;
use std::path::PathBuf;

use super::node::CryNode;
use cryfs_blobstore::BlobStore;
use cryfs_rustfs::{object_based_api::Symlink, FsError, FsResult};
use cryfs_utils::async_drop::AsyncDrop;

pub struct CrySymlink<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send,
{
    // TODO Do we need to store node here or can we just store the target directly?
    node: CryNode<B>,
}

#[async_trait]
impl<B> Symlink for CrySymlink<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send,
{
    async fn target(&self) -> FsResult<PathBuf> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }
}
