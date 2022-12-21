#[macro_use]
mod interface;
pub use interface::{
    BlockStore, BlockStoreDeleter, BlockStoreReader, BlockStoreWriter, OptimizedBlockStoreWriter,
};

mod implementations;
pub use implementations::{
    AllowIntegrityViolations, ClientId, CompressingBlockStore, EncryptedBlockStore,
    InMemoryBlockStore, IntegrityBlockStore, IntegrityConfig, MissingBlockIsIntegrityViolation,
    OnDiskBlockStore, ReadOnlyBlockStore,
};
#[cfg(any(test, feature = "testutils"))]
pub use implementations::{MockBlockStore, SharedBlockStore};
