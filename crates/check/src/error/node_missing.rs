use std::collections::BTreeSet;
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

use cryfs_blockstore::BlockId;

use crate::node_info::NodeAndBlobReference;

#[derive(Error, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct NodeMissingError {
    pub node_id: BlockId,
    pub referenced_as: BTreeSet<NodeAndBlobReference>,
}

impl NodeMissingError {
    pub fn new(node_id: BlockId, referenced_as: BTreeSet<NodeAndBlobReference>) -> Self {
        assert!(referenced_as.len() > 0, "NodeMissingError should only be created if the node is referenced by at least one other node or blob");
        Self {
            node_id,
            referenced_as,
        }
    }
}

impl Display for NodeMissingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Node {node_id} is missing. It is referenced as",
            node_id = self.node_id
        )?;
        assert!(self.referenced_as.len() > 0, "NodeMissingError should only be created if the node is referenced by at least one other node or blob");
        if self.referenced_as.len() == 1 {
            write!(
                f,
                " {referenced_as}",
                referenced_as = self.referenced_as.iter().next().unwrap(),
            )?;
        } else {
            write!(f, ":\n")?;
            for referenced_as in &self.referenced_as {
                write!(f, "  - {referenced_as}")?;
            }
        }
        Ok(())
    }
}
