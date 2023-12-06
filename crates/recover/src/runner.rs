use anyhow::{anyhow, bail, Context, Result};
use async_recursion::async_recursion;
use async_trait::async_trait;
use futures::stream::{FuturesUnordered, StreamExt, TryStreamExt};
use indicatif::ProgressBar;
use std::collections::{HashMap, HashSet};
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
        let pb = ProgressBar::new(u64::try_from(all_nodes.len()).unwrap());
        let referenced_nodes =
            get_referenced_node_ids(&blobstore, root_blob_id, Some(all_nodes.len()), &pb).await;
        let referenced_nodes: Vec<BlockId> = match referenced_nodes {
            Ok(referenced_nodes) => referenced_nodes,
            Err(e) => {
                blobstore.async_drop().await?;
                return Err(e);
            }
        };
        pb.finish_and_clear();
        log::info!(
            "Listing referenced nodes...done. Found {} nodes",
            referenced_nodes.len()
        );

        let mut unreferenced_nodes = all_nodes;
        for referenced_node in referenced_nodes {
            let found = unreferenced_nodes.remove(&referenced_node);
            if !found {
                blobstore.async_drop().await?;
                bail!("Node {:?} is referenced but not in the set of all nodes. This can happen if the file system changed while we're checking it.", referenced_node);
            }
        }

        log::info!("Found {} unreferenced nodes", unreferenced_nodes.len());

        log::info!("Checking node depths...");
        let mut num_unreferenced_nodes_per_depth = HashMap::new();
        // TODO Remember node depth when we iterate through them the first time instead of requiring another pass through the nodes
        for unreferenced_node in unreferenced_nodes {
            let depth = blobstore.load_block_depth(&unreferenced_node).await;
            let depth = match depth {
                Ok(Some(depth)) => depth,
                Ok(None) => {
                    blobstore.async_drop().await?;
                    bail!("Node {:?} found earlier but now it is gone. This can happen if the file system changed while we're checking it.", unreferenced_node);
                }
                Err(e) => {
                    blobstore.async_drop().await?;
                    return Err(e);
                }
            };
            let num_unreferenced_nodes_at_depth =
                num_unreferenced_nodes_per_depth.entry(depth).or_insert(0);
            *num_unreferenced_nodes_at_depth += 1;
        }
        let mut num_unreferenced_nodes_per_depth: Vec<(u8, u64)> =
            num_unreferenced_nodes_per_depth.into_iter().collect();
        num_unreferenced_nodes_per_depth.sort_by_key(|(depth, _)| *depth);
        log::info!("Checking node depths...done");
        for (depth, num_unreferenced_nodes_at_depth) in num_unreferenced_nodes_per_depth {
            log::info!(
                "Found {} unreferenced nodes at depth {}",
                num_unreferenced_nodes_at_depth,
                depth
            );
        }

        blobstore.async_drop().await?;
        Ok(())
    }
}

async fn get_all_node_ids(
    blockstore: &AsyncDropGuard<LockingBlockStore<impl BlockStore + Send + Sync>>,
) -> Result<HashSet<BlockId>> {
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
    progress_bar: &ProgressBar,
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
    progress_bar.inc(1);

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
                .context("We just checked that the blob is a directory but now it isn't. The filesystem changed while we're checking it.")?;

            // TODO Would FuturesOrdered be faster than FuturesUnordered here? Or look at stream processing as in [DataTree::_all_blocks_descendants_of]
            let mut child_subtrees: FuturesUnordered<_> = blob
                .entries()
                .map(|entry| *entry.blob_id())
                .collect::<Vec<_>>()
                .into_iter()
                .map(|blob_id| async move {
                    get_referenced_node_ids(blobstore, blob_id, None, progress_bar).await
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
