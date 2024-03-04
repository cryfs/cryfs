use std::num::NonZeroU8;

#[derive(PartialEq, Eq, Debug, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum NodeInfoAsSeenByLookingAtNode {
    Unreadable,
    InnerNode { depth: NonZeroU8 },
    LeafNode,
}
