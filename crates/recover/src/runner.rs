use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::stream::TryStreamExt;

use cryfs_blobstore::{BlobId, BlobStoreOnBlocks, DataNodeStore, DataTreeStore};
use cryfs_blockstore::{BlockId, BlockStore, LockingBlockStore};
use cryfs_cli_utils::BlockstoreCallback;
use cryfs_cryfs::config::ConfigLoadResult;
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

        let mut treestore = DataTreeStore::new(
            blockstore,
            // TODO No unwrap. Should we instead change blocksize_bytes in the config file struct?
            u32::try_from(self.config.config.config().blocksize_bytes).unwrap(),
        )
        .await?;

        let root_blob_id = BlobId::from_hex(&self.config.config.config().root_blob);
        let root_blob_id = match root_blob_id {
            Ok(root_blob_id) => root_blob_id,
            Err(e) => {
                treestore.async_drop().await?;
                return Err(e);
            }
        };

        log::info!("Listing referenced nodes...");
        let referenced_nodes =
            get_referenced_node_ids(&treestore, root_blob_id, all_nodes.len()).await;
        let referenced_nodes = match referenced_nodes {
            Ok(referenced_nodes) => referenced_nodes,
            Err(e) => {
                treestore.async_drop().await?;
                return Err(e);
            }
        };
        log::info!("Listing referenced nodes...done");

        treestore.async_drop().await?;
        Ok(())
    }
}

async fn get_all_node_ids(
    blockstore: &AsyncDropGuard<LockingBlockStore<impl BlockStore + Send + Sync>>,
) -> Result<Vec<BlockId>> {
    blockstore.all_blocks().await?.try_collect().await
}

async fn get_referenced_node_ids(
    treestore: &AsyncDropGuard<DataTreeStore<impl BlockStore + Send + Sync>>,
    root_blob_id: BlobId,
    capacity_hint: usize,
) -> Result<Vec<BlockId>> {
    let mut referenced_nodes = Vec::with_capacity(capacity_hint);

    let tree = treestore
        .load_tree(*root_blob_id.to_root_block_id())
        .await?
        .ok_or_else(|| {
            anyhow!(
                "Blob root {:?} vanished in the middle of the operation",
                root_blob_id
            )
        })?;

    // TODO Parallelize: If it's a directory, we can get tree blocks while we're reading the directory entries anyways.
    let tree_blocks: Vec<BlockId> = tree.all_blocks()?.try_collect().await?;
    referenced_nodes.extend(tree_blocks);

    // TODO Get directory entries and recurse into their blobs

    Ok(referenced_nodes)
}
