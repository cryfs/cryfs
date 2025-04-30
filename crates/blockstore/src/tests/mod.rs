pub mod high_level;
pub mod low_level;
pub mod utils;

#[macro_export]
macro_rules! instantiate_blockstore_tests_for_lowlevel_blockstore {
    ($type: ty) => {
        $crate::instantiate_blockstore_tests_for_lowlevel_blockstore!($type, ());
    };
    ($type: ty, $tokio_test_args: tt) => {
        mod low_level {
            use super::*;
            $crate::instantiate_lowlevel_blockstore_tests!($type, $tokio_test_args);
        }
        mod wrapped_in_high_level {
            use super::*;
            mod with_flushing {
                use super::*;
                $crate::instantiate_highlevel_blockstore_tests!(
                        $crate::tests::low_level::LockingBlockStoreFixture<$type, false>,
                    $tokio_test_args
                );
            }
            mod without_flushing {
                use super::*;
                $crate::instantiate_highlevel_blockstore_tests!(
                        $crate::tests::low_level::LockingBlockStoreFixture<$type, true>,
                    $tokio_test_args
                );
            }
        }
        mod double_wrapped {
            //! Wrapped in high level, and the result wrapped back into low level blockstore.
            use super::*;
            mod with_flushing {
                use super::*;
                $crate::instantiate_lowlevel_blockstore_tests!(
                    $crate::tests::high_level::FixtureAdapterForLLTests<
                        $crate::tests::low_level::LockingBlockStoreFixture<$type, true>,
                        false, // can be false since we don't need to double flush
                    >,
                    $tokio_test_args
                );
            }
            mod without_flushing {
                use super::*;
                $crate::instantiate_lowlevel_blockstore_tests!(
                    $crate::tests::high_level::FixtureAdapterForLLTests<
                        $crate::tests::low_level::LockingBlockStoreFixture<$type, false>,
                        false,
                    >,
                    $tokio_test_args
                );
            }
        }
    };
}
