use async_trait::async_trait;
use std::fmt::Debug;

use super::fsblobstore::FsBlobStore;
use super::{device::CryDevice, node_info::NodeInfo, open_file::CryOpenFile};
use cryfs_blobstore::BlobStore;
use cryfs_rustfs::{object_based_api::File, FsError, FsResult, NumBytes, OpenFlags};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};

pub struct CryFile<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    blobstore: AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
    node_info: NodeInfo,
}

impl<B> CryFile<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    pub fn new(
        blobstore: AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
        node_info: NodeInfo,
    ) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            blobstore,
            node_info,
        })
    }
}

#[async_trait]
impl<B> File for CryFile<B>
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
            self.node_info.clone(),
        ))
    }

    async fn truncate(&self, new_size: NumBytes) -> FsResult<()> {
        super::open_file::truncate_file(&self.blobstore, &self.node_info, new_size).await
    }
}

impl<B> Debug for CryFile<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CryFile")
            .field("node_info", &self.node_info)
            .finish()
    }
}

#[async_trait]
impl<B> AsyncDrop for CryFile<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), FsError> {
        self.blobstore.async_drop().await.map_err(|err| {
            log::error!("Failed to drop blobstore: {err:?}");
            FsError::Custom {
                error_code: libc::EIO,
            }
        })
    }
}
