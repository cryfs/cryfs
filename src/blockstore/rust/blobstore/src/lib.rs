mod blob_id;
pub use blob_id::BlobId;

mod interface;
pub use interface::{Blob, BlobStore, BLOBID_LEN};

mod on_blocks;
pub use on_blocks::{BlobOnBlocks, BlobStoreOnBlocks};

pub use cryfs_blockstore::RemoveResult;

#[cfg(test)]
mod tests;

// This is needed by rstest_reuse, at least in 0.5.0, because otherwise they can't find their macros
#[cfg(test)]
#[cfg(feature = "slow-tests-any")]
use rstest_reuse;
