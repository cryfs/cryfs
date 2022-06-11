use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use futures::stream::Stream;
use std::fmt::{self, Debug};
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

pub struct EncryptedBlockStore<C: 'static + Cipher, B: Debug + AsyncDrop<Error = anyhow::Error>> {
    underlying_block_store: AsyncDropGuard<B>,
    cipher: C,
}

impl<
        C: 'static + Cipher + Send + Sync,
        B: Debug + Send + Sync + AsyncDrop<Error = anyhow::Error>,
    > EncryptedBlockStore<C, B>
{
    pub fn new(underlying_block_store: AsyncDropGuard<B>, cipher: C) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            underlying_block_store,
            cipher,
        })
    }
}

#[async_trait]
impl<
        C: 'static + Cipher + Send + Sync,
        B: BlockStoreReader + Send + Sync + Debug + AsyncDrop<Error = anyhow::Error>,
    > BlockStoreReader for EncryptedBlockStore<C, B>
{
    async fn exists(&self, id: &BlockId) -> Result<bool> {
        self.underlying_block_store.exists(id).await
    }

    async fn load(&self, id: &BlockId) -> Result<Option<Data>> {
        let loaded = self.underlying_block_store.load(id).await.context(
            "EncryptedBlockStore failed to load the block from the underlying block store",
        )?;
        match loaded {
            None => Ok(None),
            Some(data) => Ok(Some(self._decrypt(data).await?)),
        }
    }

    async fn num_blocks(&self) -> Result<u64> {
        self.underlying_block_store.num_blocks().await
    }

    fn estimate_num_free_bytes(&self) -> Result<u64> {
        self.underlying_block_store.estimate_num_free_bytes()
    }

    fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64> {
        let ciphertext_size = self.underlying_block_store.block_size_from_physical_block_size(block_size)?.checked_sub(FORMAT_VERSION_HEADER.len() as u64)
            .with_context(|| anyhow!("Physical block size of {} is too small to hold even the FORMAT_VERSION_HEADER. Must be at least {}.", block_size, FORMAT_VERSION_HEADER.len()))?;
        ciphertext_size
            .checked_sub((C::CIPHERTEXT_OVERHEAD_PREFIX + C::CIPHERTEXT_OVERHEAD_SUFFIX) as u64)
            .with_context(|| anyhow!("Physical block size of {} is too small.", block_size))
    }

    async fn all_blocks(&self) -> Result<Pin<Box<dyn Stream<Item = Result<BlockId>> + Send>>> {
        self.underlying_block_store.all_blocks().await
    }
}

#[async_trait]
impl<
        C: 'static + Cipher + Send + Sync,
        B: BlockStoreDeleter + Send + Sync + Debug + AsyncDrop<Error = anyhow::Error>,
    > BlockStoreDeleter for EncryptedBlockStore<C, B>
{
    async fn remove(&self, id: &BlockId) -> Result<RemoveResult> {
        self.underlying_block_store.remove(id).await
    }
}

create_block_data_wrapper!(BlockData);

#[async_trait]
impl<
        C: 'static + Cipher + Send + Sync,
        B: OptimizedBlockStoreWriter + Send + Sync + Debug + AsyncDrop<Error = anyhow::Error>,
    > OptimizedBlockStoreWriter for EncryptedBlockStore<C, B>
{
    type BlockData = BlockData;

    fn allocate(size: usize) -> Self::BlockData {
        let mut data = B::allocate(
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
            .try_create_optimized(id, B::BlockData::new(ciphertext))
            .await
    }

    async fn store_optimized(&self, id: &BlockId, data: Self::BlockData) -> Result<()> {
        let ciphertext = self._encrypt(data.extract()).await?;
        self.underlying_block_store
            .store_optimized(id, B::BlockData::new(ciphertext))
            .await
    }
}

impl<C: 'static + Cipher + Send + Sync, B: Send + Debug + AsyncDrop<Error = anyhow::Error>> Debug
    for EncryptedBlockStore<C, B>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EncryptedBlockStore")
    }
}

#[async_trait]
impl<
        C: 'static + Cipher + Send + Sync,
        B: Sync + Send + Debug + AsyncDrop<Error = anyhow::Error>,
    > AsyncDrop for EncryptedBlockStore<C, B>
{
    type Error = anyhow::Error;
    async fn async_drop_impl(&mut self) -> Result<()> {
        self.underlying_block_store.async_drop().await
    }
}

impl<
        C: 'static + Cipher + Send + Sync,
        B: BlockStore + OptimizedBlockStoreWriter + Send + Sync + Debug,
    > BlockStore for EncryptedBlockStore<C, B>
{
}

impl<C: 'static + Cipher, B: Debug + AsyncDrop<Error = anyhow::Error>> EncryptedBlockStore<C, B> {
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

    use rand::{rngs::StdRng, RngCore, SeedableRng};

    use crate::blockstore::low_level::inmemory::InMemoryBlockStore;
    use crate::crypto::symmetric::{Aes128Gcm, Aes256Gcm, EncryptionKey, XChaCha20Poly1305};
    use crate::instantiate_blockstore_tests;
    use crate::utils::async_drop::SyncDrop;

    struct TestFixture<C: 'static + Cipher + Send + Sync> {
        store: SyncDrop<EncryptedBlockStore<C, InMemoryBlockStore>>,
    }
    impl<C: 'static + Cipher + Send + Sync> crate::blockstore::low_level::tests::Fixture
        for TestFixture<C>
    {
        type ConcreteBlockStore = EncryptedBlockStore<C, InMemoryBlockStore>;
        fn new() -> Self {
            let key = EncryptionKey::new(|key_data| {
                let mut rng = StdRng::seed_from_u64(0);
                rng.fill_bytes(key_data);
                Ok(())
            })
            .unwrap();
            Self {
                store: SyncDrop::new(EncryptedBlockStore::new(
                    InMemoryBlockStore::new(),
                    Cipher::new(key),
                )),
            }
        }
        fn store(&mut self) -> &mut Self::ConcreteBlockStore {
            &mut self.store
        }
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
}
