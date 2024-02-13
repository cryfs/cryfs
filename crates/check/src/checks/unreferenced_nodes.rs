use std::fmt::Debug;

use super::utils::reference_checker::ReferenceChecker;
use super::FilesystemCheck;
use crate::error::{BlobInfoAsExpectedByEntryInParent, CheckError, CorruptedError};
use cryfs_blobstore::{BlobId, BlobStoreOnBlocks, DataNode};
use cryfs_blockstore::{BlockId, BlockStore};
use cryfs_cryfs::filesystem::fsblobstore::{BlobType, EntryType, FsBlob};
use cryfs_rustfs::AbsolutePath;

#[derive(Debug, Clone)]
struct ReferencedBlobInfo {
    blob_id: BlobId,
    blob_info: BlobInfoAsExpectedByEntryInParent,
}

enum ReferencedAs {
    RootNode {
        belongs_to_blob: ReferencedBlobInfo,
    },
    NonrootNode {
        // Can be `None` if the node is unreferenced
        belongs_to_blob: Option<ReferencedBlobInfo>,
    },
}

/// Check that
/// - each existing node is referenced
/// - each referenced node exists (i.e. no dangling node exists and no node is missing)
/// - no node is referenced multiple times
///
/// Algorithm: While passing through each node, we mark the current node as **seen** and all referenced nodes as **referenced**.
/// We make sure that each node id is both **seen** and **referenced** and that there are no nodes that are only one of the two.
struct UnreferencedNodesReferenceChecker {
    reference_checker: ReferenceChecker<BlockId, (), ReferencedAs>,
    errors: Vec<CorruptedError>,
}

impl UnreferencedNodesReferenceChecker {
    pub fn new() -> Self {
        Self {
            reference_checker: ReferenceChecker::new(),
            errors: Vec::new(),
        }
    }

    pub fn process_node(
        &mut self,
        node: &DataNode<impl BlockStore + Send + Sync + Debug + 'static>,
        blob_info: Option<ReferencedBlobInfo>,
    ) -> Result<(), CheckError> {
        self.reference_checker.mark_as_seen(*node.block_id(), ());

        // Mark all referenced nodes within the same blob as referenced
        match node {
            DataNode::Leaf(_) => {
                // Leaf nodes don't have children
            }
            DataNode::Inner(node) => {
                for child in node.children() {
                    self.reference_checker.mark_as_referenced(
                        child,
                        ReferencedAs::NonrootNode {
                            belongs_to_blob: blob_info.clone(),
                        },
                    );
                }
            }
        }

        Ok(())
    }

    pub fn process_unreadable_node(&mut self, node_id: BlockId) -> Result<(), CheckError> {
        self.reference_checker.mark_as_seen(node_id, ());

        // Also make sure that the unreadable node was reported by a different check
        self.errors.push(CorruptedError::Assert(Box::new(
            CorruptedError::NodeUnreadable { node_id },
        )));

        Ok(())
    }

    // TODO `process_blob` is only called correctly when the blob is reachable. For unreachable blob, it isn't called.
    //      This means we do report each unreachable blob in the tree as an error, not just the root.
    pub fn process_blob(
        &mut self,
        blob: &FsBlob<BlobStoreOnBlocks<impl BlockStore + Send + Sync + Debug + 'static>>,
        path: &AbsolutePath,
    ) -> Result<(), CheckError> {
        match blob {
            FsBlob::File(_) | FsBlob::Symlink(_) => {
                // Files and symlinks don't reference other blobs
            }
            FsBlob::Directory(blob) => {
                for child in blob.entries() {
                    let child_blob_info = ReferencedBlobInfo {
                        blob_id: *child.blob_id(),
                        blob_info: BlobInfoAsExpectedByEntryInParent {
                            blob_type: entry_type_to_blob_type(child.entry_type()),
                            parent_id: blob.blob_id(),
                            path: path.join(child.name()),
                        },
                    };
                    self.reference_checker.mark_as_referenced(
                        *child.blob_id().to_root_block_id(),
                        ReferencedAs::RootNode {
                            belongs_to_blob: child_blob_info,
                        },
                    );
                }
            }
        }
        Ok(())
    }

    // Returns a list of errors and a list of nodes that were processed without errors
    pub fn finalize(self) -> Vec<CorruptedError> {
        let mut errors = self.errors;
        errors.extend(self.reference_checker.finalize().flat_map(
            |(node_id, seen, referenced_as)| {
                let mut errors = vec![];
                match referenced_as.first() {
                    Some(first_referenced_as) => {
                        if seen.is_none() {
                            match first_referenced_as {
                                ReferencedAs::RootNode { belongs_to_blob } => {
                                    errors.push(CorruptedError::BlobMissing {
                                        blob_id: belongs_to_blob.blob_id,
                                        expected_blob_info: belongs_to_blob.blob_info.clone(),
                                    });
                                }
                                ReferencedAs::NonrootNode { belongs_to_blob: _ } => {
                                    errors.push(CorruptedError::NodeMissing { node_id });
                                }
                            }
                        }
                        if referenced_as.len() > 1 {
                            errors.push(CorruptedError::NodeReferencedMultipleTimes { node_id });
                        }
                    }
                    None => {
                        // This node is not referenced by any other node. This is an error.
                        assert!(
                            seen.is_some(),
                            "Algorithm invariant violated: Node was neither seen nor referenced."
                        );
                        errors.push(CorruptedError::NodeUnreferenced { node_id });
                    }
                }
                errors.into_iter()
            },
        ));
        errors
    }
}

/// Check that
/// - each existing node is referenced
/// - each referenced node exists (i.e. no dangling node exists and no node is missing)
/// - no node is referenced multiple times
///
/// For unreachable nodes, this can find filesystem errors. We run [ReferenceChecker] on these nodes so that only the root of any
/// dangling blob is reported and not all the nodes below it.
///
/// For reachable nodes, this is used to assert that cryfs-check works correctly and doesn't miss any nodes.
pub struct CheckUnreferencedNodes {
    reachable_nodes_checker: UnreferencedNodesReferenceChecker,
    unreachable_nodes_checker: UnreferencedNodesReferenceChecker,
}

impl CheckUnreferencedNodes {
    pub fn new(root_blob_id: BlobId) -> Self {
        let mut reachable_nodes_checker = UnreferencedNodesReferenceChecker::new();
        reachable_nodes_checker
            .reference_checker
            .mark_as_referenced(
                *root_blob_id.to_root_block_id(),
                ReferencedAs::RootNode {
                    belongs_to_blob: ReferencedBlobInfo {
                        blob_id: root_blob_id,
                        blob_info: BlobInfoAsExpectedByEntryInParent::root_dir(),
                    },
                },
            );
        Self {
            reachable_nodes_checker,
            unreachable_nodes_checker: UnreferencedNodesReferenceChecker::new(),
        }
    }
}

impl FilesystemCheck for CheckUnreferencedNodes {
    fn process_reachable_node(
        &mut self,
        node: &DataNode<impl BlockStore + Send + Sync + Debug + 'static>,
        blob_id: BlobId,
        blob_info: &BlobInfoAsExpectedByEntryInParent,
    ) -> Result<(), CheckError> {
        self.reachable_nodes_checker.process_node(
            node,
            Some(ReferencedBlobInfo {
                blob_id,
                blob_info: blob_info.clone(),
            }),
        )
    }

    fn process_reachable_unreadable_node(
        &mut self,
        node_id: BlockId,
        _blob_id: BlobId,
        _blob_info: &BlobInfoAsExpectedByEntryInParent,
    ) -> Result<(), CheckError> {
        self.reachable_nodes_checker
            .process_unreadable_node(node_id)
    }

    fn process_unreachable_node(
        &mut self,
        node: &DataNode<impl BlockStore + Send + Sync + Debug + 'static>,
    ) -> Result<(), CheckError> {
        self.unreachable_nodes_checker.process_node(node, None)
    }

    fn process_unreachable_unreadable_node(&mut self, node_id: BlockId) -> Result<(), CheckError> {
        self.unreachable_nodes_checker
            .process_unreadable_node(node_id)
    }

    fn process_reachable_readable_blob(
        &mut self,
        blob: &FsBlob<BlobStoreOnBlocks<impl BlockStore + Send + Sync + Debug + 'static>>,
        blob_info: &BlobInfoAsExpectedByEntryInParent,
    ) -> Result<(), CheckError> {
        self.reachable_nodes_checker
            .process_blob(blob, &blob_info.path)
    }

    fn process_reachable_unreadable_blob(
        &mut self,
        _blob_id: BlobId,
        _expected_blob_info: &BlobInfoAsExpectedByEntryInParent,
    ) -> Result<(), CheckError> {
        // do nothing
        Ok(())
    }

    fn finalize(self) -> Vec<CorruptedError> {
        let mut errors = self.unreachable_nodes_checker.finalize();
        let reachable_nodes_errors = self.reachable_nodes_checker.finalize();
        for error in reachable_nodes_errors {
            match error {
                CorruptedError::NodeUnreferenced { .. } => {
                    // The check tool somehow sent us nodes further down the tree without sending us the parent node.
                    // TODO bail instead of panic
                    panic!("Algorithm invariant violated (NodeUnreferenced): {error:?}");
                }
                CorruptedError::NodeReferencedMultipleTimes { .. }
                | CorruptedError::NodeUnreadable { .. }
                | CorruptedError::NodeMissing { .. }
                | CorruptedError::BlobMissing { .. }
                | CorruptedError::Assert(_) => {
                    errors.push(error);
                }
                _ => {
                    // These errors are not expected for reachable nodes
                    // TODO bail instead of panic
                    panic!("Algorithm invariant violated (unexpected error): {error:?}");
                }
            }
        }

        errors
    }
}

// TODO This exists in multiple places. Deduplicate.
fn entry_type_to_blob_type(entry_type: EntryType) -> BlobType {
    match entry_type {
        EntryType::File => BlobType::File,
        EntryType::Dir => BlobType::Dir,
        EntryType::Symlink => BlobType::Symlink,
    }
}
