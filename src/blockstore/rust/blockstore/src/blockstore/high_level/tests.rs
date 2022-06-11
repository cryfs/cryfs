#![allow(non_snake_case)]

use std::fmt::Debug;

use crate::blockstore::high_level::LockingBlockStore;
use crate::blockstore::low_level::BlockStore;
use crate::utils::async_drop::SyncDrop;

use crate::blockstore::low_level::tests::{blockid, data, Fixture};

/// Based on a [crate::low_level::tests::Fixture], we define a [LockingBlockStoreFixture]
/// that uses the underlying fixture and wraps its blockstore into a [LockingBlockStore]
/// to run LockingBlockStore tests on it.
pub trait LockingBlockStoreFixture {
    type ConcreteBlockStore: BlockStore + Send + Sync + Debug + 'static;

    fn store(&mut self) -> SyncDrop<LockingBlockStore<Self::ConcreteBlockStore>>;
}

impl<F> LockingBlockStoreFixture for F
where
    F: Fixture,
    F::ConcreteBlockStore: Send + Sync + Debug + 'static,
{
    type ConcreteBlockStore = F::ConcreteBlockStore;
    fn store(&mut self) -> SyncDrop<LockingBlockStore<Self::ConcreteBlockStore>> {
        let inner = Fixture::store(self).into_inner_dont_drop();
        SyncDrop::new(LockingBlockStore::new(inner))
    }
}

pub mod create {
    use super::*;

    pub async fn test_twoCreatedBlocksHaveDifferentIds(mut f: impl LockingBlockStoreFixture) {
        let store = f.store();
        let first = store.create(&data(1024, 0)).await.unwrap();
        let second = store.create(&data(1024, 1)).await.unwrap();
        assert_ne!(first, second);
    }
}

#[macro_export]
macro_rules! _instantiate_locking_blockstore_tests {
    (@module $module_name: ident, $target: ty, $tokio_test_args: tt $(, $test_cases: ident)* $(,)?) => {
        mod $module_name {
            use super::*;

            $crate::_instantiate_locking_blockstore_tests!(@module_impl $module_name, $target, $tokio_test_args $(, $test_cases)*);
        }
    };
    (@module_impl $module_name: ident, $target: ty, $tokio_test_args: tt) => {
    };
    (@module_impl $module_name: ident, $target: ty, $tokio_test_args: tt, $head_test_case: ident $(, $tail_test_cases: ident)*) => {
        #[tokio::test$tokio_test_args]
        #[allow(non_snake_case)]
        async fn $head_test_case() {
            let fixture = <$target as $crate::blockstore::low_level::tests::Fixture>::new();
            $crate::blockstore::high_level::tests::$module_name::$head_test_case(fixture).await;
        }
        $crate::_instantiate_locking_blockstore_tests!(@module_impl $module_name, $target, $tokio_test_args $(, $tail_test_cases)*);
    };
}

/// This macro instantiates all LockingBlockStore tests for a given blockstore.
/// See [crate::low_level::tests::Fixture] and [LockingBlockStoreFixture] for how to invoke it.
#[macro_export]
macro_rules! instantiate_locking_blockstore_tests {
    ($target: ty) => {
        $crate::instantiate_locking_blockstore_tests!($target, ());
    };
    ($target: ty, $tokio_test_args: tt) => {
        $crate::_instantiate_locking_blockstore_tests!(@module create, $target, $tokio_test_args,
            test_twoCreatedBlocksHaveDifferentIds
        );
        $crate::_instantiate_locking_blockstore_tests!(@module try_create, $target, $tokio_test_args,
            // test_givenNonEmptyBlockStore_whenCallingTryCreateOnExistingBlock_thenFails,
            // test_givenNonEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_withEmptyData_thenSucceeds,
            // test_givenNonEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_withNonEmptyData_thenSucceeds,
            // test_givenEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_withEmptyData_thenSucceeds,
            // test_givenEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_withNonEmptyData_thenSucceeds,
        );
        $crate::_instantiate_locking_blockstore_tests!(@module load, $target, $tokio_test_args,
            // test_givenNonEmptyBlockStore_whenLoadExistingBlock_withEmptyData_thenSucceeds,
            // test_givenNonEmptyBlockStore_whenLoadExistingBlock_withNonEmptyData_thenSucceeds,
            // test_givenNonEmptyBlockStore_whenLoadNonexistingBlock_thenFails,
            // test_givenEmptyBlockStore_whenLoadNonexistingBlock_thenFails,
        );
        $crate::_instantiate_locking_blockstore_tests!(@module store, $target, $tokio_test_args,
            // test_givenEmptyBlockStore_whenStoringNonExistingBlock_withEmptyData_thenSucceeds,
            // test_givenEmptyBlockStore_whenStoringNonExistingBlock_withNonEmptyData_thenSucceeds,
            // test_givenNonEmptyBlockStore_whenStoringNonExistingBlock_withEmptyData_thenSucceeds,
            // test_givenNonEmptyBlockStore_whenStoringNonExistingBlock_withNonEmptyData_thenSucceeds,
            // test_givenNonEmptyBlockStore_whenStoringExistingBlock_withEmptyData_thenSucceeds,
            // test_givenNonEmptyBlockStore_whenStoringExistingBlock_withNonEmptyData_thenSucceeds,
        );
        $crate::_instantiate_locking_blockstore_tests!(@module remove, $target, $tokio_test_args,
            // test_givenOtherwiseEmptyBlockStore_whenRemovingEmptyBlock_thenBlockIsNotLoadableAnymore,
            // test_givenOtherwiseEmptyBlockStore_whenRemovingNonEmptyBlock_thenBlockIsNotLoadableAnymore,
            // test_givenNonEmptyBlockStore_whenRemovingEmptyBlock_thenBlockIsNotLoadableAnymore,
            // test_givenNonEmptyBlockStore_whenRemovingNonEmptyBlock_thenBlockIsNotLoadableAnymore,
            // test_givenEmptyBlockStore_whenRemovingNonexistingBlock_thenFails,
            // test_givenNonEmptyBlockStore_whenRemovingNonexistingBlock_thenFails,
        );
        $crate::_instantiate_locking_blockstore_tests!(@module num_blocks, $target, $tokio_test_args,
            // test_givenEmptyBlockStore_whenCallingNumBlocks_thenReturnsCorrectResult,
            // test_afterStoringBlocks_whenCallingNumBlocks_thenReturnsCorrectResult,
            // test_afterTryCreatingBlocks_whenCallingNumBlocks_thenReturnsCorrectResult,
            // test_afterRemovingBlocks_whenCallingNumBlocks_thenReturnsCorrectResult,
        );
        $crate::_instantiate_locking_blockstore_tests!(@module all_blocks, $target, $tokio_test_args,
            // test_givenEmptyBlockStore_whenCallingAllBlocks_thenReturnsCorrectResult,
            // test_givenBlockStoreWithOneNonEmptyBlock_whenCallingAllBlocks_thenReturnsCorrectResult,
            // test_givenBlockStoreWithOneEmptyBlock_whenCallingAllBlocks_thenReturnsCorrectResult,
            // test_givenBlockStoreWithTwoBlocks_whenCallingAllBlocks_thenReturnsCorrectResult,
            // test_givenBlockStoreWithThreeBlocks_whenCallingAllBlocks_thenReturnsCorrectResult,
            // test_afterRemovingBlock_whenCallingAllBlocks_doesntListRemovedBlocks,
        );
        $crate::_instantiate_locking_blockstore_tests!(@module exists, $target, $tokio_test_args,
            // test_givenEmptyBlockStore_whenCallingExistsOnNonExistingBlock_thenReturnsFalse,
            // test_givenNonEmptyBlockStore_whenCallingExistsOnNonExistingBlock_thenReturnsFalse,
            // test_givenNonEmptyBlockStore_whenCallingExistsOnExistingBlock_thenReturnsTrue,
        );
    };
}
