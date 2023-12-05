use anyhow::{anyhow, Result};
use async_recursion::async_recursion;
use async_trait::async_trait;
use futures::stream::{FuturesUnordered, StreamExt, TryStreamExt};
use std::fmt::Debug;

use cryfs_blobstore::{BlobId, BlobStore, BlobStoreOnBlocks};
use cryfs_blockstore::{BlockId, BlockStore, LockingBlockStore};
use cryfs_cli_utils::BlockstoreCallback;
use cryfs_cryfs::{
    config::ConfigLoadResult,
    filesystem::fsblobstore::{BlobType, FsBlob, FsBlobStore},
};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

pub struct RecoverRunner<'c> {
    pub config: &'c ConfigLoadResult,
}

#[async_trait]
impl<'l> BlockstoreCallback for RecoverRunner<'l> {
    type Result = Result<()>;

    async fn callback<B: BlockStore + AsyncDrop + Send + Sync + 'static>(
        self,
        mut blockstore: AsyncDropGuard<LockingBlockStore<B>>,
    ) -> Self::Result {
        // TODO We should be able to list all blocks in parallel with listing all known blocks

        log::info!("Listing all nodes...");
        let all_nodes = match get_all_node_ids(&blockstore).await {
            Ok(all_nodes) => all_nodes,
            Err(e) => {
                blockstore.async_drop().await?;
                return Err(e);
            }
        };
        log::info!("Listing all nodes...done. Found {} nodes", all_nodes.len(),);

        // TODO No unwrap. Should we instead change blocksize_bytes in the config file struct?
        let blobstore = BlobStoreOnBlocks::new(
            blockstore,
            u32::try_from(self.config.config.config().blocksize_bytes).unwrap(),
        )
        .await?;
        let mut blobstore = FsBlobStore::new(blobstore);

        let root_blob_id = BlobId::from_hex(&self.config.config.config().root_blob);
        let root_blob_id = match root_blob_id {
            Ok(root_blob_id) => root_blob_id,
            Err(e) => {
                blobstore.async_drop().await?;
                return Err(e);
            }
        };

        log::info!("Listing referenced nodes...");
        let referenced_nodes =
            get_referenced_node_ids(&blobstore, root_blob_id, Some(all_nodes.len())).await;
        let referenced_nodes: Vec<BlockId> = match referenced_nodes {
            Ok(referenced_nodes) => referenced_nodes,
            Err(e) => {
                blobstore.async_drop().await?;
                return Err(e);
            }
        };
        log::info!(
            "Listing referenced nodes...done. Found {} nodes",
            referenced_nodes.len()
        );

        blobstore.async_drop().await?;
        Ok(())
    }
}

async fn get_all_node_ids(
    blockstore: &AsyncDropGuard<LockingBlockStore<impl BlockStore + Send + Sync>>,
) -> Result<Vec<BlockId>> {
    blockstore.all_blocks().await?.try_collect().await
}

#[async_recursion]
async fn get_referenced_node_ids(
    blobstore: &AsyncDropGuard<
        FsBlobStore<
            impl BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        >,
    >,
    root_blob_id: BlobId,
    capacity_hint: Option<usize>,
) -> Result<Vec<BlockId>> {
    log::debug!("Entering blob {:?}", &root_blob_id);

    let mut referenced_nodes = match capacity_hint {
        Some(capacity_hint) => Vec::with_capacity(capacity_hint),
        None => Vec::new(),
    };

    let mut blob = blobstore
        .load(&root_blob_id)
        .await?
        .ok_or_else(|| anyhow!("Blob root {:?} referenced but not found", root_blob_id))?;

    // TODO Parallelize: If it's a directory, we can get blocks of the blob while we're reading the directory entries since that has to load all blocks anyways.
    let blocks_of_blob: Result<Vec<BlockId>> = blob.all_blocks()?.try_collect().await;
    let blocks_of_blob = match blocks_of_blob {
        Ok(blocks_of_blob) => blocks_of_blob,
        Err(e) => {
            blob.async_drop().await?;
            return Err(e);
        }
    };
    referenced_nodes.extend(blocks_of_blob);

    match blob.blob_type() {
        BlobType::File | BlobType::Symlink => {
            // file and symlink blobs don't have child blobs. Nothing to do.
            blob.async_drop().await?;
        }
        BlobType::Dir => {
            // Get all directory entry and recurse into their blobs, concurrently.
            let mut blob = FsBlob::into_dir(blob)
                .await
                .expect("We just checked that the blob is a directory but now it isn't");
            // TODO Would FuturesOrdered be faster than FuturesUnordered here? Or look at stream processing as in [DataTree::_all_blocks_descendants_of]
            let mut child_subtrees: FuturesUnordered<_> =
                blob.entries()
                    .map(|entry| *entry.blob_id())
                    .collect::<Vec<_>>()
                    .into_iter()
                    .map(|blob_id| async move {
                        get_referenced_node_ids(blobstore, blob_id, None).await
                    })
                    .collect();
            // TODO Concurrently async drop blob
            blob.async_drop().await?;
            while let Some(blocks_in_child_subtree) = child_subtrees.next().await {
                referenced_nodes.extend(blocks_in_child_subtree?);
            }
        }
    }

    Ok(referenced_nodes)
}
