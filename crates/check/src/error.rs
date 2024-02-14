use std::collections::BTreeSet;
use std::fmt::{self, Debug, Display};
use thiserror::Error;

use cryfs_blobstore::BlobId;
use cryfs_blockstore::BlockId;
use cryfs_cryfs::filesystem::fsblobstore::BlobType;
use cryfs_rustfs::AbsolutePathBuf;

// TOOD Add more info to each error, e.g. parent pointers, blob a node belongs to, path in filesystem, ...

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct BlobInfoAsSeenByLookingAtBlob {
    pub blob_type: BlobType,
    pub parent_pointer: BlobId,
}

impl Display for BlobInfoAsSeenByLookingAtBlob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let blob_type = match self.blob_type {
            BlobType::File => "File",
            BlobType::Dir => "Dir",
            BlobType::Symlink => "Symlink",
        };
        write!(
            f,
            "{blob_type}[parent_pointer={parent_pointer:?}]",
            parent_pointer = self.parent_pointer,
        )
    }
}

impl Debug for BlobInfoAsSeenByLookingAtBlob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlobInfoAsSeenByLookingAtBlob({self})")
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct BlobInfoAsExpectedByEntryInParent {
    pub blob_type: BlobType,
    pub parent_id: BlobId,
    pub path: AbsolutePathBuf,
}

impl BlobInfoAsExpectedByEntryInParent {
    pub fn root_dir() -> Self {
        Self {
            blob_type: BlobType::Dir,
            parent_id: BlobId::zero(),
            path: AbsolutePathBuf::root(),
        }
    }
}

impl Display for BlobInfoAsExpectedByEntryInParent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let blob_type = match self.blob_type {
            BlobType::File => "File",
            BlobType::Dir => "Dir",
            BlobType::Symlink => "Symlink",
        };
        write!(
            f,
            "{blob_type}[parent={parent_id:?}] @ {path}",
            parent_id = self.parent_id,
            path = self.path,
        )
    }
}

impl Debug for BlobInfoAsExpectedByEntryInParent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlobInfoAsExpectedByEntryInParent({self})")
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct NodeInfoAsSeenByLookingAtNode {
    pub depth: u8,
}

impl Display for NodeInfoAsSeenByLookingAtNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.depth == 0 {
            write!(f, "LeafNode")
        } else {
            write!(f, "InnerNode[depth={depth}]", depth = self.depth)
        }
    }
}

impl Debug for NodeInfoAsSeenByLookingAtNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeInfoAsSeenByLookingAtNode({self})")
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct NodeInfoAsExpectedByEntryInParent {
    pub depth: u8,
    pub parent_id: BlockId,
}

impl Display for NodeInfoAsExpectedByEntryInParent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.depth == 0 {
            write!(f, "LeafNode[parent={parent:?}]", parent = self.parent_id)
        } else {
            write!(
                f,
                "InnerNode[depth={depth}, parent={parent:?}]",
                depth = self.depth,
                parent = self.parent_id,
            )
        }
    }
}

impl Debug for NodeInfoAsExpectedByEntryInParent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeInfoAsExpectedByEntryInParent({self})")
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct BlobReference {
    pub expected_child_info: BlobInfoAsExpectedByEntryInParent,
}

/// A [CorruptedError] is an error we found in the file system when analyzing it
#[derive(Debug, Error, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum CorruptedError {
    #[error("Node {node_id:?} is unreadable and likely corrupted")]
    NodeUnreadable {
        node_id: BlockId,
        // TODO blob_info: BlobInfo,
        // TODO referenced_by: BlockId,
        // TODO error: anyhow::Error,
        // TODO expected_depth: u8,
        // TODO Re-think fields
    },

    #[error("Node {node_id:?} is referenced but does not exist")]
    NodeMissing {
        node_id: BlockId,
        // TODO blob_info: BlobInfo,
        // TODO referenced_by: BlockId,
        // TODO expected_depth: u8,
        // TODO Re-think fields
    },

    #[error("Node {node_id:?} is not referenced but exists")]
    NodeUnreferenced {
        node_id: BlockId,
        // TODO node_info: NodeInfoAsSeenByLookingAtNode
    },

    // #[error("Node {node_id:?} is referenced but is not reachable. Possibly there is a cycle in a unconnected subtree")]
    // UnreachableSubtreeWithCycle { node_id: BlockId },
    #[error("Node {node_id:?} is referenced multiple times")]
    NodeReferencedMultipleTimes {
        node_id: BlockId,
        /// `node_info` can be `None` if the node itself is missing or unreadable
        node_info: Option<NodeInfoAsSeenByLookingAtNode>,
        // TODO referenced_as: BTreeSet<(BlobInfo, NodeInfoAsExpectedByEntryInParent)>, probably should move this tuple into a NodeReference class
        // TODO Should BlobInfo become part of NodeInfoAsExpectedByEntryInParent
    },

    #[error("{blob_id:?} ({blob_info:?}) is referenced multiple times, by {referenced_as:?}")]
    BlobReferencedMultipleTimes {
        blob_id: BlobId,
        /// `blob_info` can be `None` if the blob itself is missing or unreadable
        blob_info: Option<BlobInfoAsSeenByLookingAtBlob>,
        referenced_as: BTreeSet<BlobReference>,
    },

    #[error("{blob_id:?}:{expected_blob_info} is unreadable and likely corrupted")]
    BlobUnreadable {
        blob_id: BlobId,
        expected_blob_info: BlobInfoAsExpectedByEntryInParent,
        // TODO error:  anyhow::Error,
    },

    #[error("{blob_id:?}:{expected_blob_info} is referenced but does not exist")]
    BlobMissing {
        blob_id: BlobId,
        expected_blob_info: BlobInfoAsExpectedByEntryInParent,
    },

    #[error("{blob_id:?}:{blob_info} is referenced by {referenced_as:?}, but the parent pointer doesn't match any of the references")]
    WrongParentPointer {
        blob_id: BlobId,
        blob_info: BlobInfoAsSeenByLookingAtBlob,
        referenced_as: BTreeSet<BlobReference>,
    },

    /// Not an actual error but reported by a check to indicate that we need to assert that another check reported this error.
    /// This is reported by checks who aren't the main responsible check for a condition but discovered something on the side
    /// that another check should have reported.
    #[error("We need to assert that {0} was reported")]
    Assert(Box<CorruptedError>),
}

/// A CheckError is an error found in the analysis itself. This doesn't necessarily mean that the file system is corrupted
#[derive(Error, Debug)]
pub enum CheckError {
    #[error("The filesystem was modified while the check was running. Please make sure the file system is not mounted or modified for the duration of the check.\n Details: {msg}")]
    FilesystemModified { msg: String },
}
