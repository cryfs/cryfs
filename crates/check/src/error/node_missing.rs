use std::collections::BTreeSet;
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

use cryfs_blockstore::BlockId;

use super::display::{ErrorDisplayNodeInfo, ErrorTitle, NodeErrorDisplayMessage};
use crate::MaybeNodeInfoAsSeenByLookingAtNode;
use crate::node_info::NodeAndBlobReference;

#[derive(Error, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct NodeMissingError {
    pub node_id: BlockId,
    pub referenced_as: BTreeSet<NodeAndBlobReference>,
}

impl NodeMissingError {
    pub fn new(node_id: BlockId, referenced_as: BTreeSet<NodeAndBlobReference>) -> Self {
        assert!(
            referenced_as.len() > 0,
            "NodeMissingError should only be created if the node is referenced by at least one other node or blob"
        );
        Self {
            node_id,
            referenced_as,
        }
    }
}

const ERROR_TITLE: ErrorTitle = ErrorTitle {
    error_type: "NodeMissing",
    error_message: "Node is missing.",
};

impl Display for NodeMissingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        assert!(
            self.referenced_as.len() > 0,
            "NodeMissingError should only be created if the node is referenced by at least one other node or blob"
        );

        let error_display = NodeErrorDisplayMessage {
            error_title: ERROR_TITLE,

            node_info: ErrorDisplayNodeInfo {
                node_id: self.node_id,
                node_info: MaybeNodeInfoAsSeenByLookingAtNode::Missing,
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
    use cryfs_filesystem::filesystem::fsblobstore::BlobType;
    use cryfs_rustfs::AbsolutePathBuf;

    use crate::{BlobReference, BlobReferenceWithId, MaybeBlobReferenceWithId};

    use super::*;

    #[test]
    fn test_display_unreachable_inner_node() {
        let error = NodeMissingError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            referenced_as: [NodeAndBlobReference::NonRootInnerNode {
                depth: NonZeroU8::new(4).unwrap(),
                parent_id: BlockId::from_hex("6935f4693039c90da370e99ada93ef70").unwrap(),
                belongs_to_blob: MaybeBlobReferenceWithId::UnreachableFromFilesystemRoot,
            }]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[NodeMissing]: Node is missing.
  ---> In unreachable blob
       Node referenced as: Non-root inner node [depth=4, parent_node=6935f4693039c90da370e99ada93ef70]
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Node is missing
"
            .trim(),
        );
    }

    #[test]
    fn test_display_unreachable_leaf_node() {
        let error = NodeMissingError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            referenced_as: [NodeAndBlobReference::NonRootLeafNode {
                parent_id: BlockId::from_hex("6935f4693039c90da370e99ada93ef70").unwrap(),
                belongs_to_blob: MaybeBlobReferenceWithId::UnreachableFromFilesystemRoot,
            }]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[NodeMissing]: Node is missing.
  ---> In unreachable blob
       Node referenced as: Non-root leaf node [parent_node=6935f4693039c90da370e99ada93ef70]
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Node is missing
"
            .trim(),
        );
    }

    #[test]
    fn test_display_file_root_node() {
        let error = NodeMissingError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            referenced_as: [NodeAndBlobReference::RootNode {
                belongs_to_blob: BlobReferenceWithId {
                    blob_id: BlobId::from_hex("525c7918ca6ade0cea1bc00c275615c3").unwrap(),
                    referenced_as: BlobReference {
                        blob_type: BlobType::File,
                        parent_id: BlobId::from_hex("3ef706935f4693039c90da370e99ada9").unwrap(),
                        path: AbsolutePathBuf::try_from_string("/path/to/blob".to_string())
                            .unwrap(),
                    },
                },
            }]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[NodeMissing]: Node is missing.
  ---> In file at /path/to/blob
       Blob: id=525c7918ca6ade0cea1bc00c275615c3, parent_blob=3ef706935f4693039c90da370e99ada9
       Node referenced as: Root node
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Node is missing
"
            .trim(),
        );
    }

    #[test]
    fn test_display_file_inner_node() {
        let error = NodeMissingError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            referenced_as: [NodeAndBlobReference::NonRootInnerNode {
                depth: NonZeroU8::new(4).unwrap(),
                parent_id: BlockId::from_hex("6935f4693039c90da370e99ada93ef70").unwrap(),
                belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                    blob_id: BlobId::from_hex("525c7918ca6ade0cea1bc00c275615c3").unwrap(),
                    referenced_as: BlobReference {
                        blob_type: BlobType::File,
                        parent_id: BlobId::from_hex("3ef706935f4693039c90da370e99ada9").unwrap(),
                        path: AbsolutePathBuf::try_from_string("/path/to/blob".to_string())
                            .unwrap(),
                    },
                },
            }]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[NodeMissing]: Node is missing.
  ---> In file at /path/to/blob
       Blob: id=525c7918ca6ade0cea1bc00c275615c3, parent_blob=3ef706935f4693039c90da370e99ada9
       Node referenced as: Non-root inner node [depth=4, parent_node=6935f4693039c90da370e99ada93ef70]
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Node is missing
"
            .trim(),
        );
    }

    #[test]
    fn test_display_file_leaf_node() {
        let error = NodeMissingError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            referenced_as: [NodeAndBlobReference::NonRootLeafNode {
                parent_id: BlockId::from_hex("6935f4693039c90da370e99ada93ef70").unwrap(),
                belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                    blob_id: BlobId::from_hex("525c7918ca6ade0cea1bc00c275615c3").unwrap(),
                    referenced_as: BlobReference {
                        blob_type: BlobType::File,
                        parent_id: BlobId::from_hex("3ef706935f4693039c90da370e99ada9").unwrap(),
                        path: AbsolutePathBuf::try_from_string("/path/to/blob".to_string())
                            .unwrap(),
                    },
                },
            }]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[NodeMissing]: Node is missing.
  ---> In file at /path/to/blob
       Blob: id=525c7918ca6ade0cea1bc00c275615c3, parent_blob=3ef706935f4693039c90da370e99ada9
       Node referenced as: Non-root leaf node [parent_node=6935f4693039c90da370e99ada93ef70]
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Node is missing
"
            .trim(),
        );
    }

    #[test]
    fn test_display_dir_root_node() {
        let error = NodeMissingError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            referenced_as: [NodeAndBlobReference::RootNode {
                belongs_to_blob: BlobReferenceWithId {
                    blob_id: BlobId::from_hex("525c7918ca6ade0cea1bc00c275615c3").unwrap(),
                    referenced_as: BlobReference {
                        blob_type: BlobType::Dir,
                        parent_id: BlobId::from_hex("3ef706935f4693039c90da370e99ada9").unwrap(),
                        path: AbsolutePathBuf::try_from_string("/path/to/blob".to_string())
                            .unwrap(),
                    },
                },
            }]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[NodeMissing]: Node is missing.
  ---> In dir at /path/to/blob
       Blob: id=525c7918ca6ade0cea1bc00c275615c3, parent_blob=3ef706935f4693039c90da370e99ada9
       Node referenced as: Root node
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Node is missing
"
            .trim(),
        );
    }

    #[test]
    fn test_display_dir_inner_node() {
        let error = NodeMissingError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            referenced_as: [NodeAndBlobReference::NonRootInnerNode {
                depth: NonZeroU8::new(4).unwrap(),
                parent_id: BlockId::from_hex("6935f4693039c90da370e99ada93ef70").unwrap(),
                belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                    blob_id: BlobId::from_hex("525c7918ca6ade0cea1bc00c275615c3").unwrap(),
                    referenced_as: BlobReference {
                        blob_type: BlobType::Dir,
                        parent_id: BlobId::from_hex("3ef706935f4693039c90da370e99ada9").unwrap(),
                        path: AbsolutePathBuf::try_from_string("/path/to/blob".to_string())
                            .unwrap(),
                    },
                },
            }]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[NodeMissing]: Node is missing.
  ---> In dir at /path/to/blob
       Blob: id=525c7918ca6ade0cea1bc00c275615c3, parent_blob=3ef706935f4693039c90da370e99ada9
       Node referenced as: Non-root inner node [depth=4, parent_node=6935f4693039c90da370e99ada93ef70]
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Node is missing
"
            .trim(),
        );
    }

    #[test]
    fn test_display_dir_leaf_node() {
        let error = NodeMissingError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            referenced_as: [NodeAndBlobReference::NonRootLeafNode {
                parent_id: BlockId::from_hex("6935f4693039c90da370e99ada93ef70").unwrap(),
                belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                    blob_id: BlobId::from_hex("525c7918ca6ade0cea1bc00c275615c3").unwrap(),
                    referenced_as: BlobReference {
                        blob_type: BlobType::Dir,
                        parent_id: BlobId::from_hex("3ef706935f4693039c90da370e99ada9").unwrap(),
                        path: AbsolutePathBuf::try_from_string("/path/to/blob".to_string())
                            .unwrap(),
                    },
                },
            }]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[NodeMissing]: Node is missing.
  ---> In dir at /path/to/blob
       Blob: id=525c7918ca6ade0cea1bc00c275615c3, parent_blob=3ef706935f4693039c90da370e99ada9
       Node referenced as: Non-root leaf node [parent_node=6935f4693039c90da370e99ada93ef70]
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Node is missing
"
            .trim(),
        );
    }

    #[test]
    fn test_display_symlink_root_node() {
        let error = NodeMissingError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            referenced_as: [NodeAndBlobReference::RootNode {
                belongs_to_blob: BlobReferenceWithId {
                    blob_id: BlobId::from_hex("525c7918ca6ade0cea1bc00c275615c3").unwrap(),
                    referenced_as: BlobReference {
                        blob_type: BlobType::Symlink,
                        parent_id: BlobId::from_hex("3ef706935f4693039c90da370e99ada9").unwrap(),
                        path: AbsolutePathBuf::try_from_string("/path/to/blob".to_string())
                            .unwrap(),
                    },
                },
            }]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[NodeMissing]: Node is missing.
  ---> In symlink at /path/to/blob
       Blob: id=525c7918ca6ade0cea1bc00c275615c3, parent_blob=3ef706935f4693039c90da370e99ada9
       Node referenced as: Root node
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Node is missing
"
            .trim(),
        );
    }

    #[test]
    fn test_display_symlink_inner_node() {
        let error = NodeMissingError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            referenced_as: [NodeAndBlobReference::NonRootInnerNode {
                depth: NonZeroU8::new(4).unwrap(),
                parent_id: BlockId::from_hex("6935f4693039c90da370e99ada93ef70").unwrap(),
                belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                    blob_id: BlobId::from_hex("525c7918ca6ade0cea1bc00c275615c3").unwrap(),
                    referenced_as: BlobReference {
                        blob_type: BlobType::Symlink,
                        parent_id: BlobId::from_hex("3ef706935f4693039c90da370e99ada9").unwrap(),
                        path: AbsolutePathBuf::try_from_string("/path/to/blob".to_string())
                            .unwrap(),
                    },
                },
            }]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[NodeMissing]: Node is missing.
  ---> In symlink at /path/to/blob
       Blob: id=525c7918ca6ade0cea1bc00c275615c3, parent_blob=3ef706935f4693039c90da370e99ada9
       Node referenced as: Non-root inner node [depth=4, parent_node=6935f4693039c90da370e99ada93ef70]
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Node is missing
"
            .trim(),
        );
    }

    #[test]
    fn test_display_symlink_leaf_node() {
        let error = NodeMissingError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            referenced_as: [NodeAndBlobReference::NonRootLeafNode {
                parent_id: BlockId::from_hex("6935f4693039c90da370e99ada93ef70").unwrap(),
                belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                    blob_id: BlobId::from_hex("525c7918ca6ade0cea1bc00c275615c3").unwrap(),
                    referenced_as: BlobReference {
                        blob_type: BlobType::Symlink,
                        parent_id: BlobId::from_hex("3ef706935f4693039c90da370e99ada9").unwrap(),
                        path: AbsolutePathBuf::try_from_string("/path/to/blob".to_string())
                            .unwrap(),
                    },
                },
            }]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[NodeMissing]: Node is missing.
  ---> In symlink at /path/to/blob
       Blob: id=525c7918ca6ade0cea1bc00c275615c3, parent_blob=3ef706935f4693039c90da370e99ada9
       Node referenced as: Non-root leaf node [parent_node=6935f4693039c90da370e99ada93ef70]
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Node is missing
"
            .trim(),
        );
    }

    #[test]
    fn test_display_referenced_multiple_times() {
        let error = NodeMissingError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            referenced_as: [
                NodeAndBlobReference::RootNode {
                    belongs_to_blob: BlobReferenceWithId {
                        blob_id: BlobId::from_hex("a6ade0cea1bc00c275615c3525c7918c").unwrap(),
                        referenced_as: BlobReference {
                            blob_type: BlobType::Symlink,
                            parent_id: BlobId::from_hex("693039c90da370e99ada93ef706935f4")
                                .unwrap(),
                            path: AbsolutePathBuf::try_from_string("/path/to/symlink".to_string())
                                .unwrap(),
                        },
                    },
                },
                NodeAndBlobReference::NonRootInnerNode {
                    depth: NonZeroU8::new(4).unwrap(),
                    parent_id: BlockId::from_hex("6935f4693039c90da370e99ada93ef70").unwrap(),
                    belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                        blob_id: BlobId::from_hex("525c7918ca6ade0cea1bc00c275615c3").unwrap(),
                        referenced_as: BlobReference {
                            blob_type: BlobType::File,
                            parent_id: BlobId::from_hex("3ef706935f4693039c90da370e99ada9")
                                .unwrap(),
                            path: AbsolutePathBuf::try_from_string("/path/to/file".to_string())
                                .unwrap(),
                        },
                    },
                },
                NodeAndBlobReference::NonRootLeafNode {
                    parent_id: BlockId::from_hex("9ada93ef706935f4693039c90da370e9").unwrap(),
                    belongs_to_blob: MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                        blob_id: BlobId::from_hex("0c275615c3525c7918ca6ade0cea1bc0").unwrap(),
                        referenced_as: BlobReference {
                            blob_type: BlobType::Dir,
                            parent_id: BlobId::from_hex("0da370e99ada93ef706935f4693039c9")
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
Error[NodeMissing]: Node is missing.
  ---> In symlink at /path/to/symlink
       Blob: id=a6ade0cea1bc00c275615c3525c7918c, parent_blob=693039c90da370e99ada93ef706935f4
       Node referenced as: Root node
  ---> In file at /path/to/file
       Blob: id=525c7918ca6ade0cea1bc00c275615c3, parent_blob=3ef706935f4693039c90da370e99ada9
       Node referenced as: Non-root inner node [depth=4, parent_node=6935f4693039c90da370e99ada93ef70]
  ---> In dir at /path/to/dir
       Blob: id=0c275615c3525c7918ca6ade0cea1bc0, parent_blob=0da370e99ada93ef706935f4693039c9
       Node referenced as: Non-root leaf node [parent_node=9ada93ef706935f4693039c90da370e9]
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Node is missing
"
            .trim(),
        );
    }
}
