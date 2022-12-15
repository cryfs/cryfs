use binary_layout::{define_layout, LayoutAs};

use crate::blobstore::BLOBID_LEN;

const MAGIC_NUMBER_DIR: u8 = 0x00;
const MAGIC_NUMBER_FILE: u8 = 0x01;
const MAGIC_NUMBER_SYMLINK: u8 = 0x02;

pub const FORMAT_VERSION_HEADER: u16 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlobType {
    Dir,
    File,
    Symlink,
}

impl LayoutAs<u8> for BlobType {
    fn read(v: u8) -> Self {
        match v {
            MAGIC_NUMBER_DIR => BlobType::Dir,
            MAGIC_NUMBER_FILE => BlobType::File,
            MAGIC_NUMBER_SYMLINK => BlobType::Symlink,
            magic_number => panic!("Invalid FsBlob magic number {magic_number}"),
        }
    }

    fn write(v: Self) -> u8 {
        match v {
            BlobType::Dir => MAGIC_NUMBER_DIR,
            BlobType::File => MAGIC_NUMBER_FILE,
            BlobType::Symlink => MAGIC_NUMBER_SYMLINK,
        }
    }
}

define_layout!(fsblob_header, LittleEndian, {
    format_version_header: u16,
    blob_type: BlobType as u8,
    parent: [u8; BLOBID_LEN], // TODO `BlobId as [u8; BLOBID_LEN]` with binary_layout::LayoutAs
});

define_layout!(fsblob, LittleEndian, {
        header: fsblob_header::NestedView,
        data: [u8],
    }
);
