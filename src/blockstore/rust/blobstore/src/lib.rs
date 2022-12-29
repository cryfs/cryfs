mod blob_id;
pub use blob_id::BlobId;

mod interface;
pub use interface::{Blob, BlobStore, BLOBID_LEN};

mod on_blocks;
pub use on_blocks::{BlobOnBlocks, BlobStoreOnBlocks};

pub use cryfs_blockstore::RemoveResult;

#[cfg(test)]
mod tests;
