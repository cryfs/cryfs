use thiserror::Error;

use cryfs_blobstore::BlobId;
use cryfs_blockstore::BlockId;

#[derive(Debug, Error, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub enum CorruptedError {
    #[error("Node {node_id:?} is unreadable and likely corrupted")]
    NodeUnreadable {
        node_id: BlockId,
        // TODO error: anyhow::Error,
    },

    #[error("Node {node_id:?} is referenced but does not exist")]
    NodeMissing { node_id: BlockId },

    #[error("Node {node_id:?} is not referenced but exists")]
    NodeUnreferenced { node_id: BlockId },

    // #[error("Node {node_id:?} is referenced but is not reachable. Possibly there is a cycle in a unconnected subtree")]
    // UnreachableSubtreeWithCycle { node_id: BlockId },
    #[error("Node {node_id:?} is referenced multiple times")]
    NodeReferencedMultipleTimes {
        node_id: BlockId,
        // TODO parents: HashSet<BlockId>,
    },

    #[error("Blob {blob_id:?} is referenced multiple times")]
    BlobReferencedMultipleTimes {
        blob_id: BlobId,
        // TODO parents: HashSet<BlobId>,
    },

    // #[error("Cyclic self-reference: Node {node_id:?} references itself")]
    // NodeHasCyclicSelfReference { node_id: BlockId },

    // #[error("Cyclic self-reference: Dir Blob {blob_id:?} references itself")]
    // DirBlobHasCyclicSelfReference { blob_id: BlobId },
    #[error("Blob {blob_id:?} is unreadable and likely corrupted")]
    BlobUnreadable {
        blob_id: BlobId,
        // TODO error:  anyhow::Error,
    },

    #[error("Blob {blob_id:?} is referenced but does not exist")]
    BlobMissing { blob_id: BlobId },

    #[error("Blob {blob_id:?} is referenced by parent {referenced_by:?} but has parent pointer {parent_pointer:?}")]
    WrongParentPointer {
        blob_id: BlobId,
        referenced_by: Vec<BlobId>,
        parent_pointer: BlobId,
    },

    /// Not an actual error but reported by a check to indicate that we need to assert that another check reported this error.
    /// This is reported by checks who aren't the main responsible check for a condition but discovered something on the side
    /// that another check should have reported.
    #[error("We need to assert that {0} was reported")]
    Assert(Box<CorruptedError>),
}
