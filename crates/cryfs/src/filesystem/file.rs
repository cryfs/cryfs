use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;

use super::fsblobstore::FsBlobStore;
use super::{device::CryDevice, node_info::NodeInfo, open_file::CryOpenFile};
use cryfs_blobstore::BlobStore;
use cryfs_rustfs::{object_based_api::File, FsResult, OpenFlags};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};

pub struct CryFile<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    blobstore: &'a AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
    node_info: Arc<NodeInfo>,
}

impl<'a, B> CryFile<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    pub fn new(
        blobstore: &'a AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
        node_info: Arc<NodeInfo>,
    ) -> Self {
        Self {
            blobstore,
            node_info,
        }
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
        // TODO Share the NodeInfo instance between CryFile and CryOpenFile with an Arc so that [CryFile::truncate] will not have to load the parent blob
        //      if CryOpenFile already did (or the other way round)
        // TODO Handle flags
        Ok(CryOpenFile::new(
            AsyncDropArc::clone(&self.blobstore),
            Arc::clone(&self.node_info),
        ))
    }
}

impl<'a, B> Debug for CryFile<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CryFile")
            .field("node_info", &self.node_info)
            .finish()
    }
}
