#![allow(non_snake_case)]

//! This module contains common test cases for the high level [LockingBlockStore] API.
//! It implements most tests by building a adapter to implement the low level [BlockStore] API
//! for [LockingBlockStore], and then uses [super::low_level] to run the common low level
//! tests on [LockingBlockStore] as well. On top of that, we add some tests that are specific to [LockingBlockStore].

use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use crate::blockstore::high_level::{Block, LockingBlockStore, RemoveResult};
use crate::blockstore::low_level::BlockStore;
use crate::utils::async_drop::SyncDrop;
use crate::utils::testutils::assert_data_range_eq;

use crate::blockstore::tests::{data, Fixture};

mod block_store_adapter;
pub use block_store_adapter::TestFixtureAdapter;

/// Based on a [crate::low_level::tests::Fixture], we define a [LockingBlockStoreFixture]
/// that uses the underlying fixture and wraps its blockstore into a [LockingBlockStore]
/// to run LockingBlockStore tests on it.
#[async_trait]
pub trait LockingBlockStoreFixture {
    type UnderlyingBlockStore: BlockStore + Send + Sync + Debug + 'static;

    fn new() -> Self;
    async fn store(&mut self) -> SyncDrop<LockingBlockStore<Self::UnderlyingBlockStore>>;
    async fn yield_fixture(&self, store: &LockingBlockStore<Self::UnderlyingBlockStore>);
}

pub struct LockingBlockStoreFixtureImpl<F: Fixture, const FLUSH_CACHE_ON_YIELD: bool> {
    f: F,
}

#[async_trait]
impl<F, const FLUSH_CACHE_ON_YIELD: bool> LockingBlockStoreFixture
    for LockingBlockStoreFixtureImpl<F, FLUSH_CACHE_ON_YIELD>
where
    F: Fixture + Send + Sync,
    F::ConcreteBlockStore: Send + Sync + Debug + 'static,
{
    type UnderlyingBlockStore = F::ConcreteBlockStore;
    fn new() -> Self {
        Self { f: F::new() }
    }
    async fn store(&mut self) -> SyncDrop<LockingBlockStore<Self::UnderlyingBlockStore>> {
        let inner = self.f.store().await.into_inner_dont_drop();
        SyncDrop::new(LockingBlockStore::new(inner))
    }
    async fn yield_fixture(&self, store: &LockingBlockStore<Self::UnderlyingBlockStore>) {
        if FLUSH_CACHE_ON_YIELD {
            store.clear_cache_slow().await.unwrap();
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
        let store = f.store().await;
        let first = store.create(&data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let second = store.create(&data(1024, 1)).await.unwrap();
        assert_ne!(first, second);
        f.yield_fixture(&store).await;
    }

    // TODO Test block exists and has correct data after creation
    // TODO Test creating empty blocks, both on empty and non-empty block store (see low level tests)
    // TODO Make sure all tests have an afterLoading variant
}

pub mod remove {
    use super::*;

    pub async fn test_canRemoveAModifiedBlock(mut f: impl LockingBlockStoreFixture) {
        let store = f.store().await;
        let blockid = store.create(&data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;
        let mut block = store.load(blockid).await.unwrap().unwrap();

        block.data_mut().copy_from_slice(&data(1024, 1));

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
        let store = f.store().await;
        let blockid = store.create(&data(0, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let mut block = store.load(blockid).await.unwrap().unwrap();

        block.resize(1024).await;
        assert_eq!(1024, block.data().len());

        std::mem::drop(block);
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenZeroSizeBlock_whenResizingToBeLarger_thenBlockIsStillUsable(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store().await;
        let blockid = store.create(&data(0, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let mut block = store.load(blockid).await.unwrap().unwrap();

        block.resize(1024).await;
        assert_block_is_usable(&store, block).await;

        f.yield_fixture(&store).await;
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeLarger_thenSucceeds(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store().await;
        let blockid = store.create(&data(100, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let mut block = store.load(blockid).await.unwrap().unwrap();

        block.resize(1024).await;
        assert_eq!(1024, block.data().len());

        std::mem::drop(block);
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeLarger_thenBlockIsStillUsable(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store().await;
        let blockid = store.create(&data(100, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let mut block = store.load(blockid).await.unwrap().unwrap();

        block.resize(1024).await;
        assert_block_is_usable(&store, block).await;

        f.yield_fixture(&store).await;
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeSmaller_thenSucceeds(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store().await;
        let blockid = store.create(&data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let mut block = store.load(blockid).await.unwrap().unwrap();

        block.resize(100).await;
        assert_eq!(100, block.data().len());

        std::mem::drop(block);
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeSmaller_thenBlockIsStillUsable(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store().await;
        let blockid = store.create(&data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let mut block = store.load(blockid).await.unwrap().unwrap();

        block.resize(100).await;
        assert_block_is_usable(&store, block).await;

        f.yield_fixture(&store).await;
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeZero_thenSucceeds(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store().await;
        let blockid = store.create(&data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let mut block = store.load(blockid).await.unwrap().unwrap();

        block.resize(0).await;
        assert_eq!(0, block.data().len());

        std::mem::drop(block);
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeZero_thenBlockIsStillUsable(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store().await;
        let blockid = store.create(&data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let mut block = store.load(blockid).await.unwrap().unwrap();

        block.resize(0).await;
        assert_block_is_usable(&store, block).await;
        f.yield_fixture(&store).await;
    }

    // TODO Make sure all tests have an afterLoading variant
}

pub mod data {
    use super::*;

    struct DataRange {
        blocksize: usize,
        offset: usize,
        count: usize,
    }

    impl DataRange {
        const fn new(blocksize: usize, offset: usize, count: usize) -> Self {
            Self {
                blocksize,
                offset,
                count,
            }
        }
    }

    const DATA_RANGES: &[DataRange] = &[
        DataRange::new(1024, 0, 1024), // full size leaf, access beginning to end
        DataRange::new(1024, 100, 1024 - 200), // full size leaf, access middle to middle
        DataRange::new(1024, 0, 1024 - 100), // full size leaf, access beginning to middle
        DataRange::new(1024, 100, 1024 - 100), // full size leaf, access middle to end
        DataRange::new(1024 - 100, 0, 1024 - 100), // non-full size leaf, access beginning to end
        DataRange::new(1024 - 100, 100, 1024 - 300), // non-full size leaf, access middle to middle
        DataRange::new(1024 - 100, 0, 1024 - 200), // non-full size leaf, access beginning to middle
        DataRange::new(1024 - 100, 100, 1024 - 200), // non-full size leaf, access middle to end
    ];

    pub async fn test_writeAndReadImmediately(mut f: impl LockingBlockStoreFixture) {
        for data_range in DATA_RANGES {
            let store = f.store().await;

            let blockid = store.create(&data(data_range.blocksize, 0)).await.unwrap();
            f.yield_fixture(&store).await;
            let mut block = store.load(blockid).await.unwrap().unwrap();

            block.data_mut()[data_range.offset..(data_range.offset + data_range.count)]
                .copy_from_slice(
                    &data(data_range.blocksize, 5)
                        [data_range.offset..(data_range.offset + data_range.count)],
                );

            assert_data_range_eq(
                &data(data_range.blocksize, 0),
                block.data(),
                ..data_range.offset,
            );
            assert_data_range_eq(
                &data(data_range.blocksize, 5),
                block.data(),
                data_range.offset..(data_range.offset + data_range.count),
            );
            assert_data_range_eq(
                &data(data_range.blocksize, 0),
                block.data(),
                (data_range.offset + data_range.count)..,
            );

            std::mem::drop(block);
            f.yield_fixture(&store).await;
        }
    }

    pub async fn test_writeAndReadAfterLoading(mut f: impl LockingBlockStoreFixture) {
        for data_range in DATA_RANGES {
            let store = f.store().await;

            let blockid = store.create(&data(data_range.blocksize, 0)).await.unwrap();
            f.yield_fixture(&store).await;
            let mut block = store.load(blockid).await.unwrap().unwrap();

            block.data_mut()[data_range.offset..(data_range.offset + data_range.count)]
                .copy_from_slice(
                    &data(data_range.blocksize, 5)
                        [data_range.offset..(data_range.offset + data_range.count)],
                );

            std::mem::drop(block);
            f.yield_fixture(&store).await;

            let block = store.load(blockid).await.unwrap().unwrap();

            assert_data_range_eq(
                &data(data_range.blocksize, 0),
                block.data(),
                ..data_range.offset,
            );
            assert_data_range_eq(
                &data(data_range.blocksize, 5),
                block.data(),
                data_range.offset..(data_range.offset + data_range.count),
            );
            assert_data_range_eq(
                &data(data_range.blocksize, 0),
                block.data(),
                (data_range.offset + data_range.count)..,
            );

            std::mem::drop(block);
            f.yield_fixture(&store).await;
        }
    }
}

pub mod overwrite {
    use super::*;

    pub async fn test_whenOverwritingWhileLoaded_thenBlocks(mut f: impl LockingBlockStoreFixture) {
        let store = Arc::new(f.store().await);

        let blockid = store.create(&data(1024, 0)).await.unwrap();
        let block = store.load(blockid).await.unwrap().unwrap();

        let _store = Arc::clone(&store);
        let overwrite_task = tokio::task::spawn(async move {
            _store.overwrite(&blockid, &data(1024, 1)).await.unwrap();
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(!overwrite_task.is_finished());

        std::mem::drop(block);
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(overwrite_task.is_finished());
    }

    pub async fn test_whenOverwritingWhileLoaded_thenSuccessfullyOverwrites(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = Arc::new(f.store().await);

        let blockid = store.create(&data(1024, 0)).await.unwrap();
        let block = store.load(blockid).await.unwrap().unwrap();

        let _store = Arc::clone(&store);
        let overwrite_task = tokio::task::spawn(async move {
            _store.overwrite(&blockid, &data(512, 1)).await.unwrap();
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
        std::mem::drop(block);
        overwrite_task.await.unwrap();

        let block = store.load(blockid).await.unwrap().unwrap();
        assert_eq!(&data(512, 1), block.data());
    }

    // TODO Test other locking behaviors, i.e. loading while loaded, removing while loaded, ...
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
        $crate::_instantiate_highlevel_blockstore_tests!(@module data, $target, $tokio_test_args,
            test_writeAndReadImmediately,
            test_writeAndReadAfterLoading,
        );
        $crate::_instantiate_highlevel_blockstore_tests!(@module overwrite, $target, $tokio_test_args,
            test_whenOverwritingWhileLoaded_thenBlocks,
            test_whenOverwritingWhileLoaded_thenSuccessfullyOverwrites,
        );

        // TODO Test Block::block_id()
        // TODO Test Block::data() and data_mut() return the same data after loading
        // TODO Test Block::flush()
        // TODO For  Block::flush(), test other operations with flushing inbetween
    };
}
