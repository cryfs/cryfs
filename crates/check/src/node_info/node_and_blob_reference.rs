use std::num::NonZeroU8;

use cryfs_blockstore::BlockId;

use super::{
    BlobReferenceWithId, MaybeBlobReferenceWithId, NodeAndBlobReferenceFromReachableBlob,
    NodeReference,
};

#[derive(PartialEq, Debug, Eq, PartialOrd, Ord, Hash, Clone)]
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
    use cryfs_fsblobstore::fsblobstore::BlobType;
    use cryfs_utils::path::AbsolutePathBuf;

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
            blob_id: BlobId::from_hex("3ef706935f4693039c90da370e99ada9").unwrap(),
            referenced_as: BlobReference {
                blob_type: BlobType::File,
                parent_id: BlobId::from_hex("a370e99ada93ef706935f4693039c90d").unwrap(),
                path: AbsolutePathBuf::try_from_string("/path/to/blob".to_string()).unwrap(),
            },
        };

        _test_node_info_and_blob_info(NodeReference::RootNode, blob_info.clone());
        _test_node_info_and_blob_info(
            NodeReference::NonRootInnerNode {
                depth: NonZeroU8::new(3).unwrap(),
                parent_id: BlockId::from_hex("da93ef706935f4693039c90da370e99a").unwrap(),
            },
            blob_info.clone(),
        );
        _test_node_info_and_blob_info(
            NodeReference::NonRootLeafNode {
                parent_id: BlockId::from_hex("da93ef706935f4693039c90da370e99a").unwrap(),
            },
            blob_info,
        );
    }
}
