use anyhow::{anyhow, bail, Context, Result};
use async_recursion::async_recursion;
use async_trait::async_trait;
use futures::future::{self, BoxFuture};
use futures::stream::{self, Stream, StreamExt, TryStreamExt};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedSender, WeakUnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;

use super::checks::{AllChecks, FilesystemCheck};
use super::error::CorruptedError;
use cryfs_blobstore::{BlobId, BlobStore, BlobStoreOnBlocks, DataNodeStore};
use cryfs_blockstore::{BlockId, BlockStore, LockingBlockStore};
use cryfs_cli_utils::BlockstoreCallback;
use cryfs_cryfs::{
    config::ConfigLoadResult,
    filesystem::fsblobstore::{BlobType, FsBlob, FsBlobStore},
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    progress::{Progress, Spinner},
};

pub struct RecoverRunner<'c> {
    pub config: &'c ConfigLoadResult,
}

impl<'l> BlockstoreCallback for RecoverRunner<'l> {
    type Result = Result<()>;

    async fn callback<B: BlockStore + AsyncDrop + Send + Sync + 'static>(
        self,
        mut blockstore: AsyncDropGuard<LockingBlockStore<B>>,
    ) -> Self::Result {
        // TODO It currently seems to spend some seconds here before displaying "Listing all nodes". Why? What is it doing? Show another progress bar?

        // TODO No unwrap. Should we instead change blocksize_bytes in the config file struct?
        let blocksize_bytes = u32::try_from(self.config.config.config().blocksize_bytes).unwrap();

        let mut nodestore = DataNodeStore::new(blockstore, blocksize_bytes).await?;

        // TODO --------------- From here on, it's the old algorithm
        let checks = AllChecks::new();

        // TODO Instead of autotick, we could manually tick it while listing nodes. That would mean users see it if it gets stuck.
        //      Or we could even show a Progress bar since we know we're going from AAA to ZZZ in the folder structure.
        let pb = Spinner::new_autotick("Listing all nodes");
        let all_nodes = match get_all_node_ids(&nodestore).await {
            Ok(all_nodes) => all_nodes,
            Err(e) => {
                nodestore.async_drop().await?;
                return Err(e);
            }
        };
        pb.finish();
        println!("Found {} nodes", all_nodes.len());

        // TODO Read all blobs before we're reading all nodes since it's a faster pass

        // TODO since `check_all_blobs` now also checks the nodes of the blobs, we can remove this
        //      and should replace it with code that checks which nodes haven't been processed by blobs and only go over those.
        let pb = Progress::new(
            "Checking all nodes",
            u64::try_from(all_nodes.len()).unwrap(),
        );
        check_all_nodes(&nodestore, &all_nodes, &checks, pb.clone()).await;
        pb.finish();

        // TODO --------------- From here on, it's the new algorithm
        let checks = AllChecks::new();

        // TODO Reading blobs loads those blobs again even though we already read them above. Can we prevent the double-load? e.g. using Blob::all_blocks()? but need to make sure we also load blocks that aren't referenced.

        let blobstore = BlobStoreOnBlocks::new(
            DataNodeStore::into_inner_blockstore(nodestore),
            blocksize_bytes,
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
        let pb = Progress::new(
            "Checking all blobs",
            u64::try_from(all_nodes.len()).unwrap(),
        );
        // Check all reachable nodes
        let check_all_blobs_result =
            check_all_blobs(&blobstore, root_blob_id, &checks, pb.clone()).await?;

        // Check any leftover unreferenced nodes
        let mut unreferenced_nodes = all_nodes;
        for visited_node in check_all_blobs_result.visited_nodes {
            unreferenced_nodes.remove(&visited_node);
        }
        let mut nodestore =
            BlobStoreOnBlocks::into_inner_node_store(FsBlobStore::into_inner_blobstore(blobstore));
        check_all_nodes(&nodestore, &unreferenced_nodes, &checks, pb.clone()).await;
        pb.finish();

        let errors = checks.finalize();

        // TODO Some errors may be found by multiple checks, let's deduplicate those.

        for error in &errors {
            println!("- {error}");
        }
        println!("Found {} errors", errors.len());

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

        nodestore.async_drop().await?;
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

async fn check_all_nodes<B>(
    nodestore: &AsyncDropGuard<DataNodeStore<B>>,
    all_nodes: &HashSet<BlockId>,
    checks: &AllChecks,
    pb: Progress,
) where
    B: BlockStore + Send + Sync + 'static,
{
    // TODO What's a good concurrency value here?
    const MAX_CONCURRENCY: usize = 100;
    let all_nodes = stream::iter(all_nodes.iter());
    let mut maybe_errors = all_nodes.map(move |&node_id| {
        let pb = pb.clone();
        async move {
            let loaded = nodestore.load(node_id).await;
            let result = match loaded {
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
            };
            pb.inc(1);
            result
        }
    })
    .buffer_unordered(MAX_CONCURRENCY);
    while let Some(maybe_error) = maybe_errors.next().await {
        if let Some(error) = maybe_error {
            checks.add_error(error);
        }
    }
}

#[must_use]
struct CheckAllBlobsResult {
    visited_nodes: Vec<BlockId>,
}

async fn check_all_blobs<B>(
    blobstore: &AsyncDropGuard<FsBlobStore<BlobStoreOnBlocks<B>>>,
    root_blob_id: BlobId,
    checks: &AllChecks,
    pb: Progress,
) -> Result<CheckAllBlobsResult>
where
    B: BlockStore + Send + Sync + 'static,
{
    // TODO What's a good concurrency value here?
    const MAX_CONCURRENCY: usize = 100;

    let (task_queue_sender, mut task_queue_receiver) = tokio::sync::mpsc::unbounded_channel();

    task_queue_sender
        .send(Box::pin(_check_all_blobs(
            blobstore,
            root_blob_id,
            checks,
            task_queue_sender.clone(),
            pb,
        )))
        .expect("Failed to send task to task queue");
    // Drop our instance of the sender so that `blob_queue_stream` below finishes when the last task holding a sender instance is gone
    std::mem::drop(task_queue_sender);

    let mut result = CheckAllBlobsResult {
        visited_nodes: vec![],
    };
    let mut blob_queue_stream =
        UnboundedReceiverStream::new(task_queue_receiver).buffer_unordered(MAX_CONCURRENCY);
    while let Some(subresult) = blob_queue_stream.next().await {
        result.visited_nodes.extend(subresult?.visited_nodes);
    }
    Ok(result)
}

#[async_recursion]
async fn _check_all_blobs<'a, 'b, 'f, B>(
    blobstore: &'a AsyncDropGuard<FsBlobStore<BlobStoreOnBlocks<B>>>,
    root_blob_id: BlobId,
    checks: &'b AllChecks,
    // TODO Is this possible without BoxFuture?
    task_queue_sender: UnboundedSender<BoxFuture<'f, Result<CheckAllBlobsResult>>>,
    pb: Progress,
) -> Result<CheckAllBlobsResult>
where
    B: BlockStore + Send + Sync + 'static,
    'a: 'f,
    'b: 'f,
    'f: 'async_recursion,
{
    log::debug!("Entering blob {:?}", &root_blob_id);
    let loaded = blobstore.load(&root_blob_id).await;
    let mut result = CheckAllBlobsResult {
        visited_nodes: vec![*root_blob_id.to_root_block_id()],
    };
    match loaded {
        Ok(Some(mut blob)) => {
            checks.process_reachable_blob(&blob);

            // TODO Checking children blobs for directory blobs loads the nodes of this blob.
            //      Then we load it again when we check the nodes of this blob. Can we only load it once?

            let subresult =
                check_all_children_blobs(blobstore, &blob, checks, task_queue_sender, pb.clone())
                    .await;
            match subresult {
                Ok(()) => (),
                Err(err) => {
                    blob.async_drop().await?;
                    return Err(err);
                }
            };

            // TODO Can we check nodes of this blob and children blobs concurrently?

            let subresult = check_all_nodes_of_blob(blob, checks, pb).await?;
            result.visited_nodes.extend(subresult.visited_nodes);
        }
        Ok(None) => {
            checks.add_error(CorruptedError::BlobMissing {
                blob_id: root_blob_id,
            });
        }
        Err(error) => {
            checks.add_error(CorruptedError::BlobUnreadable {
                blob_id: root_blob_id,
                error,
            });
        }
    };
    log::debug!("Exiting blob {:?}", &root_blob_id);
    Ok(result)
}

async fn check_all_children_blobs<'a, 'b, 'c, 'f, B>(
    blobstore: &'a AsyncDropGuard<FsBlobStore<BlobStoreOnBlocks<B>>>,
    blob: &FsBlob<'b, BlobStoreOnBlocks<B>>,
    checks: &'c AllChecks,
    task_queue_sender: UnboundedSender<BoxFuture<'f, Result<CheckAllBlobsResult>>>,
    pb: Progress,
) -> Result<()>
where
    B: BlockStore + Send + Sync + 'static,
    'a: 'f,
    'c: 'f,
{
    match blob.blob_type() {
        BlobType::File | BlobType::Symlink => {
            // file and symlink blobs don't have child blobs. Nothing to do.
        }
        BlobType::Dir => {
            // Get all directory entry and recurse into their blobs, concurrently.
            let blob = match blob.as_dir() {
                Ok(blob) => blob,
                Err(_) => {
                    bail!("Blob {blob_id:?} previously was a directory blob but now isn't. Please don't modify the file system while checks are running.", blob_id=blob.blob_id());
                }
            };

            for entry in blob.entries() {
                task_queue_sender
                    .send(Box::pin(_check_all_blobs(
                        blobstore,
                        *entry.blob_id(),
                        checks,
                        task_queue_sender.clone(),
                        pb.clone(),
                    )))
                    .expect("Failed to send task to task queue");
            }
        }
    };
    Ok(())
}

async fn check_all_nodes_of_blob<'a, B>(
    blob: AsyncDropGuard<FsBlob<'a, BlobStoreOnBlocks<B>>>,
    checks: &AllChecks,
    pb: Progress,
) -> Result<CheckAllBlobsResult>
where
    B: BlockStore + Send + Sync + 'static,
{
    let all_nodes = FsBlob::load_all_nodes(blob).await?;
    // TODO Should we rate-limit this instead of trying to load all at once?
    // TODO Does this actually have good concurrency? Stream handling at least ought to be ordered, which might reduce concurrency
    let mut results = all_nodes.map(|node| {
        pb.inc(1);
        match node {
            Ok(node) => {
                checks.process_existing_node(&node);
                (*node.block_id(), None) // no error
            }
            Err((node_id, error)) => {
                // return the error
                (
                    node_id,
                    Some(CorruptedError::NodeUnreadable { node_id, error }),
                )
            }
        }
    });
    let mut combined_result = CheckAllBlobsResult {
        visited_nodes: vec![],
    };
    while let Some((node_id, error)) = results.next().await {
        combined_result.visited_nodes.push(node_id);
        if let Some(error) = error {
            checks.add_error(error);
        }
    }
    Ok(combined_result)
}
