use anyhow::Result;
use async_recursion::async_recursion;
use cryfs_cryfs::filesystem::fsblobstore::EntryType;
use futures::stream::{self, StreamExt, TryStreamExt};
use itertools::{Either, Itertools};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::num::NonZeroU8;
use std::sync::{Arc, Mutex};

use crate::error::{
    BlobInfoAsExpectedByEntryInParent, NodeInfoAsSeenByLookingAtNode, NodeReference,
    NodeReferenceFromReachableBlob,
};
use crate::{
    BlobInfoAsSeenByLookingAtBlob, NodeInfoAsExpectedByEntryInParent, ReferencingBlobInfo,
};

use super::checks::{AllChecks, BlobToProcess, NodeToProcess};
use super::error::{CheckError, CorruptedError};
use super::task_queue::{self, TaskSpawner};
use cryfs_blobstore::{
    BlobId, BlobStore, BlobStoreOnBlocks, DataNode, DataNodeStore, DataTreeStore,
};
use cryfs_blockstore::{BlockId, BlockStore, LockingBlockStore};
use cryfs_cli_utils::BlockstoreCallback;
use cryfs_cryfs::{
    config::ConfigLoadResult,
    filesystem::fsblobstore::{BlobType, FsBlob, FsBlobStore},
};
// TODO Move AbsolutePathBuf to common crate and remove dependency on cryfs_rustfs.
use cryfs_rustfs::AbsolutePathBuf;
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    containers::{HashMapExt, OccupiedError},
    progress::{Progress, ProgressBarManager, Spinner},
};

// TODO What's a good concurrency value here?
const MAX_CONCURRENCY: usize = 100;

pub struct RecoverRunner<'c, PBM: ProgressBarManager> {
    pub progress_bar_manager: PBM,
    pub config: &'c ConfigLoadResult,
}

impl<'l, PBM: ProgressBarManager> BlockstoreCallback for RecoverRunner<'l, PBM> {
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
        let pb = self
            .progress_bar_manager
            .new_spinner_autotick("Listing all nodes");
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

        let pb = self
            .progress_bar_manager
            .new_progress_bar("Checking all reachable blobs", 1);
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
        let reachable_blobs: Vec<(BlobId, BlobInfoAsExpectedByEntryInParent)> = processed_blobs
            .into_iter()
            .map(|(blob_id, (blob_info, _seen_blob_info))| (blob_id, blob_info))
            .collect();
        pb.finish();

        let pb = self.progress_bar_manager.new_progress_bar(
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
    pb: impl Progress,
) -> Result<(), CheckError>
where
    B: BlockStore + Send + Sync + 'static,
{
    stream::iter(unreachable_nodes.iter())
        .map(move |&node_id| {
            let pb = pb.clone();
            async move {
                let loaded = nodestore.load(node_id).await;
                let node_to_process = match loaded {
                    Ok(Some(node)) => {
                        NodeToProcess::Readable(node)
                    }
                    Ok(None) => {
                        return Err(CheckError::FilesystemModified { msg: format!(
                            "Node {node_id:?} previously present but then vanished during our checks."
                        )});
                    }
                    Err(error) => {
                        NodeToProcess::Unreadable(node_id)
                    }
                };
                checks.process_unreachable_node(&node_to_process)?;
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
    already_processed_blobs: Arc<
        ProcessedItems<BlobId, (BlobInfoAsExpectedByEntryInParent, SeenBlobInfo)>,
    >,
    root_blob_id: BlobId,
    checks: &AllChecks,
    pb: impl Progress,
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
            BlobInfoAsExpectedByEntryInParent::root_dir(),
            checks,
            task_spawner,
            pb,
        )
    })
    .await?;

    Ok(())
}

#[async_recursion]
async fn _check_all_reachable_blobs<'a, 'b, 'c, 'd, 'f, B>(
    blobstore: &'a AsyncDropGuard<FsBlobStore<BlobStoreOnBlocks<B>>>,
    all_nodes: &'b HashSet<BlockId>,
    already_processed_blobs: Arc<
        ProcessedItems<BlobId, (BlobInfoAsExpectedByEntryInParent, SeenBlobInfo)>,
    >,
    blob_id: BlobId,
    blob_info: BlobInfoAsExpectedByEntryInParent,
    checks: &'c AllChecks,
    task_spawner: TaskSpawner<'f>,
    pb: impl Progress + 'd,
) -> Result<()>
where
    B: BlockStore + Send + Sync + 'static,
    'a: 'f,
    'b: 'f,
    'c: 'f,
    'd: 'f,
    'f: 'async_recursion,
{
    // TODO Function too large, split into subfunctions

    log::debug!("Entering blob {blob_info}");

    let loaded = blobstore.load(&blob_id).await;

    let seen_blob_info = match &loaded {
        Ok(Some(blob)) => blob_content_summary(blob),
        Ok(None) => SeenBlobInfo::Missing,
        Err(_) => SeenBlobInfo::Unreadable,
    };
    match already_processed_blobs.add(blob_id, (blob_info.clone(), seen_blob_info)) {
        AlreadySeen::AlreadySeen {
            prev_seen,
            now_seen,
        } => {
            if let Ok(Some(mut blob)) = loaded {
                blob.async_drop().await?;
            }

            // Let's make sure it still looks the same (e.g. still has the same children) and then we can skip processing it.
            if prev_seen.1 != now_seen.1 {
                Err(CheckError::FilesystemModified {
                    msg: format!(
                        "{blob_info} was previously seen as {prev_seen:?} but now as {now_seen:?}",
                        prev_seen = prev_seen,
                        now_seen = now_seen,
                    ),
                })?;
            }

            // The blob was already seen before. This can only happen if the blob is referenced multiple times.
            // TODO
            // checks.add_error(CorruptedError::Assert(Box::new(
            //     CorruptedError::BlobReferencedMultipleTimes {
            //         blob_id,
            //         blob_info: now_seen.1.to_blob_info_as_seen_by_looking_at_blob(),
            //         referenced_as: todo!(),
            //     },
            // )));
        }
        AlreadySeen::NotSeenYet => {
            match loaded {
                Ok(Some(mut blob)) => {
                    if !all_nodes.contains(blob_id.to_root_block_id()) {
                        Err(CheckError::FilesystemModified {
                            msg: format!("{blob_info} wasn't present before but is now."),
                        })?;
                    }

                    // TODO Add this assert here and a real blob type check to the list of checks
                    // if expected_blob_type != blob.blob_type() {
                    //     checks.add_error(CorruptedError::Assert(Box::new(
                    //         CorruptedError::WrongBlobType,
                    //     )));
                    // }

                    // TODO Checking children blobs for directory blobs loads the nodes of this blob.
                    //      Then we load it again when we check the nodes of this blob. Can we only load it once?

                    let subresult = check_all_reachable_children_blobs(
                        blobstore,
                        all_nodes,
                        already_processed_blobs,
                        &blob,
                        blob_info.path.clone(),
                        checks,
                        task_spawner,
                        pb.clone(),
                    )
                    .await;

                    let process_result =
                        checks.process_reachable_blob(BlobToProcess::Readable(&blob), &blob_info);

                    blob.async_drop().await?;
                    subresult?;
                    process_result?;
                }
                Ok(None) => {
                    // This is already reported by the [super::unreferenced_nodes] check but let's assert that it is
                    checks.add_error(CorruptedError::Assert(Box::new(
                        CorruptedError::BlobMissing {
                            blob_id,
                            expected_blob_info: blob_info.clone(),
                        },
                    )));
                }
                Err(error) => {
                    checks.process_reachable_blob(
                        BlobToProcess::<B>::Unreadable(blob_id),
                        &blob_info,
                    )?;
                    checks.add_error(CorruptedError::BlobUnreadable {
                        blob_id,
                        expected_blob_info: blob_info.clone(),
                        // error,
                    });
                }
            };
            pb.inc(1);
        }
    }

    log::debug!("Exiting blob {blob_info}");
    Ok(())
}

async fn check_all_reachable_children_blobs<'a, 'b, 'c, 'd, 'e, 'f, B>(
    blobstore: &'a AsyncDropGuard<FsBlobStore<BlobStoreOnBlocks<B>>>,
    all_nodes: &'b HashSet<BlockId>,
    already_processed_blobs: Arc<
        ProcessedItems<BlobId, (BlobInfoAsExpectedByEntryInParent, SeenBlobInfo)>,
    >,
    blob: &FsBlob<'c, BlobStoreOnBlocks<B>>,
    path_of_blob: AbsolutePathBuf,
    checks: &'d AllChecks,
    task_spawner: TaskSpawner<'f>,
    pb: impl Progress + 'e,
) -> Result<(), CheckError>
where
    B: BlockStore + Send + Sync + 'static,
    'a: 'f,
    'b: 'f,
    'd: 'f,
    'e: 'f,
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
                        BlobInfoAsExpectedByEntryInParent {
                            blob_type: entry_type_to_blob_type(entry.entry_type()),
                            parent_id: blob.blob_id(),
                            path: path_of_blob.join(entry.name()),
                        },
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

fn entry_type_to_blob_type(entry_type: EntryType) -> BlobType {
    match entry_type {
        EntryType::File => BlobType::File,
        EntryType::Dir => BlobType::Dir,
        EntryType::Symlink => BlobType::Symlink,
    }
}

async fn check_all_nodes_of_reachable_blobs<B>(
    nodestore: &DataNodeStore<B>,
    all_blobs: Vec<(BlobId, BlobInfoAsExpectedByEntryInParent)>,
    allowed_nodes: &HashSet<BlockId>,
    already_processed_nodes: Arc<ProcessedItems<BlockId, SeenNodeInfo>>,
    checks: &AllChecks,
    pb: impl Progress,
) -> Result<(), CheckError>
where
    B: BlockStore + Send + Sync + 'static,
{
    task_queue::run_to_completion(MAX_CONCURRENCY, move |task_spawner| async move {
        for (blob_id, blob_info) in all_blobs {
            let already_processed_nodes = Arc::clone(&already_processed_nodes);
            let pb = pb.clone();
            task_spawner.spawn(|task_spawner| {
                check_all_nodes_of_reachable_blob(
                    nodestore,
                    *blob_id.to_root_block_id(),
                    NodeReferenceFromReachableBlob {
                        node_info: NodeInfoAsExpectedByEntryInParent::RootNode,
                        blob_info: ReferencingBlobInfo { blob_id, blob_info },
                    },
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

async fn check_all_nodes_of_reachable_blob<'a, 'b, 'c, 'd, 'f, B>(
    nodestore: &'a DataNodeStore<B>,
    current_node_id: BlockId,
    current_expected_node_info: NodeReferenceFromReachableBlob,
    expected_nodes: &'b HashSet<BlockId>,
    already_processed_nodes: Arc<ProcessedItems<BlockId, SeenNodeInfo>>,
    checks: &'c AllChecks,
    task_spawner: TaskSpawner<'f, CheckError>,
    pb: impl Progress + 'd,
) -> Result<(), CheckError>
where
    B: BlockStore + Send + Sync + 'static,
    'a: 'f,
    'b: 'f,
    'c: 'f,
    'd: 'f,
{
    // TODO Function too long, split into subfunctions

    let node = nodestore.load(current_node_id).await;
    let node_info = match &node {
        Ok(Some(node)) => seen_node_info(node),
        Ok(None) => SeenNodeInfo::Missing,
        Err(err) => SeenNodeInfo::Unreadable,
    };

    // TODO Check that current_expected_node_info is correct and otherwise checks.add_error(CorruptedError::Assert) for things that are wrong

    match already_processed_nodes.add(current_node_id, node_info) {
        AlreadySeen::AlreadySeen {
            prev_seen,
            now_seen,
        } => {
            // Let's make sure it still looks the same (e.g. still has the same children) and then we can skip processing it.
            if prev_seen != now_seen {
                return Err(CheckError::FilesystemModified {
                    msg: format!(
                        "Node {current_node_id:?} was previously seen as {prev_seen:?} but now as {now_seen:?}",
                    ),
                });
            }

            // The node was already seen before. This can only happen if the node is referenced multiple times.
            // TODO
            // checks.add_error(CorruptedError::Assert(Box::new(
            //     CorruptedError::NodeReferencedMultipleTimes {
            //         node_id: current_node_id,
            //         node_info: now_seen.to_node_info_as_seen_by_looking_at_node(),
            //         referenced_as: todo!(),
            //     },
            // )));
        }
        AlreadySeen::NotSeenYet => {
            let node_to_process = match node {
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
                        current_expected_node_info.blob_info.blob_id,
                        current_expected_node_info.blob_info.blob_info.clone(),
                        &node,
                        expected_nodes,
                        already_processed_nodes,
                        checks,
                        task_spawner,
                        pb.clone(),
                    );
                    Some(NodeToProcess::Readable(node))
                }
                Ok(None) => {
                    if expected_nodes.contains(&current_node_id) {
                        return Err(CheckError::FilesystemModified {
                            msg: format!(
                                "Node {current_node_id:?} was present before but is now missing.",
                            ),
                        });
                    }
                    if current_node_id
                        == *current_expected_node_info
                            .blob_info
                            .blob_id
                            .to_root_block_id()
                    {
                        checks.add_error(CorruptedError::Assert(Box::new(
                            CorruptedError::BlobMissing {
                                blob_id: current_expected_node_info.blob_info.blob_id,
                                expected_blob_info: current_expected_node_info
                                    .blob_info
                                    .blob_info
                                    .clone(),
                            },
                        )));
                    } else {
                        let belongs_to_blob = current_expected_node_info.blob_info.clone();
                        checks.add_error(CorruptedError::Assert(Box::new(
                            CorruptedError::NodeMissing {
                                node_id: current_node_id,
                                referenced_as: match current_expected_node_info.node_info {
                                    NodeInfoAsExpectedByEntryInParent::RootNode => {
                                        NodeReference::RootNode { belongs_to_blob }
                                    }
                                    NodeInfoAsExpectedByEntryInParent::NonRootInnerNode {
                                        depth,
                                        parent_id,
                                    } => NodeReference::NonRootInnerNode {
                                        depth,
                                        parent_id,
                                        belongs_to_blob: Some(belongs_to_blob),
                                    },
                                    NodeInfoAsExpectedByEntryInParent::NonRootLeafNode {
                                        parent_id,
                                    } => NodeReference::NonRootLeafNode {
                                        parent_id,
                                        belongs_to_blob: Some(belongs_to_blob),
                                    },
                                },
                            },
                        )));
                    }
                    None
                }
                Err(error) => {
                    if !expected_nodes.contains(&current_node_id) {
                        return Err(CheckError::FilesystemModified {
                            msg: format!(
                                "Node {current_node_id:?} wasn't present before but is now.",
                            ),
                        });
                    }
                    checks.add_error(CorruptedError::Assert(Box::new(
                        CorruptedError::NodeUnreadable {
                            node_id: current_node_id,
                            expected_node_info: Some(current_expected_node_info.clone()),
                        },
                    )));
                    Some(NodeToProcess::Unreadable(current_node_id))
                }
            };
            if let Some(node_to_process) = node_to_process {
                checks.process_reachable_node(&node_to_process, &current_expected_node_info)?;
            }
            pb.inc(1);
        }
    }

    Ok(())
}

fn check_all_children_of_reachable_blob_node<'a, 'b, 'c, 'd, 'f, B>(
    nodestore: &'a DataNodeStore<B>,
    blob_id: BlobId,
    blob_info: BlobInfoAsExpectedByEntryInParent,
    current_node: &DataNode<B>,
    expected_nodes: &'b HashSet<BlockId>,
    already_processed_nodes: Arc<ProcessedItems<BlockId, SeenNodeInfo>>,
    checks: &'c AllChecks,
    task_spawner: TaskSpawner<'f, CheckError>,
    pb: impl Progress + 'd,
) where
    B: BlockStore + Send + Sync + 'static,
    'a: 'f,
    'b: 'f,
    'c: 'f,
    'd: 'f,
{
    match current_node {
        DataNode::Leaf(_) => {
            // Leaf nodes don't have children. Nothing to do.
        }
        DataNode::Inner(node) => {
            // Get all children and recurse into their nodes, concurrently.
            for child_id in node.children() {
                task_spawner.spawn(|task_spawner| {
                    let child_expected_node_info =
                        if let Some(child_depth) = NonZeroU8::new(node.depth().get() - 1) {
                            NodeInfoAsExpectedByEntryInParent::NonRootInnerNode {
                                depth: child_depth,
                                parent_id: *node.block_id(),
                            }
                        } else {
                            NodeInfoAsExpectedByEntryInParent::NonRootLeafNode {
                                parent_id: *node.block_id(),
                            }
                        };
                    let child_expected_node_info = NodeReferenceFromReachableBlob {
                        node_info: child_expected_node_info,
                        blob_info: ReferencingBlobInfo {
                            blob_id,
                            blob_info: blob_info.clone(),
                        },
                    };
                    check_all_nodes_of_reachable_blob(
                        nodestore,
                        child_id,
                        child_expected_node_info,
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
enum SeenNodeInfo {
    Unreadable,
    Missing,
    Leaf,
    Inner {
        depth: NonZeroU8,
        // We're storing children into the [SeenNodeInfo] so that if the node comes up again,
        // we can check that it still has the same children. This allows us to know that
        // we already processed those children when we saw the blob for the first time.
        // TODO Vec instead of HashSet should be enough
        children: HashSet<BlockId>,
    },
}

impl SeenNodeInfo {
    pub fn to_node_info_as_seen_by_looking_at_node(&self) -> Option<NodeInfoAsSeenByLookingAtNode> {
        match self {
            SeenNodeInfo::Unreadable | SeenNodeInfo::Missing => None,
            SeenNodeInfo::Leaf => Some(NodeInfoAsSeenByLookingAtNode::LeafNode),
            SeenNodeInfo::Inner { depth, .. } => {
                Some(NodeInfoAsSeenByLookingAtNode::InnerNode { depth: *depth })
            }
        }
    }
}

fn seen_node_info<B>(node: &DataNode<B>) -> SeenNodeInfo
where
    B: BlockStore + Send + Sync + 'static,
{
    match node {
        DataNode::Leaf(_) => SeenNodeInfo::Leaf,
        DataNode::Inner(node) => SeenNodeInfo::Inner {
            depth: node.depth(),
            children: node.children().collect(),
        },
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum SeenBlobInfo {
    Unreadable,
    Missing,
    File {
        parent_pointer: BlobId,
    },
    Symlink {
        parent_pointer: BlobId,
    },
    Dir {
        parent_pointer: BlobId,
        // We're storing children into the [SeenBlobInfo] so that if the node comes up again,
        // we can check that it still has the same children. This allows us to know that
        // we already processed those children when we saw the blob for the first time.
        children: Vec<BlobId>,
    },
}

impl SeenBlobInfo {
    pub fn to_blob_info_as_seen_by_looking_at_blob(&self) -> Option<BlobInfoAsSeenByLookingAtBlob> {
        match self {
            SeenBlobInfo::Missing => None,
            SeenBlobInfo::Unreadable => Some(BlobInfoAsSeenByLookingAtBlob::Unreadable),
            SeenBlobInfo::File { parent_pointer } => {
                Some(BlobInfoAsSeenByLookingAtBlob::Readable {
                    blob_type: BlobType::File,
                    parent_pointer: *parent_pointer,
                })
            }
            SeenBlobInfo::Symlink { parent_pointer } => {
                Some(BlobInfoAsSeenByLookingAtBlob::Readable {
                    blob_type: BlobType::Symlink,
                    parent_pointer: *parent_pointer,
                })
            }
            SeenBlobInfo::Dir {
                parent_pointer,
                children: _children,
            } => Some(BlobInfoAsSeenByLookingAtBlob::Readable {
                blob_type: BlobType::Dir,
                parent_pointer: *parent_pointer,
            }),
        }
    }
}

fn blob_content_summary<'a, B>(blob: &FsBlob<'a, B>) -> SeenBlobInfo
where
    // TODO Do we really need B: 'static ?
    B: BlobStore + Debug + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send,
{
    match blob {
        FsBlob::File(_) => SeenBlobInfo::File {
            parent_pointer: blob.parent(),
        },
        FsBlob::Symlink(_) => SeenBlobInfo::Symlink {
            parent_pointer: blob.parent(),
        },
        FsBlob::Directory(blob) => SeenBlobInfo::Dir {
            parent_pointer: blob.parent(),
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

    pub fn into_iter(self) -> impl Iterator<Item = (ItemId, ItemInfo)> {
        self.nodes.into_inner().unwrap().into_iter()
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
