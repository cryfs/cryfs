use std::collections::BTreeSet;
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

use cryfs_blockstore::BlockId;

use super::display::{ErrorDisplayNodeInfo, ErrorTitle, NodeErrorDisplayMessage};
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

const ERROR_TITLE: ErrorTitle = ErrorTitle {
    error_type: "NodeReferencedMultipleTimes",
    error_message: "Node is referenced multiple times.",
};

impl Display for NodeReferencedMultipleTimesError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        assert!(self.referenced_as.len() >= 2);

        let error_display = NodeErrorDisplayMessage {
            error_title: ERROR_TITLE,

            node_info: ErrorDisplayNodeInfo {
                node_id: self.node_id,
                node_info: self.node_info,
                node_referenced_as: self.referenced_as.iter(),
            },
        };
        error_display.display(f)
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU8;

    use console::strip_ansi_codes;
    use cryfs_blobstore::BlobId;
    use cryfs_cryfs::filesystem::fsblobstore::BlobType;
    use cryfs_rustfs::AbsolutePathBuf;

    use crate::{BlobReference, BlobReferenceWithId, MaybeBlobReferenceWithId};

    use super::*;

    #[test]
    fn test_display_missing() {
        let error = NodeReferencedMultipleTimesError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            node_info: MaybeNodeInfoAsSeenByLookingAtNode::Missing,
            referenced_as: [
                NodeAndBlobReference::RootNode {
                    belongs_to_blob: BlobReferenceWithId {
                        blob_id: BlobId::from_hex("1b918ca6acc275615c3de0525c700cea").unwrap(),
                        referenced_as: BlobReference {
                            blob_type: BlobType::File,
                            parent_id: BlobId::from_hex("3ef706935f4693039c90da370e99ada9")
                                .unwrap(),
                            path: AbsolutePathBuf::try_from_string("/path/to/file".to_string())
                                .unwrap(),
                        },
                    },
                },
                NodeAndBlobReference::NonRootInnerNode {
                    depth: NonZeroU8::new(4).unwrap(),
                    parent_id: BlockId::from_hex("a370e993ef706935f4693039c9ada90d").unwrap(),
                    belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                        blob_id: BlobId::from_hex("06935f4693039c90da370e99ada93ef7").unwrap(),
                        referenced_as: BlobReference {
                            blob_type: BlobType::Dir,
                            parent_id: BlobId::from_hex("3039c90da370e99ada93ef706935f469")
                                .unwrap(),
                            path: AbsolutePathBuf::try_from_string("/path/to/dir".to_string())
                                .unwrap(),
                        },
                    },
                },
            ]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[NodeReferencedMultipleTimes]: Node is referenced multiple times.
  ---> In file at /path/to/file
       Blob: id=1b918ca6acc275615c3de0525c700cea, parent_blob=3ef706935f4693039c90da370e99ada9
       Node referenced as: Root node
  ---> In dir at /path/to/dir
       Blob: id=06935f4693039c90da370e99ada93ef7, parent_blob=3039c90da370e99ada93ef706935f469
       Node referenced as: Non-root inner node [depth=4, parent_node=a370e993ef706935f4693039c9ada90d]
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Node is missing
"
            .trim(),
        );
    }

    #[test]
    fn test_display_unreadable() {
        let error = NodeReferencedMultipleTimesError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            node_info: MaybeNodeInfoAsSeenByLookingAtNode::Unreadable,
            referenced_as: [
                NodeAndBlobReference::NonRootLeafNode {
                    parent_id: BlockId::from_hex("4693039c9ada90da370e993ef706935f").unwrap(),
                    belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                        blob_id: BlobId::from_hex("1b918ca6acc275615c3de0525c700cea").unwrap(),
                        referenced_as: BlobReference {
                            blob_type: BlobType::File,
                            parent_id: BlobId::from_hex("3ef706935f4693039c90da370e99ada9")
                                .unwrap(),
                            path: AbsolutePathBuf::try_from_string("/path/to/file".to_string())
                                .unwrap(),
                        },
                    },
                },
                NodeAndBlobReference::NonRootInnerNode {
                    depth: NonZeroU8::new(4).unwrap(),
                    parent_id: BlockId::from_hex("a370e993ef706935f4693039c9ada90d").unwrap(),
                    belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                        blob_id: BlobId::from_hex("06935f4693039c90da370e99ada93ef7").unwrap(),
                        referenced_as: BlobReference {
                            blob_type: BlobType::Dir,
                            parent_id: BlobId::from_hex("3039c90da370e99ada93ef706935f469")
                                .unwrap(),
                            path: AbsolutePathBuf::try_from_string("/path/to/dir".to_string())
                                .unwrap(),
                        },
                    },
                },
            ]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[NodeReferencedMultipleTimes]: Node is referenced multiple times.
  ---> In dir at /path/to/dir
       Blob: id=06935f4693039c90da370e99ada93ef7, parent_blob=3039c90da370e99ada93ef706935f469
       Node referenced as: Non-root inner node [depth=4, parent_node=a370e993ef706935f4693039c9ada90d]
  ---> In file at /path/to/file
       Blob: id=1b918ca6acc275615c3de0525c700cea, parent_blob=3ef706935f4693039c90da370e99ada9
       Node referenced as: Non-root leaf node [parent_node=4693039c9ada90da370e993ef706935f]
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Node is unreadable
"
            .trim(),
        );
    }

    #[test]
    fn test_display_inner_node() {
        let error = NodeReferencedMultipleTimesError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            node_info: MaybeNodeInfoAsSeenByLookingAtNode::InnerNode {
                depth: NonZeroU8::new(5).unwrap(),
            },
            referenced_as: [
                NodeAndBlobReference::NonRootLeafNode {
                    parent_id: BlockId::from_hex("4693039c9ada90da370e993ef706935f").unwrap(),
                    belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                        blob_id: BlobId::from_hex("1b918ca6acc275615c3de0525c700cea").unwrap(),
                        referenced_as: BlobReference {
                            blob_type: BlobType::File,
                            parent_id: BlobId::from_hex("3ef706935f4693039c90da370e99ada9")
                                .unwrap(),
                            path: AbsolutePathBuf::try_from_string("/path/to/file".to_string())
                                .unwrap(),
                        },
                    },
                },
                NodeAndBlobReference::NonRootInnerNode {
                    depth: NonZeroU8::new(4).unwrap(),
                    parent_id: BlockId::from_hex("a370e993ef706935f4693039c9ada90d").unwrap(),
                    belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                        blob_id: BlobId::from_hex("06935f4693039c90da370e99ada93ef7").unwrap(),
                        referenced_as: BlobReference {
                            blob_type: BlobType::Dir,
                            parent_id: BlobId::from_hex("3039c90da370e99ada93ef706935f469")
                                .unwrap(),
                            path: AbsolutePathBuf::try_from_string("/path/to/dir".to_string())
                                .unwrap(),
                        },
                    },
                },
            ]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[NodeReferencedMultipleTimes]: Node is referenced multiple times.
  ---> In dir at /path/to/dir
       Blob: id=06935f4693039c90da370e99ada93ef7, parent_blob=3039c90da370e99ada93ef706935f469
       Node referenced as: Non-root inner node [depth=4, parent_node=a370e993ef706935f4693039c9ada90d]
  ---> In file at /path/to/file
       Blob: id=1b918ca6acc275615c3de0525c700cea, parent_blob=3ef706935f4693039c90da370e99ada9
       Node referenced as: Non-root leaf node [parent_node=4693039c9ada90da370e993ef706935f]
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Inner node [depth=5]
"
            .trim(),
        );
    }

    #[test]
    fn test_display_leaf_node() {
        let error = NodeReferencedMultipleTimesError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            node_info: MaybeNodeInfoAsSeenByLookingAtNode::LeafNode,
            referenced_as: [
                NodeAndBlobReference::NonRootLeafNode {
                    parent_id: BlockId::from_hex("4693039c9ada90da370e993ef706935f").unwrap(),
                    belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                        blob_id: BlobId::from_hex("1b918ca6acc275615c3de0525c700cea").unwrap(),
                        referenced_as: BlobReference {
                            blob_type: BlobType::File,
                            parent_id: BlobId::from_hex("3ef706935f4693039c90da370e99ada9")
                                .unwrap(),
                            path: AbsolutePathBuf::try_from_string("/path/to/file".to_string())
                                .unwrap(),
                        },
                    },
                },
                NodeAndBlobReference::NonRootInnerNode {
                    depth: NonZeroU8::new(4).unwrap(),
                    parent_id: BlockId::from_hex("a370e993ef706935f4693039c9ada90d").unwrap(),
                    belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                        blob_id: BlobId::from_hex("06935f4693039c90da370e99ada93ef7").unwrap(),
                        referenced_as: BlobReference {
                            blob_type: BlobType::Dir,
                            parent_id: BlobId::from_hex("3039c90da370e99ada93ef706935f469")
                                .unwrap(),
                            path: AbsolutePathBuf::try_from_string("/path/to/dir".to_string())
                                .unwrap(),
                        },
                    },
                },
            ]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[NodeReferencedMultipleTimes]: Node is referenced multiple times.
  ---> In dir at /path/to/dir
       Blob: id=06935f4693039c90da370e99ada93ef7, parent_blob=3039c90da370e99ada93ef706935f469
       Node referenced as: Non-root inner node [depth=4, parent_node=a370e993ef706935f4693039c9ada90d]
  ---> In file at /path/to/file
       Blob: id=1b918ca6acc275615c3de0525c700cea, parent_blob=3ef706935f4693039c90da370e99ada9
       Node referenced as: Non-root leaf node [parent_node=4693039c9ada90da370e993ef706935f]
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Leaf node
"
            .trim(),
        );
    }

    #[test]
    fn test_display_referenced_as_root_node() {
        let error = NodeReferencedMultipleTimesError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            node_info: MaybeNodeInfoAsSeenByLookingAtNode::LeafNode,
            referenced_as: [
                NodeAndBlobReference::RootNode {
                    belongs_to_blob: BlobReferenceWithId {
                        blob_id: BlobId::from_hex("1b918ca6acc275615c3de0525c700cea").unwrap(),
                        referenced_as: BlobReference {
                            blob_type: BlobType::File,
                            parent_id: BlobId::from_hex("3ef706935f4693039c90da370e99ada9")
                                .unwrap(),
                            path: AbsolutePathBuf::try_from_string("/path/to/file".to_string())
                                .unwrap(),
                        },
                    },
                },
                NodeAndBlobReference::NonRootInnerNode {
                    depth: NonZeroU8::new(4).unwrap(),
                    parent_id: BlockId::from_hex("a370e993ef706935f4693039c9ada90d").unwrap(),
                    belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                        blob_id: BlobId::from_hex("06935f4693039c90da370e99ada93ef7").unwrap(),
                        referenced_as: BlobReference {
                            blob_type: BlobType::Dir,
                            parent_id: BlobId::from_hex("3039c90da370e99ada93ef706935f469")
                                .unwrap(),
                            path: AbsolutePathBuf::try_from_string("/path/to/dir".to_string())
                                .unwrap(),
                        },
                    },
                },
            ]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[NodeReferencedMultipleTimes]: Node is referenced multiple times.
  ---> In file at /path/to/file
       Blob: id=1b918ca6acc275615c3de0525c700cea, parent_blob=3ef706935f4693039c90da370e99ada9
       Node referenced as: Root node
  ---> In dir at /path/to/dir
       Blob: id=06935f4693039c90da370e99ada93ef7, parent_blob=3039c90da370e99ada93ef706935f469
       Node referenced as: Non-root inner node [depth=4, parent_node=a370e993ef706935f4693039c9ada90d]
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Leaf node
"
            .trim(),
        );
    }

    #[test]
    fn test_display_referenced_as_inner_node_reachable() {
        let error = NodeReferencedMultipleTimesError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            node_info: MaybeNodeInfoAsSeenByLookingAtNode::InnerNode {
                depth: NonZeroU8::new(2).unwrap(),
            },
            referenced_as: [
                NodeAndBlobReference::NonRootInnerNode {
                    depth: NonZeroU8::new(5).unwrap(),
                    parent_id: BlockId::from_hex("ada90da370e993ef706935f4693039c9").unwrap(),
                    belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                        blob_id: BlobId::from_hex("1b918ca6acc275615c3de0525c700cea").unwrap(),
                        referenced_as: BlobReference {
                            blob_type: BlobType::File,
                            parent_id: BlobId::from_hex("3ef706935f4693039c90da370e99ada9")
                                .unwrap(),
                            path: AbsolutePathBuf::try_from_string("/path/to/file".to_string())
                                .unwrap(),
                        },
                    },
                },
                NodeAndBlobReference::NonRootInnerNode {
                    depth: NonZeroU8::new(4).unwrap(),
                    parent_id: BlockId::from_hex("a370e993ef706935f4693039c9ada90d").unwrap(),
                    belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                        blob_id: BlobId::from_hex("06935f4693039c90da370e99ada93ef7").unwrap(),
                        referenced_as: BlobReference {
                            blob_type: BlobType::Dir,
                            parent_id: BlobId::from_hex("3039c90da370e99ada93ef706935f469")
                                .unwrap(),
                            path: AbsolutePathBuf::try_from_string("/path/to/dir".to_string())
                                .unwrap(),
                        },
                    },
                },
            ]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[NodeReferencedMultipleTimes]: Node is referenced multiple times.
  ---> In dir at /path/to/dir
       Blob: id=06935f4693039c90da370e99ada93ef7, parent_blob=3039c90da370e99ada93ef706935f469
       Node referenced as: Non-root inner node [depth=4, parent_node=a370e993ef706935f4693039c9ada90d]
  ---> In file at /path/to/file
       Blob: id=1b918ca6acc275615c3de0525c700cea, parent_blob=3ef706935f4693039c90da370e99ada9
       Node referenced as: Non-root inner node [depth=5, parent_node=ada90da370e993ef706935f4693039c9]
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Inner node [depth=2]
"
            .trim(),
        );
    }

    #[test]
    fn test_display_referenced_as_inner_node_unreachable() {
        let error = NodeReferencedMultipleTimesError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            node_info: MaybeNodeInfoAsSeenByLookingAtNode::InnerNode {
                depth: NonZeroU8::new(2).unwrap(),
            },
            referenced_as: [
                NodeAndBlobReference::NonRootInnerNode {
                    depth: NonZeroU8::new(5).unwrap(),
                    parent_id: BlockId::from_hex("ada90da370e993ef706935f4693039c9").unwrap(),
                    belongs_to_blob: MaybeBlobReferenceWithId::UnreachableFromFilesystemRoot,
                },
                NodeAndBlobReference::NonRootInnerNode {
                    depth: NonZeroU8::new(4).unwrap(),
                    parent_id: BlockId::from_hex("a370e993ef706935f4693039c9ada90d").unwrap(),
                    belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                        blob_id: BlobId::from_hex("06935f4693039c90da370e99ada93ef7").unwrap(),
                        referenced_as: BlobReference {
                            blob_type: BlobType::Dir,
                            parent_id: BlobId::from_hex("3039c90da370e99ada93ef706935f469")
                                .unwrap(),
                            path: AbsolutePathBuf::try_from_string("/path/to/dir".to_string())
                                .unwrap(),
                        },
                    },
                },
            ]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[NodeReferencedMultipleTimes]: Node is referenced multiple times.
  ---> In unreachable blob
       Node referenced as: Non-root inner node [depth=5, parent_node=ada90da370e993ef706935f4693039c9]
  ---> In dir at /path/to/dir
       Blob: id=06935f4693039c90da370e99ada93ef7, parent_blob=3039c90da370e99ada93ef706935f469
       Node referenced as: Non-root inner node [depth=4, parent_node=a370e993ef706935f4693039c9ada90d]
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Inner node [depth=2]
"
            .trim(),
        );
    }

    #[test]
    fn test_display_referenced_as_leaf_node_reachable() {
        let error = NodeReferencedMultipleTimesError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            node_info: MaybeNodeInfoAsSeenByLookingAtNode::InnerNode {
                depth: NonZeroU8::new(2).unwrap(),
            },
            referenced_as: [
                NodeAndBlobReference::NonRootLeafNode {
                    parent_id: BlockId::from_hex("ada90da370e993ef706935f4693039c9").unwrap(),
                    belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                        blob_id: BlobId::from_hex("1b918ca6acc275615c3de0525c700cea").unwrap(),
                        referenced_as: BlobReference {
                            blob_type: BlobType::File,
                            parent_id: BlobId::from_hex("3ef706935f4693039c90da370e99ada9")
                                .unwrap(),
                            path: AbsolutePathBuf::try_from_string("/path/to/file".to_string())
                                .unwrap(),
                        },
                    },
                },
                NodeAndBlobReference::NonRootInnerNode {
                    depth: NonZeroU8::new(4).unwrap(),
                    parent_id: BlockId::from_hex("a370e993ef706935f4693039c9ada90d").unwrap(),
                    belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                        blob_id: BlobId::from_hex("06935f4693039c90da370e99ada93ef7").unwrap(),
                        referenced_as: BlobReference {
                            blob_type: BlobType::Dir,
                            parent_id: BlobId::from_hex("3039c90da370e99ada93ef706935f469")
                                .unwrap(),
                            path: AbsolutePathBuf::try_from_string("/path/to/dir".to_string())
                                .unwrap(),
                        },
                    },
                },
            ]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[NodeReferencedMultipleTimes]: Node is referenced multiple times.
  ---> In dir at /path/to/dir
       Blob: id=06935f4693039c90da370e99ada93ef7, parent_blob=3039c90da370e99ada93ef706935f469
       Node referenced as: Non-root inner node [depth=4, parent_node=a370e993ef706935f4693039c9ada90d]
  ---> In file at /path/to/file
       Blob: id=1b918ca6acc275615c3de0525c700cea, parent_blob=3ef706935f4693039c90da370e99ada9
       Node referenced as: Non-root leaf node [parent_node=ada90da370e993ef706935f4693039c9]
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Inner node [depth=2]
"
            .trim(),
        );
    }

    #[test]
    fn test_display_referenced_as_leaf_node_unreachable() {
        let error = NodeReferencedMultipleTimesError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            node_info: MaybeNodeInfoAsSeenByLookingAtNode::InnerNode {
                depth: NonZeroU8::new(2).unwrap(),
            },
            referenced_as: [
                NodeAndBlobReference::NonRootLeafNode {
                    parent_id: BlockId::from_hex("ada90da370e993ef706935f4693039c9").unwrap(),
                    belongs_to_blob: MaybeBlobReferenceWithId::UnreachableFromFilesystemRoot,
                },
                NodeAndBlobReference::NonRootInnerNode {
                    depth: NonZeroU8::new(4).unwrap(),
                    parent_id: BlockId::from_hex("a370e993ef706935f4693039c9ada90d").unwrap(),
                    belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                        blob_id: BlobId::from_hex("06935f4693039c90da370e99ada93ef7").unwrap(),
                        referenced_as: BlobReference {
                            blob_type: BlobType::Dir,
                            parent_id: BlobId::from_hex("3039c90da370e99ada93ef706935f469")
                                .unwrap(),
                            path: AbsolutePathBuf::try_from_string("/path/to/dir".to_string())
                                .unwrap(),
                        },
                    },
                },
            ]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[NodeReferencedMultipleTimes]: Node is referenced multiple times.
  ---> In dir at /path/to/dir
       Blob: id=06935f4693039c90da370e99ada93ef7, parent_blob=3039c90da370e99ada93ef706935f469
       Node referenced as: Non-root inner node [depth=4, parent_node=a370e993ef706935f4693039c9ada90d]
  ---> In unreachable blob
       Node referenced as: Non-root leaf node [parent_node=ada90da370e993ef706935f4693039c9]
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Inner node [depth=2]
"
            .trim(),
        );
    }

    #[test]
    fn test_display_many_references() {
        let error = NodeReferencedMultipleTimesError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            node_info: MaybeNodeInfoAsSeenByLookingAtNode::InnerNode {
                depth: NonZeroU8::new(2).unwrap(),
            },
            referenced_as: [
                NodeAndBlobReference::NonRootLeafNode {
                    parent_id: BlockId::from_hex("ada90da370e993ef706935f4693039c9").unwrap(),
                    belongs_to_blob: MaybeBlobReferenceWithId::UnreachableFromFilesystemRoot,
                },
                NodeAndBlobReference::NonRootLeafNode {
                    parent_id: BlockId::from_hex("ada90da370e993ef706935f4693039c9").unwrap(),
                    belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                        blob_id: BlobId::from_hex("35f4693039c90da370e99ada93ef7069").unwrap(),
                        referenced_as: BlobReference {
                            blob_type: BlobType::File,
                            parent_id: BlobId::from_hex("3da370e99ada93ef706935f46039c909")
                                .unwrap(),
                            path: AbsolutePathBuf::try_from_string("/path/to/file/1".to_string())
                                .unwrap(),
                        },
                    },
                },
                NodeAndBlobReference::NonRootInnerNode {
                    depth: NonZeroU8::new(3).unwrap(),
                    parent_id: BlockId::from_hex("9c9ada90a370e993ef706935f469303d").unwrap(),
                    belongs_to_blob: MaybeBlobReferenceWithId::UnreachableFromFilesystemRoot,
                },
                NodeAndBlobReference::NonRootInnerNode {
                    depth: NonZeroU8::new(4).unwrap(),
                    parent_id: BlockId::from_hex("a370e993ef706935f4693039c9ada90d").unwrap(),
                    belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                        blob_id: BlobId::from_hex("06935f4693039c90da370e99ada93ef7").unwrap(),
                        referenced_as: BlobReference {
                            blob_type: BlobType::Dir,
                            parent_id: BlobId::from_hex("3039c90da370e99ada93ef706935f469")
                                .unwrap(),
                            path: AbsolutePathBuf::try_from_string("/path/to/dir".to_string())
                                .unwrap(),
                        },
                    },
                },
                NodeAndBlobReference::RootNode {
                    belongs_to_blob: BlobReferenceWithId {
                        blob_id: BlobId::from_hex("175615c3de0525c700ceb918ca6acc2a").unwrap(),
                        referenced_as: BlobReference {
                            blob_type: BlobType::Symlink,
                            parent_id: BlobId::from_hex("393039c90da370e99adef706935f46a8")
                                .unwrap(),
                            path: AbsolutePathBuf::try_from_string("/path/to/symlink".to_string())
                                .unwrap(),
                        },
                    },
                },
            ]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[NodeReferencedMultipleTimes]: Node is referenced multiple times.
  ---> In symlink at /path/to/symlink
       Blob: id=175615c3de0525c700ceb918ca6acc2a, parent_blob=393039c90da370e99adef706935f46a8
       Node referenced as: Root node
  ---> In unreachable blob
       Node referenced as: Non-root inner node [depth=3, parent_node=9c9ada90a370e993ef706935f469303d]
  ---> In dir at /path/to/dir
       Blob: id=06935f4693039c90da370e99ada93ef7, parent_blob=3039c90da370e99ada93ef706935f469
       Node referenced as: Non-root inner node [depth=4, parent_node=a370e993ef706935f4693039c9ada90d]
  ---> In unreachable blob
       Node referenced as: Non-root leaf node [parent_node=ada90da370e993ef706935f4693039c9]
  ---> In file at /path/to/file/1
       Blob: id=35f4693039c90da370e99ada93ef7069, parent_blob=3da370e99ada93ef706935f46039c909
       Node referenced as: Non-root leaf node [parent_node=ada90da370e993ef706935f4693039c9]
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Inner node [depth=2]
"
            .trim(),
        );
    }
}
