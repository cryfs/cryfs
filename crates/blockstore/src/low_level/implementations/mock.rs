use anyhow::Result;
use byte_unit::Byte;
use futures::stream::BoxStream;
use mockall::mock;
use std::fmt::{self, Debug};

use crate::{
    BlockId, Overhead,
    low_level::{BlockStoreDeleter, BlockStoreReader, BlockStoreWriter, LLBlockStore},
    utils::{RemoveResult, TryCreateResult},
};
use cryfs_utils::{async_drop::AsyncDrop, data::Data};

mock! {
    pub BlockStore {
    }
    impl BlockStoreReader for BlockStore {
        fn exists(&self, id: &BlockId) -> impl Future<Output = Result<bool>> + Send;
        fn load(&self, id: &BlockId) -> impl Future<Output = Result<Option<Data>>> + Send;
        fn num_blocks(&self) -> impl Future<Output = Result<u64>> + Send;
        fn estimate_num_free_bytes(&self) -> Result<Byte>;
        fn overhead(&self) -> Overhead;

        fn all_blocks(&self) -> impl Future<Output = Result<BoxStream<'static, Result<BlockId>>>> + Send;
    }
    impl BlockStoreDeleter for BlockStore {
        fn remove(&self, id: &BlockId) -> impl Future<Output = Result<RemoveResult>> + Send;
    }
    impl BlockStoreWriter for BlockStore {
        fn try_create(&self, id: &BlockId, data: &[u8]) -> impl Future<Output = Result<TryCreateResult>> + Send;
        fn store(&self, id: &BlockId, data: &[u8]) -> impl Future<Output = Result<()>> + Send;
    }
    impl AsyncDrop for BlockStore {
        type Error = anyhow::Error;
        fn async_drop_impl(self) -> impl Future<Output = Result<()>> + Send;
    }
    impl LLBlockStore for BlockStore {}
}
impl Debug for MockBlockStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MockBlockStore")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instantiate_blockstore_tests_for_lowlevel_blockstore;
    use crate::low_level::InMemoryBlockStore;
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
        mock_store.expect_overhead().returning(move || {
            let _underlying_store = Arc::clone(&_underlying_store);
            let r = tokio::task::block_in_place(|| {
                let _underlying_store = _underlying_store.blocking_lock();
                _underlying_store
                    .as_ref()
                    .expect("Already destructed")
                    .overhead()
            });
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
    impl crate::tests::low_level::LLFixture for TestFixture {
        type ConcreteBlockStore = MockBlockStore;
        fn new() -> Self {
            Self {}
        }
        async fn store(&mut self) -> AsyncDropGuard<Self::ConcreteBlockStore> {
            make_working_mock_block_store()
        }
        async fn yield_fixture(&self, _store: &Self::ConcreteBlockStore) {}
    }

    instantiate_blockstore_tests_for_lowlevel_blockstore!(TestFixture, (flavor = "multi_thread"));
}
