//! This module defines a common set of unit tests to be run on a [LLBlockStore] implementation.
//! To use it, implement [LLFixture] for your block store and call [instantiate_blockstore_tests!](crate::instantiate_blockstore_tests!).

mod fixture;
pub use fixture::LLFixture;

pub mod tests;

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
