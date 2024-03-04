use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

use cryfs_blockstore::BlockId;

use super::display::{ErrorDisplayNodeInfo, ErrorTitle, NodeErrorDisplayMessage};
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

const ERROR_TITLE: ErrorTitle = ErrorTitle {
    error_type: "NodeUnreferenced",
    error_message: "Node is not referenced by any other nodes.",
};

impl Display for NodeUnreferencedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let error_display = NodeErrorDisplayMessage {
            error_title: ERROR_TITLE,

            node_info: ErrorDisplayNodeInfo {
                node_id: self.node_id,
                node_info: self.node_info.into(),
                node_referenced_as: std::iter::empty(),
            },
        };
        error_display.display(f)
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU8;

    use console::strip_ansi_codes;

    use super::*;

    #[test]
    fn test_display_unreadable() {
        let error = NodeUnreferencedError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            node_info: NodeInfoAsSeenByLookingAtNode::Unreadable,
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[NodeUnreferenced]: Node is not referenced by any other nodes.
  ---> No references to node found
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Node is unreadable
"
            .trim(),
        );
    }

    #[test]
    fn test_display_inner_node() {
        let error = NodeUnreferencedError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            node_info: NodeInfoAsSeenByLookingAtNode::InnerNode {
                depth: NonZeroU8::new(5).unwrap(),
            },
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[NodeUnreferenced]: Node is not referenced by any other nodes.
  ---> No references to node found
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Inner node [depth=5]
"
            .trim(),
        );
    }

    #[test]
    fn test_display_leaf_node() {
        let error = NodeUnreferencedError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            node_info: NodeInfoAsSeenByLookingAtNode::LeafNode,
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[NodeUnreferenced]: Node is not referenced by any other nodes.
  ---> No references to node found
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Leaf node
"
            .trim(),
        );
    }
}
