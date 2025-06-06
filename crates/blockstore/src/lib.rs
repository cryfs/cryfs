// TODO #![deny(missing_docs)]

mod block_id;
pub use block_id::{BLOCKID_LEN, BlockId};

mod utils;
pub use utils::{RemoveResult, TryCreateResult};

mod high_level;
pub use high_level::{Block, BlockStore, LockingBlockStore};

mod low_level;
pub use low_level::{
    AllowIntegrityViolations, BlockStoreDeleter, BlockStoreReader, BlockStoreWriter, ClientId,
    CompressingBlockStore, DynBlockStore, EncryptedBlockStore, InMemoryBlockStore,
    IntegrityBlockStore, IntegrityBlockStoreInitError, IntegrityConfig, IntegrityViolationError,
    LLBlockStore, MissingBlockIsIntegrityViolation, OnDiskBlockStore, OptimizedBlockStoreWriter,
    ReadOnlyBlockStore,
};

mod overhead;
pub use overhead::{InvalidBlockSizeError, Overhead};

#[cfg(any(test, feature = "testutils"))]
pub use high_level::{
    ActionCounts as HLActionCounts, SharedBlockStore as HLSharedBlockStore,
    TrackingBlockStore as HLTrackingBlockStore,
};
#[cfg(any(test, feature = "testutils"))]
pub use low_level::{
    ActionCounts as LLActionCounts, MockBlockStore, SharedBlockStore as LLSharedBlockStore,
    TempDirBlockStore, TrackingBlockStore as LLTrackingBlockStore,
};

#[cfg(any(test, feature = "testutils"))]
pub mod tests;

cryfs_version::assert_cargo_version_equals_git_version!();

// We're using [byte_unit] in a few places where performance might matter.
// But unfortunately, the crate will either use u64 or u128 depending on features.
// Let's make sure it uses u64 and none of our crates accidentally enabled the u128 feature.
static_assertions::const_assert_eq!(
    std::mem::size_of::<u64>(),
    std::mem::size_of::<byte_unit::Byte>()
);
