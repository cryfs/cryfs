use std::num::NonZeroU8;

use super::NodeInfoAsSeenByLookingAtNode;

#[derive(PartialEq, Debug, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum MaybeNodeInfoAsSeenByLookingAtNode {
    Missing,
    Unreadable,
    InnerNode { depth: NonZeroU8 },
    LeafNode,
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
