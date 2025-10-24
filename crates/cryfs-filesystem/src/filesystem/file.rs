use async_trait::async_trait;
use std::fmt::Debug;

use crate::filesystem::concurrentfsblobstore::ConcurrentFsBlobStore;

use super::{device::CryDevice, node_info::NodeInfo, open_file::CryOpenFile};
use cryfs_blobstore::BlobStore;
use cryfs_rustfs::{FsError, FsResult, OpenFlags, object_based_api::File};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};

pub struct CryFile<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    blobstore: &'a AsyncDropGuard<AsyncDropArc<ConcurrentFsBlobStore<B>>>,
    node_info: AsyncDropGuard<AsyncDropArc<NodeInfo<B>>>,
}

impl<'a, B> CryFile<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    pub fn new(
        blobstore: &'a AsyncDropGuard<AsyncDropArc<ConcurrentFsBlobStore<B>>>,
        node_info: AsyncDropGuard<AsyncDropArc<NodeInfo<B>>>,
    ) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            blobstore,
            node_info,
        })
    }
}

#[async_trait]
impl<'a, B> File for CryFile<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    type Device = CryDevice<B>;

    async fn into_open(
        this: AsyncDropGuard<Self>,
        flags: OpenFlags,
    ) -> FsResult<AsyncDropGuard<CryOpenFile<B>>> {
        // TODO Share the NodeInfo instance between CryFile and CryOpenFile with an Arc so that [CryFile::truncate] will not have to load the parent blob
        //      if CryOpenFile already did (or the other way round)
        // TODO Handle flags
        let this = this.unsafe_into_inner_dont_drop();
        Ok(CryOpenFile::new(
            AsyncDropArc::clone(&this.blobstore),
            this.node_info,
        ))
    }
}

impl<'a, B> Debug for CryFile<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CryFile")
            .field("node_info", &self.node_info)
            .finish()
    }
}

#[async_trait]
impl<'a, B> AsyncDrop for CryFile<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    type Error = FsError;
    async fn async_drop_impl(&mut self) -> Result<(), FsError> {
        self.node_info.async_drop().await
    }
}
