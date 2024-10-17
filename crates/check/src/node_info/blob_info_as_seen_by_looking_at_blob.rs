use cryfs_blobstore::BlobId;
use cryfs_filesystem::filesystem::fsblobstore::BlobType;

#[derive(PartialEq, Eq, Debug, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum BlobInfoAsSeenByLookingAtBlob {
    Unreadable,
    Readable {
        blob_type: BlobType,
        parent_pointer: BlobId,
    },
}
