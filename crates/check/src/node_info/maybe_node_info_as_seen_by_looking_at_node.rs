use std::fmt::{self, Debug, Display};
use std::num::NonZeroU8;

use super::NodeInfoAsSeenByLookingAtNode;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum MaybeNodeInfoAsSeenByLookingAtNode {
    Missing,
    Unreadable,
    InnerNode { depth: NonZeroU8 },
    LeafNode,
}

impl Display for MaybeNodeInfoAsSeenByLookingAtNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => write!(f, "MissingNode"),
            Self::Unreadable => write!(f, "UnreadableNode"),
            Self::LeafNode => write!(f, "LeafNode"),
            Self::InnerNode { depth } => write!(f, "InnerNode[depth={depth}]"),
        }
    }
}

impl Debug for MaybeNodeInfoAsSeenByLookingAtNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MaybeNodeInfoAsSeenByLookingAtNode({self})")
    }
}

impl From<NodeInfoAsSeenByLookingAtNode> for MaybeNodeInfoAsSeenByLookingAtNode {
    fn from(node_info: NodeInfoAsSeenByLookingAtNode) -> Self {
        match node_info {
            NodeInfoAsSeenByLookingAtNode::Unreadable => Self::Unreadable,
            NodeInfoAsSeenByLookingAtNode::InnerNode { depth } => Self::InnerNode { depth },
            NodeInfoAsSeenByLookingAtNode::LeafNode => Self::LeafNode,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroU8;

    #[test]
    fn test_display() {
        assert_eq!(
            "MissingNode",
            format!("{}", MaybeNodeInfoAsSeenByLookingAtNode::Missing)
        );

        assert_eq!(
            "UnreadableNode",
            format!("{}", MaybeNodeInfoAsSeenByLookingAtNode::Unreadable)
        );

        assert_eq!(
            "LeafNode",
            format!("{}", MaybeNodeInfoAsSeenByLookingAtNode::LeafNode),
        );

        assert_eq!(
            "InnerNode[depth=3]",
            format!(
                "{}",
                MaybeNodeInfoAsSeenByLookingAtNode::InnerNode {
                    depth: NonZeroU8::new(3).unwrap(),
                }
            ),
        );
    }
}
