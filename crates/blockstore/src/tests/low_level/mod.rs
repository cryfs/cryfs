//! This module defines a common set of unit tests to be run on a [crate::LLBlockStore] implementation.
//! To use it, implement [LLFixture] for your block store and call [instantiate_lowlevel_blockstore_specific_tests!](crate::instantiate_lowlevel_blockstore_specific_tests!).

mod fixture;
pub use fixture::LLFixture;

mod adapter_for_high_level_tests;
pub use adapter_for_high_level_tests::FixtureAdapterForHLTests;

pub mod tests;
