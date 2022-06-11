use anyhow::{anyhow, bail, Context, Result};

use super::{BlockId, BlockStore, BlockStoreDeleter, BlockStoreReader, OptimizedBlockStoreWriter};

use super::block_data::IBlockData;
use crate::crypto::symmetric::Cipher;
use crate::data::Data;

const FORMAT_VERSION_HEADER: &[u8; 2] = &1u16.to_ne_bytes();

pub struct EncryptedBlockStore<C: Cipher, B> {
    underlying_block_store: B,
    cipher: C,
}

impl<C: Cipher, B> EncryptedBlockStore<C, B> {
    pub fn new(underlying_block_store: B, cipher: C) -> Self {
        Self {
            underlying_block_store,
            cipher,
        }
    }
}

impl<C: Cipher, B: BlockStoreReader> BlockStoreReader for EncryptedBlockStore<C, B> {
    fn load(&self, id: &BlockId) -> Result<Option<Data>> {
        let loaded = self.underlying_block_store.load(id)?;
        match loaded {
            None => Ok(None),
            Some(data) => Ok(Some(self._decrypt(data)?)),
        }
    }

    fn num_blocks(&self) -> Result<u64> {
        self.underlying_block_store.num_blocks()
    }

    fn estimate_num_free_bytes(&self) -> Result<u64> {
        self.underlying_block_store.estimate_num_free_bytes()
    }

    fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64> {
        let ciphertext_size = block_size.checked_sub(FORMAT_VERSION_HEADER.len() as u64)
            .with_context(|| anyhow!("Physical block size of {} is too small to hold even the FORMAT_VERSION_HEADER. Must be at least {}.", block_size, FORMAT_VERSION_HEADER.len()))?;
        ciphertext_size
            .checked_sub(C::CIPHERTEXT_OVERHEAD as u64)
            .with_context(|| anyhow!("Physical block size of {} is too small.", block_size))
    }

    fn all_blocks(&self) -> Result<Box<dyn Iterator<Item = BlockId>>> {
        self.underlying_block_store.all_blocks()
    }
}

impl<C: Cipher, B: BlockStoreDeleter> BlockStoreDeleter for EncryptedBlockStore<C, B> {
    fn remove(&self, id: &BlockId) -> Result<bool> {
        self.underlying_block_store.remove(id)
    }
}

create_block_data_wrapper!(BlockData);

impl<C: Cipher, B: OptimizedBlockStoreWriter> OptimizedBlockStoreWriter
    for EncryptedBlockStore<C, B>
{
    type BlockData = BlockData;

    fn allocate(size: usize) -> Self::BlockData {
        let data = B::allocate(FORMAT_VERSION_HEADER.len() + C::CIPHERTEXT_OVERHEAD + size)
            .extract()
            .into_subregion((FORMAT_VERSION_HEADER.len() + C::CIPHERTEXT_OVERHEAD)..);
        BlockData::new(data)
    }

    fn try_create_optimized(&self, id: &BlockId, data: Self::BlockData) -> Result<bool> {
        let ciphertext = self._encrypt(data.extract())?;
        self.underlying_block_store
            .try_create_optimized(id, B::BlockData::new(ciphertext))
    }

    fn store_optimized(&self, id: &BlockId, data: Self::BlockData) -> Result<()> {
        let ciphertext = self._encrypt(data.extract())?;
        self.underlying_block_store
            .store_optimized(id, B::BlockData::new(ciphertext))
    }
}

impl<C: Cipher, B: BlockStore + OptimizedBlockStoreWriter> BlockStore
    for EncryptedBlockStore<C, B>
{
}

impl<C: Cipher, B> EncryptedBlockStore<C, B> {
    fn _encrypt(&self, plaintext: Data) -> Result<Data> {
        let ciphertext = self.cipher.encrypt(plaintext)?;
        Ok(_prepend_header(ciphertext))
    }

    fn _decrypt(&self, ciphertext: Data) -> Result<Data> {
        let ciphertext = _check_and_remove_header(ciphertext)?;
        self.cipher.decrypt(ciphertext).map(|d| d.into())
    }
}

fn _check_and_remove_header(data: Data) -> Result<Data> {
    if !data.starts_with(FORMAT_VERSION_HEADER) {
        bail!(
            "Couldn't parse encrypted block. Expected FORMAT_VERSION_HEADER of {:?} but found {:?}",
            FORMAT_VERSION_HEADER,
            &data[..FORMAT_VERSION_HEADER.len()]
        );
    }
    Ok(data.into_subregion(FORMAT_VERSION_HEADER.len()..))
}

fn _prepend_header(data: Data) -> Data {
    // TODO Use binary-layout here?
    let mut data = data.grow_region(FORMAT_VERSION_HEADER.len(), 0).expect(
        "Tried to grow the data to contain the header in EncryptedBlockStore::_prepend_header",
    );
    data.as_mut()[..FORMAT_VERSION_HEADER.len()].copy_from_slice(FORMAT_VERSION_HEADER);
    data
}
