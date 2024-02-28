use std::collections::BTreeSet;
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

use cryfs_blobstore::BlobId;

use crate::node_info::{BlobInfoAsSeenByLookingAtBlob, BlobReference};

#[derive(Error, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct BlobReferencedMultipleTimesError {
    pub blob_id: BlobId,
    /// `blob_info` is `None` if the blob itself is missing
    pub blob_info: Option<BlobInfoAsSeenByLookingAtBlob>,
    pub referenced_as: BTreeSet<BlobReference>,
}

impl BlobReferencedMultipleTimesError {
    pub fn new(
        blob_id: BlobId,
        blob_info: Option<BlobInfoAsSeenByLookingAtBlob>,
        referenced_as: BTreeSet<BlobReference>,
    ) -> Self {
        assert!(
            referenced_as.len() >= 2,
            "referenced_as is {} but must be at least 2",
            referenced_as.len()
        );
        Self {
            blob_id,
            blob_info,
            referenced_as,
        }
    }
}

impl Display for BlobReferencedMultipleTimesError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Blob {blob_id} is referenced multiple times",
            blob_id = self.blob_id,
        )?;
        if let Some(blob_info) = self.blob_info {
            write!(f, " and exists as {blob_info}.")?;
        } else {
            write!(f, " and is missing.")?;
        }
        write!(f, " It is referenced as:\n")?;

        for referenced_as in &self.referenced_as {
            write!(f, "  - {referenced_as}")?;
        }
        Ok(())
    }
}
