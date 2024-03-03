use std::collections::BTreeSet;
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

use cryfs_blockstore::BlockId;

use crate::node_info::{MaybeNodeInfoAsSeenByLookingAtNode, NodeAndBlobReference};

#[derive(Error, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct NodeReferencedMultipleTimesError {
    pub node_id: BlockId,
    pub node_info: MaybeNodeInfoAsSeenByLookingAtNode,
    pub referenced_as: BTreeSet<NodeAndBlobReference>,
}

impl NodeReferencedMultipleTimesError {
    pub fn new(
        node_id: BlockId,
        node_info: MaybeNodeInfoAsSeenByLookingAtNode,
        referenced_as: BTreeSet<NodeAndBlobReference>,
    ) -> Self {
        assert!(
            referenced_as.len() >= 2,
            "referenced_as is {} but must be at least 2",
            referenced_as.len()
        );
        Self {
            node_id,
            node_info,
            referenced_as,
        }
    }
}

impl Display for NodeReferencedMultipleTimesError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Node {node_id} is referenced multiple times",
            node_id = self.node_id,
        )?;
        match self.node_info {
            MaybeNodeInfoAsSeenByLookingAtNode::Missing => write!(f, " and is missing.")?,
            MaybeNodeInfoAsSeenByLookingAtNode::Unreadable => {
                write!(f, " and is unreadable and likely corrupted.")?
            }
            MaybeNodeInfoAsSeenByLookingAtNode::InnerNode { .. }
            | MaybeNodeInfoAsSeenByLookingAtNode::LeafNode => {
                write!(f, " and exists as {node_info}.", node_info = self.node_info)?
            }
        }

        write!(f, " It is referenced as:\n")?;

        for referenced_as in &self.referenced_as {
            write!(f, "  - {referenced_as}")?;
        }
        Ok(())
    }
}
