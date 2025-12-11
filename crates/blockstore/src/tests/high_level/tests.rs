#![allow(non_snake_case)]

//! This module contains common test cases for the high level [crate::LockingBlockStore] API.
//! It implements most tests by building a adapter to implement the low level [BlockStore] API
//! for [crate::LockingBlockStore], and then uses [super::super::low_level] to run the common low level
//! tests on [crate::LockingBlockStore] as well. On top of that, we add some tests that are
//! specific to [crate::LockingBlockStore].

use byte_unit::Byte;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use super::HLFixture;
use crate::{Block as _, BlockStore, tests::utils::data, utils::RemoveResult};
use cryfs_utils::{async_drop::AsyncDropGuard, testutils::asserts::assert_data_range_eq};

// TODO Other functions to test? e.g. Block::remove(self) instead of just BlockStore::remove(&self, blockid)

/// This macro instantiates all LockingBlockStore tests for a given blockstore.
/// See [crate::tests::low_level::LLFixture] and [HLFixture] for how to invoke it.
#[macro_export]
macro_rules! instantiate_highlevel_blockstore_specific_tests {
    ($target: ty) => {
        $crate::instantiate_highlevel_blockstore_specific_tests!($target, ());
    };
    ($target: ty, $tokio_test_args: tt) => {
        // And run some additional tests for APIs only we have
        $crate::_instantiate_highlevel_blockstore_specific_tests!(@module create, $target, $tokio_test_args,
            test_twoCreatedBlocksHaveDifferentIds
        );
        // try_create is tested through the low_level tests
        // load is tested through the low_level tests
        // overwrite (=store) is tested through the low_level tests
        $crate::_instantiate_highlevel_blockstore_specific_tests!(@module remove, $target, $tokio_test_args,
            // The low_level tests have further test cases for `remove`
            test_canRemoveAModifiedBlock,
        );
        // num_blocks is tested through the low_level tests
        // all_blocks is tested through the low_level tests
        $crate::_instantiate_highlevel_blockstore_specific_tests!(@module resize, $target, $tokio_test_args,
            test_givenZeroSizeBlock_whenResizingToBeLarger_thenSucceeds,
            test_givenZeroSizeBlock_whenResizingToBeLarger_thenBlockIsStillUsable,
            test_givenNonzeroSizeBlock_whenResizingToBeLarger_thenSucceeds,
            test_givenNonzeroSizeBlock_whenResizingToBeLarger_thenBlockIsStillUsable,
            test_givenNonzeroSizeBlock_whenResizingToBeSmaller_thenSucceeds,
            test_givenNonzeroSizeBlock_whenResizingToBeSmaller_thenBlockIsStillUsable,
            test_givenNonzeroSizeBlock_whenResizingToBeZero_thenSucceeds,
            test_givenNonzeroSizeBlock_whenResizingToBeZero_thenBlockIsStillUsable,
        );
        $crate::_instantiate_highlevel_blockstore_specific_tests!(@module data, $target, $tokio_test_args,
            test_writeAndReadImmediately,
            test_writeAndReadAfterLoading,
        );
        $crate::_instantiate_highlevel_blockstore_specific_tests!(@module overwrite, $target, $tokio_test_args,
            test_whenOverwritingWhileLoaded_thenBlocks,
            test_whenOverwritingWhileLoaded_thenSuccessfullyOverwrites,
        );
        $crate::_instantiate_highlevel_blockstore_specific_tests!(@module usable_block_size_from_physical_block_size, $target, $tokio_test_args,
            test_usableToPhysicalToUsable,
            test_physicalToUsableToPhysical,
        );

        // TODO Test Block::block_id()
        // TODO Test Block::data() and data_mut() return the same data after loading
        // TODO Test Block::flush()
        // TODO For  Block::flush(), test other operations with flushing inbetween
    };
}

#[macro_export]
macro_rules! _instantiate_highlevel_blockstore_specific_tests {
    (@module $module_name: ident, $target: ty, $tokio_test_args: tt $(, $test_cases: ident)* $(,)?) => {
        mod $module_name {
            use super::*;

            $crate::_instantiate_highlevel_blockstore_specific_tests!(@module_impl $module_name, $target, $tokio_test_args $(, $test_cases)*);
        }
    };
    (@module_impl $module_name: ident, $target: ty, $tokio_test_args: tt) => {
    };
    (@module_impl $module_name: ident, $target: ty, $tokio_test_args: tt, $head_test_case: ident $(, $tail_test_cases: ident)*) => {
        #[tokio::test$tokio_test_args]
        #[allow(non_snake_case)]
        async fn $head_test_case() {
            let fixture = <$target as $crate::tests::high_level::HLFixture>::new();
            $crate::tests::high_level::tests::$module_name::$head_test_case(fixture).await;
        }
        $crate::_instantiate_highlevel_blockstore_specific_tests!(@module_impl $module_name, $target, $tokio_test_args $(, $tail_test_cases)*);
    };
}

// TODO Send + Sync + Debug still needed?
async fn assert_block_is_usable<B: BlockStore + Send + Sync + Debug>(
    store: &AsyncDropGuard<B>,
    mut block: B::Block,
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

    pub async fn test_twoCreatedBlocksHaveDifferentIds(mut f: impl HLFixture) {
        let mut store = f.store().await;
        let first = store.create(&data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let second = store.create(&data(1024, 1)).await.unwrap();
        assert_ne!(first, second);
        f.yield_fixture(&store).await;

        store.async_drop().await.unwrap();
    }

    // TODO Test block exists and has correct data after creation
    // TODO Test creating empty blocks, both on empty and non-empty block store (see low level tests)
    // TODO Make sure all tests have an afterLoading variant
}

pub mod remove {
    use super::*;

    pub async fn test_canRemoveAModifiedBlock(mut f: impl HLFixture) {
        let mut store = f.store().await;
        let blockid = store.create(&data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;
        let mut block = store.load(blockid).await.unwrap().unwrap();

        block.data_mut().copy_from_slice(&data(1024, 1));

        std::mem::drop(block);
        f.yield_fixture(&store).await;

        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove_by_id(&blockid).await.unwrap()
        );
        f.yield_fixture(&store).await;

        store.async_drop().await.unwrap();
    }
}

pub mod resize {
    use super::*;

    pub async fn test_givenZeroSizeBlock_whenResizingToBeLarger_thenSucceeds(
        mut f: impl HLFixture,
    ) {
        let mut store = f.store().await;
        let blockid = store.create(&data(0, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let mut block = store.load(blockid).await.unwrap().unwrap();

        block.resize(1024).await;
        assert_eq!(1024, block.data().len());

        std::mem::drop(block);
        f.yield_fixture(&store).await;

        store.async_drop().await.unwrap();
    }

    pub async fn test_givenZeroSizeBlock_whenResizingToBeLarger_thenBlockIsStillUsable(
        mut f: impl HLFixture,
    ) {
        let mut store = f.store().await;
        let blockid = store.create(&data(0, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let mut block = store.load(blockid).await.unwrap().unwrap();

        block.resize(1024).await;
        assert_block_is_usable(&store, block).await;

        f.yield_fixture(&store).await;

        store.async_drop().await.unwrap();
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeLarger_thenSucceeds(
        mut f: impl HLFixture,
    ) {
        let mut store = f.store().await;
        let blockid = store.create(&data(100, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let mut block = store.load(blockid).await.unwrap().unwrap();

        block.resize(1024).await;
        assert_eq!(1024, block.data().len());

        std::mem::drop(block);
        f.yield_fixture(&store).await;

        store.async_drop().await.unwrap();
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeLarger_thenBlockIsStillUsable(
        mut f: impl HLFixture,
    ) {
        let mut store = f.store().await;
        let blockid = store.create(&data(100, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let mut block = store.load(blockid).await.unwrap().unwrap();

        block.resize(1024).await;
        assert_block_is_usable(&store, block).await;

        f.yield_fixture(&store).await;

        store.async_drop().await.unwrap();
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeSmaller_thenSucceeds(
        mut f: impl HLFixture,
    ) {
        let mut store = f.store().await;
        let blockid = store.create(&data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let mut block = store.load(blockid).await.unwrap().unwrap();

        block.resize(100).await;
        assert_eq!(100, block.data().len());

        std::mem::drop(block);
        f.yield_fixture(&store).await;

        store.async_drop().await.unwrap();
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeSmaller_thenBlockIsStillUsable(
        mut f: impl HLFixture,
    ) {
        let mut store = f.store().await;
        let blockid = store.create(&data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let mut block = store.load(blockid).await.unwrap().unwrap();

        block.resize(100).await;
        assert_block_is_usable(&store, block).await;

        f.yield_fixture(&store).await;

        store.async_drop().await.unwrap();
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeZero_thenSucceeds(
        mut f: impl HLFixture,
    ) {
        let mut store = f.store().await;
        let blockid = store.create(&data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let mut block = store.load(blockid).await.unwrap().unwrap();

        block.resize(0).await;
        assert_eq!(0, block.data().len());

        std::mem::drop(block);
        f.yield_fixture(&store).await;

        store.async_drop().await.unwrap();
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeZero_thenBlockIsStillUsable(
        mut f: impl HLFixture,
    ) {
        let mut store = f.store().await;
        let blockid = store.create(&data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let mut block = store.load(blockid).await.unwrap().unwrap();

        block.resize(0).await;
        assert_block_is_usable(&store, block).await;
        f.yield_fixture(&store).await;

        store.async_drop().await.unwrap();
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

    pub async fn test_writeAndReadImmediately(mut f: impl HLFixture) {
        for data_range in DATA_RANGES {
            let mut store = f.store().await;

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

            store.async_drop().await.unwrap();
        }
    }

    pub async fn test_writeAndReadAfterLoading(mut f: impl HLFixture) {
        for data_range in DATA_RANGES {
            let mut store = f.store().await;

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

            store.async_drop().await.unwrap();
        }
    }
}

pub mod overwrite {
    use super::*;

    pub async fn test_whenOverwritingWhileLoaded_thenBlocks(mut f: impl HLFixture) {
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

        Arc::into_inner(store).unwrap().async_drop().await.unwrap();
    }

    pub async fn test_whenOverwritingWhileLoaded_thenSuccessfullyOverwrites(mut f: impl HLFixture) {
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

        drop(block);
        Arc::into_inner(store).unwrap().async_drop().await.unwrap();
    }

    // TODO Test other locking behaviors, i.e. loading while loaded, removing while loaded, ...
}

pub mod usable_block_size_from_physical_block_size {
    use super::*;

    pub async fn test_physicalToUsableToPhysical(mut f: impl HLFixture) {
        let mut store = f.store().await;

        let physical = Byte::from_u64(100);
        let usable = store
            .overhead()
            .usable_block_size_from_physical_block_size(physical)
            .unwrap();
        assert!(physical >= usable);
        assert_eq!(
            physical,
            store
                .overhead()
                .physical_block_size_from_usable_block_size(usable)
        );

        store.async_drop().await.unwrap();
    }

    pub async fn test_usableToPhysicalToUsable(mut f: impl HLFixture) {
        let mut store = f.store().await;

        let usable = Byte::from_u64(100);
        let physical = store
            .overhead()
            .physical_block_size_from_usable_block_size(usable);
        assert!(physical >= usable);
        assert_eq!(
            usable,
            store
                .overhead()
                .usable_block_size_from_physical_block_size(physical)
                .unwrap()
        );

        store.async_drop().await.unwrap();
    }
}
