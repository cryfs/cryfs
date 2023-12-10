use anyhow::{anyhow, bail, Context, Result};
use async_recursion::async_recursion;
use async_trait::async_trait;
use futures::future;
use futures::stream::{FuturesUnordered, StreamExt, TryStreamExt};
use indicatif::ProgressBar;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::sync::Arc;

use super::checks::{AllChecks, FilesystemCheck};
use super::error::CorruptedError;
use cryfs_blobstore::{BlobId, BlobStore, BlobStoreOnBlocks, DataNodeStore};
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
        // TODO No unwrap. Should we instead change blocksize_bytes in the config file struct?
        let blocksize_bytes = u32::try_from(self.config.config.config().blocksize_bytes).unwrap();

        let mut nodestore = DataNodeStore::new(blockstore, blocksize_bytes).await?;

        let checks = AllChecks::new();
        let mut errors = vec![];

        log::info!("Listing all nodes...");
        let all_nodes = match get_all_node_ids(&nodestore).await {
            Ok(all_nodes) => all_nodes,
            Err(e) => {
                nodestore.async_drop().await?;
                return Err(e);
            }
        };
        log::info!("Listing all nodes...done. Found {} nodes", all_nodes.len());

        // TODO Read all blobs before we're reading all nodes since it's a faster pass

        log::info!("Checking all nodes...");
        errors.extend(check_all_nodes(&nodestore, &all_nodes, &checks).await);
        log::info!("Checking all nodes...done");

        // TODO Reading blobs loads those blobs again even though we already read them above. Can we prevent the double-load? e.g. using Blob::all_blocks()? but need to make sure we also load blocks that aren't referenced.

        let blobstore = BlobStoreOnBlocks::new(
            DataNodeStore::into_inner_blockstore(nodestore),
            blocksize_bytes,
        )
        .await?;
        let mut blobstore = FsBlobStore::new(blobstore);

        log::info!("Checking directory structure...");
        let root_blob_id = BlobId::from_hex(&self.config.config.config().root_blob);
        let root_blob_id = match root_blob_id {
            Ok(root_blob_id) => root_blob_id,
            Err(e) => {
                blobstore.async_drop().await?;
                return Err(e);
            }
        };
        errors.extend(check_all_blobs(&blobstore, root_blob_id, &checks).await?);
        log::info!("Checking directory structure...done");

        errors.extend(checks.finalize());

        // TODO println not info log for regular outputs
        log::info!("Found {} errors", errors.len());
        for error in errors {
            log::info!("- {error}");
        }

        // log::info!("Checking node depths...");
        // let mut num_unreferenced_nodes_per_depth = HashMap::new();
        // // TODO Remember node depth when we iterate through them the first time instead of requiring another pass through the nodes
        // for unreferenced_node in unreferenced_nodes {
        //     let depth = blobstore.load_block_depth(&unreferenced_node).await;
        //     let depth = match depth {
        //         Ok(Some(depth)) => depth,
        //         Ok(None) => {
        //             blobstore.async_drop().await?;
        //             bail!("Node {:?} found earlier but now it is gone. This can happen if the file system changed while we're checking it.", unreferenced_node);
        //         }
        //         Err(e) => {
        //             blobstore.async_drop().await?;
        //             return Err(e);
        //         }
        //     };
        //     let num_unreferenced_nodes_at_depth =
        //         num_unreferenced_nodes_per_depth.entry(depth).or_insert(0);
        //     *num_unreferenced_nodes_at_depth += 1;
        // }
        // let mut num_unreferenced_nodes_per_depth: Vec<(u8, u64)> =
        //     num_unreferenced_nodes_per_depth.into_iter().collect();
        // num_unreferenced_nodes_per_depth.sort_by_key(|(depth, _)| *depth);
        // log::info!("Checking node depths...done");
        // for (depth, num_unreferenced_nodes_at_depth) in num_unreferenced_nodes_per_depth {
        //     log::info!(
        //         "Found {} unreferenced nodes at depth {}",
        //         num_unreferenced_nodes_at_depth,
        //         depth
        //     );
        // }

        blobstore.async_drop().await?;
        Ok(())
    }
}

async fn get_all_node_ids<B>(
    nodestore: &AsyncDropGuard<DataNodeStore<B>>,
) -> Result<HashSet<BlockId>>
where
    B: BlockStore + Send + Sync + 'static,
{
    nodestore.all_nodes().await?.try_collect().await
}

#[must_use]
async fn check_all_nodes<B>(
    nodestore: &AsyncDropGuard<DataNodeStore<B>>,
    all_nodes: &HashSet<BlockId>,
    checks: &AllChecks,
) -> Vec<CorruptedError>
where
    B: BlockStore + Send + Sync + 'static,
{
    let pb = Arc::new(ProgressBar::new(u64::try_from(all_nodes.len()).unwrap()));
    let pb_clone = Arc::clone(&pb);
    // TODO Should we rate-limit this instead of trying to load all at once?
    let errors: FuturesUnordered<_> = all_nodes.iter().map(|&node_id| {
        let pb_clone = Arc::clone(&pb_clone);
        async move {
            let loaded = nodestore.load(node_id).await;
            pb_clone.inc(1);
            match loaded {
                Ok(Some(node)) => {
                    checks.process_existing_node(&node);
                    None // no error
                }
                Ok(None) => {
                    // TODO don't panic but return an error (i.e. change result from Vec<CorruptedError> to Result<Vec<CorruptedError>>) and exit gracefully
                    panic!("Node {node_id:?} previously present but then vanished during our checks. Please don't modify the file system while checks are running.");
                }
                Err(error) => {
                    // return the error
                    Some(CorruptedError::NodeUnreadable {
                        node_id,
                        error,
                    })
                }
            }
        }
    }).collect();
    let result = errors.filter_map(future::ready).collect().await;
    pb.finish_and_clear();
    result
}

#[async_recursion]
async fn check_all_blobs<B>(
    blobstore: &AsyncDropGuard<FsBlobStore<BlobStoreOnBlocks<B>>>,
    root_blob_id: BlobId,
    checks: &AllChecks,
) -> Result<Vec<CorruptedError>>
where
    B: BlockStore + Send + Sync + 'static,
{
    log::debug!("Entering blob {:?}", &root_blob_id);
    let loaded = blobstore.load(&root_blob_id).await;
    let errors = match loaded {
        Ok(Some(mut blob)) => {
            checks.process_reachable_blob(&blob);
            match blob.blob_type() {
                BlobType::File | BlobType::Symlink => {
                    // file and symlink blobs don't have child blobs. Nothing to do.
                    blob.async_drop().await?;
                    vec![]
                }
                BlobType::Dir => {
                    // Get all directory entry and recurse into their blobs, concurrently.
                    let blob = FsBlob::into_dir(blob).await;
                    let mut blob = match blob {
                        Ok(blob) => blob,
                        Err(_) => {
                            bail!("Blob {root_blob_id:?} previously was a directory blob but now isn't. Please don't modify the file system while checks are running.");
                        }
                    };

                    // TODO Would FuturesOrdered be faster than FuturesUnordered here? Or look at stream processing as in [DataTree::_all_blocks_descendants_of]
                    let mut child_subtrees: FuturesUnordered<_> =
                        blob.entries()
                            .map(|entry| *entry.blob_id())
                            .collect::<Vec<_>>()
                            .into_iter()
                            .map(|blob_id| async move {
                                check_all_blobs(blobstore, blob_id, checks).await
                            })
                            .collect();
                    blob.async_drop().await?;
                    let mut errors = vec![];
                    while let Some(child_subtree_errors) = child_subtrees.next().await {
                        errors.extend(child_subtree_errors?);
                    }
                    errors
                }
            }
        }
        Ok(None) => {
            vec![CorruptedError::BlobMissing {
                blob_id: root_blob_id,
            }]
        }
        Err(error) => {
            vec![CorruptedError::BlobUnreadable {
                blob_id: root_blob_id,
                error,
            }]
        }
    };
    log::debug!("Exiting blob {:?}", &root_blob_id);
    Ok(errors)
}
