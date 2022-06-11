use anyhow::{anyhow, bail, Context, Result};

use super::{BlockId, BlockStore, BlockStoreReader, BlockStoreWriter};

use crate::crypto::symmetric::Cipher;

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
    fn load(&self, id: &BlockId) -> Result<Option<Vec<u8>>> {
        let loaded = self.underlying_block_store.load(id)?;
        match loaded {
            None => Ok(None),
            Some(data) => Ok(Some(self._decrypt(&data)?)),
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
        C::plaintext_size(ciphertext_size as usize)
            .with_context(|| anyhow!("Physical block size of {} is too small.", block_size))
            .map(|a| a as u64)
    }

    fn all_blocks(&self) -> Result<Box<dyn Iterator<Item = BlockId>>> {
        self.underlying_block_store.all_blocks()
    }
}

impl<C: Cipher, B: BlockStoreWriter> BlockStoreWriter for EncryptedBlockStore<C, B> {
    fn try_create(&self, id: &BlockId, data: &[u8]) -> Result<bool> {
        let ciphertext = self._encrypt(&data)?;
        self.underlying_block_store.try_create(id, &ciphertext)
    }

    fn remove(&self, id: &BlockId) -> Result<bool> {
        self.underlying_block_store.remove(id)
    }

    fn store(&self, id: &BlockId, data: &[u8]) -> Result<()> {
        let ciphertext = self._encrypt(&data)?;
        self.underlying_block_store.store(id, &ciphertext)
    }
}

impl<C: Cipher, B: BlockStore> BlockStore for EncryptedBlockStore<C, B> {}

impl<C: Cipher, B> EncryptedBlockStore<C, B> {
    fn _encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        // TODO Avoid _prepend_header, instead directly encrypt into a pre-allocated cipherdata Vec<u8>
        let ciphertext = self.cipher.encrypt(plaintext)?;
        Ok(_prepend_header(ciphertext))
    }

    fn _decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let ciphertext = _check_and_remove_header(&ciphertext)?;
        self.cipher.decrypt(ciphertext)
    }
}

fn _check_and_remove_header(data: &[u8]) -> Result<&[u8]> {
    if !data.starts_with(FORMAT_VERSION_HEADER) {
        bail!(
            "Couldn't parse encrypted block. Expected FORMAT_VERSION_HEADER of {:?} but found {:?}",
            FORMAT_VERSION_HEADER,
            &data[..FORMAT_VERSION_HEADER.len()]
        );
    }
    Ok(&data[FORMAT_VERSION_HEADER.len()..])
}

fn _prepend_header(data: Vec<u8>) -> Vec<u8> {
    let mut result = Vec::with_capacity(FORMAT_VERSION_HEADER.len() + data.len());
    result.extend_from_slice(FORMAT_VERSION_HEADER);
    result.extend_from_slice(&data);
    result
}
