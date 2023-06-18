// TODO #![deny(missing_docs)]

mod block_id;
pub use block_id::{BlockId, BLOCKID_LEN};

mod utils;
pub use utils::{RemoveResult, TryCreateResult};

mod high_level;
pub use high_level::{Block, LockingBlockStore};

mod low_level;
pub use low_level::{
    AllowIntegrityViolations, BlockStore, BlockStoreDeleter, BlockStoreReader, BlockStoreWriter,
    ClientId, CompressingBlockStore, EncryptedBlockStore, InMemoryBlockStore, IntegrityBlockStore,
    IntegrityConfig, MissingBlockIsIntegrityViolation, OnDiskBlockStore, OptimizedBlockStoreWriter,
    ReadOnlyBlockStore,
};

#[cfg(any(test, feature = "testutils"))]
pub use low_level::{ActionCounts, MockBlockStore, SharedBlockStore, TrackingBlockStore};

#[cfg(any(test, feature = "testutils"))]
pub mod tests;

cryfs_version::assert_cargo_version_equals_git_version!();
