//! This module defines a common set of unit tests to be run on a [BlockStore] implementation.
//! To use it, implement [Fixture] for your block store and call [instantiate_blockstore_tests!].

use async_trait::async_trait;
use rand::{rngs::StdRng, RngCore, SeedableRng};

use crate::blockstore::{low_level::BlockStore, BlockId};
use crate::data::Data;
use crate::utils::async_drop::SyncDrop;

/// By writing a [Fixture] implementation and using the [instantiate_blockstore_tests] macro,
/// our suite of block store tests is instantiated for a given block store.
///
/// The fixture is kept alive for as long as the test runs, so it can hold RAII resources
/// required by the block store.
#[async_trait]
pub trait Fixture {
    type ConcreteBlockStore: BlockStore + Send + Sync;

    /// Instantiate the fixture
    fn new() -> Self;

    /// Create a new block store for testing
    async fn store(&mut self) -> SyncDrop<Self::ConcreteBlockStore>;

    /// Run some action defined by the fixture. This is often called
    /// by test cases between making changes and asserting that the changes
    /// were correctly made. Test fixtures can do things like flushing here
    /// if they want to test that flushing doesn't break anything.
    /// Most fixtures will likely implement this as a no-op.
    /// TODO Go through our low level block store implementations and see if they have a use for yield_fixture
    async fn yield_fixture(&self, store: &Self::ConcreteBlockStore);
}

pub fn blockid(seed: u64) -> BlockId {
    BlockId::from_slice(data(16, seed).as_ref()).unwrap()
}

pub fn data(size: usize, seed: u64) -> Data {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut res = vec![0; size];
    rng.fill_bytes(&mut res);
    res.into()
}

pub mod high_level;
pub mod low_level;

#[macro_export]
macro_rules! instantiate_blockstore_tests {
    ($type: ty) => {
        $crate::instantiate_blockstore_tests!($type, ());
    };
    ($type: ty, $tokio_test_args: tt) => {
        mod low_level {
            use super::*;
            $crate::instantiate_lowlevel_blockstore_tests!($type, $tokio_test_args);
        }
        mod high_level {
            use super::*;
            $crate::instantiate_highlevel_blockstore_tests!($type, $tokio_test_args);
        }
    };
}
