use std::collections::BTreeSet;
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

use cryfs_blobstore::BlobId;

use crate::node_info::{BlobReference, MaybeBlobInfoAsSeenByLookingAtBlob};

#[derive(Error, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct BlobReferencedMultipleTimesError {
    pub blob_id: BlobId,
    pub blob_info: MaybeBlobInfoAsSeenByLookingAtBlob,
    pub referenced_as: BTreeSet<BlobReference>,
}

impl BlobReferencedMultipleTimesError {
    pub fn new(
        blob_id: BlobId,
        blob_info: MaybeBlobInfoAsSeenByLookingAtBlob,
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

        match self.blob_info {
            MaybeBlobInfoAsSeenByLookingAtBlob::Missing => write!(f, " and is missing.")?,
            MaybeBlobInfoAsSeenByLookingAtBlob::Unreadable => {
                write!(f, " and is unreadable and likely corrupted.")?
            }
            MaybeBlobInfoAsSeenByLookingAtBlob::Readable { .. } => {
                write!(f, " and exists as {blob_info}.", blob_info = self.blob_info)?
            }
        }
        write!(f, " It is referenced as:\n")?;

        for referenced_as in &self.referenced_as {
            write!(f, "  - {referenced_as}")?;
        }
        Ok(())
    }
}
