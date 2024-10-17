use std::collections::BTreeSet;
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

use cryfs_blobstore::BlobId;

use super::display::{BlobErrorDisplayMessage, ErrorDisplayBlobInfo, ErrorTitle};
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

const ERROR_TITLE: ErrorTitle = ErrorTitle {
    error_type: "BlobReferencedMultipleTimes",
    error_message: "Blob is referenced multiple times.",
};

impl Display for BlobReferencedMultipleTimesError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        assert!(self.referenced_as.len() >= 2);

        let error_display = BlobErrorDisplayMessage {
            error_title: ERROR_TITLE,

            blob_info: ErrorDisplayBlobInfo {
                blob_id: self.blob_id,
                blob_info: self.blob_info,
                blob_referenced_as: self.referenced_as.iter(),
            },
        };
        error_display.display(f)
    }
}

#[cfg(test)]
mod tests {
    use console::strip_ansi_codes;
    use cryfs_filesystem::filesystem::fsblobstore::BlobType;
    use cryfs_rustfs::AbsolutePathBuf;

    use super::*;

    // TODO Here (and for other errors), tests formatting, i.e. coloring and boldness of error message

    #[test]
    fn test_display_missing() {
        let error = BlobReferencedMultipleTimesError {
            blob_id: BlobId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            blob_info: MaybeBlobInfoAsSeenByLookingAtBlob::Missing,
            referenced_as: [
                BlobReference {
                    blob_type: BlobType::File,
                    parent_id: BlobId::from_hex("3ef706935f4693039c90da370e99ada9").unwrap(),
                    path: AbsolutePathBuf::try_from_string("/path/to/blob".to_string()).unwrap(),
                },
                BlobReference {
                    blob_type: BlobType::Symlink,
                    parent_id: BlobId::from_hex("4693039c90da370e99ada93ef706935f").unwrap(),
                    path: AbsolutePathBuf::try_from_string("/path/to/another/blob".to_string())
                        .unwrap(),
                },
            ]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[BlobReferencedMultipleTimes]: Blob is referenced multiple times.
  ---> File at /path/to/blob [parent_blob=3ef706935f4693039c90da370e99ada9]
  ---> Symlink at /path/to/another/blob [parent_blob=4693039c90da370e99ada93ef706935f]
  Blob Id: 918ca6ac525c700c275615c3de0cea1b
  Blob Info: Blob is missing
"
            .trim(),
        );
    }

    #[test]
    fn test_display_unreadable() {
        let error = BlobReferencedMultipleTimesError {
            blob_id: BlobId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            blob_info: MaybeBlobInfoAsSeenByLookingAtBlob::Unreadable,
            referenced_as: [
                BlobReference {
                    blob_type: BlobType::Dir,
                    parent_id: BlobId::from_hex("3ef706935f4693039c90da370e99ada9").unwrap(),
                    path: AbsolutePathBuf::try_from_string("/path/to/blob".to_string()).unwrap(),
                },
                BlobReference {
                    blob_type: BlobType::File,
                    parent_id: BlobId::from_hex("4693039c90da370e99ada93ef706935f").unwrap(),
                    path: AbsolutePathBuf::try_from_string("/path/to/another/blob".to_string())
                        .unwrap(),
                },
            ]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[BlobReferencedMultipleTimes]: Blob is referenced multiple times.
  ---> Dir at /path/to/blob [parent_blob=3ef706935f4693039c90da370e99ada9]
  ---> File at /path/to/another/blob [parent_blob=4693039c90da370e99ada93ef706935f]
  Blob Id: 918ca6ac525c700c275615c3de0cea1b
  Blob Info: Blob is unreadable
"
            .trim(),
        );
    }

    #[test]
    fn test_display_file_blob() {
        let error = BlobReferencedMultipleTimesError {
            blob_id: BlobId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            blob_info: MaybeBlobInfoAsSeenByLookingAtBlob::Readable {
                blob_type: BlobType::File,
                parent_pointer: BlobId::from_hex("f4693039c3ef70a370e99ada9693590d").unwrap(),
            },
            referenced_as: [
                BlobReference {
                    blob_type: BlobType::Dir,
                    parent_id: BlobId::from_hex("3ef706935f4693039c90da370e99ada9").unwrap(),
                    path: AbsolutePathBuf::try_from_string("/path/to/blob".to_string()).unwrap(),
                },
                BlobReference {
                    blob_type: BlobType::File,
                    parent_id: BlobId::from_hex("4693039c90da370e99ada93ef706935f").unwrap(),
                    path: AbsolutePathBuf::try_from_string("/path/to/another/blob".to_string())
                        .unwrap(),
                },
            ]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[BlobReferencedMultipleTimes]: Blob is referenced multiple times.
  ---> Dir at /path/to/blob [parent_blob=3ef706935f4693039c90da370e99ada9]
  ---> File at /path/to/another/blob [parent_blob=4693039c90da370e99ada93ef706935f]
  Blob Id: 918ca6ac525c700c275615c3de0cea1b
  Blob Info: File [parent_pointer=f4693039c3ef70a370e99ada9693590d]
"
            .trim(),
        );
    }

    #[test]
    fn test_display_dir_blob() {
        let error = BlobReferencedMultipleTimesError {
            blob_id: BlobId::from_hex("8ca6ac525c700c275615c3de0cea1b91").unwrap(),
            blob_info: MaybeBlobInfoAsSeenByLookingAtBlob::Readable {
                blob_type: BlobType::Dir,
                parent_pointer: BlobId::from_hex("693039c3ef70a370e99ada9693590df4").unwrap(),
            },
            referenced_as: [
                BlobReference {
                    blob_type: BlobType::Dir,
                    parent_id: BlobId::from_hex("f706935f4693039c90da370e99ada93e").unwrap(),
                    path: AbsolutePathBuf::try_from_string("/path/to/blob".to_string()).unwrap(),
                },
                BlobReference {
                    blob_type: BlobType::File,
                    parent_id: BlobId::from_hex("93039c90da370e99ada93ef706935f46").unwrap(),
                    path: AbsolutePathBuf::try_from_string("/path/to/another/blob".to_string())
                        .unwrap(),
                },
            ]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[BlobReferencedMultipleTimes]: Blob is referenced multiple times.
  ---> Dir at /path/to/blob [parent_blob=f706935f4693039c90da370e99ada93e]
  ---> File at /path/to/another/blob [parent_blob=93039c90da370e99ada93ef706935f46]
  Blob Id: 8ca6ac525c700c275615c3de0cea1b91
  Blob Info: Dir [parent_pointer=693039c3ef70a370e99ada9693590df4]
"
            .trim(),
        );
    }

    #[test]
    fn test_display_symlink_blob() {
        let error = BlobReferencedMultipleTimesError {
            blob_id: BlobId::from_hex("8ca6ac525c700c275615c3de0cea1b91").unwrap(),
            blob_info: MaybeBlobInfoAsSeenByLookingAtBlob::Readable {
                blob_type: BlobType::Symlink,
                parent_pointer: BlobId::from_hex("693039c3ef70a370e99ada9693590df4").unwrap(),
            },
            referenced_as: [
                BlobReference {
                    blob_type: BlobType::Dir,
                    parent_id: BlobId::from_hex("f706935f4693039c90da370e99ada93e").unwrap(),
                    path: AbsolutePathBuf::try_from_string("/path/to/blob".to_string()).unwrap(),
                },
                BlobReference {
                    blob_type: BlobType::File,
                    parent_id: BlobId::from_hex("93039c90da370e99ada93ef706935f46").unwrap(),
                    path: AbsolutePathBuf::try_from_string("/path/to/another/blob".to_string())
                        .unwrap(),
                },
            ]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[BlobReferencedMultipleTimes]: Blob is referenced multiple times.
  ---> Dir at /path/to/blob [parent_blob=f706935f4693039c90da370e99ada93e]
  ---> File at /path/to/another/blob [parent_blob=93039c90da370e99ada93ef706935f46]
  Blob Id: 8ca6ac525c700c275615c3de0cea1b91
  Blob Info: Symlink [parent_pointer=693039c3ef70a370e99ada9693590df4]
"
            .trim(),
        );
    }

    #[test]
    fn test_display_many_references() {
        let error = BlobReferencedMultipleTimesError {
            blob_id: BlobId::from_hex("8ca6ac525c700c275615c3de0cea1b91").unwrap(),
            blob_info: MaybeBlobInfoAsSeenByLookingAtBlob::Readable {
                blob_type: BlobType::Symlink,
                parent_pointer: BlobId::from_hex("693039c3ef70a370e99ada9693590df4").unwrap(),
            },
            referenced_as: [
                BlobReference {
                    blob_type: BlobType::Dir,
                    parent_id: BlobId::from_hex("f706935f4693039c90da370e99ada93e").unwrap(),
                    path: AbsolutePathBuf::try_from_string("/path/to/blob".to_string()).unwrap(),
                },
                BlobReference {
                    blob_type: BlobType::File,
                    parent_id: BlobId::from_hex("93039c90da370e99ada93ef706935f46").unwrap(),
                    path: AbsolutePathBuf::try_from_string("/path/to/another/blob".to_string())
                        .unwrap(),
                },
                BlobReference {
                    blob_type: BlobType::Symlink,
                    parent_id: BlobId::from_hex("3039c90da370e99ada93ef706935f469").unwrap(),
                    path: AbsolutePathBuf::try_from_string("/path/to/yet/another/blob".to_string())
                        .unwrap(),
                },
                BlobReference {
                    blob_type: BlobType::File,
                    parent_id: BlobId::from_hex("f46cf70693590da370e99ada93e93039").unwrap(),
                    path: AbsolutePathBuf::try_from_string("/path/to/file/blob".to_string())
                        .unwrap(),
                },
                BlobReference {
                    blob_type: BlobType::Dir,
                    parent_id: BlobId::from_hex("ab9ada93ef706935f4693039c90da370").unwrap(),
                    path: AbsolutePathBuf::try_from_string("/path/to/dir/blob".to_string())
                        .unwrap(),
                },
            ]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[BlobReferencedMultipleTimes]: Blob is referenced multiple times.
  ---> Dir at /path/to/dir/blob [parent_blob=ab9ada93ef706935f4693039c90da370]
  ---> Dir at /path/to/blob [parent_blob=f706935f4693039c90da370e99ada93e]
  ---> File at /path/to/another/blob [parent_blob=93039c90da370e99ada93ef706935f46]
  ---> File at /path/to/file/blob [parent_blob=f46cf70693590da370e99ada93e93039]
  ---> Symlink at /path/to/yet/another/blob [parent_blob=3039c90da370e99ada93ef706935f469]
  Blob Id: 8ca6ac525c700c275615c3de0cea1b91
  Blob Info: Symlink [parent_pointer=693039c3ef70a370e99ada9693590df4]
"
            .trim(),
        );
    }
}
