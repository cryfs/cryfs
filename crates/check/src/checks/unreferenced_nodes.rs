use std::fmt::Debug;
use std::num::NonZeroU8;

use super::utils::reference_checker::ReferenceChecker;
use super::{BlobToProcess, FilesystemCheck, NodeToProcess};
use crate::error::{
    BlobInfoAsExpectedByEntryInParent, CheckError, CorruptedError, NodeInfoAsSeenByLookingAtNode,
    NodeReference, NodeReferenceFromReachableBlob, ReferencingBlobInfo,
};
use cryfs_blobstore::{BlobId, BlobStoreOnBlocks, DataNode};
use cryfs_blockstore::{BlockId, BlockStore};
use cryfs_cryfs::filesystem::fsblobstore::{BlobType, EntryType, FsBlob};
use cryfs_rustfs::AbsolutePath;

struct ReferencedAs {
    referenced_as: NodeReference,
}

#[derive(Debug, Clone)]
struct SeenInfo {
    node_info: NodeInfoAsSeenByLookingAtNode,
}

/// Check that
/// - each existing node is referenced
/// - each referenced node exists (i.e. no dangling node exists and no node is missing)
/// - no node is referenced multiple times
///
/// Algorithm: While passing through each node, we mark the current node as **seen** and all referenced nodes as **referenced**.
/// We make sure that each node id is both **seen** and **referenced** and that there are no nodes that are only one of the two.
struct UnreferencedNodesReferenceChecker {
    reference_checker: ReferenceChecker<BlockId, SeenInfo, ReferencedAs>,
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
        node: &NodeToProcess<impl BlockStore + Send + Sync + Debug + 'static>,
        expected_node_info: Option<NodeReferenceFromReachableBlob>,
    ) -> Result<(), CheckError> {
        match node {
            NodeToProcess::Readable(node) => self.process_readable_node(node, expected_node_info),
            NodeToProcess::Unreadable(node_id) => {
                self.process_unreadable_node(*node_id, expected_node_info)
            }
        }
    }

    fn process_readable_node(
        &mut self,
        node: &DataNode<impl BlockStore + Send + Sync + Debug + 'static>,
        expected_node_info: Option<NodeReferenceFromReachableBlob>,
    ) -> Result<(), CheckError> {
        let depth = NonZeroU8::new(node.depth());
        let node_info = if let Some(depth) = depth {
            NodeInfoAsSeenByLookingAtNode::InnerNode { depth }
        } else {
            NodeInfoAsSeenByLookingAtNode::LeafNode
        };
        self.reference_checker
            .mark_as_seen(*node.block_id(), SeenInfo { node_info });

        // Mark all referenced nodes within the same blob as referenced
        match node {
            DataNode::Leaf(_) => {
                // Leaf nodes don't have children
            }
            DataNode::Inner(node) => {
                for child in node.children() {
                    let parent_id = *node.block_id();
                    let depth = node.depth().get() - 1;
                    let referenced_as = if depth == 0 {
                        NodeReference::NonRootLeafNode {
                            belongs_to_blob: expected_node_info
                                .as_ref()
                                .map(|a| a.blob_info.clone()),
                            parent_id,
                        }
                    } else {
                        NodeReference::NonRootInnerNode {
                            belongs_to_blob: expected_node_info
                                .as_ref()
                                .map(|a| a.blob_info.clone()),
                            depth: NonZeroU8::new(depth).unwrap(),
                            parent_id,
                        }
                    };
                    self.reference_checker
                        .mark_as_referenced(child, ReferencedAs { referenced_as });
                }
            }
        }

        Ok(())
    }

    fn process_unreadable_node(
        &mut self,
        node_id: BlockId,
        expected_node_info: Option<NodeReferenceFromReachableBlob>,
    ) -> Result<(), CheckError> {
        self.reference_checker.mark_as_seen(
            node_id,
            SeenInfo {
                node_info: NodeInfoAsSeenByLookingAtNode::Unreadable,
            },
        );

        // Also make sure that the unreadable node was reported by a different check
        self.errors.push(CorruptedError::Assert(Box::new(
            CorruptedError::NodeUnreadable {
                node_id,
                expected_node_info,
            },
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
                    let child_blob_info = ReferencingBlobInfo {
                        blob_id: *child.blob_id(),
                        blob_info: BlobInfoAsExpectedByEntryInParent {
                            blob_type: entry_type_to_blob_type(child.entry_type()),
                            parent_id: blob.blob_id(),
                            path: path.join(child.name()),
                        },
                    };
                    self.reference_checker.mark_as_referenced(
                        *child.blob_id().to_root_block_id(),
                        ReferencedAs {
                            referenced_as: NodeReference::RootNode {
                                belongs_to_blob: child_blob_info,
                            },
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
                if seen.is_none() && !referenced_as.is_empty() {
                    errors.push(CorruptedError::NodeMissing {
                        node_id,
                        referenced_as: referenced_as
                            .iter()
                            .map(|referenced_as| referenced_as.referenced_as.clone())
                            .collect(),
                    })
                }
                if referenced_as.len() > 1 {
                    errors.push(CorruptedError::NodeReferencedMultipleTimes {
                        node_id,
                        node_info: seen.as_ref().map(|seen| seen.node_info.clone()),
                        // TODO How to handle the case where referenced_as Vec has duplicate entries?
                        referenced_as: referenced_as
                            .iter()
                            .map(|referenced_as| referenced_as.referenced_as.clone())
                            .collect(),
                    });
                }
                if referenced_as.is_empty() {
                    // This node is not referenced by any other node. This is an error.
                    let seen = seen.expect(
                        "Algorithm invariant violated: Node was neither seen nor referenced.",
                    );
                    errors.push(CorruptedError::NodeUnreferenced {
                        node_id,
                        node_info: seen.node_info,
                    });
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
                ReferencedAs {
                    referenced_as: NodeReference::RootNode {
                        belongs_to_blob: ReferencingBlobInfo {
                            blob_id: root_blob_id,
                            blob_info: BlobInfoAsExpectedByEntryInParent::root_dir(),
                        },
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
    fn process_reachable_node<'a>(
        &mut self,
        node: &NodeToProcess<impl BlockStore + Send + Sync + Debug + 'static>,
        expected_node_info: &NodeReferenceFromReachableBlob,
    ) -> Result<(), CheckError> {
        self.reachable_nodes_checker
            .process_node(node, Some(expected_node_info.clone()))?;
        Ok(())
    }

    fn process_unreachable_node<'a>(
        &mut self,
        node: &NodeToProcess<impl BlockStore + Send + Sync + Debug + 'static>,
    ) -> Result<(), CheckError> {
        self.unreachable_nodes_checker.process_node(node, None)?;
        Ok(())
    }

    fn process_reachable_blob<'a, 'b>(
        &mut self,
        blob: BlobToProcess<'a, 'b, impl BlockStore + Send + Sync + Debug + 'static>,
        expected_blob_info: &BlobInfoAsExpectedByEntryInParent,
    ) -> Result<(), CheckError> {
        match blob {
            BlobToProcess::Readable(blob) => {
                self.reachable_nodes_checker
                    .process_blob(blob, &expected_blob_info.path)?;
            }
            BlobToProcess::Unreadable(_blob_id) => {
                // do nothing
            }
        }
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
