use std::fmt::{self, Debug, Display};
use std::num::NonZeroU8;

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
