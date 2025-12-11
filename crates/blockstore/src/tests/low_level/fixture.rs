use async_trait::async_trait;
use cryfs_utils::async_drop::AsyncDropGuard;

use crate::LLBlockStore;

/// By writing a [LLFixture] implementation and using the [instantiate_lowlevel_blockstore_specific_tests!](crate::instantiate_lowlevel_blockstore_specific_tests!) macro,
/// our suite of low level block store tests is instantiated for a given block store.
///
/// The fixture is kept alive for as long as the test runs, so it can hold RAII resources
/// required by the block store.
#[async_trait]
pub trait LLFixture {
    type ConcreteBlockStore: LLBlockStore + Send + Sync;

    /// Instantiate the fixture
    fn new() -> Self;

    /// Create a new block store for testing
    async fn store(&mut self) -> AsyncDropGuard<Self::ConcreteBlockStore>;

    /// Run some action defined by the fixture. This is often called
    /// by test cases between making changes and asserting that the changes
    /// were correctly made. Test fixtures can do things like flushing here
    /// if they want to test that flushing doesn't break anything.
    /// Most fixtures will likely implement this as a no-op.
    /// TODO Go through our low level block store implementations and see if they have a use for yield_fixture
    async fn yield_fixture(&self, store: &Self::ConcreteBlockStore);
}
