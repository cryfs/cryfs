#![allow(non_snake_case)]

//! This module contains common test cases for the high level [LockingBlockStore] API.
//! It implements most tests by building a adapter to implement the low level [BlockStore] API
//! for [LockingBlockStore], and then uses [super::low_level] to run the common low level
//! tests on [LockingBlockStore] as well. On top of that, we add some tests that are specific to [LockingBlockStore].

use futures::{StreamExt, TryStreamExt};
use std::fmt::Debug;

use crate::blockstore::high_level::{Block, LockingBlockStore, RemoveResult, TryCreateResult};
use crate::blockstore::low_level::BlockStore;
use crate::blockstore::BlockId;
use crate::data::Data;
use crate::utils::async_drop::SyncDrop;

use crate::blockstore::tests::{blockid, data, Fixture};
use crate::utils::testutils::assert_unordered_vec_eq;

// TODO Go through tests and make sure we have versions that flush. Otherwise, we're just testing the cache.

mod block_store_adapter;
pub use block_store_adapter::TestFixtureAdapter;

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

async fn create_block<B: BlockStore + Send + Sync + Debug + 'static>(
    store: &LockingBlockStore<B>,
    block_id: &BlockId,
    data: &Data,
) {
    assert_eq!(
        TryCreateResult::SuccessfullyCreated,
        store.try_create(block_id, &data).await.unwrap()
    );
}

async fn remove_block<B: BlockStore + Send + Sync + Debug + 'static>(
    store: &LockingBlockStore<B>,
    block_id: &BlockId,
) {
    assert_eq!(
        RemoveResult::SuccessfullyRemoved,
        store.remove(block_id).await.unwrap()
    );
}

async fn assert_block_is_usable<B: BlockStore + Send + Sync + Debug>(
    store: &LockingBlockStore<B>,
    mut block: Block<B>,
) {
    // Write full block space and check it was correctly written
    let fixture = data(block.data().len(), 102);
    block.data_mut().copy_from_slice(&fixture);
    assert_eq!(&fixture, block.data());

    // Store and reload block and check data is still correct
    let block_id = *block.block_id();
    std::mem::drop(block);
    let block = store.load(block_id).await.unwrap().unwrap();
    assert_eq!(&fixture, block.data());
}

pub mod create {
    use super::*;

    pub async fn test_twoCreatedBlocksHaveDifferentIds(mut f: impl LockingBlockStoreFixture) {
        let store = f.store();
        let first = store.create(&data(1024, 0)).await.unwrap();
        let second = store.create(&data(1024, 1)).await.unwrap();
        assert_ne!(first, second);
    }

    // TODO Test block exists and has correct data after creation
}

pub mod remove {
    use super::*;

    pub async fn test_canRemoveAModifiedBlock(mut f: impl LockingBlockStoreFixture) {
        let store = f.store();
        let blockid = store.create(&data(1024, 0)).await.unwrap();
        let mut block = store.load(blockid).await.unwrap().unwrap();
        block.data_mut().copy_from_slice(&data(1024, 1));
        std::mem::drop(block);
        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&blockid).await.unwrap()
        );
    }
}

pub mod resize {
    use super::*;

    pub async fn test_givenZeroSizeBlock_whenResizingToBeLarger_thenSucceeds(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store();
        let blockid = store.create(&data(0, 0)).await.unwrap();
        let mut block = store.load(blockid).await.unwrap().unwrap();
        block.resize(1024).await;
        assert_eq!(1024, block.data().len());
    }

    pub async fn test_givenZeroSizeBlock_whenResizingToBeLarger_thenBlockIsStillUsable(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store();
        let blockid = store.create(&data(0, 0)).await.unwrap();
        let mut block = store.load(blockid).await.unwrap().unwrap();
        block.resize(1024).await;
        assert_block_is_usable(&store, block).await;
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeLarger_thenSucceeds(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store();
        let blockid = store.create(&data(100, 0)).await.unwrap();
        let mut block = store.load(blockid).await.unwrap().unwrap();
        block.resize(1024).await;
        assert_eq!(1024, block.data().len());
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeLarger_thenBlockIsStillUsable(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store();
        let blockid = store.create(&data(100, 0)).await.unwrap();
        let mut block = store.load(blockid).await.unwrap().unwrap();
        block.resize(1024).await;
        assert_block_is_usable(&store, block).await;
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeSmaller_thenSucceeds(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store();
        let blockid = store.create(&data(1024, 0)).await.unwrap();
        let mut block = store.load(blockid).await.unwrap().unwrap();
        block.resize(100).await;
        assert_eq!(100, block.data().len());
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeSmaller_thenBlockIsStillUsable(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store();
        let blockid = store.create(&data(1024, 0)).await.unwrap();
        let mut block = store.load(blockid).await.unwrap().unwrap();
        block.resize(100).await;
        assert_block_is_usable(&store, block).await;
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeZero_thenSucceeds(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store();
        let blockid = store.create(&data(1024, 0)).await.unwrap();
        let mut block = store.load(blockid).await.unwrap().unwrap();
        block.resize(0).await;
        assert_eq!(0, block.data().len());
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeZero_thenBlockIsStillUsable(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store();
        let blockid = store.create(&data(1024, 0)).await.unwrap();
        let mut block = store.load(blockid).await.unwrap().unwrap();
        block.resize(0).await;
        assert_block_is_usable(&store, block).await;
    }
}

// TODO Other functions to test?

#[macro_export]
macro_rules! _instantiate_highlevel_blockstore_tests {
    (@module $module_name: ident, $target: ty, $tokio_test_args: tt $(, $test_cases: ident)* $(,)?) => {
        mod $module_name {
            use super::*;

            $crate::_instantiate_highlevel_blockstore_tests!(@module_impl $module_name, $target, $tokio_test_args $(, $test_cases)*);
        }
    };
    (@module_impl $module_name: ident, $target: ty, $tokio_test_args: tt) => {
    };
    (@module_impl $module_name: ident, $target: ty, $tokio_test_args: tt, $head_test_case: ident $(, $tail_test_cases: ident)*) => {
        #[tokio::test$tokio_test_args]
        #[allow(non_snake_case)]
        async fn $head_test_case() {
            let fixture = <$target as $crate::blockstore::tests::Fixture>::new();
            $crate::blockstore::tests::high_level::$module_name::$head_test_case(fixture).await;
        }
        $crate::_instantiate_highlevel_blockstore_tests!(@module_impl $module_name, $target, $tokio_test_args $(, $tail_test_cases)*);
    };
}

/// This macro instantiates all LockingBlockStore tests for a given blockstore.
/// See [crate::low_level::tests::Fixture] and [LockingBlockStoreFixture] for how to invoke it.
#[macro_export]
macro_rules! instantiate_highlevel_blockstore_tests {
    ($target: ty) => {
        $crate::instantiate_highlevel_blockstore_tests!($target, ());
    };
    ($target: ty, $tokio_test_args: tt) => {
        // Run all low level tests on this block store (using an adapter to map the APIs)
        mod low_level_adapter {
            use super::*;
            $crate::instantiate_lowlevel_blockstore_tests!($crate::blockstore::tests::high_level::TestFixtureAdapter<$target>, $tokio_test_args);
        }
        // And run some additional tests for APIs only we have
        $crate::_instantiate_highlevel_blockstore_tests!(@module create, $target, $tokio_test_args,
            test_twoCreatedBlocksHaveDifferentIds
        );
        // try_create is tested through the low_level tests
        // load is tested through the low_level tests
        // overwrite (=store) is tested through the low_level tests
        $crate::_instantiate_highlevel_blockstore_tests!(@module remove, $target, $tokio_test_args,
            // The low_level tests have further test cases for `remove`
            test_canRemoveAModifiedBlock,
        );
        // num_blocks is tested through the low_level tests
        // all_blocks is tested through the low_level tests
        $crate::_instantiate_highlevel_blockstore_tests!(@module resize, $target, $tokio_test_args,
            test_givenZeroSizeBlock_whenResizingToBeLarger_thenSucceeds,
            test_givenZeroSizeBlock_whenResizingToBeLarger_thenBlockIsStillUsable,
            test_givenNonzeroSizeBlock_whenResizingToBeLarger_thenSucceeds,
            test_givenNonzeroSizeBlock_whenResizingToBeLarger_thenBlockIsStillUsable,
            test_givenNonzeroSizeBlock_whenResizingToBeSmaller_thenSucceeds,
            test_givenNonzeroSizeBlock_whenResizingToBeSmaller_thenBlockIsStillUsable,
            test_givenNonzeroSizeBlock_whenResizingToBeZero_thenSucceeds,
            test_givenNonzeroSizeBlock_whenResizingToBeZero_thenBlockIsStillUsable,
        );
    };
}
