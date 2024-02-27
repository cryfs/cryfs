use std::fmt::{self, Debug, Display};
use std::num::NonZeroU8;

use cryfs_blockstore::BlockId;

/// Info about how we expect a node to look like, based on the reference to it from its parent node or parent blob.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum NodeReference {
    RootNode,
    NonRootInnerNode {
        depth: NonZeroU8,
        parent_id: BlockId,
    },
    NonRootLeafNode {
        parent_id: BlockId,
    },
}

impl Display for NodeReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO Better format for belongs_to_blob. Maybe layout more hierarchically like show blob_info first, then node_info?
        match self {
            Self::RootNode => {
                write!(f, "RootNode")
            }
            Self::NonRootInnerNode { depth, parent_id } => {
                write!(f, "NonRootInnerNode[depth={depth}, parent={parent_id:?}]")
            }
            Self::NonRootLeafNode { parent_id } => {
                write!(f, "NonRootLeafNode[parent={parent_id:?}]",)
            }
        }
    }
}

impl Debug for NodeReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeReference({self})")
    }
}
