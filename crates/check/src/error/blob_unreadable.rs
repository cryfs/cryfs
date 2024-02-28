use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

use cryfs_blobstore::BlobId;

use crate::node_info::BlobReference;

#[derive(Error, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct BlobUnreadableError {
    pub blob_id: BlobId,
    pub referenced_as: BlobReference,
    // TODO error:  anyhow::Error,
}

impl BlobUnreadableError {
    pub fn new(blob_id: BlobId, referenced_as: BlobReference) -> Self {
        Self {
            blob_id,
            referenced_as,
            // TODO error: anyhow::Error,
        }
    }
}

impl Display for BlobUnreadableError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Blob {blob_id} is unreadable and likely corrupted. It is referenced as {referenced_as}.",
            blob_id = self.blob_id,
            referenced_as = self.referenced_as,
        )
    }
}
