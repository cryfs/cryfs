use thiserror::Error;

use cryfs_blobstore::BlobId;
use cryfs_blockstore::BlockId;

#[derive(Debug, Error)]
pub enum CorruptedError {
    #[error("Node {node_id:?} is unreadable and likely corrupted")]
    NodeUnreadable {
        node_id: BlockId,
        error: anyhow::Error,
    },

    #[error("Node {node_id:?} is referenced but does not exist")]
    NodeMissing { node_id: BlockId },

    #[error("Node {node_id:?} is not referenced but exists")]
    NodeUnreferenced { node_id: BlockId },

    #[error("Node {node_id:?} is referenced multiple times")]
    NodeReferencedMultipleTimes { node_id: BlockId },

    #[error("Cyclic self-reference: Node {node_id:?} references itself")]
    NodeHasCyclicSelfReference { node_id: BlockId },

    #[error("Cyclic self-reference: Dir Blob {blob_id:?} references itself")]
    DirBlobHasCyclicSelfReference { blob_id: BlobId },

    #[error("Blob {blob_id:?} is unreadable and likely corrupted")]
    BlobUnreadable {
        blob_id: BlobId,
        error: anyhow::Error,
    },

    #[error("Blob {blob_id:?} is referenced but does not exist")]
    BlobMissing { blob_id: BlobId },
}
