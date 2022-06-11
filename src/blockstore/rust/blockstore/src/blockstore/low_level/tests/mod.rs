#![allow(non_snake_case)]

use anyhow::Result;
use futures::stream::StreamExt;
use rand::{rngs::StdRng, RngCore, SeedableRng};

use super::{
    BlockId, BlockStore, BlockStoreDeleter, BlockStoreReader, BlockStoreWriter, RemoveResult,
    TryCreateResult,
};
use crate::data::Data;

// TODO Test try_create_optimized(), store_optimized()

/// By writing a [Fixture] implementation and using the [instantiate_blockstore_tests] macro,
/// our suite of block store tests is instantiated for a given block store.
///
/// The fixture is kept alive for as long as the test runs, so it can hold RAII resources
/// required by the block store.
pub trait Fixture {
    type ConcreteBlockStore: BlockStore;

    fn new() -> Self;
    fn store(&mut self) -> &mut Self::ConcreteBlockStore;
}

fn blockid(seed: u64) -> BlockId {
    BlockId::from_slice(data(16, seed).as_ref()).unwrap()
}

fn data(size: usize, seed: u64) -> Data {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut res = vec![0; size];
    rng.fill_bytes(&mut res);
    res.into()
}

pub mod try_create {
    use super::*;

    pub async fn test_givenNonEmptyBlockStore_whenCallingTryCreateOnExistingBlock_thenFails(
        mut f: impl Fixture,
    ) {
        let store = f.store();

        store.store(&blockid(1), &data(1024, 0)).await.unwrap();
        let status = store
            .try_create(&blockid(1), data(1024, 1).as_ref())
            .await
            .unwrap();
        assert_eq!(
            TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists,
            status
        );

        assert_eq!(Some(data(1024, 0)), store.load(&blockid(1)).await.unwrap());
    }

    pub async fn test_givenNonEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_withNonEmptyData_thenSucceeds(
        mut f: impl Fixture,
    ) {
        let store = f.store();

        store.store(&blockid(1), &data(1024, 0)).await.unwrap();
        let status = store
            .try_create(&blockid(2), data(1024, 1).as_ref())
            .await
            .unwrap();
        assert_eq!(TryCreateResult::SuccessfullyCreated, status);

        assert_eq!(Some(data(1024, 1)), store.load(&blockid(2)).await.unwrap());
    }

    pub async fn test_givenNonEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_withEmptyData_thenSucceeds(
        mut f: impl Fixture,
    ) {
        let store = f.store();

        store.store(&blockid(1), &data(1024, 0)).await.unwrap();
        let status = store
            .try_create(&blockid(2), data(0, 1).as_ref())
            .await
            .unwrap();
        assert_eq!(TryCreateResult::SuccessfullyCreated, status);

        assert_eq!(Some(data(0, 1)), store.load(&blockid(2)).await.unwrap());
    }

    pub async fn test_givenEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_withNonEmptyData_thenSucceeds(
        mut f: impl Fixture,
    ) {
        let store = f.store();

        let status = store
            .try_create(&blockid(1), data(1024, 1).as_ref())
            .await
            .unwrap();
        assert_eq!(TryCreateResult::SuccessfullyCreated, status);

        assert_eq!(Some(data(1024, 1)), store.load(&blockid(1)).await.unwrap());
    }

    pub async fn test_givenEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_withEmptyData_thenSucceeds(
        mut f: impl Fixture,
    ) {
        let store = f.store();

        let status = store
            .try_create(&blockid(1), data(0, 1).as_ref())
            .await
            .unwrap();
        assert_eq!(TryCreateResult::SuccessfullyCreated, status);

        assert_eq!(Some(data(0, 1)), store.load(&blockid(1)).await.unwrap());
    }
}

pub mod load {
    use super::*;

    pub async fn test_givenNonEmptyBlockStore_whenLoadExistingBlock_withNonEmptyData_thenSucceeds(
        mut f: impl Fixture,
    ) {
        let store = f.store();

        store
            .store(&blockid(0), data(1024, 0).as_ref())
            .await
            .unwrap();
        store
            .store(&blockid(1), data(1024, 1).as_ref())
            .await
            .unwrap();
        let loaded = store.load(&blockid(1)).await.unwrap();
        assert_eq!(Some(data(1024, 1)), loaded);
    }

    pub async fn test_givenNonEmptyBlockStore_whenLoadExistingBlock_withEmptyData_thenSucceeds(
        mut f: impl Fixture,
    ) {
        let store = f.store();

        store
            .store(&blockid(0), data(1024, 0).as_ref())
            .await
            .unwrap();
        store.store(&blockid(1), data(0, 1).as_ref()).await.unwrap();
        let loaded = store.load(&blockid(1)).await.unwrap();
        assert_eq!(Some(data(0, 1)), loaded);
    }

    pub async fn test_givenNonEmptyBlockStore_whenLoadNonexistingBlock_thenFails(
        mut f: impl Fixture,
    ) {
        let store = f.store();

        store
            .store(&blockid(0), data(1024, 0).as_ref())
            .await
            .unwrap();
        store
            .store(&blockid(1), data(1024, 1).as_ref())
            .await
            .unwrap();
        let loaded = store.load(&blockid(2)).await.unwrap();
        assert_eq!(None, loaded);
    }

    pub async fn test_givenEmptyBlockStore_whenLoadNonexistingBlock_thenFails(mut f: impl Fixture) {
        let store = f.store();

        let loaded = store.load(&blockid(1)).await.unwrap();
        assert_eq!(None, loaded);
    }
}

pub mod store {
    use super::*;

    pub async fn test_givenEmptyBlockStore_whenStoringNonExistingBlock_withNonEmptyData_thenSucceeds(
        mut f: impl Fixture,
    ) {
        let store = f.store();

        store
            .store(&blockid(1), data(1024, 1).as_ref())
            .await
            .unwrap();

        assert_eq!(Some(data(1024, 1)), store.load(&blockid(1)).await.unwrap());
    }

    pub async fn test_givenEmptyBlockStore_whenStoringNonExistingBlock_withEmptyData_thenSucceeds(
        mut f: impl Fixture,
    ) {
        let store = f.store();

        store.store(&blockid(1), data(0, 1).as_ref()).await.unwrap();

        assert_eq!(Some(data(0, 1)), store.load(&blockid(1)).await.unwrap());
    }

    pub async fn test_givenNonEmptyBlockStore_whenStoringNonExistingBlock_withNonEmptyData_thenSucceeds(
        mut f: impl Fixture,
    ) {
        let store = f.store();

        store
            .store(&blockid(1), data(1024, 0).as_ref())
            .await
            .unwrap();
        store
            .store(&blockid(2), data(1024, 1).as_ref())
            .await
            .unwrap();

        assert_eq!(Some(data(1024, 1)), store.load(&blockid(2)).await.unwrap());
    }

    pub async fn test_givenNonEmptyBlockStore_whenStoringNonExistingBlock_withEmptyData_thenSucceeds(
        mut f: impl Fixture,
    ) {
        let store = f.store();

        store
            .store(&blockid(1), data(1024, 0).as_ref())
            .await
            .unwrap();
        store.store(&blockid(2), data(0, 1).as_ref()).await.unwrap();

        assert_eq!(Some(data(0, 1)), store.load(&blockid(2)).await.unwrap());
    }

    pub async fn test_givenNonEmptyBlockStore_whenStoringExistingBlock_withNonEmptyData_thenSucceeds(
        mut f: impl Fixture,
    ) {
        let store = f.store();

        store
            .store(&blockid(1), data(1024, 0).as_ref())
            .await
            .unwrap();
        store
            .store(&blockid(2), data(1024, 1).as_ref())
            .await
            .unwrap();
        store
            .store(&blockid(2), data(1024, 2).as_ref())
            .await
            .unwrap();

        // Test the unrelated block still has the old value
        assert_eq!(Some(data(1024, 0)), store.load(&blockid(1)).await.unwrap());

        // Check it got successfully overwritten
        assert_eq!(Some(data(1024, 2)), store.load(&blockid(2)).await.unwrap());
    }

    pub async fn test_givenNonEmptyBlockStore_whenStoringExistingBlock_withEmptyData_thenSucceeds(
        mut f: impl Fixture,
    ) {
        let store = f.store();

        store
            .store(&blockid(1), data(1024, 0).as_ref())
            .await
            .unwrap();
        store
            .store(&blockid(2), data(1024, 1).as_ref())
            .await
            .unwrap();
        store.store(&blockid(2), data(0, 2).as_ref()).await.unwrap();

        // Test the unrelated block still has the old value
        assert_eq!(Some(data(1024, 0)), store.load(&blockid(1)).await.unwrap());

        // Check it got successfully overwritten
        assert_eq!(Some(data(0, 2)), store.load(&blockid(2)).await.unwrap());
    }
}

pub mod remove {
    use super::*;

    pub async fn test_givenOtherwiseEmptyBlockStore_whenRemovingNonEmptyBlock_thenBlockIsNotLoadableAnymore(
        mut f: impl Fixture,
    ) {
        let store = f.store();

        store
            .store(&blockid(1), data(1024, 1).as_ref())
            .await
            .unwrap();
        assert!(store.load(&blockid(1)).await.unwrap().is_some());
        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&blockid(1)).await.unwrap()
        );
        assert!(store.load(&blockid(1)).await.unwrap().is_none());
    }

    pub async fn test_givenOtherwiseEmptyBlockStore_whenRemovingEmptyBlock_thenBlockIsNotLoadableAnymore(
        mut f: impl Fixture,
    ) {
        let store = f.store();

        store.store(&blockid(1), data(0, 1).as_ref()).await.unwrap();
        assert!(store.load(&blockid(1)).await.unwrap().is_some());
        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&blockid(1)).await.unwrap()
        );
        assert!(store.load(&blockid(1)).await.unwrap().is_none());
    }

    pub async fn test_givenNonEmptyBlockStore_whenRemovingNonEmptyBlock_thenBlockIsNotLoadableAnymore(
        mut f: impl Fixture,
    ) {
        let store = f.store();

        store
            .store(&blockid(1), data(1024, 2).as_ref())
            .await
            .unwrap();

        store
            .store(&blockid(2), data(1024, 1).as_ref())
            .await
            .unwrap();
        assert!(store.load(&blockid(2)).await.unwrap().is_some());
        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&blockid(2)).await.unwrap()
        );
        assert!(store.load(&blockid(2)).await.unwrap().is_none());
    }

    pub async fn test_givenNonEmptyBlockStore_whenRemovingEmptyBlock_thenBlockIsNotLoadableAnymore(
        mut f: impl Fixture,
    ) {
        let store = f.store();

        store
            .store(&blockid(1), data(1024, 2).as_ref())
            .await
            .unwrap();

        store.store(&blockid(2), data(0, 1).as_ref()).await.unwrap();
        assert!(store.load(&blockid(2)).await.unwrap().is_some());
        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&blockid(2)).await.unwrap()
        );
        assert!(store.load(&blockid(2)).await.unwrap().is_none());
    }

    pub async fn test_givenEmptyBlockStore_whenRemovingNonexistingBlock_thenFails(
        mut f: impl Fixture,
    ) {
        let store = f.store();

        assert_eq!(
            RemoveResult::NotRemovedBecauseItDoesntExist,
            store.remove(&blockid(1)).await.unwrap()
        );
        assert!(store.load(&blockid(1)).await.unwrap().is_none());
    }

    pub async fn test_givenNonEmptyBlockStore_whenRemovingNonexistingBlock_thenFails(
        mut f: impl Fixture,
    ) {
        let store = f.store();

        store
            .store(&blockid(1), data(1024, 2).as_ref())
            .await
            .unwrap();

        assert_eq!(
            RemoveResult::NotRemovedBecauseItDoesntExist,
            store.remove(&blockid(2)).await.unwrap()
        );
        assert!(store.load(&blockid(2)).await.unwrap().is_none());
    }
}

pub mod num_blocks {
    use super::*;

    pub async fn test_givenEmptyBlockStore_whenCallingNumBlocks_thenReturnsCorrectResult(
        mut f: impl Fixture,
    ) {
        let store = f.store();
        assert_eq!(0, store.num_blocks().await.unwrap());
    }

    pub async fn test_afterStoringBlocks_whenCallingNumBlocks_thenReturnsCorrectResult(
        mut f: impl Fixture,
    ) {
        let store = f.store();
        assert_eq!(0, store.num_blocks().await.unwrap());
        store.store(&blockid(0), &data(1024, 0)).await.unwrap();
        assert_eq!(1, store.num_blocks().await.unwrap());
        store.store(&blockid(1), &data(1024, 1)).await.unwrap();
        assert_eq!(2, store.num_blocks().await.unwrap());
        store.store(&blockid(2), &data(0, 2)).await.unwrap();
        assert_eq!(3, store.num_blocks().await.unwrap());
        store.store(&blockid(1), &data(1024, 3)).await.unwrap();
        assert_eq!(3, store.num_blocks().await.unwrap());
    }

    pub async fn test_afterTryCreatingBlocks_whenCallingNumBlocks_thenReturnsCorrectResult(
        mut f: impl Fixture,
    ) {
        let store = f.store();
        assert_eq!(0, store.num_blocks().await.unwrap());
        assert_eq!(
            TryCreateResult::SuccessfullyCreated,
            store.try_create(&blockid(0), &data(1024, 0)).await.unwrap()
        );
        assert_eq!(1, store.num_blocks().await.unwrap());
        assert_eq!(
            TryCreateResult::SuccessfullyCreated,
            store.try_create(&blockid(1), &data(1024, 1)).await.unwrap()
        );
        assert_eq!(2, store.num_blocks().await.unwrap());
        assert_eq!(
            TryCreateResult::SuccessfullyCreated,
            store.try_create(&blockid(2), &data(0, 2)).await.unwrap()
        );
        assert_eq!(3, store.num_blocks().await.unwrap());
        assert_eq!(
            TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists,
            store.try_create(&blockid(1), &data(1024, 3)).await.unwrap()
        );
        assert_eq!(3, store.num_blocks().await.unwrap());
    }

    pub async fn test_afterRemovingBlocks_whenCallingNumBlocks_thenReturnsCorrectResult(
        mut f: impl Fixture,
    ) {
        let store = f.store();
        store.store(&blockid(0), &data(1024, 0)).await.unwrap();
        store.store(&blockid(1), &data(1024, 1)).await.unwrap();
        store.store(&blockid(2), &data(0, 2)).await.unwrap();

        assert_eq!(3, store.num_blocks().await.unwrap());
        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&blockid(0)).await.unwrap()
        );
        assert_eq!(2, store.num_blocks().await.unwrap());
        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&blockid(1)).await.unwrap()
        );
        assert_eq!(1, store.num_blocks().await.unwrap());
        assert_eq!(
            RemoveResult::NotRemovedBecauseItDoesntExist,
            store.remove(&blockid(1)).await.unwrap()
        );
        assert_eq!(1, store.num_blocks().await.unwrap());
        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&blockid(2)).await.unwrap()
        );
        assert_eq!(0, store.num_blocks().await.unwrap());
    }
}

pub mod all_blocks {
    use super::*;

    fn assert_unordered_vec_eq(mut lhs: Vec<BlockId>, mut rhs: Vec<BlockId>) {
        lhs.sort();
        rhs.sort();
        assert_eq!(lhs, rhs);
    }

    async fn call_all_blocks(store: &impl BlockStoreReader) -> Vec<BlockId> {
        store
            .all_blocks()
            .await
            .unwrap()
            .collect::<Vec<Result<BlockId>>>()
            .await
            .into_iter()
            .collect::<Result<Vec<BlockId>>>()
            .unwrap()
    }

    pub async fn test_givenEmptyBlockStore_whenCallingAllBlocks_thenReturnsCorrectResult(
        mut f: impl Fixture,
    ) {
        let store = f.store();
        assert_unordered_vec_eq(vec![], call_all_blocks(store).await);
    }

    pub async fn test_givenBlockStoreWithOneNonEmptyBlock_whenCallingAllBlocks_thenReturnsCorrectResult(
        mut f: impl Fixture,
    ) {
        let store = f.store();
        store.store(&blockid(0), &data(1024, 0)).await.unwrap();
        assert_unordered_vec_eq(vec![blockid(0)], call_all_blocks(store).await);
    }

    pub async fn test_givenBlockStoreWithOneEmptyBlock_whenCallingAllBlocks_thenReturnsCorrectResult(
        mut f: impl Fixture,
    ) {
        let store = f.store();
        store.store(&blockid(0), &data(0, 0)).await.unwrap();
        assert_unordered_vec_eq(vec![blockid(0)], call_all_blocks(store).await);
    }

    pub async fn test_givenBlockStoreWithTwoBlocks_whenCallingAllBlocks_thenReturnsCorrectResult(
        mut f: impl Fixture,
    ) {
        let store = f.store();
        store.store(&blockid(0), &data(1024, 0)).await.unwrap();
        store.store(&blockid(1), &data(1024, 0)).await.unwrap();
        assert_unordered_vec_eq(vec![blockid(0), blockid(1)], call_all_blocks(store).await);
    }

    pub async fn test_givenBlockStoreWithThreeBlocks_whenCallingAllBlocks_thenReturnsCorrectResult(
        mut f: impl Fixture,
    ) {
        let store = f.store();
        store.store(&blockid(0), &data(1024, 0)).await.unwrap();
        store.store(&blockid(1), &data(1024, 0)).await.unwrap();
        store.store(&blockid(2), &data(1024, 0)).await.unwrap();
        assert_unordered_vec_eq(
            vec![blockid(0), blockid(1), blockid(2)],
            call_all_blocks(store).await,
        );
    }

    pub async fn test_afterRemovingBlock_whenCallingAllBlocks_doesntListRemovedBlocks(
        mut f: impl Fixture,
    ) {
        let store = f.store();
        store.store(&blockid(0), &data(1024, 0)).await.unwrap();
        store.store(&blockid(1), &data(1024, 0)).await.unwrap();
        store.store(&blockid(2), &data(1024, 0)).await.unwrap();

        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&blockid(1)).await.unwrap()
        );
        assert_unordered_vec_eq(vec![blockid(0), blockid(2)], call_all_blocks(store).await);

        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&blockid(2)).await.unwrap()
        );
        assert_unordered_vec_eq(vec![blockid(0)], call_all_blocks(store).await);

        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&blockid(0)).await.unwrap()
        );
        assert_unordered_vec_eq(vec![], call_all_blocks(store).await);
    }
}

pub mod exists {
    use super::*;

    pub async fn test_givenEmptyBlockStore_whenCallingExistsOnNonExistingBlock_thenReturnsFalse(
        mut f: impl Fixture,
    ) {
        let store = f.store();
        assert_eq!(false, store.exists(&blockid(0)).await.unwrap());
    }

    pub async fn test_givenNonEmptyBlockStore_whenCallingExistsOnNonExistingBlock_thenReturnsFalse(
        mut f: impl Fixture,
    ) {
        let store = f.store();
        store.store(&blockid(0), &data(1024, 0)).await.unwrap();
        assert_eq!(false, store.exists(&blockid(1)).await.unwrap());
    }

    pub async fn test_givenNonEmptyBlockStore_whenCallingExistsOnExistingBlock_thenReturnsTrue(
        mut f: impl Fixture,
    ) {
        let store = f.store();
        store.store(&blockid(0), &data(1024, 0)).await.unwrap();
        assert_eq!(true, store.exists(&blockid(0)).await.unwrap());
    }
}

#[macro_export]
macro_rules! _instantiate_blockstore_tests {
    (@module $module_name: ident, $target: ty, $tokio_test_args: tt $(, $test_cases: ident)* $(,)?) => {
        mod $module_name {
            use super::*;

            $crate::_instantiate_blockstore_tests!(@module_impl $module_name, $target, $tokio_test_args $(, $test_cases)*);
        }
    };
    (@module_impl $module_name: ident, $target: ty, $tokio_test_args: tt) => {
    };
    (@module_impl $module_name: ident, $target: ty, $tokio_test_args: tt, $head_test_case: ident $(, $tail_test_cases: ident)*) => {
        #[tokio::test$tokio_test_args]
        #[allow(non_snake_case)]
        async fn $head_test_case() {
            let fixture = <$target as $crate::blockstore::low_level::tests::Fixture>::new();
            $crate::blockstore::low_level::tests::$module_name::$head_test_case(fixture).await
        }
        $crate::_instantiate_blockstore_tests!(@module_impl $module_name, $target, $tokio_test_args $(, $tail_test_cases)*);
    };
}

/// This macro instantiates all blockstore tests for a given blockstore.
/// See [Fixture] for how to invoke it.
#[macro_export]
macro_rules! instantiate_blockstore_tests {
    ($target: ty) => {
        $crate::instantiate_blockstore_tests!($target, ());
    };
    ($target: ty, $tokio_test_args: tt) => {
        $crate::_instantiate_blockstore_tests!(@module try_create, $target, $tokio_test_args,
            test_givenNonEmptyBlockStore_whenCallingTryCreateOnExistingBlock_thenFails,
            test_givenNonEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_withEmptyData_thenSucceeds,
            test_givenNonEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_withNonEmptyData_thenSucceeds,
            test_givenEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_withEmptyData_thenSucceeds,
            test_givenEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_withNonEmptyData_thenSucceeds,
        );
        $crate::_instantiate_blockstore_tests!(@module load, $target, $tokio_test_args,
            test_givenNonEmptyBlockStore_whenLoadExistingBlock_withEmptyData_thenSucceeds,
            test_givenNonEmptyBlockStore_whenLoadExistingBlock_withNonEmptyData_thenSucceeds,
            test_givenNonEmptyBlockStore_whenLoadNonexistingBlock_thenFails,
            test_givenEmptyBlockStore_whenLoadNonexistingBlock_thenFails,
        );
        $crate::_instantiate_blockstore_tests!(@module store, $target, $tokio_test_args,
            test_givenEmptyBlockStore_whenStoringNonExistingBlock_withEmptyData_thenSucceeds,
            test_givenEmptyBlockStore_whenStoringNonExistingBlock_withNonEmptyData_thenSucceeds,
            test_givenNonEmptyBlockStore_whenStoringNonExistingBlock_withEmptyData_thenSucceeds,
            test_givenNonEmptyBlockStore_whenStoringNonExistingBlock_withNonEmptyData_thenSucceeds,
            test_givenNonEmptyBlockStore_whenStoringExistingBlock_withEmptyData_thenSucceeds,
            test_givenNonEmptyBlockStore_whenStoringExistingBlock_withNonEmptyData_thenSucceeds,
        );
        $crate::_instantiate_blockstore_tests!(@module remove, $target, $tokio_test_args,
            test_givenOtherwiseEmptyBlockStore_whenRemovingEmptyBlock_thenBlockIsNotLoadableAnymore,
            test_givenOtherwiseEmptyBlockStore_whenRemovingNonEmptyBlock_thenBlockIsNotLoadableAnymore,
            test_givenNonEmptyBlockStore_whenRemovingEmptyBlock_thenBlockIsNotLoadableAnymore,
            test_givenNonEmptyBlockStore_whenRemovingNonEmptyBlock_thenBlockIsNotLoadableAnymore,
            test_givenEmptyBlockStore_whenRemovingNonexistingBlock_thenFails,
            test_givenNonEmptyBlockStore_whenRemovingNonexistingBlock_thenFails,
        );
        $crate::_instantiate_blockstore_tests!(@module num_blocks, $target, $tokio_test_args,
            test_givenEmptyBlockStore_whenCallingNumBlocks_thenReturnsCorrectResult,
            test_afterStoringBlocks_whenCallingNumBlocks_thenReturnsCorrectResult,
            test_afterTryCreatingBlocks_whenCallingNumBlocks_thenReturnsCorrectResult,
            test_afterRemovingBlocks_whenCallingNumBlocks_thenReturnsCorrectResult,
        );
        $crate::_instantiate_blockstore_tests!(@module all_blocks, $target, $tokio_test_args,
            test_givenEmptyBlockStore_whenCallingAllBlocks_thenReturnsCorrectResult,
            test_givenBlockStoreWithOneNonEmptyBlock_whenCallingAllBlocks_thenReturnsCorrectResult,
            test_givenBlockStoreWithOneEmptyBlock_whenCallingAllBlocks_thenReturnsCorrectResult,
            test_givenBlockStoreWithTwoBlocks_whenCallingAllBlocks_thenReturnsCorrectResult,
            test_givenBlockStoreWithThreeBlocks_whenCallingAllBlocks_thenReturnsCorrectResult,
            test_afterRemovingBlock_whenCallingAllBlocks_doesntListRemovedBlocks,
        );
        $crate::_instantiate_blockstore_tests!(@module exists, $target, $tokio_test_args,
            test_givenEmptyBlockStore_whenCallingExistsOnNonExistingBlock_thenReturnsFalse,
            test_givenNonEmptyBlockStore_whenCallingExistsOnNonExistingBlock_thenReturnsFalse,
            test_givenNonEmptyBlockStore_whenCallingExistsOnExistingBlock_thenReturnsTrue,
        );
    };
}
