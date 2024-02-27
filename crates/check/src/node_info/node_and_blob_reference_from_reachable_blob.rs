use std::fmt::{self, Debug, Display};

use crate::node_info::{BlobReferenceWithId, NodeReference};

/// A reference to a node that is reachable from a blob reachable from the root blob of the file system.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct NodeAndBlobReferenceFromReachableBlob {
    pub node_info: NodeReference,
    pub blob_info: BlobReferenceWithId,
}

impl Display for NodeAndBlobReferenceFromReachableBlob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO Better format
        write!(
            f,
            "{blob_info}:{node_info}",
            blob_info = self.blob_info,
            node_info = self.node_info,
        )
    }
}

impl Debug for NodeAndBlobReferenceFromReachableBlob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeAndBlobReferenceFromReachableBlob({self})")
    }
}
