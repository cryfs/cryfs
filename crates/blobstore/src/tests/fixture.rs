use async_trait::async_trait;
use std::fmt::Debug;

use crate::BlobStore;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

/// By writing a [Fixture] implementation and using the [instantiate_blobstore_tests] macro,
/// our suite of blob store tests is instantiated for a given blob store.
///
/// The fixture is kept alive for as long as the test runs, so it can hold RAII resources
/// required by the block store.
#[async_trait]
pub trait Fixture {
    type ConcreteBlobStore: BlobStore + Debug + AsyncDrop + Send + Sync;

    /// Instantiate the fixture
    fn new() -> Self;

    /// Create a new block store for testing
    async fn store(&mut self) -> AsyncDropGuard<Self::ConcreteBlobStore>;

    /// Run some action defined by the fixture. This is often called
    /// by test cases between making changes and asserting that the changes
    /// were correctly made. Test fixtures can do things like flushing here
    /// if they want to test that flushing doesn't break anything.
    /// Most fixtures will likely implement this as a no-op.
    /// TODO Go through our low level block store implementations and see if they have a use for yield_fixture
    async fn yield_fixture(&self, store: &Self::ConcreteBlobStore);
}
