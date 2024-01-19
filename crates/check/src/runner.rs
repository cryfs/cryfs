use anyhow::Result;
use async_recursion::async_recursion;
use futures::stream::{self, StreamExt, TryStreamExt};
use itertools::{Either, Itertools};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use super::checks::AllChecks;
use super::error::{CheckError, CorruptedError};
use super::task_queue::{self, TaskSpawner};
use cryfs_blobstore::{
    BlobId, BlobStore, BlobStoreOnBlocks, DataNode, DataNodeStore, DataTreeStore, LoadNodeError,
};
use cryfs_blockstore::{BlockId, BlockStore, LockingBlockStore};
use cryfs_cli_utils::BlockstoreCallback;
use cryfs_cryfs::{
    config::ConfigLoadResult,
    filesystem::fsblobstore::{BlobType, FsBlob, FsBlobStore},
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    containers::{HashMapExt, OccupiedError},
    progress::{Progress, Spinner},
};

// TODO What's a good concurrency value here?
const MAX_CONCURRENCY: usize = 100;

pub struct RecoverRunner<'c> {
    pub config: &'c ConfigLoadResult,
}

impl<'l> BlockstoreCallback for RecoverRunner<'l> {
    type Result = Result<Vec<CorruptedError>>;

    async fn callback<B: BlockStore + AsyncDrop + Send + Sync + 'static>(
        self,
        mut blockstore: AsyncDropGuard<LockingBlockStore<B>>,
    ) -> Self::Result {
        // TODO Function too large. Split into subfunctions

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
        let processed_blobs = Arc::new(ProcessedItems::new());
        let check_all_reachable_blobs_result = check_all_reachable_blobs(
            &blobstore,
            &all_nodes,
            Arc::clone(&processed_blobs),
            root_blob_id,
            &checks,
            pb.clone(),
        )
        .await;
        match check_all_reachable_blobs_result {
            Ok(()) => (),
            Err(e) => {
                blobstore.async_drop().await?;
                return Err(e);
            }
        };
        let processed_blobs = Arc::try_unwrap(processed_blobs)
            .expect("All tasks are finished here and we should be able to unwrap the Arc");
        let reachable_blobs: Vec<BlobId> = processed_blobs.into_keys().collect();
        pb.finish();

        let pb = Progress::new(
            "Checking all nodes",
            u64::try_from(all_nodes.len()).unwrap(),
        );
        let mut nodestore = DataTreeStore::into_inner_node_store(
            BlobStoreOnBlocks::into_inner_tree_store(FsBlobStore::into_inner_blobstore(blobstore)),
        );
        let processed_nodes = Arc::new(ProcessedItems::new());
        let check_all_nodes_of_reachable_blobs_result = check_all_nodes_of_reachable_blobs(
            &nodestore,
            reachable_blobs,
            &all_nodes,
            Arc::clone(&processed_nodes),
            &checks,
            pb.clone(),
        )
        .await;
        match check_all_nodes_of_reachable_blobs_result {
            Ok(()) => (),
            Err(e) => {
                nodestore.async_drop().await?;
                return Err(e.into());
            }
        };

        let processed_nodes = Arc::try_unwrap(processed_nodes)
            .expect("All tasks are finished here and we should be able to unwrap the Arc");
        let unreachable_nodes = set_remove_all(all_nodes, processed_nodes.into_keys());
        let check_unreachable_nodes_result =
            check_all_unreachable_nodes(&nodestore, &unreachable_nodes, &checks, pb.clone()).await;
        pb.finish();
        match check_unreachable_nodes_result {
            Ok(()) => (),
            Err(e) => {
                nodestore.async_drop().await?;
                return Err(e.into());
            }
        };

        let errors = checks.finalize();

        let errors = run_assertions(errors);

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
) -> Result<(), CheckError>
where
    B: BlockStore + Send + Sync + 'static,
{
    stream::iter(unreachable_nodes.iter())
        .map(move |&node_id| {
            let pb = pb.clone();
            async move {
                let loaded = nodestore.load(node_id).await;
                match loaded {
                    Ok(Some(node)) => {
                        checks.process_unreachable_node(&node)?;
                    }
                    Ok(None) => {
                        return Err(CheckError::FilesystemModified { msg: format!(
                            "Node {node_id:?} previously present but then vanished during our checks."
                        )});
                    }
                    Err(error) => {
                        checks.process_unreachable_unreadable_node(node_id)?;
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
    already_processed_blobs: Arc<ProcessedItems<BlobId, BlobInfo>>,
    root_blob_id: BlobId,
    checks: &AllChecks,
    pb: Progress,
) -> Result<()>
where
    B: BlockStore + Send + Sync + 'static,
{
    task_queue::run_to_completion(MAX_CONCURRENCY, |task_spawner| {
        _check_all_reachable_blobs(
            blobstore,
            all_nodes,
            Arc::clone(&already_processed_blobs),
            root_blob_id,
            checks,
            task_spawner,
            pb,
        )
    })
    .await?;

    Ok(())
}

#[async_recursion]
async fn _check_all_reachable_blobs<'a, 'b, 'c, 'f, B>(
    blobstore: &'a AsyncDropGuard<FsBlobStore<BlobStoreOnBlocks<B>>>,
    all_nodes: &'b HashSet<BlockId>,
    already_processed_blobs: Arc<ProcessedItems<BlobId, BlobInfo>>,
    root_blob_id: BlobId,
    checks: &'c AllChecks,
    task_spawner: TaskSpawner<'f>,
    pb: Progress,
) -> Result<()>
where
    B: BlockStore + Send + Sync + 'static,
    'a: 'f,
    'b: 'f,
    'c: 'f,
    'f: 'async_recursion,
{
    // TODO Function too large, split into subfunctions

    log::debug!("Entering blob {:?}", &root_blob_id);

    let loaded = blobstore.load(&root_blob_id).await;

    let blob_info = match &loaded {
        Ok(Some(blob)) => processed_blob_info(blob),
        Ok(None) => BlobInfo::Missing,
        Err(_) => BlobInfo::Unreadable,
    };
    match already_processed_blobs.add(root_blob_id, blob_info) {
        AlreadySeen::AlreadySeen {
            prev_seen,
            now_seen,
        } => {
            // The blob was already seen before. This can only happen if the blob is referenced multiple times.
            checks.add_error(CorruptedError::Assert(Box::new(
                CorruptedError::BlobReferencedMultipleTimes {
                    blob_id: root_blob_id,
                },
            )));

            // Let's make sure it still looks the same (e.g. still has the same children) and then we can skip processing it.
            if prev_seen != now_seen {
                Err(CheckError::FilesystemModified {
                    msg: format!(
                        "Blob {blob_id:?} was previously seen as {prev_seen:?} but now as {now_seen:?}",
                        blob_id = root_blob_id,
                        prev_seen = prev_seen,
                        now_seen = now_seen,
                    ),
                })?;
            }
        }
        AlreadySeen::NotSeenYet => {
            match loaded {
                Ok(Some(mut blob)) => {
                    if !all_nodes.contains(root_blob_id.to_root_block_id()) {
                        Err(CheckError::FilesystemModified {
                            msg: format!("Blob {root_blob_id:?} wasn't present before but is now."),
                        })?;
                    }

                    checks.process_reachable_blob(&blob)?;

                    // TODO Checking children blobs for directory blobs loads the nodes of this blob.
                    //      Then we load it again when we check the nodes of this blob. Can we only load it once?

                    let subresult = check_all_reachable_children_blobs(
                        blobstore,
                        all_nodes,
                        already_processed_blobs,
                        &blob,
                        checks,
                        task_spawner,
                        pb.clone(),
                    )
                    .await;
                    blob.async_drop().await?;
                    subresult?;
                }
                Ok(None) => {
                    // This is already reported by the [super::unreferenced_nodes] check but let's assert that it is
                    checks.add_error(CorruptedError::Assert(Box::new(
                        CorruptedError::BlobMissing {
                            blob_id: root_blob_id,
                        },
                    )));
                }
                Err(error) => {
                    checks.add_error(CorruptedError::BlobUnreadable {
                        blob_id: root_blob_id,
                        // error,
                    });
                }
            };
            pb.inc(1);
        }
    }

    log::debug!("Exiting blob {:?}", &root_blob_id);
    Ok(())
}

async fn check_all_reachable_children_blobs<'a, 'b, 'c, 'd, 'f, B>(
    blobstore: &'a AsyncDropGuard<FsBlobStore<BlobStoreOnBlocks<B>>>,
    all_nodes: &'b HashSet<BlockId>,
    already_processed_blobs: Arc<ProcessedItems<BlobId, BlobInfo>>,
    blob: &FsBlob<'c, BlobStoreOnBlocks<B>>,
    checks: &'d AllChecks,
    task_spawner: TaskSpawner<'f>,
    pb: Progress,
) -> Result<(), CheckError>
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
                    return Err(CheckError::FilesystemModified {
                        msg: format!(
                            "Blob {blob_id:?} previously was a directory blob but now isn't",
                            blob_id = blob.blob_id(),
                        ),
                    });
                }
            };

            // TODO If we manage to prioritize directory blobs over file blobs in the processing, then the progress bar max would be correct much more quickly.
            //      We could for example do it with two task queues, one for directory blobs and one for file/symlink blobs, and then chain them together.
            pb.inc_length(blob.entries().len() as u64);

            for entry in blob.entries() {
                task_spawner.spawn(|task_spawner| {
                    _check_all_reachable_blobs(
                        blobstore,
                        all_nodes,
                        Arc::clone(&already_processed_blobs),
                        *entry.blob_id(),
                        checks,
                        task_spawner,
                        pb.clone(),
                    )
                });
            }
        }
    };
    Ok(())
}

async fn check_all_nodes_of_reachable_blobs<B>(
    nodestore: &DataNodeStore<B>,
    all_blobs: Vec<BlobId>,
    allowed_nodes: &HashSet<BlockId>,
    already_processed_nodes: Arc<ProcessedItems<BlockId, NodeInfo>>,
    checks: &AllChecks,
    pb: Progress,
) -> Result<(), CheckError>
where
    B: BlockStore + Send + Sync + 'static,
{
    task_queue::run_to_completion(MAX_CONCURRENCY, move |task_spawner| async move {
        for blob_id in all_blobs {
            let already_processed_nodes = Arc::clone(&already_processed_nodes);
            let pb = pb.clone();
            task_spawner.spawn(|task_spawner| {
                check_all_nodes_of_reachable_blob(
                    nodestore,
                    blob_id,
                    *blob_id.to_root_block_id(),
                    allowed_nodes,
                    already_processed_nodes,
                    checks,
                    task_spawner,
                    pb,
                )
            })
        }
        Ok(())
    })
    .await?;

    Ok(())
}

async fn check_all_nodes_of_reachable_blob<'a, 'b, 'c, 'f, B>(
    nodestore: &'a DataNodeStore<B>,
    blob_id: BlobId,
    current_node_id: BlockId,
    expected_nodes: &'b HashSet<BlockId>,
    already_processed_nodes: Arc<ProcessedItems<BlockId, NodeInfo>>,
    checks: &'c AllChecks,
    task_spawner: TaskSpawner<'f, CheckError>,
    pb: Progress,
) -> Result<(), CheckError>
where
    B: BlockStore + Send + Sync + 'static,
    'a: 'f,
    'b: 'f,
    'c: 'f,
{
    // TODO Function too long, split into subfunctions

    let node = nodestore.load(current_node_id).await;
    let node_info = match &node {
        Ok(Some(node)) => processed_node_info(node),
        Ok(None) => NodeInfo::Missing,
        Err(err) => NodeInfo::Unreadable,
    };

    match already_processed_nodes.add(current_node_id, node_info) {
        AlreadySeen::AlreadySeen {
            prev_seen,
            now_seen,
        } => {
            // The node was already seen before. This can only happen if the node is referenced multiple times.
            checks.add_error(CorruptedError::Assert(Box::new(
                CorruptedError::NodeReferencedMultipleTimes {
                    node_id: current_node_id,
                },
            )));

            // Let's make sure it still looks the same (e.g. still has the same children) and then we can skip processing it.

            if prev_seen != now_seen {
                return Err(CheckError::FilesystemModified {
                        msg: format!(
                            "Node {current_node_id:?} was previously seen as {prev_seen:?} but now as {now_seen:?}",
                        ),
                    });
            }
        }
        AlreadySeen::NotSeenYet => {
            match node {
                Ok(Some(node)) => {
                    if !expected_nodes.contains(&current_node_id) {
                        return Err(CheckError::FilesystemModified {
                            msg: format!(
                                "Node {current_node_id:?} wasn't present before but is now.",
                            ),
                        });
                    }
                    check_all_children_of_reachable_blob_node(
                        nodestore,
                        blob_id,
                        &node,
                        expected_nodes,
                        already_processed_nodes,
                        checks,
                        task_spawner,
                        pb.clone(),
                    );
                    checks.process_reachable_node(&node)?;
                }
                Ok(None) => {
                    if expected_nodes.contains(&current_node_id) {
                        return Err(CheckError::FilesystemModified {
                            msg: format!(
                                "Node {current_node_id:?} was present before but is now missing.",
                            ),
                        });
                    }
                    if current_node_id == *blob_id.to_root_block_id() {
                        checks.add_error(CorruptedError::Assert(Box::new(
                            CorruptedError::BlobMissing { blob_id },
                        )));
                    } else {
                        checks.add_error(CorruptedError::Assert(Box::new(
                            CorruptedError::NodeMissing {
                                node_id: current_node_id,
                            },
                        )));
                    }
                }
                Err(error) => {
                    if !expected_nodes.contains(&current_node_id) {
                        return Err(CheckError::FilesystemModified {
                            msg: format!(
                                "Node {current_node_id:?} wasn't present before but is now.",
                            ),
                        });
                    }
                    checks.process_reachable_unreadable_node(current_node_id)?;
                    checks.add_error(CorruptedError::Assert(Box::new(
                        CorruptedError::NodeUnreadable {
                            node_id: current_node_id,
                        },
                    )));
                }
            };
            pb.inc(1);
        }
    }

    Ok(())
}

fn check_all_children_of_reachable_blob_node<'a, 'b, 'c, 'f, B>(
    nodestore: &'a DataNodeStore<B>,
    blob_id: BlobId,
    current_node: &DataNode<B>,
    expected_nodes: &'b HashSet<BlockId>,
    already_processed_nodes: Arc<ProcessedItems<BlockId, NodeInfo>>,
    checks: &'c AllChecks,
    task_spawner: TaskSpawner<'f, CheckError>,
    pb: Progress,
) where
    B: BlockStore + Send + Sync + 'static,
    'a: 'f,
    'b: 'f,
    'c: 'f,
{
    match current_node {
        DataNode::Leaf(_) => {
            // Leaf nodes don't have children. Nothing to do.
        }
        DataNode::Inner(node) => {
            // Get all children and recurse into their nodes, concurrently.
            for child_id in node.children() {
                task_spawner.spawn(|task_spawner| {
                    check_all_nodes_of_reachable_blob(
                        nodestore,
                        blob_id,
                        child_id,
                        expected_nodes,
                        Arc::clone(&already_processed_nodes),
                        checks,
                        task_spawner,
                        pb.clone(),
                    )
                });
            }
        }
    }
}

fn set_remove_all<T: Hash + Eq>(
    mut set: HashSet<T>,
    to_remove: impl Iterator<Item = T>,
) -> HashSet<T> {
    for item in to_remove {
        set.remove(&item);
    }
    set
}

fn run_assertions(errors: Vec<CorruptedError>) -> Vec<CorruptedError> {
    // Run assertions to make sure all errors that some checks found on the side were reported by the check responsible for them
    let (errors, assertions): (Vec<CorruptedError>, Vec<CorruptedError>) =
        errors.into_iter().partition_map(|error| match error {
            CorruptedError::Assert(err) => Either::Right(*err),
            err => Either::Left(err),
        });
    for assertion in assertions {
        assert!(
            errors.contains(&assertion),
            "CorruptedError::Assert failed: {assertion:?}"
        );
    }
    errors
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum NodeInfo {
    Unreadable,
    Missing,
    Leaf,
    Inner {
        // We're storing children into the [NodeInfo] so that if the node comes up again,
        // we can check that it still has the same children. This allows us to know that
        // we already processed those children when we saw the blob for the first time.
        // TODO Vec instead of HashSet should be enough
        children: HashSet<BlockId>,
    },
}

fn processed_node_info<B>(node: &DataNode<B>) -> NodeInfo
where
    B: BlockStore + Send + Sync + 'static,
{
    match node {
        DataNode::Leaf(_) => NodeInfo::Leaf,
        DataNode::Inner(node) => NodeInfo::Inner {
            children: node.children().collect(),
        },
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum BlobInfo {
    Unreadable,
    Missing,
    File,
    Symlink,
    Dir {
        // We're storing children into the [NodeInfo] so that if the node comes up again,
        // we can check that it still has the same children. This allows us to know that
        // we already processed those children when we saw the blob for the first time.
        children: Vec<BlobId>,
    },
}

fn processed_blob_info<'a, B>(blob: &FsBlob<'a, B>) -> BlobInfo
where
    // TODO Do we really need B: 'static ?
    B: BlobStore + Debug + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send,
{
    match blob {
        FsBlob::File(_) => BlobInfo::File,
        FsBlob::Symlink(_) => BlobInfo::Symlink,
        FsBlob::Directory(blob) => BlobInfo::Dir {
            children: blob.entries().map(|entry| *entry.blob_id()).collect(),
        },
    }
}

#[derive(Debug)]
struct ProcessedItems<ItemId, ItemInfo>
where
    ItemId: Debug + PartialEq + Eq + Hash,
    ItemInfo: Clone,
{
    nodes: Mutex<HashMap<ItemId, ItemInfo>>,
}

impl<ItemId, ItemInfo> ProcessedItems<ItemId, ItemInfo>
where
    ItemId: Debug + PartialEq + Eq + Hash,
    ItemInfo: Clone,
{
    pub fn new() -> Self {
        Self {
            nodes: Mutex::new(HashMap::new()),
        }
    }

    #[must_use]
    pub fn add(&self, id: ItemId, node: ItemInfo) -> AlreadySeen<ItemInfo> {
        match HashMapExt::try_insert(&mut *self.nodes.lock().unwrap(), id, node) {
            Ok(_) => AlreadySeen::NotSeenYet,
            Err(OccupiedError { entry, value, .. }) => AlreadySeen::AlreadySeen {
                prev_seen: entry.get().clone(),
                now_seen: value,
            },
        }
    }

    pub fn into_keys(self) -> impl Iterator<Item = ItemId> {
        self.nodes.into_inner().unwrap().into_keys()
    }
}

#[must_use]
enum AlreadySeen<ItemInfo> {
    NotSeenYet,
    AlreadySeen {
        // TODO Return `prev_seen` by reference, not clone
        prev_seen: ItemInfo,
        now_seen: ItemInfo,
    },
}
