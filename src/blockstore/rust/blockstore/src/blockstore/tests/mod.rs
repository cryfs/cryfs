use rand::{rngs::StdRng, RngCore, SeedableRng};

use crate::blockstore::{low_level::BlockStore, BlockId};
use crate::data::Data;
use crate::utils::async_drop::SyncDrop;

/// By writing a [Fixture] implementation and using the [instantiate_blockstore_tests] macro,
/// our suite of block store tests is instantiated for a given block store.
///
/// The fixture is kept alive for as long as the test runs, so it can hold RAII resources
/// required by the block store.
pub trait Fixture {
    type ConcreteBlockStore: BlockStore;

    fn new() -> Self;
    fn store(&mut self) -> SyncDrop<Self::ConcreteBlockStore>;
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
