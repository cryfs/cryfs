use std::collections::BTreeSet;
use std::fmt::Debug;
use thiserror::Error;

use cryfs_blobstore::BlobId;
use cryfs_blockstore::BlockId;

use crate::node_info::{
    BlobInfoAsSeenByLookingAtBlob, BlobReference, NodeAndBlobReference,
    NodeAndBlobReferenceFromReachableBlob, NodeInfoAsSeenByLookingAtNode,
};

// TODO Add more info to each error, e.g. parent pointers, blob a node belongs to, path in filesystem, ...

// TODO Improve error messages

/// A [CorruptedError] is an error we found in the file system when analyzing it
#[derive(Debug, Error, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum CorruptedError {
    #[error("Node {node_id:?} is unreadable and likely corrupted")]
    NodeUnreadable {
        node_id: BlockId,
        // `referenced_as` is `None` if the node itself isn't reachable from the root blob of the file system
        referenced_as: Option<NodeAndBlobReferenceFromReachableBlob>,
        // TODO referenced_as: BTreeSet<NodeAndBlobReference>,
        // TODO error: anyhow::Error,
    },

    #[error("Node {node_id:?} is referenced but does not exist")]
    NodeMissing {
        node_id: BlockId,
        referenced_as: BTreeSet<NodeAndBlobReference>,
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
        referenced_as: BTreeSet<NodeAndBlobReference>,
    },

    #[error("{blob_id:?} ({blob_info:?}) is referenced multiple times, by {referenced_as:?}")]
    BlobReferencedMultipleTimes {
        blob_id: BlobId,
        /// `blob_info` is `None` if the blob itself is missing
        blob_info: Option<BlobInfoAsSeenByLookingAtBlob>,
        referenced_as: BTreeSet<BlobReference>,
    },

    #[error("{blob_id:?}:{referenced_as} is unreadable and likely corrupted")]
    BlobUnreadable {
        blob_id: BlobId,
        referenced_as: BlobReference,
        // TODO error:  anyhow::Error,
    },

    #[error("{blob_id:?}:{blob_info} is referenced by {referenced_as:?}, but the parent pointer doesn't match any of the references")]
    WrongParentPointer {
        blob_id: BlobId,
        blob_info: BlobInfoAsSeenByLookingAtBlob,
        referenced_as: BTreeSet<BlobReference>,
    },
}

/// A CheckError is an error found in the analysis itself. This doesn't necessarily mean that the file system is corrupted
#[derive(Error, Debug)]
pub enum CheckError {
    #[error("The filesystem was modified while the check was running. Please make sure the file system is not mounted or modified for the duration of the check.\n Details: {msg}")]
    FilesystemModified { msg: String },
}
