use std::collections::BTreeSet;
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

use cryfs_blockstore::BlockId;

use super::display::{ErrorDisplayNodeInfo, ErrorTitle, NodeErrorDisplayMessage};
use crate::node_info::NodeAndBlobReference;
use crate::MaybeNodeInfoAsSeenByLookingAtNode;

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

const ERROR_TITLE: ErrorTitle = ErrorTitle {
    error_type: "NodeUnreadable",
    error_message: "Node is unreadable and likely corrupted.",
};

impl Display for NodeUnreadableError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let error_display = NodeErrorDisplayMessage {
            error_title: ERROR_TITLE,

            node_info: ErrorDisplayNodeInfo {
                node_id: self.node_id,
                node_info: MaybeNodeInfoAsSeenByLookingAtNode::Unreadable,
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
    fn test_display_unreferenced() {
        let error = NodeUnreadableError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            referenced_as: [].into_iter().collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[NodeUnreadable]: Node is unreadable and likely corrupted.
  ---> No references to node found
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Node is unreadable
"
            .trim(),
        );
    }

    #[test]
    fn test_display_referenced_as_root_node() {
        let error = NodeUnreadableError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
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
Error[NodeUnreadable]: Node is unreadable and likely corrupted.
  ---> In file at /path/to/file
       Blob: id=1b918ca6acc275615c3de0525c700cea, parent_blob=3ef706935f4693039c90da370e99ada9
       Node referenced as: Root node
  ---> In dir at /path/to/dir
       Blob: id=06935f4693039c90da370e99ada93ef7, parent_blob=3039c90da370e99ada93ef706935f469
       Node referenced as: Non-root inner node [depth=4, parent_node=a370e993ef706935f4693039c9ada90d]
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Node is unreadable
"
            .trim(),
        );
    }

    #[test]
    fn test_display_referenced_as_inner_node_reachable() {
        let error = NodeUnreadableError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
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
Error[NodeUnreadable]: Node is unreadable and likely corrupted.
  ---> In dir at /path/to/dir
       Blob: id=06935f4693039c90da370e99ada93ef7, parent_blob=3039c90da370e99ada93ef706935f469
       Node referenced as: Non-root inner node [depth=4, parent_node=a370e993ef706935f4693039c9ada90d]
  ---> In file at /path/to/file
       Blob: id=1b918ca6acc275615c3de0525c700cea, parent_blob=3ef706935f4693039c90da370e99ada9
       Node referenced as: Non-root inner node [depth=5, parent_node=ada90da370e993ef706935f4693039c9]
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Node is unreadable
"
            .trim(),
        );
    }

    #[test]
    fn test_display_referenced_as_inner_node_unreachable() {
        let error = NodeUnreadableError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
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
Error[NodeUnreadable]: Node is unreadable and likely corrupted.
  ---> In unreachable blob
       Node referenced as: Non-root inner node [depth=5, parent_node=ada90da370e993ef706935f4693039c9]
  ---> In dir at /path/to/dir
       Blob: id=06935f4693039c90da370e99ada93ef7, parent_blob=3039c90da370e99ada93ef706935f469
       Node referenced as: Non-root inner node [depth=4, parent_node=a370e993ef706935f4693039c9ada90d]
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Node is unreadable
"
            .trim(),
        );
    }

    #[test]
    fn test_display_referenced_as_leaf_node_reachable() {
        let error = NodeUnreadableError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
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
Error[NodeUnreadable]: Node is unreadable and likely corrupted.
  ---> In dir at /path/to/dir
       Blob: id=06935f4693039c90da370e99ada93ef7, parent_blob=3039c90da370e99ada93ef706935f469
       Node referenced as: Non-root inner node [depth=4, parent_node=a370e993ef706935f4693039c9ada90d]
  ---> In file at /path/to/file
       Blob: id=1b918ca6acc275615c3de0525c700cea, parent_blob=3ef706935f4693039c90da370e99ada9
       Node referenced as: Non-root leaf node [parent_node=ada90da370e993ef706935f4693039c9]
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Node is unreadable
"
            .trim(),
        );
    }

    #[test]
    fn test_display_referenced_as_leaf_node_unreachable() {
        let error = NodeUnreadableError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
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
Error[NodeUnreadable]: Node is unreadable and likely corrupted.
  ---> In dir at /path/to/dir
       Blob: id=06935f4693039c90da370e99ada93ef7, parent_blob=3039c90da370e99ada93ef706935f469
       Node referenced as: Non-root inner node [depth=4, parent_node=a370e993ef706935f4693039c9ada90d]
  ---> In unreachable blob
       Node referenced as: Non-root leaf node [parent_node=ada90da370e993ef706935f4693039c9]
  Node Id: 918ca6ac525c700c275615c3de0cea1b
  Node Info: Node is unreadable
"
            .trim(),
        );
    }

    #[test]
    fn test_display_many_references() {
        let error = NodeUnreadableError {
            node_id: BlockId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
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
Error[NodeUnreadable]: Node is unreadable and likely corrupted.
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
  Node Info: Node is unreadable
"
            .trim(),
        );
    }
}
