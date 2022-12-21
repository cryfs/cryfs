#![allow(non_snake_case)]

//! This module contains common test cases for the low level [BlockStore] API

use futures::stream::TryStreamExt;
use std::ops::Deref;

use super::{blockid, data, Fixture};
use crate::{
    low_level::{BlockStoreDeleter, BlockStoreReader, BlockStoreWriter},
    utils::{RemoveResult, TryCreateResult},
    BlockId,
};
use cryfs_utils::testutils::assert_unordered_vec_eq;

pub mod try_create {
    use super::*;

    pub async fn test_givenNonEmptyBlockStore_whenCallingTryCreateOnExistingBlock_thenFails(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;

        store.store(&blockid(1), &data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let status = store
            .try_create(&blockid(1), data(1024, 1).as_ref())
            .await
            .unwrap();
        assert_eq!(
            TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists,
            status
        );
        f.yield_fixture(&store).await;

        assert_eq!(Some(data(1024, 0)), store.load(&blockid(1)).await.unwrap());
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenNonEmptyBlockStore_whenCallingTryCreateOnExistingEmptyBlock_thenFails(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;

        store.store(&blockid(1), &data(0, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let status = store
            .try_create(&blockid(1), data(1024, 1).as_ref())
            .await
            .unwrap();
        assert_eq!(
            TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists,
            status
        );
        f.yield_fixture(&store).await;

        assert_eq!(Some(data(0, 0)), store.load(&blockid(1)).await.unwrap());
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenNonEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_withNonEmptyData_thenSucceeds(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;

        store.store(&blockid(1), &data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let status = store
            .try_create(&blockid(2), data(1024, 1).as_ref())
            .await
            .unwrap();
        assert_eq!(TryCreateResult::SuccessfullyCreated, status);
        f.yield_fixture(&store).await;

        assert_eq!(Some(data(1024, 1)), store.load(&blockid(2)).await.unwrap());
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenNonEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_withEmptyData_thenSucceeds(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;

        store.store(&blockid(1), &data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        let status = store
            .try_create(&blockid(2), data(0, 1).as_ref())
            .await
            .unwrap();
        assert_eq!(TryCreateResult::SuccessfullyCreated, status);
        f.yield_fixture(&store).await;

        assert_eq!(Some(data(0, 1)), store.load(&blockid(2)).await.unwrap());
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_withNonEmptyData_thenSucceeds(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;

        let status = store
            .try_create(&blockid(1), data(1024, 1).as_ref())
            .await
            .unwrap();
        assert_eq!(TryCreateResult::SuccessfullyCreated, status);
        f.yield_fixture(&store).await;

        assert_eq!(Some(data(1024, 1)), store.load(&blockid(1)).await.unwrap());
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_withEmptyData_thenSucceeds(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;

        let status = store
            .try_create(&blockid(1), data(0, 1).as_ref())
            .await
            .unwrap();
        assert_eq!(TryCreateResult::SuccessfullyCreated, status);
        f.yield_fixture(&store).await;

        assert_eq!(Some(data(0, 1)), store.load(&blockid(1)).await.unwrap());
        f.yield_fixture(&store).await;
    }
}

pub mod load {
    use super::*;

    pub async fn test_givenNonEmptyBlockStore_whenLoadExistingBlock_withNonEmptyData_thenSucceeds(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;

        store
            .store(&blockid(0), data(1024, 0).as_ref())
            .await
            .unwrap();
        f.yield_fixture(&store).await;

        store
            .store(&blockid(1), data(1024, 1).as_ref())
            .await
            .unwrap();
        f.yield_fixture(&store).await;

        let loaded = store.load(&blockid(1)).await.unwrap();
        assert_eq!(Some(data(1024, 1)), loaded);
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenNonEmptyBlockStore_whenLoadExistingBlock_withEmptyData_thenSucceeds(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;

        store
            .store(&blockid(0), data(1024, 0).as_ref())
            .await
            .unwrap();
        f.yield_fixture(&store).await;

        store.store(&blockid(1), data(0, 1).as_ref()).await.unwrap();
        f.yield_fixture(&store).await;

        let loaded = store.load(&blockid(1)).await.unwrap();
        assert_eq!(Some(data(0, 1)), loaded);
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenNonEmptyBlockStore_whenLoadNonexistingBlock_thenFails(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;

        store
            .store(&blockid(0), data(1024, 0).as_ref())
            .await
            .unwrap();
        f.yield_fixture(&store).await;

        store
            .store(&blockid(1), data(1024, 1).as_ref())
            .await
            .unwrap();
        f.yield_fixture(&store).await;

        let loaded = store.load(&blockid(2)).await.unwrap();
        assert_eq!(None, loaded);
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenEmptyBlockStore_whenLoadNonexistingBlock_thenFails(mut f: impl Fixture) {
        let store = f.store().await;

        let loaded = store.load(&blockid(1)).await.unwrap();
        assert_eq!(None, loaded);
        f.yield_fixture(&store).await;
    }
}

pub mod store {
    use super::*;

    pub async fn test_givenEmptyBlockStore_whenStoringNonExistingBlock_withNonEmptyData_thenSucceeds(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;

        store
            .store(&blockid(1), data(1024, 1).as_ref())
            .await
            .unwrap();
        f.yield_fixture(&store).await;

        assert_eq!(Some(data(1024, 1)), store.load(&blockid(1)).await.unwrap());
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenEmptyBlockStore_whenStoringNonExistingBlock_withEmptyData_thenSucceeds(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;

        store.store(&blockid(1), data(0, 1).as_ref()).await.unwrap();
        f.yield_fixture(&store).await;

        assert_eq!(Some(data(0, 1)), store.load(&blockid(1)).await.unwrap());
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenNonEmptyBlockStore_whenStoringNonExistingBlock_withNonEmptyData_thenSucceeds(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;

        store
            .store(&blockid(1), data(1024, 0).as_ref())
            .await
            .unwrap();
        f.yield_fixture(&store).await;
        store
            .store(&blockid(2), data(1024, 1).as_ref())
            .await
            .unwrap();
        f.yield_fixture(&store).await;

        assert_eq!(Some(data(1024, 1)), store.load(&blockid(2)).await.unwrap());
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenNonEmptyBlockStore_whenStoringNonExistingBlock_withEmptyData_thenSucceeds(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;

        store
            .store(&blockid(1), data(1024, 0).as_ref())
            .await
            .unwrap();
        f.yield_fixture(&store).await;

        store.store(&blockid(2), data(0, 1).as_ref()).await.unwrap();
        f.yield_fixture(&store).await;

        assert_eq!(Some(data(0, 1)), store.load(&blockid(2)).await.unwrap());
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenNonEmptyBlockStore_whenStoringExistingBlock_withNonEmptyData_thenSucceeds(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;

        store
            .store(&blockid(1), data(1024, 0).as_ref())
            .await
            .unwrap();
        f.yield_fixture(&store).await;

        store
            .store(&blockid(2), data(1024, 1).as_ref())
            .await
            .unwrap();
        f.yield_fixture(&store).await;

        store
            .store(&blockid(2), data(1024, 2).as_ref())
            .await
            .unwrap();
        f.yield_fixture(&store).await;

        // Test the unrelated block still has the old value
        assert_eq!(Some(data(1024, 0)), store.load(&blockid(1)).await.unwrap());
        f.yield_fixture(&store).await;

        // Check it got successfully overwritten
        assert_eq!(Some(data(1024, 2)), store.load(&blockid(2)).await.unwrap());
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenNonEmptyBlockStore_whenStoringExistingBlock_withEmptyData_thenSucceeds(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;

        store
            .store(&blockid(1), data(1024, 0).as_ref())
            .await
            .unwrap();
        f.yield_fixture(&store).await;

        store
            .store(&blockid(2), data(1024, 1).as_ref())
            .await
            .unwrap();
        f.yield_fixture(&store).await;

        store.store(&blockid(2), data(0, 2).as_ref()).await.unwrap();
        f.yield_fixture(&store).await;

        // Test the unrelated block still has the old value
        assert_eq!(Some(data(1024, 0)), store.load(&blockid(1)).await.unwrap());
        f.yield_fixture(&store).await;

        // Check it got successfully overwritten
        assert_eq!(Some(data(0, 2)), store.load(&blockid(2)).await.unwrap());
        f.yield_fixture(&store).await;
    }

    // TODO Test that overwriting an existing block with larger/smaller blocksize works
}

pub mod remove {
    use super::*;

    pub async fn test_givenOtherwiseEmptyBlockStore_whenRemovingNonEmptyBlock_thenBlockIsNotLoadableAnymore(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;

        store
            .store(&blockid(1), data(1024, 1).as_ref())
            .await
            .unwrap();
        f.yield_fixture(&store).await;

        assert!(store.load(&blockid(1)).await.unwrap().is_some());
        f.yield_fixture(&store).await;

        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&blockid(1)).await.unwrap()
        );
        f.yield_fixture(&store).await;

        assert!(store.load(&blockid(1)).await.unwrap().is_none());
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenOtherwiseEmptyBlockStore_whenRemovingEmptyBlock_thenBlockIsNotLoadableAnymore(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;

        store.store(&blockid(1), data(0, 1).as_ref()).await.unwrap();
        f.yield_fixture(&store).await;

        assert!(store.load(&blockid(1)).await.unwrap().is_some());
        f.yield_fixture(&store).await;

        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&blockid(1)).await.unwrap()
        );
        f.yield_fixture(&store).await;

        assert!(store.load(&blockid(1)).await.unwrap().is_none());
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenNonEmptyBlockStore_whenRemovingNonEmptyBlock_thenBlockIsNotLoadableAnymore(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;

        store
            .store(&blockid(1), data(1024, 2).as_ref())
            .await
            .unwrap();
        f.yield_fixture(&store).await;

        store
            .store(&blockid(2), data(1024, 1).as_ref())
            .await
            .unwrap();
        f.yield_fixture(&store).await;

        assert!(store.load(&blockid(2)).await.unwrap().is_some());
        f.yield_fixture(&store).await;

        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&blockid(2)).await.unwrap()
        );
        f.yield_fixture(&store).await;

        assert!(store.load(&blockid(2)).await.unwrap().is_none());
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenNonEmptyBlockStore_whenRemovingEmptyBlock_thenBlockIsNotLoadableAnymore(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;

        store
            .store(&blockid(1), data(1024, 2).as_ref())
            .await
            .unwrap();
        f.yield_fixture(&store).await;

        store.store(&blockid(2), data(0, 1).as_ref()).await.unwrap();
        f.yield_fixture(&store).await;

        assert!(store.load(&blockid(2)).await.unwrap().is_some());
        f.yield_fixture(&store).await;

        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&blockid(2)).await.unwrap()
        );
        f.yield_fixture(&store).await;

        assert!(store.load(&blockid(2)).await.unwrap().is_none());
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenEmptyBlockStore_whenRemovingNonexistingBlock_thenFails(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;

        assert_eq!(
            RemoveResult::NotRemovedBecauseItDoesntExist,
            store.remove(&blockid(1)).await.unwrap()
        );
        f.yield_fixture(&store).await;

        assert!(store.load(&blockid(1)).await.unwrap().is_none());
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenNonEmptyBlockStore_whenRemovingNonexistingBlock_thenFails(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;

        store
            .store(&blockid(1), data(1024, 2).as_ref())
            .await
            .unwrap();
        f.yield_fixture(&store).await;

        assert_eq!(
            RemoveResult::NotRemovedBecauseItDoesntExist,
            store.remove(&blockid(2)).await.unwrap()
        );
        f.yield_fixture(&store).await;

        assert!(store.load(&blockid(2)).await.unwrap().is_none());
        f.yield_fixture(&store).await;
    }
}

pub mod num_blocks {
    use super::*;

    pub async fn test_givenEmptyBlockStore_whenCallingNumBlocks_thenReturnsCorrectResult(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;
        assert_eq!(0, store.num_blocks().await.unwrap());
        f.yield_fixture(&store).await;
    }

    pub async fn test_afterStoringBlocks_whenCallingNumBlocks_thenReturnsCorrectResult(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;
        assert_eq!(0, store.num_blocks().await.unwrap());
        f.yield_fixture(&store).await;

        store.store(&blockid(0), &data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;
        assert_eq!(1, store.num_blocks().await.unwrap());
        f.yield_fixture(&store).await;

        store.store(&blockid(1), &data(1024, 1)).await.unwrap();
        f.yield_fixture(&store).await;
        assert_eq!(2, store.num_blocks().await.unwrap());
        f.yield_fixture(&store).await;

        store.store(&blockid(2), &data(0, 2)).await.unwrap();
        f.yield_fixture(&store).await;
        assert_eq!(3, store.num_blocks().await.unwrap());
        f.yield_fixture(&store).await;

        store.store(&blockid(1), &data(1024, 3)).await.unwrap();
        f.yield_fixture(&store).await;
        assert_eq!(3, store.num_blocks().await.unwrap());
        f.yield_fixture(&store).await;
    }

    pub async fn test_afterStoringBlocks_withSameId_whenCallingNumBlocks_thenReturnsCorrectResult(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;
        assert_eq!(0, store.num_blocks().await.unwrap());
        f.yield_fixture(&store).await;

        store.store(&blockid(0), &data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;
        assert_eq!(1, store.num_blocks().await.unwrap());
        f.yield_fixture(&store).await;

        store.store(&blockid(0), &data(1024, 1)).await.unwrap();
        f.yield_fixture(&store).await;
        assert_eq!(1, store.num_blocks().await.unwrap());
        f.yield_fixture(&store).await;
    }

    pub async fn test_afterTryCreatingBlocks_whenCallingNumBlocks_thenReturnsCorrectResult(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;

        assert_eq!(0, store.num_blocks().await.unwrap());
        f.yield_fixture(&store).await;

        assert_eq!(
            TryCreateResult::SuccessfullyCreated,
            store.try_create(&blockid(0), &data(1024, 0)).await.unwrap()
        );
        f.yield_fixture(&store).await;
        assert_eq!(1, store.num_blocks().await.unwrap());
        f.yield_fixture(&store).await;

        assert_eq!(
            TryCreateResult::SuccessfullyCreated,
            store.try_create(&blockid(1), &data(1024, 1)).await.unwrap()
        );
        f.yield_fixture(&store).await;
        assert_eq!(2, store.num_blocks().await.unwrap());
        f.yield_fixture(&store).await;

        assert_eq!(
            TryCreateResult::SuccessfullyCreated,
            store.try_create(&blockid(2), &data(0, 2)).await.unwrap()
        );
        f.yield_fixture(&store).await;
        assert_eq!(3, store.num_blocks().await.unwrap());
        f.yield_fixture(&store).await;

        assert_eq!(
            TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists,
            store.try_create(&blockid(1), &data(1024, 3)).await.unwrap()
        );
        f.yield_fixture(&store).await;
        assert_eq!(3, store.num_blocks().await.unwrap());
        f.yield_fixture(&store).await;
    }

    pub async fn test_afterRemovingBlocks_whenCallingNumBlocks_thenReturnsCorrectResult(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;

        store.store(&blockid(0), &data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;
        store.store(&blockid(1), &data(1024, 1)).await.unwrap();
        f.yield_fixture(&store).await;
        store.store(&blockid(2), &data(0, 2)).await.unwrap();
        f.yield_fixture(&store).await;

        assert_eq!(3, store.num_blocks().await.unwrap());
        f.yield_fixture(&store).await;

        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&blockid(0)).await.unwrap()
        );
        f.yield_fixture(&store).await;
        assert_eq!(2, store.num_blocks().await.unwrap());
        f.yield_fixture(&store).await;

        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&blockid(1)).await.unwrap()
        );
        f.yield_fixture(&store).await;
        assert_eq!(1, store.num_blocks().await.unwrap());
        f.yield_fixture(&store).await;

        assert_eq!(
            RemoveResult::NotRemovedBecauseItDoesntExist,
            store.remove(&blockid(1)).await.unwrap()
        );
        f.yield_fixture(&store).await;
        assert_eq!(1, store.num_blocks().await.unwrap());
        f.yield_fixture(&store).await;

        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&blockid(2)).await.unwrap()
        );
        f.yield_fixture(&store).await;
        assert_eq!(0, store.num_blocks().await.unwrap());
        f.yield_fixture(&store).await;
    }
}

pub mod all_blocks {
    use super::*;

    async fn call_all_blocks(store: &impl BlockStoreReader) -> Vec<BlockId> {
        store
            .all_blocks()
            .await
            .unwrap()
            .try_collect()
            .await
            .unwrap()
    }

    pub async fn test_givenEmptyBlockStore_whenCallingAllBlocks_thenReturnsCorrectResult(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;
        assert_unordered_vec_eq(vec![], call_all_blocks(store.deref()).await);
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenBlockStoreWithOneNonEmptyBlock_whenCallingAllBlocks_thenReturnsCorrectResult(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;
        store.store(&blockid(0), &data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;
        assert_unordered_vec_eq(vec![blockid(0)], call_all_blocks(store.deref()).await);
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenBlockStoreWithOneEmptyBlock_whenCallingAllBlocks_thenReturnsCorrectResult(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;
        store.store(&blockid(0), &data(0, 0)).await.unwrap();
        f.yield_fixture(&store).await;
        assert_unordered_vec_eq(vec![blockid(0)], call_all_blocks(store.deref()).await);
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenBlockStoreWithTwoBlocks_whenCallingAllBlocks_thenReturnsCorrectResult(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;
        store.store(&blockid(0), &data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;
        store.store(&blockid(1), &data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;
        assert_unordered_vec_eq(
            vec![blockid(0), blockid(1)],
            call_all_blocks(store.deref()).await,
        );
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenBlockStoreWithThreeBlocks_whenCallingAllBlocks_thenReturnsCorrectResult(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;
        store.store(&blockid(0), &data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;
        store.store(&blockid(1), &data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;
        store.store(&blockid(2), &data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;
        assert_unordered_vec_eq(
            vec![blockid(0), blockid(1), blockid(2)],
            call_all_blocks(store.deref()).await,
        );
        f.yield_fixture(&store).await;
    }

    pub async fn test_afterRemovingBlock_whenCallingAllBlocks_doesntListRemovedBlocks(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;
        store.store(&blockid(0), &data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;
        store.store(&blockid(1), &data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;
        store.store(&blockid(2), &data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;

        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&blockid(1)).await.unwrap()
        );
        f.yield_fixture(&store).await;
        assert_unordered_vec_eq(
            vec![blockid(0), blockid(2)],
            call_all_blocks(store.deref()).await,
        );
        f.yield_fixture(&store).await;

        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&blockid(2)).await.unwrap()
        );
        f.yield_fixture(&store).await;
        assert_unordered_vec_eq(vec![blockid(0)], call_all_blocks(store.deref()).await);
        f.yield_fixture(&store).await;

        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&blockid(0)).await.unwrap()
        );
        f.yield_fixture(&store).await;
        assert_unordered_vec_eq(vec![], call_all_blocks(store.deref()).await);
        f.yield_fixture(&store).await;
    }
}

pub mod exists {
    use super::*;

    pub async fn test_givenEmptyBlockStore_whenCallingExistsOnNonExistingBlock_thenReturnsFalse(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;
        assert_eq!(false, store.exists(&blockid(0)).await.unwrap());
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenNonEmptyBlockStore_whenCallingExistsOnNonExistingBlock_thenReturnsFalse(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;
        store.store(&blockid(0), &data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;
        assert_eq!(false, store.exists(&blockid(1)).await.unwrap());
        f.yield_fixture(&store).await;
    }

    pub async fn test_givenNonEmptyBlockStore_whenCallingExistsOnExistingBlock_thenReturnsTrue(
        mut f: impl Fixture,
    ) {
        let store = f.store().await;
        store.store(&blockid(0), &data(1024, 0)).await.unwrap();
        f.yield_fixture(&store).await;
        assert_eq!(true, store.exists(&blockid(0)).await.unwrap());
        f.yield_fixture(&store).await;
    }
}

#[macro_export]
macro_rules! _instantiate_lowlevel_blockstore_tests {
    (@module $module_name: ident, $target: ty, $tokio_test_args: tt $(, $test_cases: ident)* $(,)?) => {
        mod $module_name {
            use super::*;

            $crate::_instantiate_lowlevel_blockstore_tests!(@module_impl $module_name, $target, $tokio_test_args $(, $test_cases)*);
        }
    };
    (@module_impl $module_name: ident, $target: ty, $tokio_test_args: tt) => {
    };
    (@module_impl $module_name: ident, $target: ty, $tokio_test_args: tt, $head_test_case: ident $(, $tail_test_cases: ident)*) => {
        #[tokio::test$tokio_test_args]
        #[allow(non_snake_case)]
        async fn $head_test_case() {
            let fixture = <$target as $crate::tests::Fixture>::new();
            $crate::tests::low_level::$module_name::$head_test_case(fixture).await
        }
        $crate::_instantiate_lowlevel_blockstore_tests!(@module_impl $module_name, $target, $tokio_test_args $(, $tail_test_cases)*);
    };
}

/// This macro instantiates all blockstore tests for a given blockstore.
/// See [Fixture] for how to invoke it.
#[macro_export]
macro_rules! instantiate_lowlevel_blockstore_tests {
    ($target: ty) => {
        $crate::instantiate_lowlevel_blockstore_tests!($target, ());
    };
    ($target: ty, $tokio_test_args: tt) => {
        $crate::_instantiate_lowlevel_blockstore_tests!(@module try_create, $target, $tokio_test_args,
            test_givenNonEmptyBlockStore_whenCallingTryCreateOnExistingBlock_thenFails,
            test_givenNonEmptyBlockStore_whenCallingTryCreateOnExistingEmptyBlock_thenFails,
            test_givenNonEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_withEmptyData_thenSucceeds,
            test_givenNonEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_withNonEmptyData_thenSucceeds,
            test_givenEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_withEmptyData_thenSucceeds,
            test_givenEmptyBlockStore_whenCallingTryCreateOnNonExistingBlock_withNonEmptyData_thenSucceeds,
        );
        $crate::_instantiate_lowlevel_blockstore_tests!(@module load, $target, $tokio_test_args,
            test_givenNonEmptyBlockStore_whenLoadExistingBlock_withEmptyData_thenSucceeds,
            test_givenNonEmptyBlockStore_whenLoadExistingBlock_withNonEmptyData_thenSucceeds,
            test_givenNonEmptyBlockStore_whenLoadNonexistingBlock_thenFails,
            test_givenEmptyBlockStore_whenLoadNonexistingBlock_thenFails,
        );
        $crate::_instantiate_lowlevel_blockstore_tests!(@module store, $target, $tokio_test_args,
            test_givenEmptyBlockStore_whenStoringNonExistingBlock_withEmptyData_thenSucceeds,
            test_givenEmptyBlockStore_whenStoringNonExistingBlock_withNonEmptyData_thenSucceeds,
            test_givenNonEmptyBlockStore_whenStoringNonExistingBlock_withEmptyData_thenSucceeds,
            test_givenNonEmptyBlockStore_whenStoringNonExistingBlock_withNonEmptyData_thenSucceeds,
            test_givenNonEmptyBlockStore_whenStoringExistingBlock_withEmptyData_thenSucceeds,
            test_givenNonEmptyBlockStore_whenStoringExistingBlock_withNonEmptyData_thenSucceeds,
        );
        $crate::_instantiate_lowlevel_blockstore_tests!(@module remove, $target, $tokio_test_args,
            test_givenOtherwiseEmptyBlockStore_whenRemovingEmptyBlock_thenBlockIsNotLoadableAnymore,
            test_givenOtherwiseEmptyBlockStore_whenRemovingNonEmptyBlock_thenBlockIsNotLoadableAnymore,
            test_givenNonEmptyBlockStore_whenRemovingEmptyBlock_thenBlockIsNotLoadableAnymore,
            test_givenNonEmptyBlockStore_whenRemovingNonEmptyBlock_thenBlockIsNotLoadableAnymore,
            test_givenEmptyBlockStore_whenRemovingNonexistingBlock_thenFails,
            test_givenNonEmptyBlockStore_whenRemovingNonexistingBlock_thenFails,
        );
        $crate::_instantiate_lowlevel_blockstore_tests!(@module num_blocks, $target, $tokio_test_args,
            test_givenEmptyBlockStore_whenCallingNumBlocks_thenReturnsCorrectResult,
            test_afterStoringBlocks_whenCallingNumBlocks_thenReturnsCorrectResult,
            test_afterStoringBlocks_withSameId_whenCallingNumBlocks_thenReturnsCorrectResult,
            test_afterTryCreatingBlocks_whenCallingNumBlocks_thenReturnsCorrectResult,
            test_afterRemovingBlocks_whenCallingNumBlocks_thenReturnsCorrectResult,
        );
        $crate::_instantiate_lowlevel_blockstore_tests!(@module all_blocks, $target, $tokio_test_args,
            test_givenEmptyBlockStore_whenCallingAllBlocks_thenReturnsCorrectResult,
            test_givenBlockStoreWithOneNonEmptyBlock_whenCallingAllBlocks_thenReturnsCorrectResult,
            test_givenBlockStoreWithOneEmptyBlock_whenCallingAllBlocks_thenReturnsCorrectResult,
            test_givenBlockStoreWithTwoBlocks_whenCallingAllBlocks_thenReturnsCorrectResult,
            test_givenBlockStoreWithThreeBlocks_whenCallingAllBlocks_thenReturnsCorrectResult,
            test_afterRemovingBlock_whenCallingAllBlocks_doesntListRemovedBlocks,
        );
        $crate::_instantiate_lowlevel_blockstore_tests!(@module exists, $target, $tokio_test_args,
            test_givenEmptyBlockStore_whenCallingExistsOnNonExistingBlock_thenReturnsFalse,
            test_givenNonEmptyBlockStore_whenCallingExistsOnNonExistingBlock_thenReturnsFalse,
            test_givenNonEmptyBlockStore_whenCallingExistsOnExistingBlock_thenReturnsTrue,
        );

        // TODO Test estimate_num_free_bytes
        // TODO Test block_size_from_physical_block_size
        // TODO Test OptimizedBlockStoreWriter
    };
}
