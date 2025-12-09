use std::collections::BTreeSet;
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

use cryfs_blobstore::BlobId;

use super::display::{BlobErrorDisplayMessage, ErrorDisplayBlobInfo, ErrorTitle};
use crate::{MaybeBlobInfoAsSeenByLookingAtBlob, node_info::BlobReference};

#[derive(Error, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct BlobUnreadableError {
    pub blob_id: BlobId,
    pub referenced_as: BTreeSet<BlobReference>,
    // TODO error:  anyhow::Error,
}

impl BlobUnreadableError {
    pub fn new(blob_id: BlobId, referenced_as: BTreeSet<BlobReference>) -> Self {
        Self {
            blob_id,
            referenced_as,
            // TODO error: anyhow::Error,
        }
    }
}

const ERROR_TITLE: ErrorTitle = ErrorTitle {
    error_type: "BlobUnreadable",
    error_message: "Blob is unreadable and likely corrupted.",
};

impl Display for BlobUnreadableError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let error_display = BlobErrorDisplayMessage {
            error_title: ERROR_TITLE,

            blob_info: ErrorDisplayBlobInfo {
                blob_id: self.blob_id,
                blob_info: MaybeBlobInfoAsSeenByLookingAtBlob::Unreadable,
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
    use cryfs_utils::path::AbsolutePathBuf;

    use super::*;

    #[test]
    fn test_display_file() {
        let error = BlobUnreadableError::new(
            BlobId::from_hex("918ca6ac525c700c275615c3de0cea1b").unwrap(),
            [BlobReference {
                blob_type: BlobType::File,
                parent_id: BlobId::from_hex("3ef706935f4693039c90da370e99ada9").unwrap(),
                path: AbsolutePathBuf::try_from_string("/path/to/blob".to_string()).unwrap(),
            }]
            .into_iter()
            .collect(),
        );
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[BlobUnreadable]: Blob is unreadable and likely corrupted.
  ---> File at /path/to/blob [parent_blob=3ef706935f4693039c90da370e99ada9]
  Blob Id: 918ca6ac525c700c275615c3de0cea1b
  Blob Info: Blob is unreadable
"
            .trim(),
        );
    }

    #[test]
    fn test_display_dir() {
        let error = BlobUnreadableError::new(
            BlobId::from_hex("25c700c275615c3de0cea1b918ca6ac5").unwrap(),
            [BlobReference {
                blob_type: BlobType::Dir,
                parent_id: BlobId::from_hex("6935f4693039c90da370e99ada93ef70").unwrap(),
                path: AbsolutePathBuf::try_from_string("/path/to/another/blob".to_string())
                    .unwrap(),
            }]
            .into_iter()
            .collect(),
        );
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[BlobUnreadable]: Blob is unreadable and likely corrupted.
  ---> Dir at /path/to/another/blob [parent_blob=6935f4693039c90da370e99ada93ef70]
  Blob Id: 25c700c275615c3de0cea1b918ca6ac5
  Blob Info: Blob is unreadable
"
            .trim(),
        );
    }

    #[test]
    fn test_display_symlink() {
        let error = BlobUnreadableError::new(
            BlobId::from_hex("0c275615c3de0cea1b918ca6ac525c70").unwrap(),
            [BlobReference {
                blob_type: BlobType::Symlink,
                parent_id: BlobId::from_hex("93039c90da370e99ada93ef706935f46").unwrap(),
                path: AbsolutePathBuf::try_from_string("/path/to/yet/another/blob".to_string())
                    .unwrap(),
            }]
            .into_iter()
            .collect(),
        );
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[BlobUnreadable]: Blob is unreadable and likely corrupted.
  ---> Symlink at /path/to/yet/another/blob [parent_blob=93039c90da370e99ada93ef706935f46]
  Blob Id: 0c275615c3de0cea1b918ca6ac525c70
  Blob Info: Blob is unreadable
"
            .trim(),
        );
    }

    #[test]
    fn test_display_referenced_multiple_times() {
        let error = BlobUnreadableError::new(
            BlobId::from_hex("0c275615c3de0cea1b918ca6ac525c70").unwrap(),
            [
                BlobReference {
                    blob_type: BlobType::File,
                    parent_id: BlobId::from_hex("3ef706935f4693039c90da370e99ada9").unwrap(),
                    path: AbsolutePathBuf::try_from_string("/path/to/blob".to_string()).unwrap(),
                },
                BlobReference {
                    blob_type: BlobType::Dir,
                    parent_id: BlobId::from_hex("6935f4693039c90da370e99ada93ef70").unwrap(),
                    path: AbsolutePathBuf::try_from_string("/path/to/another/blob".to_string())
                        .unwrap(),
                },
                BlobReference {
                    blob_type: BlobType::Symlink,
                    parent_id: BlobId::from_hex("93039c90da370e99ada93ef706935f46").unwrap(),
                    path: AbsolutePathBuf::try_from_string("/path/to/yet/another/blob".to_string())
                        .unwrap(),
                },
            ]
            .into_iter()
            .collect(),
        );
        assert_eq!(
            strip_ansi_codes(&format!("{}", error)).trim(),
            "
Error[BlobUnreadable]: Blob is unreadable and likely corrupted.
  ---> Dir at /path/to/another/blob [parent_blob=6935f4693039c90da370e99ada93ef70]
  ---> File at /path/to/blob [parent_blob=3ef706935f4693039c90da370e99ada9]
  ---> Symlink at /path/to/yet/another/blob [parent_blob=93039c90da370e99ada93ef706935f46]
  Blob Id: 0c275615c3de0cea1b918ca6ac525c70
  Blob Info: Blob is unreadable
"
            .trim(),
        );
    }
}
