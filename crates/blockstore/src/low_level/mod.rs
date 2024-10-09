#[macro_use]
mod interface;
pub use interface::{
    BlockStore, BlockStoreDeleter, BlockStoreReader, BlockStoreWriter, InvalidBlockSizeError,
    OptimizedBlockStoreWriter,
};

mod implementations;
#[cfg(any(test, feature = "testutils"))]
pub use implementations::{ActionCounts, MockBlockStore, SharedBlockStore, TrackingBlockStore};
pub use implementations::{
    AllowIntegrityViolations, ClientId, CompressingBlockStore, DynBlockStore, EncryptedBlockStore,
    InMemoryBlockStore, IntegrityBlockStore, IntegrityBlockStoreInitError, IntegrityConfig,
    MissingBlockIsIntegrityViolation, OnDiskBlockStore, ReadOnlyBlockStore,
};
