use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

use cryfs_blockstore::BlockId;

use crate::node_info::NodeInfoAsSeenByLookingAtNode;

#[derive(Error, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct NodeUnreferencedError {
    pub node_id: BlockId,
    pub node_info: NodeInfoAsSeenByLookingAtNode,
}

impl NodeUnreferencedError {
    pub fn new(node_id: BlockId, node_info: NodeInfoAsSeenByLookingAtNode) -> Self {
        Self { node_id, node_info }
    }
}

impl Display for NodeUnreferencedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Node {node_id} exists as {node_info} but is unreferenced.",
            node_id = self.node_id,
            node_info = self.node_info,
        )
    }
}
