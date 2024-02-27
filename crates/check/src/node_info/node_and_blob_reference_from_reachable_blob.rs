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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::BlobReference;
    use cryfs_blobstore::BlobId;
    use cryfs_blockstore::BlockId;
    use cryfs_cryfs::filesystem::fsblobstore::BlobType;
    use cryfs_rustfs::AbsolutePathBuf;
    use std::num::NonZeroU8;

    #[test]
    fn test_display() {
        let blob_info = BlobReferenceWithId {
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
                NodeAndBlobReferenceFromReachableBlob {
                    node_info: NodeReference::RootNode,
                    blob_info: blob_info.clone(),
                }
            ),
        );

        assert_eq!(
            "File[path=/path/to/blob, id=3EF706935F4693039C90DA370E99ADA9, parent=A370E99ADA93EF706935F4693039C90D]:NonRootInnerNode[depth=3, parent=DA93EF706935F4693039C90DA370E99A]",
            format!(
                "{}",
                NodeAndBlobReferenceFromReachableBlob{
                    node_info: NodeReference::NonRootInnerNode {
                        depth: NonZeroU8::new(3).unwrap(),
                        parent_id: BlockId::from_hex("DA93EF706935F4693039C90DA370E99A").unwrap(),
                    },
                    blob_info: blob_info.clone(),
                }
            ),
        );

        assert_eq!(
            "File[path=/path/to/blob, id=3EF706935F4693039C90DA370E99ADA9, parent=A370E99ADA93EF706935F4693039C90D]:NonRootLeafNode[parent=DA93EF706935F4693039C90DA370E99A]",
            format!(
                "{}",
                NodeAndBlobReferenceFromReachableBlob {
                    node_info: NodeReference::NonRootLeafNode {
                        parent_id: BlockId::from_hex("DA93EF706935F4693039C90DA370E99A").unwrap(),
                    },
                    blob_info: blob_info.clone(),
                }
            ),
        );
    }
}
