use anyhow::{anyhow, bail, Context, Result};
use std::convert::TryInto;
use std::ops::Add;
use typenum::Unsigned;

use super::{
    BlockId, BlockStore, BlockStoreDeleter, BlockStoreReader, OptimizedBlockStoreWriter,
    OptimizedBlockStoreWriterMetadata,
};

use super::block_data::IBlockData;
use crate::crypto::symmetric::Cipher;
use crate::data::{Data, GrowableData};

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

impl<C: Cipher, B: OptimizedBlockStoreWriterMetadata> OptimizedBlockStoreWriterMetadata
    for EncryptedBlockStore<C, B>
{
    type RequiredPrefixBytesBase =
        <B::RequiredPrefixBytesBase as Add<B::RequiredPrefixBytesSelf>>::Output;
    type RequiredPrefixBytesSelf = typenum::U50; // TODO FORMAT_VERSION_HEADER.len() + C::CIPHERTEXT_OVERHEAD;
}

impl<C: Cipher, B: OptimizedBlockStoreWriter> OptimizedBlockStoreWriter
    for EncryptedBlockStore<C, B>
{
    fn allocate(
        size: usize,
    ) -> GrowableData<
        <Self::RequiredPrefixBytesBase as Add<Self::RequiredPrefixBytesSelf>>::Output,
        typenum::U0,
    > {
        Data::from(vec![
            0;
            Self::RequiredPrefixBytesBase::USIZE
                + Self::RequiredPrefixBytesSelf::USIZE
                + size
        ])
        .into_subregion(
            (Self::RequiredPrefixBytesBase::USIZE + Self::RequiredPrefixBytesSelf::USIZE)..,
        )
        .try_into()
        .unwrap()
    }

    fn try_create_optimized(
        &self,
        id: &BlockId,
        data: GrowableData<
            <Self::RequiredPrefixBytesBase as Add<Self::RequiredPrefixBytesSelf>>::Output,
            typenum::U0,
        >,
    ) -> Result<bool> {
        // TODO remove try_into / extract
        let ciphertext: GrowableData<
            <Self::RequiredPrefixBytesBase as Add<Self::RequiredPrefixBytesSelf>>::Output,
            typenum::U0,
        > = data.extract().try_into().unwrap();
        let ciphertext: GrowableData<Self::RequiredPrefixBytesBase, typenum::U0> =
            self._encrypt(ciphertext)?;
        self.underlying_block_store
            .try_create_optimized(id, ciphertext)
    }

    fn store_optimized(
        &self,
        id: &BlockId,
        data: GrowableData<
            <Self::RequiredPrefixBytesBase as Add<Self::RequiredPrefixBytesSelf>>::Output,
            typenum::U0,
        >,
    ) -> Result<()> {
        // TODO remove try_into / extract
        let ciphertext: GrowableData<
            <Self::RequiredPrefixBytesBase as Add<Self::RequiredPrefixBytesSelf>>::Output,
            typenum::U0,
        > = data.extract().try_into().unwrap();
        let ciphertext: GrowableData<Self::RequiredPrefixBytesBase, typenum::U0> =
            self._encrypt(ciphertext)?;
        self.underlying_block_store.store_optimized(id, ciphertext)
    }
}

impl<C: Cipher, B: BlockStore + OptimizedBlockStoreWriter> BlockStore
    for EncryptedBlockStore<C, B>
{
}

impl<C: Cipher, B: OptimizedBlockStoreWriter> EncryptedBlockStore<C, B> {
    fn _encrypt(
        &self,
        plaintext: GrowableData<
            <<Self as OptimizedBlockStoreWriterMetadata>::RequiredPrefixBytesBase as Add<
                <Self as OptimizedBlockStoreWriterMetadata>::RequiredPrefixBytesSelf,
            >>::Output,
            typenum::U0,
        >,
    ) -> Result<
        GrowableData<
            <Self as OptimizedBlockStoreWriterMetadata>::RequiredPrefixBytesBase,
            typenum::U0,
        >,
    > {
        // TODO Avoid _prepend_header, instead directly encrypt into a pre-allocated cipherdata Vec<u8>
        let ciphertext = self.cipher.encrypt(plaintext)?;
        todo!()
        // TODO
        // Ok(_prepend_header(ciphertext))
    }
}
impl<C: Cipher, B> EncryptedBlockStore<C, B> {
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

// fn _prepend_header<const PREFIX_BYTES: usize>(
//     data: GrowableData<PREFIX_BYTES, 0>,
// ) -> GrowableData<{ sub_header_len(PREFIX_BYTES) }, 0> {
//     // TODO Use binary-layout here?
//     let mut data = data.grow_region::<{ FORMAT_VERSION_HEADER.len() }, 0>();
//     data.as_mut()[..FORMAT_VERSION_HEADER.len()].copy_from_slice(FORMAT_VERSION_HEADER);
//     data
// }

// const fn sub_header_len(size: usize) -> usize {
//     size - FORMAT_VERSION_HEADER.len()
// }
