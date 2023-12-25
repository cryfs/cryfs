use anyhow::{anyhow, bail, ensure, Context, Result};
use async_recursion::async_recursion;
use async_trait::async_trait;
use futures::future::{self, BoxFuture};
use futures::stream::{self, Stream, StreamExt, TryStreamExt};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
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
    type Result = Result<Vec<CorruptedError>>;

    async fn callback<B: BlockStore + AsyncDrop + Send + Sync + 'static>(
        self,
        mut blockstore: AsyncDropGuard<LockingBlockStore<B>>,
    ) -> Self::Result {
        let root_blob_id = BlobId::from_hex(&self.config.config.config().root_blob);
        let root_blob_id = match root_blob_id {
            Ok(root_blob_id) => root_blob_id,
            Err(e) => {
                blockstore.async_drop().await?;
                return Err(e);
            }
        };

        // TODO Instead of autotick, we could manually tick it while listing nodes. That would mean users see it if it gets stuck.
        //      Or we could even show a Progress bar since we know we're going from AAA to ZZZ in the folder structure.
        let pb = Spinner::new_autotick("Listing all nodes");
        let all_nodes = match get_all_node_ids(&blockstore).await {
            Ok(all_nodes) => all_nodes,
            Err(e) => {
                blockstore.async_drop().await?;
                return Err(e);
            }
        };
        pb.finish();
        println!("Found {} nodes", all_nodes.len());

        let checks = AllChecks::new(root_blob_id);

        // TODO No unwrap. Should we instead change blocksize_bytes in the config file struct?
        let blocksize_bytes = u32::try_from(self.config.config.config().blocksize_bytes).unwrap();
        let mut blobstore =
            FsBlobStore::new(BlobStoreOnBlocks::new(blockstore, blocksize_bytes).await?);

        let pb = Progress::new(
            "Checking all blobs",
            u64::try_from(all_nodes.len()).unwrap(),
        );
        let check_all_blobs_result =
            check_all_reachable_blobs(&blobstore, &all_nodes, root_blob_id, &checks, pb.clone())
                .await;
        let check_all_blobs_result = match check_all_blobs_result {
            Ok(check_all_blobs_result) => check_all_blobs_result,
            Err(e) => {
                blobstore.async_drop().await?;
                return Err(e);
            }
        };

        let mut unreachable_nodes = set_remove_all(all_nodes, check_all_blobs_result.visited_nodes);

        let mut nodestore =
            BlobStoreOnBlocks::into_inner_node_store(FsBlobStore::into_inner_blobstore(blobstore));
        let check_unreachable_nodes_result =
            check_all_unreachable_nodes(&nodestore, &unreachable_nodes, &checks, pb.clone()).await;
        pb.finish();
        match check_unreachable_nodes_result {
            Ok(()) => (),
            Err(e) => {
                nodestore.async_drop().await?;
                return Err(e);
            }
        };

        let errors = checks.finalize();

        // Some errors may be found by multiple checks, let's deduplicate those.
        let errors = deduplicate(errors);

        nodestore.async_drop().await?;
        Ok(errors)
    }
}

async fn get_all_node_ids<B>(
    blockstore: &AsyncDropGuard<LockingBlockStore<B>>,
) -> Result<HashSet<BlockId>>
where
    B: BlockStore + Send + Sync + 'static,
{
    blockstore.all_blocks().await?.try_collect().await
}

async fn check_all_unreachable_nodes<B>(
    nodestore: &AsyncDropGuard<DataNodeStore<B>>,
    unreachable_nodes: &HashSet<BlockId>,
    checks: &AllChecks,
    pb: Progress,
) -> Result<()>
where
    B: BlockStore + Send + Sync + 'static,
{
    // TODO What's a good concurrency value here?
    const MAX_CONCURRENCY: usize = 100;

    let unreachable_nodes = stream::iter(unreachable_nodes.iter());
    let mut maybe_errors = unreachable_nodes.map(move |&node_id| {
        let pb = pb.clone();
        async move {
            let loaded = nodestore.load(node_id).await;
            let result = match loaded {
                Ok(Some(node)) => {
                    checks.process_unreachable_node(&node);
                    None // no error
                }
                Ok(None) => {
                    bail!("Node {node_id:?} previously present but then vanished during our checks. Please don't modify the file system while checks are running.");
                }
                Err(error) => {
                    // return the error
                    Some(CorruptedError::NodeUnreadable {
                        node_id,
                        // error,
                    })
                }
            };
            pb.inc(1);
            Ok(result)
        }
    })
    .buffer_unordered(MAX_CONCURRENCY);
    while let Some(maybe_error) = maybe_errors.next().await {
        if let Some(error) = maybe_error? {
            checks.add_error(error);
        }
    }
    Ok(())
}

#[must_use]
struct CheckAllBlobsResult {
    visited_nodes: Vec<BlockId>,
}

async fn check_all_reachable_blobs<B>(
    blobstore: &AsyncDropGuard<FsBlobStore<BlobStoreOnBlocks<B>>>,
    all_nodes: &HashSet<BlockId>,
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
        .send(Box::pin(_check_all_reachable_blobs(
            blobstore,
            all_nodes,
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
async fn _check_all_reachable_blobs<'a, 'b, 'c, 'f, B>(
    blobstore: &'a AsyncDropGuard<FsBlobStore<BlobStoreOnBlocks<B>>>,
    all_nodes: &'b HashSet<BlockId>,
    root_blob_id: BlobId,
    checks: &'c AllChecks,
    // TODO Is this possible without BoxFuture?
    task_queue_sender: UnboundedSender<BoxFuture<'f, Result<CheckAllBlobsResult>>>,
    pb: Progress,
) -> Result<CheckAllBlobsResult>
where
    B: BlockStore + Send + Sync + 'static,
    'a: 'f,
    'b: 'f,
    'c: 'f,
    'f: 'async_recursion,
{
    log::debug!("Entering blob {:?}", &root_blob_id);
    ensure!(all_nodes.contains(root_blob_id.to_root_block_id()), "Blob {root_blob_id:?} wasn't present before but is now. Please don't modify the file system while checks are running.");

    let loaded = blobstore.load(&root_blob_id).await;
    let mut result = CheckAllBlobsResult {
        visited_nodes: vec![*root_blob_id.to_root_block_id()],
    };
    match loaded {
        Ok(Some(mut blob)) => {
            checks.process_reachable_blob(&blob);

            // TODO Checking children blobs for directory blobs loads the nodes of this blob.
            //      Then we load it again when we check the nodes of this blob. Can we only load it once?

            // First, add tasks for all children blobs (if we're a directory blob).
            let subresult = check_all_reachable_children_blobs(
                blobstore,
                all_nodes,
                &blob,
                checks,
                task_queue_sender,
                pb.clone(),
            )
            .await;
            match subresult {
                Ok(()) => (),
                Err(err) => {
                    blob.async_drop().await?;
                    return Err(err);
                }
            };

            // Then, check all nodes of the current blob. This will be processed concurrently to the
            // children blobs added to the task queue above.
            let subresult = check_all_nodes_of_reachable_blob(blob, all_nodes, checks, pb).await?;
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
                // error,
            });
        }
    };
    log::debug!("Exiting blob {:?}", &root_blob_id);
    Ok(result)
}

async fn check_all_reachable_children_blobs<'a, 'b, 'c, 'd, 'f, B>(
    blobstore: &'a AsyncDropGuard<FsBlobStore<BlobStoreOnBlocks<B>>>,
    all_nodes: &'b HashSet<BlockId>,
    blob: &FsBlob<'c, BlobStoreOnBlocks<B>>,
    checks: &'d AllChecks,
    task_queue_sender: UnboundedSender<BoxFuture<'f, Result<CheckAllBlobsResult>>>,
    pb: Progress,
) -> Result<()>
where
    B: BlockStore + Send + Sync + 'static,
    'a: 'f,
    'b: 'f,
    'd: 'f,
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
                    .send(Box::pin(_check_all_reachable_blobs(
                        blobstore,
                        all_nodes,
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

async fn check_all_nodes_of_reachable_blob<'a, B>(
    blob: AsyncDropGuard<FsBlob<'a, BlobStoreOnBlocks<B>>>,
    all_nodes: &HashSet<BlockId>,
    checks: &AllChecks,
    pb: Progress,
) -> Result<CheckAllBlobsResult>
where
    B: BlockStore + Send + Sync + 'static,
{
    let all_nodes_of_blob = FsBlob::load_all_nodes(blob).await?;
    // TODO Should we rate-limit this instead of trying to load all at once?
    let mut results = all_nodes_of_blob.map(|node| {
        let (node_id, error) = match node {
            Ok(node) => {
                checks.process_reachable_node(&node);
                // no error
                (*node.block_id(), None)
            }
            Err((node_id, load_error)) => {
                // return the CorruptedError we found
                (
                    node_id,
                    Some(CorruptedError::NodeUnreadable { node_id/* , error: load_error*/ }),
                )
            }
        };
        ensure!(all_nodes.contains(&node_id), "Node {node_id:?} wasn't present before but is now. Please don't modify the file system while checks are running.");
        pb.inc(1);
        Ok((node_id, error))
    });
    let mut combined_result = CheckAllBlobsResult {
        visited_nodes: vec![],
    };
    while let Some(result) = results.next().await {
        let (node_id, error) = result?;
        combined_result.visited_nodes.push(node_id);
        if let Some(error) = error {
            checks.add_error(error);
        }
    }
    Ok(combined_result)
}

fn set_remove_all<T: Hash + Eq>(mut set: HashSet<T>, to_remove: Vec<T>) -> HashSet<T> {
    for item in to_remove {
        set.remove(&item);
    }
    set
}

fn deduplicate<T>(mut items: Vec<T>) -> Vec<T>
where
    T: Eq + Hash + Clone,
{
    // TODO Without clone?
    let mut seen: HashSet<T> = HashSet::new();
    items.retain(|item| seen.insert(item.clone()));
    items
}
