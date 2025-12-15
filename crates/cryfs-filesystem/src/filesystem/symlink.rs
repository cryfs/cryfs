use async_trait::async_trait;
use std::fmt::Debug;

use cryfs_fsblobstore::{
    concurrentfsblobstore::{ConcurrentFsBlob, ConcurrentFsBlobStore},
    fsblobstore::{FsBlob, SymlinkBlob},
};

use super::{device::CryDevice, node::CryNode, node_info::NodeInfo};
use cryfs_blobstore::BlobStore;
use cryfs_rustfs::{FsError, FsResult, object_based_api::Symlink};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard},
    with_async_drop_2,
};

#[derive(Debug)]
pub struct CrySymlink<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    // TODO Do we need to store blobstore + node_info here or can we just store the target directly?
    blobstore: &'a AsyncDropGuard<AsyncDropArc<ConcurrentFsBlobStore<B>>>,
    node_info: AsyncDropGuard<AsyncDropArc<NodeInfo<B>>>,
}

impl<'a, B> CrySymlink<'a, B>
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

    async fn load_blob(&self) -> FsResult<AsyncDropGuard<ConcurrentFsBlob<B>>> {
        self.node_info.load_blob(&self.blobstore).await
    }

    fn blob_as_symlink_mut<'b>(blob: &'b mut FsBlob<B>) -> Result<&'b mut SymlinkBlob<B>, FsError> {
        let blob_id = blob.blob_id();
        blob.as_symlink_mut().map_err(|err| {
            FsError::CorruptedFilesystem {
                // TODO Add to message what it actually is
                message: format!("Blob {:?} is listed as a symlink in its parent directory but is actually not a symlink: {err:?}", blob_id),
            }
        })
    }
}

#[async_trait]
impl<'a, B> Symlink for CrySymlink<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    type Device = CryDevice<B>;

    fn into_node(this: AsyncDropGuard<Self>) -> AsyncDropGuard<CryNode<B>> {
        let this = this.unsafe_into_inner_dont_drop();
        CryNode::new_internal(AsyncDropArc::clone(&this.blobstore), this.node_info)
    }

    async fn target(&self) -> FsResult<String> {
        self.node_info
            .concurrently_maybe_update_access_timestamp_in_parent(async || {
                let blob = self.load_blob().await?;
                with_async_drop_2!(blob, {
                    blob.with_lock(async |mut blob| {
                        let blob = Self::blob_as_symlink_mut(&mut blob)?;
                        let target = blob.target().await.map_err(|err| {
                            FsError::CorruptedFilesystem {
                                // TODO Add to message what it actually is
                                message: format!("Unparseable symlink blob: {err:?}"),
                            }
                        });
                        Ok::<_, FsError>(target)
                    })
                    .await
                })?
            })
            .await
    }
}

#[async_trait]
impl<'a, B> AsyncDrop for CrySymlink<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> FsResult<()> {
        self.node_info.async_drop().await
    }
}
