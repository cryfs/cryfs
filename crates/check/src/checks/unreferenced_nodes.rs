use std::collections::HashSet;
use std::fmt::Debug;

use super::utils::reference_checker::{MarkAsSeenResult, ReferenceChecker};
use super::FilesystemCheck;
use crate::error::{CheckError, CorruptedError};
use cryfs_blobstore::{BlobId, BlobStoreOnBlocks, DataNode};
use cryfs_blockstore::{BlockId, BlockStore};
use cryfs_cryfs::filesystem::fsblobstore::FsBlob;

enum NodeType {
    RootNode,
    NonrootNode,
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum ChildrenOfNode {
    Unreadable,
    Leaf,
    Inner(HashSet<BlockId>),
}

impl ChildrenOfNode {
    pub fn into_iter(self) -> impl Iterator<Item = BlockId> {
        match self {
            Self::Unreadable | Self::Leaf => HashSet::new().into_iter(),
            Self::Inner(children) => children.into_iter(),
        }
    }
}

/// Check that
/// - each existing node is referenced
/// - each referenced node exists (i.e. no dangling node exists and no node is missing)
/// - no node is referenced multiple times
///
/// Algorithm: While passing through each node, we mark the current node as **seen** and all referenced nodes as **referenced**.
/// We make sure that each node id is both **seen** and **referenced** and that there are no nodes that are only one of the two.
struct UnreferencedNodesReferenceChecker {
    reference_checker: ReferenceChecker<BlockId, ChildrenOfNode, NodeType>,
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
    ) -> Result<(), CheckError> {
        let children = match node {
            DataNode::Inner(node) => ChildrenOfNode::Inner(node.children().collect()),
            DataNode::Leaf(_) => ChildrenOfNode::Leaf,
        };
        self._mark_as_seen_and_children_as_referenced(*node.block_id(), children)
    }

    pub fn process_unreadable_node(&mut self, node_id: BlockId) -> Result<(), CheckError> {
        self._mark_as_seen_and_children_as_referenced(node_id, ChildrenOfNode::Unreadable)?;

        // Also make sure that the unreadable node was reported by a different check
        self.errors.push(CorruptedError::Assert(Box::new(
            CorruptedError::NodeUnreadable { node_id },
        )));

        Ok(())
    }

    fn _mark_as_seen_and_children_as_referenced(
        &mut self,
        node_id: BlockId,
        children: ChildrenOfNode,
    ) -> Result<(), CheckError> {
        let mark_as_seen_result = self
            .reference_checker
            .mark_as_seen(node_id, children.clone());
        match mark_as_seen_result {
            MarkAsSeenResult::AlreadySeenBefore {
                prev_seen_info: prev_seen_children,
            } => {
                // This should only happen if a node is referenced multiple times.
                // So let's make sure that error was caught and reported.
                self.errors.push(CorruptedError::Assert(Box::new(
                    CorruptedError::NodeReferencedMultipleTimes { node_id },
                )));

                if children != prev_seen_children {
                    // If the children changed, then the file system must have changed during the analysis. This isn't supported.
                    return Err(CheckError::FilesystemModified{msg: format!("Node {node_id:?} was seen multiple times and with a different list of children ({prev_seen_children:?} vs {children:?}).")});
                }
            }
            MarkAsSeenResult::NotSeenBeforeYet => {
                // Mark all referenced nodes within the same blob as referenced
                for child in children.into_iter() {
                    self.reference_checker
                        .mark_as_referenced(child, NodeType::NonrootNode);
                }
            }
        }
        Ok(())
    }

    // TODO `process_blob` is only called correctly when the blob is reachable. For unreachable blob, it isn't.
    //      This means we do report each unreachable blob in the tree as an error, not just the root.
    pub fn process_blob(
        &mut self,
        blob: &FsBlob<BlobStoreOnBlocks<impl BlockStore + Send + Sync + Debug + 'static>>,
    ) -> Result<(), CheckError> {
        match blob {
            FsBlob::File(_) | FsBlob::Symlink(_) => {
                // Files and symlinks don't reference other blobs
            }
            FsBlob::Directory(blob) => {
                for child in blob.entries() {
                    self.reference_checker.mark_as_referenced(
                        *child.blob_id().to_root_block_id(),
                        NodeType::RootNode,
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
                    Some(node_type) => {
                        if seen.is_none() {
                            match node_type {
                                NodeType::RootNode => {
                                    errors.push(CorruptedError::BlobMissing {
                                        blob_id: BlobId::from_root_block_id(node_id.into()),
                                    });
                                }
                                NodeType::NonrootNode => {
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
            .mark_as_referenced(*root_blob_id.to_root_block_id(), NodeType::RootNode);
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
    ) -> Result<(), CheckError> {
        self.reachable_nodes_checker.process_node(node)
    }

    fn process_reachable_unreadable_node(&mut self, node_id: BlockId) -> Result<(), CheckError> {
        self.reachable_nodes_checker
            .process_unreadable_node(node_id)
    }

    fn process_unreachable_node(
        &mut self,
        node: &DataNode<impl BlockStore + Send + Sync + Debug + 'static>,
    ) -> Result<(), CheckError> {
        self.unreachable_nodes_checker.process_node(node)
    }

    fn process_unreachable_unreadable_node(&mut self, node_id: BlockId) -> Result<(), CheckError> {
        self.unreachable_nodes_checker
            .process_unreadable_node(node_id)
    }

    fn process_reachable_blob(
        &mut self,
        blob: &FsBlob<BlobStoreOnBlocks<impl BlockStore + Send + Sync + Debug + 'static>>,
    ) -> Result<(), CheckError> {
        self.reachable_nodes_checker.process_blob(blob)
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
