use anyhow::Result;
use futures::{future::BoxFuture, stream::Stream};
use mockall::mock;
use std::fmt::{self, Debug};
use std::pin::Pin;

use crate::{
    low_level::{BlockStore, BlockStoreDeleter, BlockStoreReader, BlockStoreWriter},
    utils::{RemoveResult, TryCreateResult},
    BlockId,
};
use cryfs_utils::{async_drop::AsyncDrop, data::Data};

mock! {
    pub BlockStore {
    }
    impl BlockStoreReader for BlockStore {
        fn exists<'a, 'b, 'r>(&'a self, id: &'b BlockId) -> BoxFuture<'r, Result<bool>> where 'a: 'r, 'b: 'r;
        fn load<'a, 'b, 'r>(&'a self, id: &'b BlockId) -> BoxFuture<'r, Result<Option<Data>>> where 'a: 'r, 'b: 'r;
        fn num_blocks<'a, 'r>(&'a self) -> BoxFuture<'r, Result<u64>> where 'a: 'r;
        fn estimate_num_free_bytes(&self) -> Result<u64>;
        fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64>;

        fn all_blocks<'a, 'r>(&'a self) -> BoxFuture<'r, Result<Pin<Box<dyn Stream<Item = Result<BlockId>> + Send>>>> where 'a: 'r;
    }
    impl BlockStoreDeleter for BlockStore {
        fn remove<'a, 'b, 'r>(&'a self, id: &'b BlockId) -> BoxFuture<'r, Result<RemoveResult>> where 'a: 'r, 'b: 'r;
    }
    impl BlockStoreWriter for BlockStore {
        fn try_create<'a, 'b, 'c, 'r>(&'a self, id: &'b BlockId, data: &'c [u8]) -> BoxFuture<'r, Result<TryCreateResult>> where 'a: 'r, 'b: 'r, 'c: 'r;
        fn store<'a, 'b, 'c, 'r>(&'a self, id: &'b BlockId, data: &'c[u8]) -> BoxFuture<'r, Result<()>> where 'a: 'r, 'b: 'r, 'c: 'r;
    }
    impl AsyncDrop for BlockStore {
        type Error = anyhow::Error;
        fn async_drop_impl<'a, 'r>(&'a mut self) -> BoxFuture<'r, Result<()>> where 'a: 'r;
    }
    impl BlockStore for BlockStore {}
}
impl Debug for MockBlockStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MockBlockStore")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instantiate_blockstore_tests;
    use crate::low_level::InMemoryBlockStore;
    use async_trait::async_trait;
    use cryfs_utils::async_drop::AsyncDropGuard;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // Build a MockBlockStore that sets up all mock methods so that
    // they work based on an underlying InMemoryBlockStore.
    fn make_working_mock_block_store() -> AsyncDropGuard<MockBlockStore> {
        let underlying_store = Arc::new(Mutex::new(Some(InMemoryBlockStore::new())));
        let mut mock_store = AsyncDropGuard::new(MockBlockStore::new());

        let _underlying_store = Arc::clone(&underlying_store);
        mock_store.expect_exists().returning(move |id| {
            let _underlying_store = Arc::clone(&_underlying_store);
            let id = *id;
            Box::pin(async move {
                _underlying_store
                    .lock()
                    .await
                    .as_ref()
                    .expect("Already destructed")
                    .exists(&id)
                    .await
            })
        });

        let _underlying_store = Arc::clone(&underlying_store);
        mock_store.expect_num_blocks().returning(move || {
            let _underlying_store = Arc::clone(&_underlying_store);
            Box::pin(async move {
                _underlying_store
                    .lock()
                    .await
                    .as_ref()
                    .expect("Already destructed")
                    .num_blocks()
                    .await
            })
        });

        let _underlying_store = Arc::clone(&underlying_store);
        mock_store
            .expect_estimate_num_free_bytes()
            .returning(move || {
                let _underlying_store = Arc::clone(&_underlying_store);
                let r = _underlying_store
                    .blocking_lock()
                    .as_ref()
                    .expect("Already destructed")
                    .estimate_num_free_bytes();
                r
            });

        let _underlying_store = Arc::clone(&underlying_store);
        mock_store
            .expect_block_size_from_physical_block_size()
            .returning(move |block_size| {
                let _underlying_store = Arc::clone(&_underlying_store);
                let r = _underlying_store
                    .blocking_lock()
                    .as_ref()
                    .expect("Already destructed")
                    .block_size_from_physical_block_size(block_size);
                r
            });

        let _underlying_store = Arc::clone(&underlying_store);
        mock_store.expect_all_blocks().returning(move || {
            let _underlying_store = Arc::clone(&_underlying_store);
            Box::pin(async move {
                _underlying_store
                    .lock()
                    .await
                    .as_ref()
                    .expect("Already destructed")
                    .all_blocks()
                    .await
            })
        });

        let _underlying_store = Arc::clone(&underlying_store);
        mock_store.expect_load().returning(move |id| {
            let _underlying_store = Arc::clone(&_underlying_store);
            let id = *id;
            Box::pin(async move {
                _underlying_store
                    .lock()
                    .await
                    .as_ref()
                    .expect("Already destructed")
                    .load(&id)
                    .await
            })
        });

        let _underlying_store = Arc::clone(&underlying_store);
        mock_store.expect_remove().returning(move |id| {
            let _underlying_store = Arc::clone(&_underlying_store);
            let id = *id;
            Box::pin(async move {
                _underlying_store
                    .lock()
                    .await
                    .as_ref()
                    .expect("Already destructed")
                    .remove(&id)
                    .await
            })
        });

        let _underlying_store = Arc::clone(&underlying_store);
        mock_store.expect_try_create().returning(move |id, data| {
            let _underlying_store = Arc::clone(&_underlying_store);
            let id = *id;
            let data = data.to_vec();
            Box::pin(async move {
                _underlying_store
                    .lock()
                    .await
                    .as_ref()
                    .expect("Already destructed")
                    .try_create(&id, &data)
                    .await
            })
        });

        let _underlying_store = Arc::clone(&underlying_store);
        mock_store.expect_store().returning(move |id, data| {
            let _underlying_store = Arc::clone(&_underlying_store);
            let id = *id;
            let data = data.to_vec();
            Box::pin(async move {
                _underlying_store
                    .lock()
                    .await
                    .as_ref()
                    .expect("Already destructed")
                    .store(&id, &data)
                    .await
            })
        });

        let _underlying_store = Arc::clone(&underlying_store);
        mock_store
            .expect_async_drop_impl()
            .times(1)
            .returning(move || {
                let _underlying_store = Arc::clone(&_underlying_store);
                Box::pin(async move {
                    _underlying_store
                        .lock()
                        .await
                        .take()
                        .expect("Already destructed")
                        .async_drop()
                        .await
                })
            });

        mock_store
    }

    struct TestFixture {}
    #[async_trait]
    impl crate::tests::Fixture for TestFixture {
        type ConcreteBlockStore = MockBlockStore;
        fn new() -> Self {
            Self {}
        }
        async fn store(&mut self) -> AsyncDropGuard<Self::ConcreteBlockStore> {
            make_working_mock_block_store()
        }
        async fn yield_fixture(&self, _store: &Self::ConcreteBlockStore) {}
    }

    instantiate_blockstore_tests!(TestFixture, (flavor = "multi_thread"));
}
