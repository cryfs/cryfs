use std::fmt::{self, Debug, Display};

use cryfs_blobstore::BlobId;
use cryfs_cryfs::filesystem::fsblobstore::BlobType;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
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
                write!(f, "{blob_type}[parent={parent_pointer}]",)
            }
        }
    }
}

impl Debug for BlobInfoAsSeenByLookingAtBlob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlobInfoAsSeenByLookingAtBlob({self})")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_display() {
        let parent_pointer = BlobId::from_hex("A370E99ADA93EF706935F4693039C90D").unwrap();

        assert_eq!(
            "UnreadableBlob",
            format!("{}", BlobInfoAsSeenByLookingAtBlob::Unreadable),
        );

        assert_eq!(
            "File[parent=A370E99ADA93EF706935F4693039C90D]",
            format!(
                "{}",
                BlobInfoAsSeenByLookingAtBlob::Readable {
                    blob_type: BlobType::File,
                    parent_pointer,
                }
            ),
        );

        assert_eq!(
            "Dir[parent=A370E99ADA93EF706935F4693039C90D]",
            format!(
                "{}",
                BlobInfoAsSeenByLookingAtBlob::Readable {
                    blob_type: BlobType::Dir,
                    parent_pointer,
                }
            ),
        );

        assert_eq!(
            "Symlink[parent=A370E99ADA93EF706935F4693039C90D]",
            format!(
                "{}",
                BlobInfoAsSeenByLookingAtBlob::Readable {
                    blob_type: BlobType::Symlink,
                    parent_pointer,
                }
            ),
        );
    }
}
