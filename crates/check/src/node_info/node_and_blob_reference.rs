use std::fmt::{self, Debug, Display};
use std::num::NonZeroU8;

use cryfs_blockstore::BlockId;
use cryfs_cryfs::filesystem::fsblobstore::BlobType;

use super::{
    BlobReferenceWithId, MaybeBlobReferenceWithId, NodeAndBlobReferenceFromReachableBlob,
    NodeReference,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum NodeAndBlobReference {
    RootNode {
        belongs_to_blob: BlobReferenceWithId,
    },
    NonRootInnerNode {
        belongs_to_blob: MaybeBlobReferenceWithId,

        depth: NonZeroU8,
        parent_id: BlockId,
    },
    NonRootLeafNode {
        belongs_to_blob: MaybeBlobReferenceWithId,

        parent_id: BlockId,
    },
}

impl NodeAndBlobReference {
    pub fn blob_info(self) -> MaybeBlobReferenceWithId {
        match self {
            Self::RootNode { belongs_to_blob } => belongs_to_blob.into(),
            Self::NonRootInnerNode {
                belongs_to_blob, ..
            } => belongs_to_blob,
            Self::NonRootLeafNode {
                belongs_to_blob, ..
            } => belongs_to_blob,
        }
    }

    pub fn node_info(&self) -> NodeReference {
        match self {
            Self::RootNode { .. } => NodeReference::RootNode,
            Self::NonRootInnerNode {
                depth, parent_id, ..
            } => NodeReference::NonRootInnerNode {
                depth: *depth,
                parent_id: *parent_id,
            },
            Self::NonRootLeafNode { parent_id, .. } => NodeReference::NonRootLeafNode {
                parent_id: *parent_id,
            },
        }
    }
}

impl Display for NodeAndBlobReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let belongs_to_blob = self.clone().blob_info();

        match belongs_to_blob {
            MaybeBlobReferenceWithId::UnreachableFromFilesystemRoot => {
                write!(f, "UnreachableBlob:")?
            }
            MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                blob_id,
                referenced_as,
            } => {
                let blob_type = match referenced_as.blob_type {
                    BlobType::File => "File",
                    BlobType::Dir => "Dir",
                    BlobType::Symlink => "Symlink",
                };
                write!(
                    f,
                    "{blob_type}[path={path}, id={blob_id}, parent={parent_id}]:",
                    blob_id = blob_id,
                    parent_id = referenced_as.parent_id,
                    path = referenced_as.path,
                    blob_type = blob_type,
                )?;
            }
        }

        write!(f, "{}", self.node_info())
    }
}

impl Debug for NodeAndBlobReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeAndBlobReference({self})")
    }
}

impl From<NodeAndBlobReferenceFromReachableBlob> for NodeAndBlobReference {
    fn from(node_reference: NodeAndBlobReferenceFromReachableBlob) -> Self {
        match node_reference.node_info {
            NodeReference::RootNode => Self::RootNode {
                belongs_to_blob: node_reference.blob_info,
            },
            NodeReference::NonRootInnerNode { depth, parent_id } => Self::NonRootInnerNode {
                belongs_to_blob: node_reference.blob_info.into(),
                depth,
                parent_id,
            },
            NodeReference::NonRootLeafNode { parent_id } => Self::NonRootLeafNode {
                belongs_to_blob: node_reference.blob_info.into(),
                parent_id,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::BlobReference;
    use cryfs_blobstore::BlobId;
    use cryfs_blockstore::BlockId;
    use cryfs_cryfs::filesystem::fsblobstore::BlobType;
    use cryfs_rustfs::AbsolutePathBuf;

    #[test]
    fn test_display() {
        let belongs_to_blob = BlobReferenceWithId {
            blob_id: BlobId::from_hex("3EF706935F4693039C90DA370E99ADA9").unwrap(),
            referenced_as: BlobReference {
                blob_type: BlobType::File,
                parent_id: BlobId::from_hex("A370E99ADA93EF706935F4693039C90D").unwrap(),
                path: AbsolutePathBuf::try_from_string("/path/to/blob".to_string()).unwrap(),
            },
        };

        assert_eq!(
            "File[path=/path/to/blob, id=3EF706935F4693039C90DA370E99ADA9, parent=A370E99ADA93EF706935F4693039C90D]:RootNode",
            format!(
                "{}",
                NodeAndBlobReference::RootNode {
                    belongs_to_blob: belongs_to_blob.clone(),
                }
            ),
        );

        assert_eq!(
            "UnreachableBlob:NonRootInnerNode[depth=3, parent=DA93EF706935F4693039C90DA370E99A]",
            format!(
                "{}",
                NodeAndBlobReference::NonRootInnerNode {
                    belongs_to_blob: MaybeBlobReferenceWithId::UnreachableFromFilesystemRoot,
                    depth: NonZeroU8::new(3).unwrap(),
                    parent_id: BlockId::from_hex("DA93EF706935F4693039C90DA370E99A").unwrap(),
                }
            ),
        );

        assert_eq!(
            "File[path=/path/to/blob, id=3EF706935F4693039C90DA370E99ADA9, parent=A370E99ADA93EF706935F4693039C90D]:NonRootInnerNode[depth=3, parent=DA93EF706935F4693039C90DA370E99A]",
            format!(
                "{}",
                NodeAndBlobReference::NonRootInnerNode {
                    belongs_to_blob: belongs_to_blob.clone().into(),
                    depth: NonZeroU8::new(3).unwrap(),
                    parent_id: BlockId::from_hex("DA93EF706935F4693039C90DA370E99A").unwrap(),
                }
            ),
        );

        assert_eq!(
            "UnreachableBlob:NonRootLeafNode[parent=DA93EF706935F4693039C90DA370E99A]",
            format!(
                "{}",
                NodeAndBlobReference::NonRootLeafNode {
                    belongs_to_blob: MaybeBlobReferenceWithId::UnreachableFromFilesystemRoot,
                    parent_id: BlockId::from_hex("DA93EF706935F4693039C90DA370E99A").unwrap(),
                }
            ),
        );

        assert_eq!(
            "File[path=/path/to/blob, id=3EF706935F4693039C90DA370E99ADA9, parent=A370E99ADA93EF706935F4693039C90D]:NonRootLeafNode[parent=DA93EF706935F4693039C90DA370E99A]",
            format!(
                "{}",
                NodeAndBlobReference::NonRootLeafNode {
                    belongs_to_blob: belongs_to_blob.clone().into(),
                    parent_id: BlockId::from_hex("DA93EF706935F4693039C90DA370E99A").unwrap(),
                }
            ),
        );
    }

    fn _test_node_info_and_blob_info(node_info: NodeReference, blob_info: BlobReferenceWithId) {
        let converted = NodeAndBlobReference::from(NodeAndBlobReferenceFromReachableBlob {
            node_info,
            blob_info: blob_info.clone(),
        });
        assert_eq!(node_info, converted.node_info());
        assert_eq!(
            MaybeBlobReferenceWithId::from(blob_info),
            converted.blob_info(),
        );
    }

    #[test]
    fn test_node_info_and_blob_info() {
        let blob_info = BlobReferenceWithId {
            blob_id: BlobId::from_hex("3EF706935F4693039C90DA370E99ADA9").unwrap(),
            referenced_as: BlobReference {
                blob_type: BlobType::File,
                parent_id: BlobId::from_hex("A370E99ADA93EF706935F4693039C90D").unwrap(),
                path: AbsolutePathBuf::try_from_string("/path/to/blob".to_string()).unwrap(),
            },
        };

        _test_node_info_and_blob_info(NodeReference::RootNode, blob_info.clone());
        _test_node_info_and_blob_info(
            NodeReference::NonRootInnerNode {
                depth: NonZeroU8::new(3).unwrap(),
                parent_id: BlockId::from_hex("DA93EF706935F4693039C90DA370E99A").unwrap(),
            },
            blob_info.clone(),
        );
        _test_node_info_and_blob_info(
            NodeReference::NonRootLeafNode {
                parent_id: BlockId::from_hex("DA93EF706935F4693039C90DA370E99A").unwrap(),
            },
            blob_info,
        );
    }
}
