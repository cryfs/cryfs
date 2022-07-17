use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use futures::stream::Stream;
use std::borrow::Borrow;
use std::fmt::{self, Debug};
use std::marker::PhantomData;
use std::ops::Deref;
use std::pin::Pin;

use super::block_data::IBlockData;
use super::{
    BlockId, BlockStore, BlockStoreDeleter, BlockStoreReader, OptimizedBlockStoreWriter,
    RemoveResult, TryCreateResult,
};
use crate::crypto::symmetric::Cipher;
use crate::data::Data;
use crate::utils::async_drop::{AsyncDrop, AsyncDropGuard};

// TODO Here and in other files: Add more .context() to errors

const FORMAT_VERSION_HEADER: &[u8; 2] = &1u16.to_ne_bytes();

pub struct EncryptedBlockStore<
    C: 'static + Cipher,
    _B: Debug,
    B: 'static + Debug + AsyncDrop<Error = anyhow::Error> + Borrow<_B> + Send + Sync,
> {
    underlying_block_store: AsyncDropGuard<B>,
    cipher: C,
    _phantom: PhantomData<_B>,
}

impl<
        // TODO Are all those bounds on C, _B, B still needed ?
        C: 'static + Cipher + Send + Sync,
        _B: Debug + Send + Sync,
        B: 'static + Debug + AsyncDrop<Error = anyhow::Error> + Borrow<_B> + Send + Sync,
    > EncryptedBlockStore<C, _B, B>
{
    pub fn new(underlying_block_store: AsyncDropGuard<B>, cipher: C) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            underlying_block_store,
            cipher,
            _phantom: PhantomData,
        })
    }
}

#[async_trait]
impl<
        C: 'static + Cipher + Send + Sync,
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

    fn estimate_num_free_bytes(&self) -> Result<u64> {
        self.underlying_block_store
            .deref()
            .borrow()
            .estimate_num_free_bytes()
    }

    fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64> {
        let ciphertext_size = self.underlying_block_store.deref().borrow().block_size_from_physical_block_size(block_size)?.checked_sub(FORMAT_VERSION_HEADER.len() as u64)
            .with_context(|| anyhow!("Physical block size of {} is too small to hold even the FORMAT_VERSION_HEADER. Must be at least {}.", block_size, FORMAT_VERSION_HEADER.len()))?;
        ciphertext_size
            .checked_sub((C::CIPHERTEXT_OVERHEAD_PREFIX + C::CIPHERTEXT_OVERHEAD_SUFFIX) as u64)
            .with_context(|| anyhow!("Physical block size of {} is too small.", block_size))
    }

    async fn all_blocks(&self) -> Result<Pin<Box<dyn Stream<Item = Result<BlockId>> + Send>>> {
        self.underlying_block_store
            .deref()
            .borrow()
            .all_blocks()
            .await
    }
}

#[async_trait]
impl<
        C: 'static + Cipher + Send + Sync,
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
        C: 'static + Cipher + Send + Sync,
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
        C: 'static + Cipher + Send + Sync,
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
        C: 'static + Cipher + Send + Sync,
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
        C: 'static + Cipher + Send + Sync,
        _B: BlockStore + OptimizedBlockStoreWriter + Send + Sync + Debug,
        B: 'static + Debug + AsyncDrop<Error = anyhow::Error> + Borrow<_B> + Send + Sync,
    > BlockStore for EncryptedBlockStore<C, _B, B>
{
}

impl<
        C: 'static + Cipher,
        _B: Debug,
        B: 'static + Debug + AsyncDrop<Error = anyhow::Error> + Borrow<_B> + Send + Sync,
    > EncryptedBlockStore<C, _B, B>
{
    async fn _encrypt(&self, plaintext: Data) -> Result<Data> {
        // TODO Is it better to move encryption/decryption to a dedicated threadpool instead of block_in_place?
        let ciphertext = tokio::task::block_in_place(move || self.cipher.encrypt(plaintext))?;
        Ok(_prepend_header(ciphertext))
    }

    async fn _decrypt(&self, ciphertext: Data) -> Result<Data> {
        let ciphertext = _check_and_remove_header(ciphertext)?;
        tokio::task::block_in_place(move || self.cipher.decrypt(ciphertext)).map(|d| d.into())
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

    use generic_array::ArrayLength;
    use rand::{rngs::StdRng, RngCore, SeedableRng};
    use std::marker::PhantomData;

    use crate::blockstore::low_level::{
        inmemory::InMemoryBlockStore, BlockStoreReader, BlockStoreWriter,
    };
    use crate::blockstore::tests::{blockid, data, Fixture};
    use crate::crypto::symmetric::{Aes128Gcm, Aes256Gcm, EncryptionKey, XChaCha20Poly1305};
    use crate::instantiate_blockstore_tests;
    use crate::utils::async_drop::AsyncDropArc;
    use crate::utils::async_drop::SyncDrop;

    fn key<KeySize: ArrayLength<u8>>(seed: u64) -> EncryptionKey<KeySize> {
        EncryptionKey::new(|key_data| {
            let mut rng = StdRng::seed_from_u64(seed);
            rng.fill_bytes(key_data);
            Ok(())
        })
        .unwrap()
    }

    struct TestFixture<C: 'static + Cipher + Send + Sync> {
        _c: PhantomData<C>,
    }
    #[async_trait]
    impl<C: 'static + Cipher + Send + Sync> Fixture for TestFixture<C> {
        type ConcreteBlockStore = EncryptedBlockStore<C, InMemoryBlockStore, InMemoryBlockStore>;
        fn new() -> Self {
            Self { _c: PhantomData }
        }
        async fn store(&mut self) -> SyncDrop<Self::ConcreteBlockStore> {
            SyncDrop::new(EncryptedBlockStore::new(
                InMemoryBlockStore::new(),
                Cipher::new(key(0)),
            ))
        }
        async fn yield_fixture(&self, _store: &Self::ConcreteBlockStore) {}
    }

    mod aes256gcm {
        use super::*;
        instantiate_blockstore_tests!(super::TestFixture<Aes256Gcm>, (flavor = "multi_thread"));
    }
    mod aes128gcm {
        use super::*;
        crate::instantiate_blockstore_tests!(
            super::TestFixture<Aes128Gcm>,
            (flavor = "multi_thread")
        );
    }

    mod xchachapoly1305 {
        use super::*;
        crate::instantiate_blockstore_tests!(
            super::TestFixture<XChaCha20Poly1305>,
            (flavor = "multi_thread")
        );
    }

    #[tokio::test]
    async fn test_block_size_from_physical_block_size() {
        async fn _test_block_size_from_physical_block_size<C: 'static + Cipher + Send + Sync>() {
            let mut fixture = TestFixture::<C>::new();
            let store = fixture.store().await;
            let expected_overhead: u64 = FORMAT_VERSION_HEADER.len() as u64
                + C::CIPHERTEXT_OVERHEAD_PREFIX as u64
                + C::CIPHERTEXT_OVERHEAD_SUFFIX as u64;

            assert_eq!(
                0u64,
                store
                    .block_size_from_physical_block_size(expected_overhead)
                    .unwrap()
            );
            assert_eq!(
                20u64,
                store
                    .block_size_from_physical_block_size(expected_overhead + 20u64)
                    .unwrap()
            );
            assert!(store.block_size_from_physical_block_size(0).is_err());
        }

        _test_block_size_from_physical_block_size::<Aes256Gcm>().await;
        _test_block_size_from_physical_block_size::<Aes128Gcm>().await;
        _test_block_size_from_physical_block_size::<XChaCha20Poly1305>().await;
    }

    async fn _store(
        bs: &AsyncDropGuard<AsyncDropArc<InMemoryBlockStore>>,
        key: EncryptionKey<<Aes256Gcm as Cipher>::KeySize>,
        block_id: &BlockId,
        data: &Data,
    ) {
        let mut store = EncryptedBlockStore::<
            Aes256Gcm,
            InMemoryBlockStore,
            AsyncDropArc<InMemoryBlockStore>,
        >::new(AsyncDropArc::clone(&bs), Aes256Gcm::new(key));

        store.store(block_id, data).await.unwrap();
        store.async_drop().await.unwrap();
    }

    async fn _load(
        bs: &AsyncDropGuard<AsyncDropArc<InMemoryBlockStore>>,
        key: EncryptionKey<<Aes256Gcm as Cipher>::KeySize>,
        block_id: &BlockId,
    ) -> Result<Option<Data>> {
        let mut store = EncryptedBlockStore::<
            Aes256Gcm,
            InMemoryBlockStore,
            AsyncDropArc<InMemoryBlockStore>,
        >::new(AsyncDropArc::clone(&bs), Aes256Gcm::new(key));
        let result = store.load(block_id).await;

        store.async_drop().await.unwrap();
        result
    }

    async fn _manipulate(
        bs: &AsyncDropGuard<AsyncDropArc<InMemoryBlockStore>>,
        block_id: &BlockId,
    ) {
        let mut data = bs.load(block_id).await.unwrap().unwrap();
        data[0] += 1;
        bs.store(block_id, &data).await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_loading_with_same_key_works() {
        let mut inner = AsyncDropArc::new(InMemoryBlockStore::new());

        _store(&inner, key(0), &blockid(0), &data(1024, 0)).await;
        assert_eq!(
            Some(data(1024, 0)),
            _load(&inner, key(0), &blockid(0)).await.unwrap()
        );

        inner.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_loading_with_different_key_doesnt_work() {
        let mut inner = AsyncDropArc::new(InMemoryBlockStore::new());

        _store(&inner, key(0), &blockid(0), &data(1024, 0)).await;
        _load(&inner, key(1), &blockid(0)).await.unwrap_err();

        inner.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_loading_manipulated_block_doesnt_work() {
        let mut inner = AsyncDropArc::new(InMemoryBlockStore::new());

        _store(&inner, key(0), &blockid(0), &data(1024, 0)).await;
        _manipulate(&inner, &blockid(0)).await;
        _load(&inner, key(0), &blockid(0)).await.unwrap_err();

        inner.async_drop().await.unwrap();
    }
}
