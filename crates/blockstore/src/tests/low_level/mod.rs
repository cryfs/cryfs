//! This module defines a common set of unit tests to be run on a [LLBlockStore] implementation.
//! To use it, implement [LLFixture] for your block store and call [instantiate_blockstore_tests!](crate::instantiate_blockstore_tests!).

mod fixture;
pub use fixture::LLFixture;

mod adapter_for_high_level_tests;
pub use adapter_for_high_level_tests::LockingBlockStoreFixture;

pub mod tests;
