use std::fmt::{self, Debug, Display};

use cryfs_blobstore::BlobId;
use cryfs_cryfs::filesystem::fsblobstore::BlobType;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum BlobInfoAsSeenByLookingAtBlob {
    Unreadable,
    Readable {
        blob_type: BlobType,
        parent_pointer: BlobId,
    },
}

impl Display for BlobInfoAsSeenByLookingAtBlob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable => write!(f, "UnreadableBlob"),
            Self::Readable {
                blob_type,
                parent_pointer,
            } => {
                let blob_type = match blob_type {
                    BlobType::File => "File",
                    BlobType::Dir => "Dir",
                    BlobType::Symlink => "Symlink",
                };
                write!(f, "{blob_type}[parent_pointer={parent_pointer:?}]",)
            }
        }
    }
}

impl Debug for BlobInfoAsSeenByLookingAtBlob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlobInfoAsSeenByLookingAtBlob({self})")
    }
}
