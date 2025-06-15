use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;

use super::{
    device::CryDevice,
    fsblobstore::{FsBlob, FsBlobStore, SymlinkBlob},
    node::CryNode,
    node_info::NodeInfo,
};
use cryfs_blobstore::BlobStore;
use cryfs_rustfs::{FsError, FsResult, object_based_api::Symlink};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard},
    with_async_drop_2,
};

pub struct CrySymlink<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    // TODO Do we need to store blobstore + node_info here or can we just store the target directly?
    blobstore: &'a AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
    node_info: Arc<NodeInfo>,
}

impl<'a, B> CrySymlink<'a, B>
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

    async fn load_blob(&self) -> FsResult<AsyncDropGuard<FsBlob<'a, B>>> {
        self.node_info.load_blob(&self.blobstore).await
    }

    fn blob_as_symlink_mut<'b>(
        blob: &'b mut FsBlob<'a, B>,
    ) -> Result<&'b mut SymlinkBlob<'a, B>, FsError> {
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
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    type Device = CryDevice<B>;

    fn as_node(&self) -> AsyncDropGuard<CryNode<B>> {
        CryNode::new_internal(
            AsyncDropArc::clone(&self.blobstore),
            Arc::clone(&self.node_info),
        )
    }

    async fn target(&self) -> FsResult<String> {
        self.node_info
            .concurrently_maybe_update_access_timestamp_in_parent(&self.blobstore, async || {
                let mut blob = self.load_blob().await?;
                with_async_drop_2!(blob, {
                    let blob = Self::blob_as_symlink_mut(&mut blob)?;
                    let target = blob.target().await.map_err(|err| {
                        FsError::CorruptedFilesystem {
                            // TODO Add to message what it actually is
                            message: format!("Unparseable symlink blob: {err:?}"),
                        }
                    });
                    Ok(target)
                })?
            })
            .await
    }
}
