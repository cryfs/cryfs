use async_trait::async_trait;
use std::fmt::Debug;

use super::{device::CryDevice, node::CryNode, open_file::CryOpenFile};
use cryfs_blobstore::BlobStore;
use cryfs_rustfs::{object_based_api::File, FsError, FsResult, NumBytes, OpenFlags};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

pub struct CryFile<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    node: CryNode<'a, B>,
}

impl<'a, B> CryFile<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    pub fn new(node: CryNode<'a, B>) -> Self {
        Self { node }
    }
}

#[async_trait]
impl<'a, B> File for CryFile<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    type Device = CryDevice<B>;

    async fn open(&self, flags: OpenFlags) -> FsResult<AsyncDropGuard<CryOpenFile<B>>> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }

    async fn truncate(&self, new_size: NumBytes) -> FsResult<()> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }
}
