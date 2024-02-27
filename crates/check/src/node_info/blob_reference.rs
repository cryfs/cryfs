use std::fmt::{self, Debug, Display};

use cryfs_blobstore::BlobId;
use cryfs_cryfs::filesystem::fsblobstore::BlobType;
use cryfs_rustfs::AbsolutePathBuf;

/// Reference to a blob as seen by looking at its parent dir blob
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
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

impl Display for BlobReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let blob_type = match self.blob_type {
            BlobType::File => "File",
            BlobType::Dir => "Dir",
            BlobType::Symlink => "Symlink",
        };
        write!(
            f,
            "{blob_type}[parent={parent_id:?}] @ {path}",
            parent_id = self.parent_id,
            path = self.path,
        )
    }
}

impl Debug for BlobReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlobReference({self})")
    }
}
