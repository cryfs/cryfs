#[macro_use]
mod interface;
pub use interface::{
    BlockStore, BlockStoreDeleter, BlockStoreReader, BlockStoreWriter, OptimizedBlockStoreWriter,
};

mod implementations;
#[cfg(any(test, feature = "testutils"))]
pub use implementations::{ActionCounts, MockBlockStore, SharedBlockStore, TrackingBlockStore};
pub use implementations::{
    AllowIntegrityViolations, ClientId, CompressingBlockStore, EncryptedBlockStore,
    InMemoryBlockStore, IntegrityBlockStore, IntegrityConfig, MissingBlockIsIntegrityViolation,
    OnDiskBlockStore, ReadOnlyBlockStore,
};
