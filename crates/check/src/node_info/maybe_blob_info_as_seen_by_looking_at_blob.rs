use cryfs_blobstore::BlobId;
use cryfs_cryfs::filesystem::fsblobstore::BlobType;

use super::BlobInfoAsSeenByLookingAtBlob;

#[derive(PartialEq, Debug, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum MaybeBlobInfoAsSeenByLookingAtBlob {
    Missing,
    Unreadable,
    Readable {
        blob_type: BlobType,
        parent_pointer: BlobId,
    },
}

impl From<BlobInfoAsSeenByLookingAtBlob> for MaybeBlobInfoAsSeenByLookingAtBlob {
    fn from(blob_info: BlobInfoAsSeenByLookingAtBlob) -> Self {
        match blob_info {
            BlobInfoAsSeenByLookingAtBlob::Unreadable => Self::Unreadable,
            BlobInfoAsSeenByLookingAtBlob::Readable {
                blob_type,
                parent_pointer,
            } => Self::Readable {
                blob_type,
                parent_pointer,
            },
        }
    }
}
