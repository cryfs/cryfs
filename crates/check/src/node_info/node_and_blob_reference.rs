use std::fmt::{self, Debug, Display};
use std::num::NonZeroU8;

use cryfs_blockstore::BlockId;

use super::{BlobReferenceWithId, NodeAndBlobReferenceFromReachableBlob, NodeReference};

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum NodeAndBlobReference {
    RootNode {
        belongs_to_blob: BlobReferenceWithId,
    },
    NonRootInnerNode {
        // `belongs_to_blob` can be `None` if the node is part of a subtree that is unreachable from the filesystem root
        belongs_to_blob: Option<BlobReferenceWithId>,

        depth: NonZeroU8,
        parent_id: BlockId,
    },
    NonRootLeafNode {
        // `belongs_to_blob` can be `None` if the node is part of a subtree that is unreachable from the filesystem root
        belongs_to_blob: Option<BlobReferenceWithId>,

        parent_id: BlockId,
    },
}

impl Display for NodeAndBlobReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO Better format
        match self {
            Self::RootNode { belongs_to_blob } => {
                write!(f, "{belongs_to_blob}:RootNode")
            }
            Self::NonRootInnerNode {
                belongs_to_blob,
                depth,
                parent_id,
            } => {
                write!(
                    f,
                    "{belongs_to_blob:?}:NonRootInnerNode[depth={depth}, parent={parent_id:?}]"
                )
            }
            Self::NonRootLeafNode {
                belongs_to_blob,
                parent_id,
            } => {
                write!(
                    f,
                    "{belongs_to_blob:?}:NonRootLeafNode[parent={parent_id:?}]",
                )
            }
        }
    }
}

impl Debug for NodeAndBlobReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeAndBlobReference({self})")
    }
}

impl From<NodeAndBlobReferenceFromReachableBlob> for NodeAndBlobReference {
    fn from(node_reference: NodeAndBlobReferenceFromReachableBlob) -> Self {
        match node_reference.node_info {
            NodeReference::RootNode => Self::RootNode {
                belongs_to_blob: node_reference.blob_info,
            },
            NodeReference::NonRootInnerNode { depth, parent_id } => Self::NonRootInnerNode {
                belongs_to_blob: Some(node_reference.blob_info),
                depth,
                parent_id,
            },
            NodeReference::NonRootLeafNode { parent_id } => Self::NonRootLeafNode {
                belongs_to_blob: Some(node_reference.blob_info),
                parent_id,
            },
        }
    }
}
