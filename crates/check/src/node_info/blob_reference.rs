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
            "{blob_type}[path={path}, parent={parent_id}]",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        let parent_id = BlobId::from_hex("A370E99ADA93EF706935F4693039C90D").unwrap();
        let path = AbsolutePathBuf::try_from_string("/path/to/blob".to_string()).unwrap();

        assert_eq!(
            "File[path=/path/to/blob, parent=A370E99ADA93EF706935F4693039C90D]",
            format!(
                "{}",
                BlobReference {
                    blob_type: BlobType::File,
                    parent_id,
                    path: path.clone(),
                }
            ),
        );

        assert_eq!(
            "Dir[path=/path/to/blob, parent=A370E99ADA93EF706935F4693039C90D]",
            format!(
                "{}",
                BlobReference {
                    blob_type: BlobType::Dir,
                    parent_id,
                    path: path.clone(),
                }
            ),
        );

        assert_eq!(
            "Symlink[path=/path/to/blob, parent=A370E99ADA93EF706935F4693039C90D]",
            format!(
                "{}",
                BlobReference {
                    blob_type: BlobType::Symlink,
                    parent_id,
                    path: path.clone(),
                }
            ),
        );
    }
}
