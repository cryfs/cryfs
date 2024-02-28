use std::collections::BTreeSet;
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

use cryfs_blobstore::BlobId;

use crate::node_info::{BlobInfoAsSeenByLookingAtBlob, BlobReference};

#[derive(Error, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct WrongParentPointerError {
    pub blob_id: BlobId,
    pub blob_info: BlobInfoAsSeenByLookingAtBlob,
    pub referenced_as: BTreeSet<BlobReference>,
}

impl WrongParentPointerError {
    pub fn new(
        blob_id: BlobId,
        blob_info: BlobInfoAsSeenByLookingAtBlob,
        referenced_as: BTreeSet<BlobReference>,
    ) -> Self {
        assert!(
            referenced_as.len() >= 1,
            "referenced_as is {} but must be at least 1",
            referenced_as.len()
        );
        Self {
            blob_id,
            blob_info,
            referenced_as,
        }
    }
}

impl Display for WrongParentPointerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.referenced_as.len() == 1 {
            write!(
                f,
                "Blob {blob_id} has a parent pointer that does not match the blob referencing it. The blob exists as {blob_info} and it is referenced as {referenced_as}.",
                blob_id = self.blob_id,
                blob_info = self.blob_info,
                referenced_as = self.referenced_as.iter().next().unwrap(),
            )
        } else {
            write!(
                f,
                "Blob {blob_id} has a parent pointer that does not match any of the blobs referencing it. The blob exists as {blob_info} and it is referenced as:\n",
                blob_id = self.blob_id,
                blob_info = self.blob_info,
            )?;
            for referenced_as in &self.referenced_as {
                write!(f, "  - {referenced_as}")?;
            }
            Ok(())
        }
    }
}
