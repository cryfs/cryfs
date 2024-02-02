use std::collections::BTreeSet;
use thiserror::Error;

use cryfs_blobstore::BlobId;
use cryfs_blockstore::BlockId;

// TOOD Add more info to each error, e.g. parent pointers, blob a node belongs to, path in filesystem, ...

/// A [CorruptedError] is an error we found in the file system when analyzing it
#[derive(Debug, Error, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum CorruptedError {
    #[error("Node {node_id:?} is unreadable and likely corrupted")]
    NodeUnreadable {
        node_id: BlockId,
        // TODO referenced_by: BlockId,
        // TODO error: anyhow::Error,
        // TODO expected_depth: u8,
    },

    #[error("Node {node_id:?} is referenced but does not exist")]
    NodeMissing {
        node_id: BlockId,
        // TODO referenced_by: BlockId,
        // TODO expected_depth: u8,
    },

    #[error("Node {node_id:?} is not referenced but exists")]
    NodeUnreferenced {
        node_id: BlockId,
        // TODO depth: u8,
    },

    // #[error("Node {node_id:?} is referenced but is not reachable. Possibly there is a cycle in a unconnected subtree")]
    // UnreachableSubtreeWithCycle { node_id: BlockId },
    #[error("Node {node_id:?} is referenced multiple times")]
    NodeReferencedMultipleTimes {
        node_id: BlockId,
        // TODO referenced_by: BTreeSet<BlockId>,
        // TODO expected_depths: u8,
    },

    #[error("Blob {blob_id:?} is referenced multiple times")]
    BlobReferencedMultipleTimes {
        blob_id: BlobId,
        // TODO blob_type: ,
        // TODO referenced_by: BTreeSet<BlobId>,
    },

    #[error("Blob {blob_id:?} is unreadable and likely corrupted")]
    BlobUnreadable {
        blob_id: BlobId,
        // TODO expected_blob_type: ,
        // TODO referenced_by: BlobId,
        // TODO error:  anyhow::Error,
    },

    #[error("Blob {blob_id:?} is referenced but does not exist")]
    BlobMissing {
        blob_id: BlobId,
        // TODO expected_blob_type: ,
        // TODO referenced_by: BlobId,
    },

    #[error("Blob {blob_id:?} is referenced by parent {referenced_by:?} but has parent pointer {parent_pointer:?}")]
    WrongParentPointer {
        blob_id: BlobId,
        // TODO blob_type: ,
        referenced_by: BTreeSet<BlobId>,
        parent_pointer: BlobId,
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
