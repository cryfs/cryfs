use anyhow::{bail, ensure, Result};
use async_recursion::async_recursion;
use futures::future::BoxFuture;
use futures::stream::{self, StreamExt, TryStreamExt};
use std::collections::HashSet;
use std::hash::Hash;
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::wrappers::UnboundedReceiverStream;

use super::checks::AllChecks;
use super::error::CorruptedError;
use cryfs_blobstore::{BlobId, BlobStoreOnBlocks, DataNodeStore, DataTreeStore, LoadNodeError};
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

        let pb = Progress::new("Checking all reachable blobs", 1);
        let reachable_blobs =
            check_all_reachable_blobs(&blobstore, &all_nodes, root_blob_id, &checks, pb.clone())
                .await;
        let reachable_blobs = match reachable_blobs {
            Ok(reachable_blobs) => reachable_blobs,
            Err(e) => {
                blobstore.async_drop().await?;
                return Err(e);
            }
        };
        pb.finish();

        let pb = Progress::new(
            "Checking all nodes",
            u64::try_from(all_nodes.len()).unwrap(),
        );
        let mut treestore =
            BlobStoreOnBlocks::into_inner_tree_store(FsBlobStore::into_inner_blobstore(blobstore));
        let reachable_nodes = check_all_nodes_of_reachable_blobs(
            &treestore,
            reachable_blobs,
            &all_nodes,
            &checks,
            pb.clone(),
        )
        .await;
        let reachable_nodes = match reachable_nodes {
            Ok(reachable_nodes) => reachable_nodes,
            Err(e) => {
                treestore.async_drop().await?;
                return Err(e);
            }
        };

        let unreachable_nodes = set_remove_all(all_nodes, reachable_nodes);
        let mut nodestore = DataTreeStore::into_inner_node_store(treestore);
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

    stream::iter(unreachable_nodes.iter())
        .map(move |&node_id| {
            let pb = pb.clone();
            async move {
                let loaded = nodestore.load(node_id).await;
                match loaded {
                    Ok(Some(node)) => {
                        checks.process_unreachable_node(&node);
                    }
                    Ok(None) => {
                        bail!("Node {node_id:?} previously present but then vanished during our checks. Please don't modify the file system while checks are running.");
                    }
                    Err(error) => {
                        checks.process_unreachable_unreadable_node(node_id);
                    }
                };
                pb.inc(1);
                Ok(())
            }
        })
        .buffer_unordered(MAX_CONCURRENCY)
        .try_collect::<Vec<()>>()
        .await?;
    Ok(())
}

async fn check_all_reachable_blobs<B>(
    blobstore: &AsyncDropGuard<FsBlobStore<BlobStoreOnBlocks<B>>>,
    all_nodes: &HashSet<BlockId>,
    root_blob_id: BlobId,
    checks: &AllChecks,
    pb: Progress,
) -> Result<Vec<BlobId>>
where
    B: BlockStore + Send + Sync + 'static,
{
    // TODO What's a good concurrency value here?
    const MAX_CONCURRENCY: usize = 100;

    let (task_queue_sender, task_queue_receiver) = tokio::sync::mpsc::unbounded_channel();

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

    UnboundedReceiverStream::new(task_queue_receiver)
        .buffer_unordered(MAX_CONCURRENCY)
        .try_collect()
        .await
}

#[async_recursion]
async fn _check_all_reachable_blobs<'a, 'b, 'c, 'f, B>(
    blobstore: &'a AsyncDropGuard<FsBlobStore<BlobStoreOnBlocks<B>>>,
    all_nodes: &'b HashSet<BlockId>,
    root_blob_id: BlobId,
    checks: &'c AllChecks,
    // TODO Is this possible without BoxFuture?
    task_queue_sender: UnboundedSender<BoxFuture<'f, Result<BlobId>>>,
    pb: Progress,
) -> Result<BlobId>
where
    B: BlockStore + Send + Sync + 'static,
    'a: 'f,
    'b: 'f,
    'c: 'f,
    'f: 'async_recursion,
{
    log::debug!("Entering blob {:?}", &root_blob_id);

    let loaded = blobstore.load(&root_blob_id).await;

    match loaded {
        Ok(Some(mut blob)) => {
            ensure!(all_nodes.contains(root_blob_id.to_root_block_id()), "Blob {root_blob_id:?} wasn't present before but is now. Please don't modify the file system while checks are running.");

            checks.process_reachable_blob(&blob);

            // TODO Checking children blobs for directory blobs loads the nodes of this blob.
            //      Then we load it again when we check the nodes of this blob. Can we only load it once?

            let subresult = check_all_reachable_children_blobs(
                blobstore,
                all_nodes,
                &blob,
                checks,
                task_queue_sender,
                pb.clone(),
            )
            .await;
            blob.async_drop().await?;
            subresult?;
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
    pb.inc(1);
    log::debug!("Exiting blob {:?}", &root_blob_id);
    Ok(root_blob_id)
}

async fn check_all_reachable_children_blobs<'a, 'b, 'c, 'd, 'f, B>(
    blobstore: &'a AsyncDropGuard<FsBlobStore<BlobStoreOnBlocks<B>>>,
    all_nodes: &'b HashSet<BlockId>,
    blob: &FsBlob<'c, BlobStoreOnBlocks<B>>,
    checks: &'d AllChecks,
    task_queue_sender: UnboundedSender<BoxFuture<'f, Result<BlobId>>>,
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

            // TODO If we manage to prioritize directory blobs over file blobs in the processing, then the progress bar max would be correct much more quickly.
            //      We could for example do it with two task queues, one for directory blobs and one for file/symlink blobs, and then chain them together.
            pb.inc_length(blob.entries().len() as u64);

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

async fn check_all_nodes_of_reachable_blobs<B>(
    treestore: &DataTreeStore<B>,
    all_blobs: Vec<BlobId>,
    allowed_nodes: &HashSet<BlockId>,
    checks: &AllChecks,
    pb: Progress,
) -> Result<Vec<BlockId>>
where
    B: BlockStore + Send + Sync + 'static,
{
    // TODO What's a good concurrency value here?
    const MAX_CONCURRENCY: usize = 100;

    let mut tasks = stream::iter(all_blobs.into_iter().map(|blob_root| {
        check_all_nodes_of_reachable_blob(
            treestore,
            *blob_root.to_root_block_id(),
            allowed_nodes,
            checks,
            pb.clone(),
        )
    }))
    .buffer_unordered(MAX_CONCURRENCY);

    let mut result = vec![];
    while let Some(task) = tasks.next().await {
        result.extend(task?);
    }
    Ok(result)
}

async fn check_all_nodes_of_reachable_blob<B>(
    treestore: &DataTreeStore<B>,
    root_node_id: BlockId,
    expected_nodes: &HashSet<BlockId>,
    checks: &AllChecks,
    pb: Progress,
    // TODO Return stream? But measure that it's not slower
) -> Result<Vec<BlockId>>
where
    B: BlockStore + Send + Sync + 'static,
{
    let mut all_nodes_of_blob = treestore
        .load_all_nodes_in_subtree_of_id(root_node_id)
        .await;
    let mut visited_nodes = vec![];
    while let Some(node) = all_nodes_of_blob.next().await {
        let node_id = match node {
            Ok(node) => {
                ensure!(expected_nodes.contains(node.block_id()), "Node {node_id:?} wasn't present before but is now. Please don't modify the file system while checks are running.", node_id=node.block_id());
                checks.process_reachable_node(&node);
                *node.block_id()
            }
            Err(LoadNodeError::NodeNotFound { node_id }) => {
                ensure!(!expected_nodes.contains(&node_id), "Node {node_id:?} was present before but is now missing. Please don't modify the file system while checks are running.");
                if node_id == root_node_id {
                    checks.add_error(CorruptedError::BlobMissing {
                        blob_id: BlobId::from_root_block_id(node_id),
                    });
                } else {
                    checks.add_error(CorruptedError::NodeMissing { node_id });
                }
                node_id
            }
            Err(LoadNodeError::NodeLoadError { node_id, error }) => {
                ensure!(expected_nodes.contains(&node_id), "Node {node_id:?} wasn't present before but is now. Please don't modify the file system while checks are running.");
                checks.process_reachable_unreadable_node(node_id);
                node_id
            }
        };
        visited_nodes.push(node_id);
        pb.inc(1);
    }
    Ok(visited_nodes)
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
