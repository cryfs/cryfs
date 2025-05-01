pub mod high_level;
pub mod low_level;
pub mod utils;

/// Instantiate all blockstore tests for a given lowlevel blockstore implementation.
/// This will instantiate low level blockstore tests directly, and high level blockstore tests by wrapping the blockstore into a high level blockstore.
#[macro_export]
macro_rules! instantiate_blockstore_tests_for_lowlevel_blockstore {
    ($type: ty) => {
        $crate::instantiate_blockstore_tests_for_lowlevel_blockstore!($type, ());
    };
    ($type: ty, $tokio_test_args: tt) => {
        mod low_level {
            use super::*;
            $crate::instantiate_lowlevel_blockstore_specific_tests!($type, $tokio_test_args);
        }
        mod wrapped_in_high_level {
            use super::*;
            mod with_flushing {
                use super::*;
                $crate::instantiate_highlevel_blockstore_specific_tests!(
                        $crate::tests::low_level::FixtureAdapterForHLTests<$type, false>,
                    $tokio_test_args
                );
            }
            mod without_flushing {
                use super::*;
                $crate::instantiate_highlevel_blockstore_specific_tests!(
                        $crate::tests::low_level::FixtureAdapterForHLTests<$type, true>,
                    $tokio_test_args
                );
            }
        }
        mod double_wrapped {
            //! Wrapped in high level, and the result wrapped back into low level blockstore.
            use super::*;
            mod with_flushing {
                use super::*;
                $crate::instantiate_lowlevel_blockstore_specific_tests!(
                    $crate::tests::high_level::FixtureAdapterForLLTests<
                        $crate::tests::low_level::FixtureAdapterForHLTests<$type, true>,
                        false, // can be false since we don't need to double flush
                    >,
                    $tokio_test_args
                );
            }
            mod without_flushing {
                use super::*;
                $crate::instantiate_lowlevel_blockstore_specific_tests!(
                    $crate::tests::high_level::FixtureAdapterForLLTests<
                        $crate::tests::low_level::FixtureAdapterForHLTests<$type, false>,
                        false,
                    >,
                    $tokio_test_args
                );
            }
        }
    };
}

/// Instantiate all blockstore tests for a given highlevel blockstore implementation.
/// This will instantiate high level blockstore tests directly, and low level blockstore tests by wrapping the blockstore into a low level blockstore.
#[macro_export]
macro_rules! instantiate_blockstore_tests_for_highlevel_blockstore {
    ($type: ty) => {
        $crate::instantiate_blockstore_tests_for_highlevel_blockstore!($type, ());
    };
    ($type: ty, $tokio_test_args: tt) => {
        mod high_level {
            use super::*;
            $crate::instantiate_highlevel_blockstore_specific_tests!($type, $tokio_test_args);
        }
        mod wrapped_in_low_level {
            use super::*;
            mod with_flushing {
                use super::*;
                $crate::instantiate_lowlevel_blockstore_specific_tests!(
                    $crate::tests::high_level::FixtureAdapterForLLTests<$type, false>,
                    $tokio_test_args
                );
            }
            mod without_flushing {
                use super::*;
                $crate::instantiate_lowlevel_blockstore_specific_tests!(
                    $crate::tests::high_level::FixtureAdapterForLLTests<$type, true>,
                    $tokio_test_args
                );
            }
        }
        mod double_wrapped {
            //! Wrapped in high level, and the result wrapped back into low level blockstore.
            use super::*;
            mod with_flushing {
                use super::*;
                $crate::instantiate_highlevel_blockstore_specific_tests!(
                    $crate::tests::low_level::FixtureAdapterForHLTests<
                        $crate::tests::high_level::FixtureAdapterForLLTests<$type, true>,
                        false, // can be false since we don't need to double flush
                    >,
                    $tokio_test_args
                );
            }
            mod without_flushing {
                use super::*;
                $crate::instantiate_highlevel_blockstore_specific_tests!(
                    $crate::tests::low_level::FixtureAdapterForHLTests<
                        $crate::tests::high_level::FixtureAdapterForLLTests<$type, false>,
                        false,
                    >,
                    $tokio_test_args
                );
            }
        }
    };
}
