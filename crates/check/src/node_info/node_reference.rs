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
                write!(f, "NonRootInnerNode[depth={depth}, parent={parent_id}]")
            }
            Self::NonRootLeafNode { parent_id } => {
                write!(f, "NonRootLeafNode[parent={parent_id}]",)
            }
        }
    }
}

impl Debug for NodeReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeReference({self})")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroU8;

    #[test]
    fn test_display() {
        let parent_id = BlockId::from_hex("A370E99ADA93EF706935F4693039C90D").unwrap();

        assert_eq!("RootNode", format!("{}", NodeReference::RootNode));

        assert_eq!(
            "NonRootInnerNode[depth=3, parent=A370E99ADA93EF706935F4693039C90D]",
            format!(
                "{}",
                NodeReference::NonRootInnerNode {
                    depth: NonZeroU8::new(3).unwrap(),
                    parent_id,
                }
            ),
        );

        assert_eq!(
            "NonRootLeafNode[parent=A370E99ADA93EF706935F4693039C90D]",
            format!("{}", NodeReference::NonRootLeafNode { parent_id },),
        );
    }
}
