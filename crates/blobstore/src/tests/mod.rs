#![allow(non_snake_case)]

pub mod fixture;
pub mod test_as_blockstore;
pub mod tests;

/// Instantiate all blobstore tests for a given blobstore implementation.
/// This will instantiate some blobstore tests directly, and also wrap it into an adapter making it a [BlockStore] and then run block store tests on it.
#[macro_export]
macro_rules! instantiate_tests_for_blobstore {
    ($type: ty) => {
        $crate::instantiate_tests_for_blobstore!($type, ());
    };
    ($type: ty, $tokio_test_args: tt) => {
        mod as_blobstore {
            use super::*;
            $crate::instantiate_blobstore_specific_tests!($type, $tokio_test_args);
        }
        mod as_blockstore {
            use super::*;
            mod with_flushing {
                use super::*;
                cryfs_blockstore::instantiate_blockstore_tests_for_lowlevel_blockstore!(
                    $crate::tests::test_as_blockstore::TestFixtureAdapter<$type, true>,
                    (flavor = "multi_thread")
                );
            }
            mod without_flushing {
                use super::*;
                cryfs_blockstore::instantiate_blockstore_tests_for_lowlevel_blockstore!(
                    $crate::tests::test_as_blockstore::TestFixtureAdapter<$type, false>,
                    (flavor = "multi_thread")
                );
            }
        }
    };
}
