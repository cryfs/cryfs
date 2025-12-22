use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use byte_unit::Byte;
use cryfs_utils::threadpool::ThreadPool;
use futures::stream::BoxStream;
use std::borrow::Borrow;
use std::fmt::{self, Debug};
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;

use crate::{
    BlockId, Overhead,
    low_level::{
        BlockStoreDeleter, BlockStoreReader, LLBlockStore, OptimizedBlockStoreWriter,
        interface::block_data::{IBlockData, create_block_data_wrapper},
    },
    utils::{RemoveResult, TryCreateResult},
};
use cryfs_crypto::symmetric::CipherDef;
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    data::Data,
};

// TODO Here and in other files: Add more .context() to errors

const FORMAT_VERSION_HEADER: &[u8; 2] = &1u16.to_ne_bytes();

pub struct EncryptedBlockStore<
    C: 'static + CipherDef + Sync,
    _B: Debug,
    B: 'static + Debug + AsyncDrop<Error = anyhow::Error> + Borrow<_B> + Send + Sync,
> {
    underlying_block_store: AsyncDropGuard<B>,
    cipher: Arc<C>,
    crypto_task_pool: ThreadPool,
    _phantom: PhantomData<_B>,
}

impl<
    // TODO Are all those bounds on C, _B, B still needed ?
    C: 'static + CipherDef + Send + Sync,
    _B: Debug + Send + Sync,
    B: 'static + Debug + AsyncDrop<Error = anyhow::Error> + Borrow<_B> + Send + Sync,
> EncryptedBlockStore<C, _B, B>
{
    pub fn new(underlying_block_store: AsyncDropGuard<B>, cipher: C) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            underlying_block_store,
            cipher: Arc::new(cipher),
            crypto_task_pool: ThreadPool::new("EncryptedBlockStore")
                .expect("Failed to create thread pool for crypto tasks"),
            _phantom: PhantomData,
        })
    }
}

#[async_trait]
impl<
    C: 'static + CipherDef + Send + Sync,
    _B: BlockStoreReader + Send + Sync + Debug,
    B: 'static + Debug + AsyncDrop<Error = anyhow::Error> + Borrow<_B> + Send + Sync,
> BlockStoreReader for EncryptedBlockStore<C, _B, B>
{
    async fn exists(&self, id: &BlockId) -> Result<bool> {
        self.underlying_block_store
            .deref()
            .borrow()
            .exists(id)
            .await
    }

    async fn load(&self, id: &BlockId) -> Result<Option<Data>> {
        let loaded = self
            .underlying_block_store
            .deref()
            .borrow()
            .load(id)
            .await
            .context(
                "EncryptedBlockStore failed to load the block from the underlying block store",
            )?;
        match loaded {
            None => Ok(None),
            Some(data) => Ok(Some(self._decrypt(data).await?)),
        }
    }

    async fn num_blocks(&self) -> Result<u64> {
        self.underlying_block_store
            .deref()
            .borrow()
            .num_blocks()
            .await
    }

    fn estimate_num_free_bytes(&self) -> Result<Byte> {
        self.underlying_block_store
            .deref()
            .borrow()
            .estimate_num_free_bytes()
    }

    fn overhead(&self) -> Overhead {
        self.underlying_block_store.deref().borrow().overhead()
            + Overhead::new(Byte::from_u64(
                (FORMAT_VERSION_HEADER.len()
                    + C::CIPHERTEXT_OVERHEAD_PREFIX
                    + C::CIPHERTEXT_OVERHEAD_SUFFIX) as u64,
            ))
    }

    async fn all_blocks(&self) -> Result<BoxStream<'static, Result<BlockId>>> {
        self.underlying_block_store
            .deref()
            .borrow()
            .all_blocks()
            .await
    }
}

#[async_trait]
impl<
    C: 'static + CipherDef + Send + Sync,
    _B: BlockStoreDeleter + Send + Sync + Debug,
    B: 'static + Debug + AsyncDrop<Error = anyhow::Error> + Borrow<_B> + Send + Sync,
> BlockStoreDeleter for EncryptedBlockStore<C, _B, B>
{
    async fn remove(&self, id: &BlockId) -> Result<RemoveResult> {
        self.underlying_block_store
            .deref()
            .borrow()
            .remove(id)
            .await
    }
}

create_block_data_wrapper!(BlockData);

#[async_trait]
impl<
    C: 'static + CipherDef + Send + Sync,
    _B: OptimizedBlockStoreWriter + Send + Sync + Debug,
    B: 'static + Debug + AsyncDrop<Error = anyhow::Error> + Borrow<_B> + Send + Sync,
> OptimizedBlockStoreWriter for EncryptedBlockStore<C, _B, B>
{
    type BlockData = BlockData;

    fn allocate(size: usize) -> Self::BlockData {
        let mut data = _B::allocate(
            FORMAT_VERSION_HEADER.len()
                + C::CIPHERTEXT_OVERHEAD_PREFIX
                + C::CIPHERTEXT_OVERHEAD_SUFFIX
                + size,
        )
        .extract();
        data.shrink_to_subregion(
            (FORMAT_VERSION_HEADER.len() + C::CIPHERTEXT_OVERHEAD_PREFIX)
                ..(data.len() - C::CIPHERTEXT_OVERHEAD_SUFFIX),
        );
        BlockData::new(data)
    }

    async fn try_create_optimized(
        &self,
        id: &BlockId,
        data: Self::BlockData,
    ) -> Result<TryCreateResult> {
        let ciphertext = self._encrypt(data.extract()).await?;
        self.underlying_block_store
            .deref()
            .borrow()
            .try_create_optimized(id, _B::BlockData::new(ciphertext))
            .await
    }

    async fn store_optimized(&self, id: &BlockId, data: Self::BlockData) -> Result<()> {
        let ciphertext = self._encrypt(data.extract()).await?;
        self.underlying_block_store
            .deref()
            .borrow()
            .store_optimized(id, _B::BlockData::new(ciphertext))
            .await
    }
}

impl<
    C: 'static + CipherDef + Send + Sync,
    _B: Send + Debug,
    B: 'static + Debug + AsyncDrop<Error = anyhow::Error> + Borrow<_B> + Send + Sync,
> Debug for EncryptedBlockStore<C, _B, B>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EncryptedBlockStore")
    }
}

#[async_trait]
impl<
    C: 'static + CipherDef + Send + Sync,
    _B: Sync + Send + Debug,
    B: 'static + Debug + AsyncDrop<Error = anyhow::Error> + Borrow<_B> + Send + Sync,
> AsyncDrop for EncryptedBlockStore<C, _B, B>
{
    type Error = anyhow::Error;
    async fn async_drop_impl(&mut self) -> Result<()> {
        self.underlying_block_store.async_drop().await
    }
}

impl<
    C: 'static + CipherDef + Send + Sync,
    _B: LLBlockStore + OptimizedBlockStoreWriter + Send + Sync + Debug,
    B: 'static + Debug + AsyncDrop<Error = anyhow::Error> + Borrow<_B> + Send + Sync,
> LLBlockStore for EncryptedBlockStore<C, _B, B>
{
}

impl<
    C: 'static + CipherDef + Send + Sync,
    _B: Debug + Sync,
    B: 'static + Debug + AsyncDrop<Error = anyhow::Error> + Borrow<_B> + Send + Sync,
> EncryptedBlockStore<C, _B, B>
{
    async fn _encrypt(&self, plaintext: Data) -> Result<Data> {
        let cipher = Arc::clone(&self.cipher);
        self.crypto_task_pool
            .execute_job(move || {
                let ciphertext = cipher.encrypt(plaintext)?;
                Ok(_prepend_header(ciphertext))
            })
            .await
    }

    async fn _decrypt(&self, ciphertext: Data) -> Result<Data> {
        let cipher = Arc::clone(&self.cipher);
        self.crypto_task_pool
            .execute_job(move || {
                let ciphertext = _check_and_remove_header(ciphertext)?;
                cipher.decrypt(ciphertext)
            })
            .await
    }
}

fn _check_and_remove_header(mut data: Data) -> Result<Data> {
    if !data.starts_with(FORMAT_VERSION_HEADER) {
        bail!(
            "Couldn't parse encrypted block. Expected FORMAT_VERSION_HEADER of {:?} but found {:?}",
            FORMAT_VERSION_HEADER,
            &data[..FORMAT_VERSION_HEADER.len()]
        );
    }
    data.shrink_to_subregion(FORMAT_VERSION_HEADER.len()..);
    Ok(data)
}

fn _prepend_header(mut data: Data) -> Data {
    // TODO Use binary-layout here?
    data.grow_region_fail_if_reallocation_necessary(FORMAT_VERSION_HEADER.len(), 0)
        .expect(
            "Tried to grow the data to contain the header in EncryptedBlockStore::_prepend_header",
        );
    data.as_mut()[..FORMAT_VERSION_HEADER.len()].copy_from_slice(FORMAT_VERSION_HEADER);
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::{RngCore, SeedableRng, rngs::StdRng};
    use std::marker::PhantomData;

    use crate::instantiate_blockstore_tests_for_lowlevel_blockstore;
    use crate::low_level::{BlockStoreReader, BlockStoreWriter, InMemoryBlockStore};
    use crate::tests::{
        low_level::LLFixture,
        utils::{blockid, data},
    };
    use cryfs_crypto::symmetric::{
        Aes128Gcm, Aes256Gcm, DefaultNonceSize, EncryptionKey, XChaCha20Poly1305,
    };
    use cryfs_utils::async_drop::AsyncDropArc;
    // TODO Separate out InfallibleUnwrap from lockable and depend on that instead of on lockable
    use lockable::InfallibleUnwrap;

    fn key(size: usize, seed: u64) -> EncryptionKey {
        EncryptionKey::new(size, |key_data| {
            let mut rng = StdRng::seed_from_u64(seed);
            rng.fill_bytes(key_data);
            Ok(())
        })
        .infallible_unwrap()
    }

    struct TestFixture<C: 'static + CipherDef + Send + Sync> {
        _c: PhantomData<C>,
    }
    #[async_trait]
    impl<C: 'static + CipherDef + Send + Sync> LLFixture for TestFixture<C> {
        type ConcreteBlockStore = EncryptedBlockStore<C, InMemoryBlockStore, InMemoryBlockStore>;
        fn new() -> Self {
            Self { _c: PhantomData }
        }
        async fn store(&mut self) -> AsyncDropGuard<Self::ConcreteBlockStore> {
            EncryptedBlockStore::new(
                InMemoryBlockStore::new(),
                C::new(key(C::KEY_SIZE, 0)).unwrap(),
            )
        }
        async fn yield_fixture(&self, _store: &Self::ConcreteBlockStore) {}
    }

    mod aes256gcm {
        use super::*;
        instantiate_blockstore_tests_for_lowlevel_blockstore!(
            super::TestFixture<Aes256Gcm>,
            (flavor = "multi_thread")
        );
    }
    mod aes128gcm {
        use super::*;
        crate::instantiate_blockstore_tests_for_lowlevel_blockstore!(
            super::TestFixture<Aes128Gcm>,
            (flavor = "multi_thread")
        );
    }

    mod xchachapoly1305 {
        use super::*;
        crate::instantiate_blockstore_tests_for_lowlevel_blockstore!(
            super::TestFixture<XChaCha20Poly1305>,
            (flavor = "multi_thread")
        );
    }

    #[tokio::test]
    async fn test_usable_block_size_from_physical_block_size() {
        async fn _test_usable_block_size_from_physical_block_size<
            C: 'static + CipherDef + Send + Sync,
        >() {
            let mut fixture = TestFixture::<C>::new();
            let mut store = fixture.store().await;
            let expected_overhead = Byte::from_u64(
                FORMAT_VERSION_HEADER.len() as u64
                    + C::CIPHERTEXT_OVERHEAD_PREFIX as u64
                    + C::CIPHERTEXT_OVERHEAD_SUFFIX as u64,
            );

            // usable_block_size_from_physical_block_size
            assert_eq!(
                0u64,
                store
                    .overhead()
                    .usable_block_size_from_physical_block_size(expected_overhead)
                    .unwrap()
            );
            assert_eq!(
                20u64,
                store
                    .overhead()
                    .usable_block_size_from_physical_block_size(
                        expected_overhead.add(Byte::from_u64(20)).unwrap()
                    )
                    .unwrap()
            );
            assert!(
                store
                    .overhead()
                    .usable_block_size_from_physical_block_size(Byte::from_u64(0))
                    .is_err()
            );

            // physical_block_size_from_usable_block_size
            assert_eq!(
                expected_overhead,
                store
                    .overhead()
                    .physical_block_size_from_usable_block_size(Byte::from_u64(0))
            );
            assert_eq!(
                Byte::from_u64(20).add(expected_overhead).unwrap(),
                store
                    .overhead()
                    .physical_block_size_from_usable_block_size(Byte::from_u64(20))
            );

            store.async_drop().await.unwrap();
        }

        _test_usable_block_size_from_physical_block_size::<Aes256Gcm>().await;
        _test_usable_block_size_from_physical_block_size::<Aes128Gcm>().await;
        _test_usable_block_size_from_physical_block_size::<XChaCha20Poly1305>().await;
    }

    async fn _store(
        bs: &AsyncDropGuard<AsyncDropArc<InMemoryBlockStore>>,
        key: EncryptionKey,
        block_id: &BlockId,
        data: &Data,
    ) {
        let mut store = EncryptedBlockStore::<
            Aes256Gcm,
            InMemoryBlockStore,
            AsyncDropArc<InMemoryBlockStore>,
        >::new(AsyncDropArc::clone(&bs), Aes256Gcm::new(key).unwrap());

        store.store(block_id, data).await.unwrap();
        store.async_drop().await.unwrap();
    }

    async fn _load(
        bs: &AsyncDropGuard<AsyncDropArc<InMemoryBlockStore>>,
        key: EncryptionKey,
        block_id: &BlockId,
    ) -> Result<Option<Data>> {
        let mut store = EncryptedBlockStore::<
            Aes256Gcm,
            InMemoryBlockStore,
            AsyncDropArc<InMemoryBlockStore>,
        >::new(AsyncDropArc::clone(&bs), Aes256Gcm::new(key)?);
        let result = store.load(block_id).await;

        store.async_drop().await.unwrap();
        result
    }

    async fn _manipulate(
        bs: &AsyncDropGuard<AsyncDropArc<InMemoryBlockStore>>,
        block_id: &BlockId,
    ) {
        let mut data = bs.load(block_id).await.unwrap().unwrap();
        data[0] = data[0].overflowing_add(1).0;
        bs.store(block_id, &data).await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_loading_with_same_key_works() {
        let mut inner = AsyncDropArc::new(InMemoryBlockStore::new());

        _store(
            &inner,
            key(Aes256Gcm::<DefaultNonceSize>::KEY_SIZE, 0),
            &blockid(0),
            &data(1024, 0),
        )
        .await;
        assert_eq!(
            Some(data(1024, 0)),
            _load(
                &inner,
                key(Aes256Gcm::<DefaultNonceSize>::KEY_SIZE, 0),
                &blockid(0)
            )
            .await
            .unwrap()
        );

        inner.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_loading_with_different_key_doesnt_work() {
        let mut inner = AsyncDropArc::new(InMemoryBlockStore::new());

        _store(
            &inner,
            key(Aes256Gcm::<DefaultNonceSize>::KEY_SIZE, 0),
            &blockid(0),
            &data(1024, 0),
        )
        .await;
        _load(
            &inner,
            key(Aes256Gcm::<DefaultNonceSize>::KEY_SIZE, 1),
            &blockid(0),
        )
        .await
        .unwrap_err();

        inner.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_loading_manipulated_block_doesnt_work() {
        let mut inner = AsyncDropArc::new(InMemoryBlockStore::new());

        _store(
            &inner,
            key(Aes256Gcm::<DefaultNonceSize>::KEY_SIZE, 0),
            &blockid(0),
            &data(1024, 0),
        )
        .await;
        _manipulate(&inner, &blockid(0)).await;
        _load(
            &inner,
            key(Aes256Gcm::<DefaultNonceSize>::KEY_SIZE, 0),
            &blockid(0),
        )
        .await
        .unwrap_err();

        inner.async_drop().await.unwrap();
    }
}
