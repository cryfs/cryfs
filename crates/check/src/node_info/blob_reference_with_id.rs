use std::fmt::{self, Debug, Display};

use cryfs_blobstore::BlobId;
use cryfs_filesystem::filesystem::fsblobstore::BlobType;

use crate::node_info::BlobReference;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct BlobReferenceWithId {
    pub blob_id: BlobId,
    pub referenced_as: BlobReference,
}

impl Display for BlobReferenceWithId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let blob_type = match self.referenced_as.blob_type {
            BlobType::File => "File",
            BlobType::Dir => "Dir",
            BlobType::Symlink => "Symlink",
        };
        write!(
            f,
            "{blob_type}(path={path}, blob_id={blob_id}, parent_blob={parent_id})",
            path = self.referenced_as.path,
            blob_id = self.blob_id,
            parent_id = self.referenced_as.parent_id,
        )
    }
}

impl Debug for BlobReferenceWithId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlobReferenceWithId({self})")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use console::strip_ansi_codes;
    use cryfs_rustfs::AbsolutePathBuf;

    #[test]
    fn test_display() {
        let blob_id = BlobId::from_hex("3ef706935f4693039c90da370e99ada9").unwrap();
        let parent_id = BlobId::from_hex("a370e99ada93ef706935f4693039c90d").unwrap();
        let path = AbsolutePathBuf::try_from_string("/path/to/blob".to_string()).unwrap();

        assert_eq!(
            "File(path=/path/to/blob, blob_id=3ef706935f4693039c90da370e99ada9, parent_blob=a370e99ada93ef706935f4693039c90d)",
            strip_ansi_codes(&format!(
                "{}",
                BlobReferenceWithId {
                    blob_id,
                    referenced_as: BlobReference {
                        blob_type: BlobType::File,
                        parent_id,
                        path: path.clone(),
                    }
                }
            )),
        );

        assert_eq!(
            "Dir(path=/path/to/blob, blob_id=3ef706935f4693039c90da370e99ada9, parent_blob=a370e99ada93ef706935f4693039c90d)",
            strip_ansi_codes(&format!(
                "{}",
                BlobReferenceWithId {
                    blob_id,
                    referenced_as: BlobReference {
                        blob_type: BlobType::Dir,
                        parent_id,
                        path: path.clone(),
                    }
                }
            )),
        );

        assert_eq!(
            "Symlink(path=/path/to/blob, blob_id=3ef706935f4693039c90da370e99ada9, parent_blob=a370e99ada93ef706935f4693039c90d)",
            strip_ansi_codes(&format!(
                "{}",
                BlobReferenceWithId {
                    blob_id,
                    referenced_as: BlobReference {
                        blob_type: BlobType::Symlink,
                        parent_id,
                        path: path.clone(),
                    }
                }
            )),
        );
    }
}
