use std::collections::BTreeSet;
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

use cryfs_blockstore::BlockId;

use crate::node_info::NodeAndBlobReference;

#[derive(Error, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct NodeUnreadableError {
    pub node_id: BlockId,
    // `referenced_as` can be empty if the node itself isn't referenced from any readable blocks or blobs
    pub referenced_as: BTreeSet<NodeAndBlobReference>,
    // TODO error: anyhow::Error,
}

impl NodeUnreadableError {
    pub fn new(node_id: BlockId, referenced_as: BTreeSet<NodeAndBlobReference>) -> Self {
        Self {
            node_id,
            referenced_as,
            // TODO error: anyhow::Error,
        }
    }
}

impl Display for NodeUnreadableError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Node {node_id} is unreadable and likely corrupted. ",
            node_id = self.node_id
        )?;
        if self.referenced_as.is_empty() {
            write!(f, "It isn't referenced by any readable nodes or blobs.")?;
        } else if self.referenced_as.len() == 1 {
            write!(
                f,
                "It is referenced as {referenced_as}.",
                referenced_as = self.referenced_as.iter().next().unwrap()
            )?;
        } else {
            write!(f, "It is referenced as:\n")?;
            for referenced_as in &self.referenced_as {
                write!(f, "  - {referenced_as}")?;
            }
        }
        Ok(())
    }
}
