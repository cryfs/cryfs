use async_trait::async_trait;
use std::fmt::Debug;

use super::{device::CryDevice, node::CryNode, open_file::CryOpenFile};
use cryfs_blobstore::BlobStore;
use cryfs_rustfs::{object_based_api::File, FsError, FsResult, NumBytes, OpenFlags};
use cryfs_utils::async_drop::AsyncDrop;

pub struct CryFile<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send,
{
    node: CryNode<B>,
}

#[async_trait]
impl<B> File for CryFile<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send,
{
    type Device = CryDevice<B>;

    async fn open(&self, flags: OpenFlags) -> FsResult<CryOpenFile<B>> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }

    async fn truncate(&self, new_size: NumBytes) -> FsResult<()> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }
}
