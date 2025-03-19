use anyhow::bail;
use binary_layout::{LayoutAs, binary_layout};
use std::convert::Infallible;

use cryfs_blobstore::BLOBID_LEN;

const MAGIC_NUMBER_DIR: u8 = 0x00;
const MAGIC_NUMBER_FILE: u8 = 0x01;
const MAGIC_NUMBER_SYMLINK: u8 = 0x02;

pub const FORMAT_VERSION_HEADER: u16 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum BlobType {
    Dir,
    File,
    Symlink,
}

impl LayoutAs<u8> for BlobType {
    type ReadError = anyhow::Error;
    type WriteError = Infallible;

    fn try_read(v: u8) -> Result<Self, anyhow::Error> {
        match v {
            MAGIC_NUMBER_DIR => Ok(BlobType::Dir),
            MAGIC_NUMBER_FILE => Ok(BlobType::File),
            MAGIC_NUMBER_SYMLINK => Ok(BlobType::Symlink),
            magic_number => bail!("Invalid FsBlob magic number {magic_number}"),
        }
    }

    fn try_write(v: Self) -> Result<u8, Infallible> {
        match v {
            BlobType::Dir => Ok(MAGIC_NUMBER_DIR),
            BlobType::File => Ok(MAGIC_NUMBER_FILE),
            BlobType::Symlink => Ok(MAGIC_NUMBER_SYMLINK),
        }
    }
}

binary_layout!(fsblob_header, LittleEndian, {
    format_version_header: u16,
    blob_type: BlobType as u8,
    // TODO We're currently not checking the parent pointers when loading a blob. We should probably check all the parent pointers on a path from the root to a loaded blob when traversing down the tree to load that blob.
    parent: [u8; BLOBID_LEN], // TODO `BlobId as [u8; BLOBID_LEN]` with binary_layout::LayoutAs
});

binary_layout!(fsblob, LittleEndian, {
        header: fsblob_header::NestedView,
        data: [u8],
    }
);
