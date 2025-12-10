// TODO Figure out which functions actually should or shouldn't be #[inline]

#![forbid(unsafe_code)]
// TODO #![deny(missing_docs)]

mod blob_id;
pub use blob_id::BlobId;

mod interface;
pub use interface::{BLOBID_LEN, Blob, BlobStore};

mod implementations;
pub use implementations::{
    BlobOnBlocks, BlobStoreOnBlocks, DataInnerNode, DataLeafNode, DataNode, DataNodeStore,
    DataTree, DataTreeStore, LoadNodeError,
};
#[cfg(any(test, feature = "testutils"))]
pub use implementations::{BlobStoreActionCounts, TrackingBlobStore};

pub use cryfs_blockstore::RemoveResult;

#[cfg(test)]
mod tests;

cryfs_version::assert_cargo_version_equals_git_version!();
