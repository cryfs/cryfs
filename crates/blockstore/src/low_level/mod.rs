#[macro_use]
mod interface;
pub use interface::{
    BlockStoreDeleter, BlockStoreReader, BlockStoreWriter, InvalidBlockSizeError, LLBlockStore,
    OptimizedBlockStoreWriter,
};

mod implementations;
#[cfg(any(test, feature = "testutils"))]
pub use implementations::{
    ActionCounts, MockBlockStore, SharedBlockStore, TempDirBlockStore, TrackingBlockStore,
};
pub use implementations::{
    AllowIntegrityViolations, ClientId, CompressingBlockStore, DynBlockStore, EncryptedBlockStore,
    InMemoryBlockStore, IntegrityBlockStore, IntegrityBlockStoreInitError, IntegrityConfig,
    IntegrityViolationError, MissingBlockIsIntegrityViolation, OnDiskBlockStore,
    ReadOnlyBlockStore,
};
