use console::style;
use std::fmt::{self, Formatter};

use cryfs_blobstore::BlobId;
use cryfs_filesystem::filesystem::fsblobstore::BlobType;

use super::ErrorTitle;
use crate::{BlobReference, MaybeBlobInfoAsSeenByLookingAtBlob};

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct BlobErrorDisplayMessage<'a, RI>
where
    RI: Iterator<Item = &'a BlobReference>,
{
    pub error_title: ErrorTitle,
    pub blob_info: ErrorDisplayBlobInfo<'a, RI>,
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct ErrorDisplayBlobInfo<'a, RI>
where
    RI: Iterator<Item = &'a BlobReference>,
{
    pub blob_id: BlobId,
    pub blob_info: MaybeBlobInfoAsSeenByLookingAtBlob,
    pub blob_referenced_as: RI,
}

impl<'a, RI> BlobErrorDisplayMessage<'a, RI>
where
    RI: Iterator<Item = &'a BlobReference>,
{
    pub fn display(self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{error_title}", error_title = self.error_title)?;
        display_error_display_blob_info(f, self.blob_info)?;

        Ok(())
    }
}

fn display_error_display_blob_info<'a, RI>(
    f: &mut Formatter<'_>,
    obj: ErrorDisplayBlobInfo<'a, RI>,
) -> fmt::Result
where
    RI: Iterator<Item = &'a BlobReference>,
{
    let mut has_references = false;
    for referenced_as in obj.blob_referenced_as {
        has_references = true;
        write!(f, "  ---> ")?;
        display_blob_reference(f, referenced_as)?;
        write!(f, "\n")?;
    }
    if !has_references {
        write!(f, "  ---> No references to blob found\n")?;
    }
    write!(
        f,
        "  {blob_id_title} {blob_id}\n  {blob_info_title} ",
        blob_id_title = style("Blob Id:").bold(),
        blob_id = &obj.blob_id,
        blob_info_title = style("Blob Info:").bold(),
    )?;
    display_maybe_blob_info_as_seen_by_looking_at_blob(f, &obj.blob_info)?;
    write!(f, "\n")?;

    Ok(())
}

fn display_blob_reference(f: &mut fmt::Formatter<'_>, obj: &BlobReference) -> fmt::Result {
    let blob_type = match obj.blob_type {
        BlobType::File => "File",
        BlobType::Dir => "Dir",
        BlobType::Symlink => "Symlink",
    };
    write!(
        f,
        "{blob_type} at {path} [parent_blob={parent}]",
        path = obj.path,
        parent = obj.parent_id,
    )
}

fn display_maybe_blob_info_as_seen_by_looking_at_blob(
    f: &mut fmt::Formatter<'_>,
    obj: &MaybeBlobInfoAsSeenByLookingAtBlob,
) -> fmt::Result {
    match obj {
        MaybeBlobInfoAsSeenByLookingAtBlob::Missing => write!(f, "Blob is missing"),
        MaybeBlobInfoAsSeenByLookingAtBlob::Unreadable => write!(f, "Blob is unreadable"),
        MaybeBlobInfoAsSeenByLookingAtBlob::Readable {
            blob_type,
            parent_pointer,
        } => {
            let blob_type = match blob_type {
                BlobType::File => "File",
                BlobType::Dir => "Dir",
                BlobType::Symlink => "Symlink",
            };
            write!(f, "{blob_type} [parent_pointer={parent_pointer}]",)
        }
    }
}
