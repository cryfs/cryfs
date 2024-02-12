use std::collections::BTreeSet;
use std::fmt::{self, Display};
use thiserror::Error;

use cryfs_blobstore::BlobId;
use cryfs_blockstore::BlockId;
use cryfs_cryfs::filesystem::fsblobstore::BlobType;
use cryfs_rustfs::AbsolutePathBuf;

// TOOD Add more info to each error, e.g. parent pointers, blob a node belongs to, path in filesystem, ...
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct BlobInfo {
    pub blob_id: BlobId,
    pub blob_type: BlobType,
    pub path: AbsolutePathBuf,
}

impl Display for BlobInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let blob_type = match self.blob_type {
            BlobType::File => "File",
            BlobType::Dir => "Dir",
            BlobType::Symlink => "Symlink",
        };
        write!(
            f,
            "{blob_type}[{blob_id:?}] @ {path}",
            blob_id = self.blob_id,
            path = self.path,
        )
    }
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
    },

    #[error("Node {node_id:?} is referenced but does not exist")]
    NodeMissing {
        node_id: BlockId,
        // TODO blob_info: BlobInfo,
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
        // TODO referenced_by: BTreeSet<(BlockId, BlobInfo)>,
        // TODO expected_depths: u8,
    },

    #[error("Blob {blob_id:?} is referenced multiple times")]
    BlobReferencedMultipleTimes {
        blob_id: BlobId,
        // TODO replace blob_id with blob_info: BlobInfo,
        // TODO referenced_by: BTreeSet<BlobInfo>,
    },

    #[error("{expected_blob_info} is unreadable and likely corrupted")]
    BlobUnreadable {
        expected_blob_info: BlobInfo,
        // TODO expected_blob_type: ,
        // TODO referenced_by: BlobId,
        // TODO error:  anyhow::Error,
    },

    #[error("{expected_blob_info:?} is referenced but does not exist")]
    BlobMissing {
        expected_blob_info: BlobInfo,
        // TODO replace blob_id with blob_info: BlobInfo,
        // TODO expected_blob_type: ,
        // TODO referenced_by: BlobId,
    },

    #[error("Blob {blob_id:?} is referenced by parent {referenced_by:?} but has parent pointer {parent_pointer:?}")]
    WrongParentPointer {
        blob_id: BlobId,
        // TODO replace blob_id with blob_info: BlobInfo,
        referenced_by: BTreeSet<BlobId>,
        // TODO replace referenced_by with referenced_by: BTreeSet<BlobInfo>,
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
