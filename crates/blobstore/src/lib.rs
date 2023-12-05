// TODO Figure out which functions actually should or shouldn't be #[inline]

#![forbid(unsafe_code)]
// TODO #![deny(missing_docs)]

mod blob_id;
pub use blob_id::BlobId;

mod interface;
pub use interface::{Blob, BlobStore, BLOBID_LEN};

mod on_blocks;
pub use on_blocks::{BlobOnBlocks, BlobStoreOnBlocks, DataNodeStore, DataTreeStore};

pub use cryfs_blockstore::RemoveResult;

#[cfg(test)]
mod tests;

// This is needed by rstest_reuse, at least in 0.5.0, because otherwise they can't find their macros
#[cfg(test)]
#[cfg(feature = "slow-tests-any")]
use rstest_reuse;

cryfs_version::assert_cargo_version_equals_git_version!();
