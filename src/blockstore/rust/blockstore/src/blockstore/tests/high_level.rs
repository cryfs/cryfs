#![allow(non_snake_case)]

use std::fmt::Debug;
use futures::{TryStreamExt, StreamExt};

use crate::blockstore::high_level::{LockingBlockStore, Block, RemoveResult, TryCreateResult};
use crate::blockstore::low_level::BlockStore;
use crate::blockstore::BlockId;
use crate::data::Data;
use crate::utils::async_drop::SyncDrop;

use crate::blockstore::tests::{blockid, data, Fixture};
use crate::utils::testutils::assert_unordered_vec_eq;

// TODO Go through tests and make sure we have versions that flush. Otherwise, we're just testing the cache.

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

async fn assert_block_is_usable<B: BlockStore + Send + Sync + Debug>(store: &LockingBlockStore<B>, mut block: Block<B>) {
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

    pub async fn test_blockIsNotLoadableAfterRemoving(mut f: impl LockingBlockStoreFixture) {
        let store = f.store();
        let blockid = store.create(&data(1024, 0)).await.unwrap();
        assert!(store.load(blockid).await.unwrap().is_some());

        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&blockid).await.unwrap(),
        );
        assert!(store.load(blockid).await.unwrap().is_none());
    }

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

    // TODO Test removing when there are still entries left
    // TODO Test removing nonexisting block
}

pub mod num_blocks {
    use super::*;

    pub async fn test_whenAddingBlocks_thenNumBlocksIsCorrect(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store();
        assert_eq!(0, store.num_blocks().await.unwrap());
        store.create(&data(1024, 0)).await.unwrap();
        assert_eq!(1, store.num_blocks().await.unwrap());
        store.create(&data(1024, 0)).await.unwrap();
        assert_eq!(2, store.num_blocks().await.unwrap());
        store.create(&data(1024, 0)).await.unwrap();
        assert_eq!(3, store.num_blocks().await.unwrap());
    }

    pub async fn test_whenRemovingBlocks_thenNumBlocksIsCorrect(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store();
        let block1 = store.create(&data(1024, 0)).await.unwrap();
        let block2 = store.create(&data(1024, 0)).await.unwrap();
        let block3 = store.create(&data(1024, 0)).await.unwrap();
        let block4 = store.create(&data(1024, 0)).await.unwrap();
        assert_eq!(4, store.num_blocks().await.unwrap());
        remove_block(&store, &block2).await;
        assert_eq!(3, store.num_blocks().await.unwrap());
        remove_block(&store, &block4).await;
        assert_eq!(2, store.num_blocks().await.unwrap());
        remove_block(&store, &block1).await;
        assert_eq!(1, store.num_blocks().await.unwrap());
        remove_block(&store, &block3).await;
        assert_eq!(0, store.num_blocks().await.unwrap());
    }

    pub async fn test_whenRemovingNonExistingBlocks_thenNumBlocksIsCorrect(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store();
        create_block(&store, &blockid(1), &data(1024, 0)).await;
        create_block(&store, &blockid(2), &data(1024, 0)).await;
        create_block(&store, &blockid(3), &data(1024, 0)).await;
        create_block(&store, &blockid(4), &data(1024, 0)).await;
        assert_eq!(4, store.num_blocks().await.unwrap());

        assert_eq!(
            RemoveResult::NotRemovedBecauseItDoesntExist,
            store.remove(&blockid(0)).await.unwrap()
        );
        assert_eq!(4, store.num_blocks().await.unwrap());
    }

    // TODO More tests?
}

pub mod all_blocks {
    use super::*;

    async fn call_all_blocks<B: BlockStore + Send + Sync + Debug + 'static>(
        store: &LockingBlockStore<B>,
    ) -> Vec<BlockId> {
        store.all_blocks().await.unwrap().try_collect().await.unwrap()
    }

    pub async fn test_givenEmptyBlockStore_whenCallingAllBlocks_thenReturnsCorrectResult(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store();
        assert_unordered_vec_eq(vec![], call_all_blocks(&store).await);
    }

    pub async fn test_givenBlockStoreWithOneNonEmptyBlock_whenCallingAllBlocks_thenReturnsCorrectResult(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store();
        create_block(&store, &blockid(0), &data(1024, 0)).await;
        assert_unordered_vec_eq(vec![blockid(0)], call_all_blocks(&store).await);
    }

    pub async fn test_givenBlockStoreWithOneEmptyBlock_whenCallingAllBlocks_thenReturnsCorrectResult(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store();
        create_block(&store, &blockid(0), &data(0, 0)).await;
        assert_unordered_vec_eq(vec![blockid(0)], call_all_blocks(&store).await);
    }

    pub async fn test_givenBlockStoreWithTwoBlocks_whenCallingAllBlocks_thenReturnsCorrectResult(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store();
        create_block(&store, &blockid(0), &data(1024, 0)).await;
        create_block(&store, &blockid(1), &data(1024, 0)).await;
        assert_unordered_vec_eq(
            vec![blockid(0), blockid(1)],
            call_all_blocks(&store).await,
        );
    }

    pub async fn test_givenBlockStoreWithThreeBlocks_whenCallingAllBlocks_thenReturnsCorrectResult(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store();
        create_block(&store, &blockid(0), &data(1024, 0)).await;
        create_block(&store, &blockid(1), &data(1024, 0)).await;
        create_block(&store, &blockid(2), &data(1024, 0)).await;
        assert_unordered_vec_eq(
            vec![blockid(0), blockid(1), blockid(2)],
            call_all_blocks(&store).await,
        );
    }

    pub async fn test_afterRemovingBlock_whenCallingAllBlocks_doesntListRemovedBlocks(
        mut f: impl LockingBlockStoreFixture,
    ) {
        let store = f.store();
        create_block(&store, &blockid(0), &data(1024, 0)).await;
        create_block(&store, &blockid(1), &data(1024, 0)).await;
        create_block(&store, &blockid(2), &data(1024, 0)).await;

        remove_block(&store, &blockid(1)).await;
        assert_unordered_vec_eq(
            vec![blockid(0), blockid(2)],
            call_all_blocks(&store).await,
        );

        remove_block(&store, &blockid(2)).await;
        assert_unordered_vec_eq(vec![blockid(0)], call_all_blocks(&store).await);

        remove_block(&store, &blockid(0)).await;
        assert_unordered_vec_eq(vec![], call_all_blocks(&store).await);
    }
}

pub mod resize {
    use super::*;

    pub async fn test_givenZeroSizeBlock_whenResizingToBeLarger_thenSucceeds(mut f: impl LockingBlockStoreFixture) {
        let store = f.store();
        let blockid = store.create(&data(0, 0)).await.unwrap();
        let mut block = store.load(blockid).await.unwrap().unwrap();
        block.resize(1024).await;
        assert_eq!(1024, block.data().len());
    }

    pub async fn test_givenZeroSizeBlock_whenResizingToBeLarger_thenBlockIsStillUsable(mut f: impl LockingBlockStoreFixture) {
        let store = f.store();
        let blockid = store.create(&data(0, 0)).await.unwrap();
        let mut block = store.load(blockid).await.unwrap().unwrap();
        block.resize(1024).await;
        assert_block_is_usable(&store, block).await;
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeLarger_thenSucceeds(mut f: impl LockingBlockStoreFixture) {
        let store = f.store();
        let blockid = store.create(&data(100, 0)).await.unwrap();
        let mut block = store.load(blockid).await.unwrap().unwrap();
        block.resize(1024).await;
        assert_eq!(1024, block.data().len());
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeLarger_thenBlockIsStillUsable(mut f: impl LockingBlockStoreFixture) {
        let store = f.store();
        let blockid = store.create(&data(100, 0)).await.unwrap();
        let mut block = store.load(blockid).await.unwrap().unwrap();
        block.resize(1024).await;
        assert_block_is_usable(&store, block).await;
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeSmaller_thenSucceeds(mut f: impl LockingBlockStoreFixture) {
        let store = f.store();
        let blockid = store.create(&data(1024, 0)).await.unwrap();
        let mut block = store.load(blockid).await.unwrap().unwrap();
        block.resize(100).await;
        assert_eq!(100, block.data().len());
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeSmaller_thenBlockIsStillUsable(mut f: impl LockingBlockStoreFixture) {
        let store = f.store();
        let blockid = store.create(&data(1024, 0)).await.unwrap();
        let mut block = store.load(blockid).await.unwrap().unwrap();
        block.resize(100).await;
        assert_block_is_usable(&store, block).await;
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeZero_thenSucceeds(mut f: impl LockingBlockStoreFixture) {
        let store = f.store();
        let blockid = store.create(&data(1024, 0)).await.unwrap();
        let mut block = store.load(blockid).await.unwrap().unwrap();
        block.resize(0).await;
        assert_eq!(0, block.data().len());
    }

    pub async fn test_givenNonzeroSizeBlock_whenResizingToBeZero_thenBlockIsStillUsable(mut f: impl LockingBlockStoreFixture) {
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
        $crate::_instantiate_highlevel_blockstore_tests!(@module create, $target, $tokio_test_args,
            test_twoCreatedBlocksHaveDifferentIds
        );
        $crate::_instantiate_highlevel_blockstore_tests!(@module try_create, $target, $tokio_test_args,
            // test_givenNonEmptyBlockStore_whenCallingTryCreateOnExistingBlock_thenFails,
            // test_givenNonEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_withEmptyData_thenSucceeds,
            // test_givenNonEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_withNonEmptyData_thenSucceeds,
            // test_givenEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_withEmptyData_thenSucceeds,
            // test_givenEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_withNonEmptyData_thenSucceeds,
        );
        $crate::_instantiate_highlevel_blockstore_tests!(@module load, $target, $tokio_test_args,
            // test_givenNonEmptyBlockStore_whenLoadExistingBlock_withEmptyData_thenSucceeds,
            // test_givenNonEmptyBlockStore_whenLoadExistingBlock_withNonEmptyData_thenSucceeds,
            // test_givenNonEmptyBlockStore_whenLoadNonexistingBlock_thenFails,
            // test_givenEmptyBlockStore_whenLoadNonexistingBlock_thenFails,
        );
        $crate::_instantiate_highlevel_blockstore_tests!(@module store, $target, $tokio_test_args,
            // test_givenEmptyBlockStore_whenStoringNonExistingBlock_withEmptyData_thenSucceeds,
            // test_givenEmptyBlockStore_whenStoringNonExistingBlock_withNonEmptyData_thenSucceeds,
            // test_givenNonEmptyBlockStore_whenStoringNonExistingBlock_withEmptyData_thenSucceeds,
            // test_givenNonEmptyBlockStore_whenStoringNonExistingBlock_withNonEmptyData_thenSucceeds,
            // test_givenNonEmptyBlockStore_whenStoringExistingBlock_withEmptyData_thenSucceeds,
            // test_givenNonEmptyBlockStore_whenStoringExistingBlock_withNonEmptyData_thenSucceeds,
        );
        $crate::_instantiate_highlevel_blockstore_tests!(@module remove, $target, $tokio_test_args,
            test_blockIsNotLoadableAfterRemoving,
            test_canRemoveAModifiedBlock,
            // test_givenOtherwiseEmptyBlockStore_whenRemovingEmptyBlock_thenBlockIsNotLoadableAnymore,
            // test_givenOtherwiseEmptyBlockStore_whenRemovingNonEmptyBlock_thenBlockIsNotLoadableAnymore,
            // test_givenNonEmptyBlockStore_whenRemovingEmptyBlock_thenBlockIsNotLoadableAnymore,
            // test_givenNonEmptyBlockStore_whenRemovingNonEmptyBlock_thenBlockIsNotLoadableAnymore,
            // test_givenEmptyBlockStore_whenRemovingNonexistingBlock_thenFails,
            // test_givenNonEmptyBlockStore_whenRemovingNonexistingBlock_thenFails,
        );
        $crate::_instantiate_highlevel_blockstore_tests!(@module num_blocks, $target, $tokio_test_args,
            test_whenAddingBlocks_thenNumBlocksIsCorrect,
            test_whenRemovingBlocks_thenNumBlocksIsCorrect,
            test_whenRemovingNonExistingBlocks_thenNumBlocksIsCorrect,
            // test_givenEmptyBlockStore_whenCallingNumBlocks_thenReturnsCorrectResult,
            // test_afterStoringBlocks_whenCallingNumBlocks_thenReturnsCorrectResult,
            // test_afterTryCreatingBlocks_whenCallingNumBlocks_thenReturnsCorrectResult,
            // test_afterRemovingBlocks_whenCallingNumBlocks_thenReturnsCorrectResult,
        );
        $crate::_instantiate_highlevel_blockstore_tests!(@module all_blocks, $target, $tokio_test_args,
            test_givenEmptyBlockStore_whenCallingAllBlocks_thenReturnsCorrectResult,
            test_givenBlockStoreWithOneNonEmptyBlock_whenCallingAllBlocks_thenReturnsCorrectResult,
            test_givenBlockStoreWithOneEmptyBlock_whenCallingAllBlocks_thenReturnsCorrectResult,
            test_givenBlockStoreWithTwoBlocks_whenCallingAllBlocks_thenReturnsCorrectResult,
            test_givenBlockStoreWithThreeBlocks_whenCallingAllBlocks_thenReturnsCorrectResult,
            test_afterRemovingBlock_whenCallingAllBlocks_doesntListRemovedBlocks,
        );
        $crate::_instantiate_highlevel_blockstore_tests!(@module exists, $target, $tokio_test_args,
            // test_givenEmptyBlockStore_whenCallingExistsOnNonExistingBlock_thenReturnsFalse,
            // test_givenNonEmptyBlockStore_whenCallingExistsOnNonExistingBlock_thenReturnsFalse,
            // test_givenNonEmptyBlockStore_whenCallingExistsOnExistingBlock_thenReturnsTrue,
        );
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
