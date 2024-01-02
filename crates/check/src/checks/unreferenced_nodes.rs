use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

use super::FilesystemCheck;
use crate::error::CorruptedError;
use cryfs_blobstore::{BlobId, BlobStoreOnBlocks, DataNode};
use cryfs_blockstore::{BlockId, BlockStore};
use cryfs_cryfs::filesystem::fsblobstore::FsBlob;

enum NodeType {
    RootNode,
    NonrootNode,
}

/// Check that
/// - each existing node is referenced
/// - each referenced node exists (i.e. no dangling node exists and no node is missing)
///
/// Algorithm: While passing through each node, we mark the current node as **seen** and all referenced nodes as **referenced**.
/// We make sure that each node id is both **seen** and **referenced** and that there are no nodes that are only one of the two.
struct ReferenceChecker {
    // Remember all nodes we've seen but we haven't seen a reference to yet
    seen_and_unreferenced: HashSet<BlockId>,

    // Remember all nodes we've seen a reference to but we haven't seen the node itself yet
    unseen_and_referenced: HashMap<BlockId, NodeType>,

    // Remember all nodes we've seen and have seen the reference to.
    // Invariant: Nodes in `seen_and_referenced` don't get added to `seen_and_unreferenced` or `unseen_and_referenced` anymore.
    seen_and_referenced: HashSet<BlockId>,

    errors: Vec<CorruptedError>,
}

impl ReferenceChecker {
    pub fn new() -> Self {
        Self {
            seen_and_unreferenced: HashSet::new(),
            unseen_and_referenced: HashMap::new(),
            seen_and_referenced: HashSet::new(),
            errors: Vec::new(),
        }
    }

    pub fn process_node(
        &mut self,
        node: &DataNode<impl BlockStore + Send + Sync + Debug + 'static>,
    ) {
        self.mark_as_seen(*node.block_id());

        // Mark all referenced nodes within the same blob as referenced
        match node {
            DataNode::Inner(node) => {
                for child in node.children() {
                    self.mark_as_referenced(child, NodeType::NonrootNode);
                }
            }
            DataNode::Leaf(_) => {
                // A leaf node doesn't reference other nodes
            }
        }
    }

    pub fn process_unreadable_node(&mut self, node_id: BlockId) {
        self.mark_as_seen(node_id);
    }

    // TODO `process_blob` is only called correctly when the blob is reachable. For unreachable blob, it isn't.
    //      This means we do report each unreachable blob in the tree as an error, not just the root.
    pub fn process_blob(
        &mut self,
        blob: &FsBlob<BlobStoreOnBlocks<impl BlockStore + Send + Sync + Debug + 'static>>,
    ) {
        match blob {
            FsBlob::File(_) | FsBlob::Symlink(_) => {
                // Files and symlinks don't reference other blobs
            }
            FsBlob::Directory(blob) => {
                for child in blob.entries() {
                    self.mark_as_referenced(
                        *child.blob_id().to_root_block_id(),
                        NodeType::RootNode,
                    );
                }
            }
        }
    }

    fn mark_as_seen(&mut self, node_id: BlockId) {
        if self.unseen_and_referenced.remove(&node_id).is_some() {
            // We already saw a reference to this node previously and now we saw the node itself. Everything is fine.
            assert!(!self.seen_and_unreferenced.contains(&node_id), "Algorithm invariant violated: A node was in both `seen_and_unreferenced` and in `unseen_and_referenced`.");
            if !self.seen_and_referenced.insert(node_id) {
                panic!("Algorithm invariant violated: A node was in both `unseen_and_referenced` and in `seen_and_referenced`.");
            }
        } else if self.seen_and_referenced.contains(&node_id) {
            // We've already seen the node before. We shouldn't see it again. This is a bug in the check tool.
            panic!("Algorithm invariant violated: Node {node_id:?} was seen twice",);
        } else {
            // We haven't seen a reference to this node yet. Remember it.
            if !self.seen_and_unreferenced.insert(node_id) {
                panic!("Algorithm invariant violated: node {node_id:?} was seen twice");
            }
        }
    }

    fn mark_as_referenced(&mut self, node_id: BlockId, node_type: NodeType) {
        if self.seen_and_unreferenced.remove(&node_id) {
            // We already saw this node previously and now we saw the reference to it. Everything is fine.
            assert!(!self.unseen_and_referenced.contains_key(&node_id), "Algorithm invariant violated: A node was in both `unseen_and_referenced` and in `seen_and_unreferenced`.");
            if !self.seen_and_referenced.insert(node_id) {
                panic!("Algorithm invariant violated: A node was in both `seen_and_unreferenced` and in `seen_and_referenced`.");
            }
        } else if self.seen_and_referenced.contains(&node_id) {
            // We've already seen this node and the reference to it. This is now the second reference to it.
            self.errors
                .push(CorruptedError::NodeReferencedMultipleTimes { node_id });
        } else {
            // We haven't seen this node yet. Remember it.
            if self
                .unseen_and_referenced
                .insert(node_id, node_type)
                .is_some()
            {
                // TODO Specifically test scenarios for NodeReferencedMultipleTimes, both this and the case above
                self.errors
                    .push(CorruptedError::NodeReferencedMultipleTimes { node_id });
            }
        }
    }

    // Returns a list of errors and a list of nodes that were processed without errors
    pub fn finalize(self) -> Vec<CorruptedError> {
        let mut errors = self.errors;
        errors.extend(
            self.seen_and_unreferenced
                .into_iter()
                .map(|node_id| CorruptedError::NodeUnreferenced { node_id }),
        );
        errors.extend(
            self.unseen_and_referenced
                .into_iter()
                .map(|(node_id, node_type)| match node_type {
                    NodeType::RootNode => CorruptedError::BlobMissing {
                        blob_id: BlobId::from_root_block_id(node_id.into()),
                    },
                    NodeType::NonrootNode => CorruptedError::NodeMissing { node_id },
                }),
        );
        errors
    }
}

/// Check that
/// - each existing node is referenced
/// - each referenced node exists (i.e. no dangling node exists and no node is missing)
///
/// For unreachable nodes, this can find filesystem errors. We run [ReferenceChecker] on these nodes so that only the root of any
/// dangling blob is reported and not all the nodes below it.
///
/// For reachable nodes, this is used to assert that cryfs-check works correctly and doesn't miss any nodes.
pub struct CheckUnreferencedNodes {
    reachable_nodes_checker: ReferenceChecker,
    unreachable_nodes_checker: ReferenceChecker,
}

impl CheckUnreferencedNodes {
    pub fn new(root_blob_id: BlobId) -> Self {
        let mut reachable_nodes_checker = ReferenceChecker::new();
        reachable_nodes_checker
            .mark_as_referenced(*root_blob_id.to_root_block_id(), NodeType::RootNode);
        Self {
            reachable_nodes_checker,
            unreachable_nodes_checker: ReferenceChecker::new(),
        }
    }
}

impl FilesystemCheck for CheckUnreferencedNodes {
    fn process_reachable_node(
        &mut self,
        node: &DataNode<impl BlockStore + Send + Sync + Debug + 'static>,
    ) {
        self.reachable_nodes_checker.process_node(node);
    }

    fn process_reachable_unreadable_node(&mut self, node_id: BlockId) {
        self.reachable_nodes_checker
            .process_unreadable_node(node_id);
    }

    fn process_unreachable_node(
        &mut self,
        node: &DataNode<impl BlockStore + Send + Sync + Debug + 'static>,
    ) {
        self.unreachable_nodes_checker.process_node(node);
    }

    fn process_unreachable_unreadable_node(&mut self, node_id: BlockId) {
        self.unreachable_nodes_checker
            .process_unreadable_node(node_id);
    }

    fn process_reachable_blob(
        &mut self,
        blob: &FsBlob<BlobStoreOnBlocks<impl BlockStore + Send + Sync + Debug + 'static>>,
    ) {
        self.reachable_nodes_checker.process_blob(blob);
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
                | CorruptedError::BlobMissing { .. } => {
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
