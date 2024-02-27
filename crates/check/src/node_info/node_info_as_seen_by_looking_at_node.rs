use std::fmt::{self, Debug, Display};
use std::num::NonZeroU8;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroU8;

    #[test]
    fn test_display() {
        assert_eq!(
            "UnreadableNode",
            format!("{}", NodeInfoAsSeenByLookingAtNode::Unreadable)
        );

        assert_eq!(
            "LeafNode",
            format!("{}", NodeInfoAsSeenByLookingAtNode::LeafNode),
        );

        assert_eq!(
            "InnerNode[depth=3]",
            format!(
                "{}",
                NodeInfoAsSeenByLookingAtNode::InnerNode {
                    depth: NonZeroU8::new(3).unwrap(),
                }
            ),
        );
    }
}
