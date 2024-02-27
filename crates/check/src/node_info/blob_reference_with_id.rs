use std::fmt::{self, Debug, Display};

use cryfs_blobstore::BlobId;

use crate::node_info::BlobReference;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct BlobReferenceWithId {
    pub blob_id: BlobId,
    pub referenced_as: BlobReference,
}

impl Display for BlobReferenceWithId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{blob_id:?}:{referenced_as}",
            blob_id = self.blob_id,
            referenced_as = self.referenced_as
        )
    }
}

impl Debug for BlobReferenceWithId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlobReferenceWithId({self})")
    }
}
