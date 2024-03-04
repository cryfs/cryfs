use std::fmt::Debug;

use crate::node_info::{BlobReferenceWithId, NodeReference};

/// A reference to a node that is reachable from a blob reachable from the root blob of the file system.
#[derive(PartialEq, Debug, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct NodeAndBlobReferenceFromReachableBlob {
    pub node_info: NodeReference,
    pub blob_info: BlobReferenceWithId,
}
