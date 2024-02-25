use std::collections::BTreeSet;
use std::fmt::{self, Debug, Display};
use std::num::NonZeroU8;
use thiserror::Error;

use cryfs_blobstore::BlobId;
use cryfs_blockstore::BlockId;
use cryfs_cryfs::filesystem::fsblobstore::BlobType;
use cryfs_rustfs::AbsolutePathBuf;

// TODO Add more info to each error, e.g. parent pointers, blob a node belongs to, path in filesystem, ...

// TODO Improve error messages

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum BlobInfoAsSeenByLookingAtBlob {
    Unreadable,
    Readable {
        blob_type: BlobType,
        parent_pointer: BlobId,
    },
}

impl Display for BlobInfoAsSeenByLookingAtBlob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable => write!(f, "UnreadableBlob"),
            Self::Readable {
                blob_type,
                parent_pointer,
            } => {
                let blob_type = match blob_type {
                    BlobType::File => "File",
                    BlobType::Dir => "Dir",
                    BlobType::Symlink => "Symlink",
                };
                write!(f, "{blob_type}[parent_pointer={parent_pointer:?}]",)
            }
        }
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
pub enum NodeInfoAsSeenByLookingAtNode {
    Unreadable,
    InnerNode { depth: NonZeroU8 },
    LeafNode,
}

impl Display for NodeInfoAsSeenByLookingAtNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable => write!(f, "UnreadableNode"),
            Self::LeafNode => write!(f, "LeafNode"),
            Self::InnerNode { depth } => write!(f, "InnerNode[depth={depth}]"),
        }
    }
}

impl Debug for NodeInfoAsSeenByLookingAtNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeInfoAsSeenByLookingAtNode({self})")
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct ReferencingBlobInfo {
    pub blob_id: BlobId,
    pub blob_info: BlobInfoAsExpectedByEntryInParent,
}

impl Display for ReferencingBlobInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{blob_id:?}:{blob_info}",
            blob_id = self.blob_id,
            blob_info = self.blob_info
        )
    }
}

impl Debug for ReferencingBlobInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ReferencingBlobInfo({self})")
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum NodeInfoAsExpectedByEntryInParent {
    RootNode {
        belongs_to_blob: ReferencingBlobInfo,
    },
    NonRootInnerNode {
        // `belongs_to_blob` can be `None` if the node is not (transitively) referenced by a blob
        belongs_to_blob: Option<ReferencingBlobInfo>,

        depth: NonZeroU8,
        parent_id: BlockId,
    },
    NonRootLeafNode {
        // `belongs_to_blob` can be `None` if the node is not (transitively) referenced by a blob
        belongs_to_blob: Option<ReferencingBlobInfo>,

        parent_id: BlockId,
    },
}

impl Display for NodeInfoAsExpectedByEntryInParent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO Better format for belongs_to_blob. Maybe layout more hierarchically like show blob_info first, then node_info?
        match self {
            Self::RootNode { belongs_to_blob } => {
                write!(f, "RootNode[belongs_to_blob={belongs_to_blob:?}]")
            }
            Self::NonRootInnerNode {
                belongs_to_blob,
                depth,
                parent_id,
            } => {
                write!(f, "NonRootInnerNode[belongs_to_blob={belongs_to_blob:?}, depth={depth}, parent={parent_id:?}]")
            }
            Self::NonRootLeafNode {
                belongs_to_blob,
                parent_id,
            } => {
                write!(
                    f,
                    "NonRootLeafNode[belongs_to_blob={belongs_to_blob:?}, parent={parent_id:?}]",
                )
            }
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct NodeReference {
    pub node_info: NodeInfoAsExpectedByEntryInParent,
}

/// A [CorruptedError] is an error we found in the file system when analyzing it
#[derive(Debug, Error, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum CorruptedError {
    #[error("Node {node_id:?} is unreadable and likely corrupted")]
    NodeUnreadable {
        node_id: BlockId,
        // `expected_node_info` is `None` if the node itself isn't reachable from the root blob of the file system
        // TODO Can this be Some but with NodeInfoAsExpectedByEntryInParent::belongs_to_blob==None? If yes, add tests for it. If not, change data model here.
        expected_node_info: Option<NodeInfoAsExpectedByEntryInParent>,
        // TODO error: anyhow::Error,
    },

    #[error("Node {node_id:?} is referenced but does not exist")]
    NodeMissing {
        node_id: BlockId,
        expected_node_info: NodeInfoAsExpectedByEntryInParent,
    },

    #[error("Node {node_id:?} is not referenced but exists")]
    NodeUnreferenced {
        node_id: BlockId,
        node_info: NodeInfoAsSeenByLookingAtNode,
    },

    // #[error("Node {node_id:?} is referenced but is not reachable. Possibly there is a cycle in a unconnected subtree")]
    // UnreachableSubtreeWithCycle { node_id: BlockId },
    #[error("{node_id:?} ({node_info:?}) is referenced multiple times, by {referenced_as:?}")]
    NodeReferencedMultipleTimes {
        node_id: BlockId,
        /// `node_info` is `None` if the node itself is missing
        node_info: Option<NodeInfoAsSeenByLookingAtNode>,
        referenced_as: BTreeSet<NodeReference>,
    },

    #[error("{blob_id:?} ({blob_info:?}) is referenced multiple times, by {referenced_as:?}")]
    BlobReferencedMultipleTimes {
        blob_id: BlobId,
        /// `blob_info` is `None` if the blob itself is missing
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
