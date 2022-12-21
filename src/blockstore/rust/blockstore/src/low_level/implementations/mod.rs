mod compressing;
pub use compressing::CompressingBlockStore;

mod encrypted;
pub use encrypted::EncryptedBlockStore;

mod inmemory;
pub use inmemory::InMemoryBlockStore;

mod integrity;
pub use integrity::{
    AllowIntegrityViolations, ClientId, IntegrityBlockStore, IntegrityConfig,
    MissingBlockIsIntegrityViolation,
};

mod ondisk;
pub use ondisk::OnDiskBlockStore;

mod readonly;
pub use readonly::ReadOnlyBlockStore;

#[cfg(any(test, feature = "testutils"))]
mod mock;
#[cfg(any(test, feature = "testutils"))]
pub use mock::MockBlockStore;

#[cfg(any(test, feature = "testutils"))]
mod shared;
#[cfg(any(test, feature = "testutils"))]
pub use shared::SharedBlockStore;
