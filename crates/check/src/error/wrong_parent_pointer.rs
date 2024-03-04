use cryfs_cryfs::filesystem::fsblobstore::BlobType;
use std::collections::BTreeSet;
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

use cryfs_blobstore::BlobId;

use crate::node_info::BlobReference;

#[derive(Error, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct WrongParentPointerError {
    pub blob_id: BlobId,
    pub blob_type: BlobType,
    pub parent_pointer: BlobId,
    pub referenced_as: BTreeSet<BlobReference>,
}

impl Display for WrongParentPointerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        assert!(
            self.referenced_as.len() >= 1,
            "referenced_as is {} but must be at least 1",
            self.referenced_as.len()
        );
        let blob_type = match self.blob_type {
            BlobType::File => "File",
            BlobType::Dir => "Dir",
            BlobType::Symlink => "Symlink",
        };
        if self.referenced_as.len() == 1 {
            write!(
                f,
                "Blob {blob_id} has a parent pointer that does not match the blob referencing it. The blob exists as {blob_type}, has parent pointer {parent_pointer} and it is referenced as {referenced_as}.",
                blob_id = self.blob_id,
                parent_pointer = self.parent_pointer,
                referenced_as = self.referenced_as.iter().next().unwrap(),
            )
        } else {
            write!(
                f,
                "Blob {blob_id} has a parent pointer that does not match any of the blobs referencing it. The blob exists as {blob_type}, has parent pointer {parent_pointer} and it is referenced as:\n",
                blob_id = self.blob_id,
                parent_pointer = self.parent_pointer,
            )?;
            for referenced_as in &self.referenced_as {
                write!(f, "  - {referenced_as}")?;
            }
            Ok(())
        }
    }
}
