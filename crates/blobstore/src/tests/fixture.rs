use std::fmt::Debug;

use crate::BlobStore;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

/// By writing a [Fixture] implementation and using the [instantiate_blobstore_tests] macro,
/// our suite of blob store tests is instantiated for a given blob store.
///
/// The fixture is kept alive for as long as the test runs, so it can hold RAII resources
/// required by the block store.
pub trait Fixture {
    type ConcreteBlobStore: BlobStore
        + Debug
        + AsyncDrop<Error = anyhow::Error>
        + Send
        + Sync
        + 'static;

    /// Instantiate the fixture
    fn new() -> Self;

    /// Create a new block store for testing
    fn store(&mut self) -> impl Future<Output = AsyncDropGuard<Self::ConcreteBlobStore>> + Send;

    /// Run some action defined by the fixture. This is often called
    /// by test cases between making changes and asserting that the changes
    /// were correctly made. Test fixtures can do things like flushing here
    /// if they want to test that flushing doesn't break anything.
    /// Most fixtures will likely implement this as a no-op.
    /// TODO Go through our low level block store implementations and see if they have a use for yield_fixture
    fn yield_fixture(&self, store: &Self::ConcreteBlobStore) -> impl Future<Output = ()> + Send;
}
