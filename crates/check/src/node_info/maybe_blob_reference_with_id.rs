use cryfs_blobstore::BlobId;

use crate::{BlobReferenceWithId, node_info::BlobReference};

#[derive(PartialEq, Debug, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum MaybeBlobReferenceWithId {
    UnreachableFromFilesystemRoot,
    ReachableFromFilesystemRoot {
        blob_id: BlobId,
        referenced_as: BlobReference,
    },
}

impl From<BlobReferenceWithId> for MaybeBlobReferenceWithId {
    fn from(blob_reference_with_id: BlobReferenceWithId) -> Self {
        Self::ReachableFromFilesystemRoot {
            blob_id: blob_reference_with_id.blob_id,
            referenced_as: blob_reference_with_id.referenced_as,
        }
    }
}
