#![allow(non_snake_case)]

//! This module contains common test cases for the high level [LockingBlockStore] API.
//! It implements most tests by building a adapter to implement the low level [BlockStore] API
//! for [LockingBlockStore], and then uses [super::low_level] to run the common low level
//! tests on [LockingBlockStore] as well. On top of that, we add some tests that are specific to [LockingBlockStore].

use async_trait::async_trait;
use std::fmt::Debug;

use crate::blockstore::high_level::{Block, LockingBlockStore, RemoveResult};
use crate::blockstore::low_level::BlockStore;
use crate::utils::async_drop::SyncDrop;

use crate::blockstore::tests::{blockid, data, Fixture};

mod block_store_adapter;
pub use block_store_adapter::TestFixtureAdapter;

/// Based on a [crate::low_level::tests::Fixture], we define a [LockingBlockStoreFixture]
/// that uses the underlying fixture and wraps its blockstore into a [LockingBlockStore]
/// to run LockingBlockStore tests on it.
#[async_trait]
pub trait LockingBlockStoreFixture {
    type UnderlyingBlockStore: BlockStore + Send + Sync + Debug + 'static;

    fn new() -> Self;
    fn store(&mut self) -> SyncDrop<LockingBlockStore<Self::UnderlyingBlockStore>>;
    async fn yield_fixture(&self, store: &LockingBlockStore<Self::UnderlyingBlockStore>);
}

pub struct LockingBlockStoreFixtureImpl<F: Fixture, const FLUSH_CACHE_ON_YIELD: bool> {
    f: F,
}

#[async_trait]
impl<F, const FLUSH_CACHE_ON_YIELD: bool> LockingBlockStoreFixture
    for LockingBlockStoreFixtureImpl<F, FLUSH_CACHE_ON_YIELD>
where
    F: Fixture + Sync,
    F::ConcreteBlockStore: Send + Sync + Debug + 'static,
{
    type UnderlyingBlockStore = F::ConcreteBlockStore;
    fn new() -> Self {
        Self { f: F::new() }
    }
    fn store(&mut self) -> SyncDrop<LockingBlockStore<Self::UnderlyingBlockStore>> {
        let inner = self.f.store().into_inner_dont_drop();
        SyncDrop::new(LockingBlockStore::new(inner))
    }
    async fn yield_fixture(&self, store: &LockingBlockStore<Self::UnderlyingBlockStore>) {
        if FLUSH_CACHE_ON_YIELD {
            // We can't call clear_cache_slow() here because that would clear the whole cache
            // and wait for all blocks to be released. But the test cases here (as opposed
            // to the low level one through block_store_adapter) usually keep a Block object
            // around while calling yield_fixture.
            store.clear_unlocked_cache_entries().await.unwrap();
        }
    }
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
        f.yield_fixture(&store).await;

        let second = store.create(&data(1024, 1)).await.unwrap();
        assert_ne!(first, second);
        f.yield_fixture(&store).await;
    }

    // TODO Test block exists and has correct data after creation
}

pub mod remove {
    use super::*;

    pub async fn test_canRemoveAModifiedBlock(mut f: impl LockingBlockStoreFixture) {
        let store = f.store();
        let blockid = store.create(&data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;
        let mut block = store.load(blockid).await.unwrap().unwrap();
        f.yield_fixture(&store).await;

        block.data_mut().copy_from_slice(&data(1024, 1));
        f.yield_fixture(&store).await;

        std::mem::drop(block);
        f.yield_fixture(&store).await;

        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&blockid).await.unwrap()
        );
        f.yield_fixture(&store).await;
    }
}

pub mod resize {
    use super::*;

    pub async fn test_givenZeroSizeBlock_whenResizingToBeLarger_thenSucceeds(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store();
        let blockid = store.create(&data(0, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let mut block = store.load(blockid).await.unwrap().unwrap();
        f.yield_fixture(&store).await;

        block.resize(1024).await;
        f.yield_fixture(&store).await;

        assert_eq!(1024, block.data().len());
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenZeroSizeBlock_whenResizingToBeLarger_thenBlockIsStillUsable(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store();
        let blockid = store.create(&data(0, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let mut block = store.load(blockid).await.unwrap().unwrap();
        f.yield_fixture(&store).await;

        block.resize(1024).await;
        f.yield_fixture(&store).await;

        assert_block_is_usable(&store, block).await;
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeLarger_thenSucceeds(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store();
        let blockid = store.create(&data(100, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let mut block = store.load(blockid).await.unwrap().unwrap();
        f.yield_fixture(&store).await;

        block.resize(1024).await;
        f.yield_fixture(&store).await;

        assert_eq!(1024, block.data().len());
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeLarger_thenBlockIsStillUsable(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store();
        let blockid = store.create(&data(100, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let mut block = store.load(blockid).await.unwrap().unwrap();
        f.yield_fixture(&store).await;

        block.resize(1024).await;
        f.yield_fixture(&store).await;

        assert_block_is_usable(&store, block).await;
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeSmaller_thenSucceeds(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store();
        let blockid = store.create(&data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let mut block = store.load(blockid).await.unwrap().unwrap();
        f.yield_fixture(&store).await;

        block.resize(100).await;
        f.yield_fixture(&store).await;

        assert_eq!(100, block.data().len());
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeSmaller_thenBlockIsStillUsable(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store();
        let blockid = store.create(&data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let mut block = store.load(blockid).await.unwrap().unwrap();
        f.yield_fixture(&store).await;

        block.resize(100).await;
        f.yield_fixture(&store).await;

        assert_block_is_usable(&store, block).await;
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeZero_thenSucceeds(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store();
        let blockid = store.create(&data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let mut block = store.load(blockid).await.unwrap().unwrap();
        f.yield_fixture(&store).await;

        block.resize(0).await;
        f.yield_fixture(&store).await;

        assert_eq!(0, block.data().len());
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeZero_thenBlockIsStillUsable(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store();
        let blockid = store.create(&data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let mut block = store.load(blockid).await.unwrap().unwrap();
        f.yield_fixture(&store).await;

        block.resize(0).await;
        f.yield_fixture(&store).await;

        assert_block_is_usable(&store, block).await;
        f.yield_fixture(&store).await;
    }
}

// TODO Other functions to test?

#[macro_export]
macro_rules! _instantiate_highlevel_blockstore_tests {
    (@module $module_name: ident, $target: ty, $tokio_test_args: tt $(, $test_cases: ident)* $(,)?) => {
        mod $module_name {
            use super::*;

            mod without_flushing {
                use super::*;
                $crate::_instantiate_highlevel_blockstore_tests!(@module_impl $module_name, $target, $tokio_test_args, false $(, $test_cases)*);
            }
            mod with_flushing {
                use super::*;
                $crate::_instantiate_highlevel_blockstore_tests!(@module_impl $module_name, $target, $tokio_test_args, true $(, $test_cases)*);
            }
        }
    };
    (@module_impl $module_name: ident, $target: ty, $tokio_test_args: tt, $flush_cache_on_yield: expr) => {
    };
    (@module_impl $module_name: ident, $target: ty, $tokio_test_args: tt, $flush_cache_on_yield: expr, $head_test_case: ident $(, $tail_test_cases: ident)*) => {
        #[tokio::test$tokio_test_args]
        #[allow(non_snake_case)]
        async fn $head_test_case() {
            let fixture = <$crate::blockstore::tests::high_level::LockingBlockStoreFixtureImpl::<$target, $flush_cache_on_yield> as $crate::blockstore::tests::high_level::LockingBlockStoreFixture>::new();
            $crate::blockstore::tests::high_level::$module_name::$head_test_case(fixture).await;
        }
        $crate::_instantiate_highlevel_blockstore_tests!(@module_impl $module_name, $target, $tokio_test_args, $flush_cache_on_yield $(, $tail_test_cases)*);
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
            mod without_flushing {
                use super::*;
                $crate::instantiate_lowlevel_blockstore_tests!($crate::blockstore::tests::high_level::TestFixtureAdapter<$target, false>, $tokio_test_args);
            }
            mod with_flushing {
                use super::*;
                $crate::instantiate_lowlevel_blockstore_tests!($crate::blockstore::tests::high_level::TestFixtureAdapter<$target, true>, $tokio_test_args);
            }
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

        // TODO Test Block::block_id()
        // TODO Test Block::data() and data_mut()
        // TODO Test Block::flush()
    };
}
