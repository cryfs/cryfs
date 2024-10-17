use cryfs_blobstore::BlobId;
use cryfs_filesystem::filesystem::fsblobstore::BlobType;
use cryfs_rustfs::AbsolutePathBuf;

/// Reference to a blob as seen by looking at its parent dir blob
#[derive(PartialEq, Debug, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct BlobReference {
    pub blob_type: BlobType,
    pub parent_id: BlobId,
    pub path: AbsolutePathBuf,
}

impl BlobReference {
    pub fn root_dir() -> Self {
        Self {
            blob_type: BlobType::Dir,
            parent_id: BlobId::zero(),
            path: AbsolutePathBuf::root(),
        }
    }
}
