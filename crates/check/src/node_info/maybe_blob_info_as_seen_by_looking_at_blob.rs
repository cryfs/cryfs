use std::fmt::{self, Debug, Display};

use cryfs_blobstore::BlobId;
use cryfs_cryfs::filesystem::fsblobstore::BlobType;

use super::BlobInfoAsSeenByLookingAtBlob;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum MaybeBlobInfoAsSeenByLookingAtBlob {
    Missing,
    Unreadable,
    Readable {
        blob_type: BlobType,
        parent_pointer: BlobId,
    },
}

impl Display for MaybeBlobInfoAsSeenByLookingAtBlob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => write!(f, "MissingBlob"),
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
                write!(f, "{blob_type}[parent={parent_pointer}]",)
            }
        }
    }
}

impl Debug for MaybeBlobInfoAsSeenByLookingAtBlob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MaybeBlobInfoAsSeenByLookingAtBlob({self})")
    }
}

impl From<BlobInfoAsSeenByLookingAtBlob> for MaybeBlobInfoAsSeenByLookingAtBlob {
    fn from(blob_info: BlobInfoAsSeenByLookingAtBlob) -> Self {
        match blob_info {
            BlobInfoAsSeenByLookingAtBlob::Unreadable => Self::Unreadable,
            BlobInfoAsSeenByLookingAtBlob::Readable {
                blob_type,
                parent_pointer,
            } => Self::Readable {
                blob_type,
                parent_pointer,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_display() {
        let parent_pointer = BlobId::from_hex("A370E99ADA93EF706935F4693039C90D").unwrap();

        assert_eq!(
            "MissingBlob",
            format!("{}", MaybeBlobInfoAsSeenByLookingAtBlob::Missing),
        );

        assert_eq!(
            "UnreadableBlob",
            format!("{}", MaybeBlobInfoAsSeenByLookingAtBlob::Unreadable),
        );

        assert_eq!(
            "File[parent=A370E99ADA93EF706935F4693039C90D]",
            format!(
                "{}",
                MaybeBlobInfoAsSeenByLookingAtBlob::Readable {
                    blob_type: BlobType::File,
                    parent_pointer,
                }
            ),
        );

        assert_eq!(
            "Dir[parent=A370E99ADA93EF706935F4693039C90D]",
            format!(
                "{}",
                MaybeBlobInfoAsSeenByLookingAtBlob::Readable {
                    blob_type: BlobType::Dir,
                    parent_pointer,
                }
            ),
        );

        assert_eq!(
            "Symlink[parent=A370E99ADA93EF706935F4693039C90D]",
            format!(
                "{}",
                MaybeBlobInfoAsSeenByLookingAtBlob::Readable {
                    blob_type: BlobType::Symlink,
                    parent_pointer,
                }
            ),
        );
    }
}
