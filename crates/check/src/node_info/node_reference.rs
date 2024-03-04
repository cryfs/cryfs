use std::num::NonZeroU8;

use cryfs_blockstore::BlockId;

/// Info about how we expect a node to look like, based on the reference to it from its parent node or parent blob.
#[derive(PartialEq, Debug, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
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
