use anyhow::{bail, Result};
use binread::{BinRead, BinResult, ReadOptions};
use binwrite::{BinWrite, WriterOption};
use futures::TryStreamExt;
use rand::{thread_rng, Rng};
use std::convert::TryInto;
use std::io::{Read, Seek, Write};
use std::path::Path;

use super::{
    encrypted::EncryptedBlockStore,
    inmemory::InMemoryBlockStore,
    integrity::{ClientId, IntegrityBlockStore, IntegrityConfig},
    ondisk::OnDiskBlockStore,
    BlockStore,
};
use crate::crypto::symmetric::{Aes256Gcm, Cipher, EncryptionKey};
use crate::data::Data;

pub const BLOCKID_LEN: usize = 16;

#[cxx::bridge]
mod ffi {
    #[namespace = "blockstore::rust::bridge"]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    struct BlockId {
        id: [u8; 16],
    }

    #[namespace = "blockstore::rust::bridge"]
    extern "Rust" {
        type OptionData;
        fn has_value(&self) -> bool;
        fn value(&self) -> Result<&[u8]>;
    }

    #[namespace = "blockstore::rust::bridge"]
    extern "Rust" {
        type RustBlockStore2Bridge;
        fn try_create(&self, id: &BlockId, data: &[u8]) -> Result<bool>;
        fn remove(&self, id: &BlockId) -> Result<bool>;
        fn load(&self, id: &BlockId) -> Result<Box<OptionData>>;
        fn store(&self, id: &BlockId, data: &[u8]) -> Result<()>;
        fn num_blocks(&self) -> Result<u64>;
        fn estimate_num_free_bytes(&self) -> Result<u64>;
        fn block_size_from_physical_block_size(&self, block_size: u64) -> u64;
        fn all_blocks(&self) -> Result<Vec<BlockId>>;

        fn new_inmemory_blockstore() -> Box<RustBlockStore2Bridge>;
        fn new_encrypted_inmemory_blockstore() -> Box<RustBlockStore2Bridge>;
        fn new_integrity_inmemory_blockstore(
            integrity_file_path: &str,
        ) -> Result<Box<RustBlockStore2Bridge>>;
        fn new_ondisk_blockstore(basedir: &str) -> Box<RustBlockStore2Bridge>;
    }
}

pub use ffi::BlockId;

impl BlockId {
    pub fn new_random() -> Self {
        let mut result = Self {
            id: [0; BLOCKID_LEN],
        };
        let mut rng = thread_rng();
        rng.fill(&mut result.id);
        result
    }
    pub fn from_slice(id_data: &[u8]) -> Result<Self> {
        Ok(Self::from_array(id_data.try_into()?))
    }
    pub fn from_array(id: &[u8; BLOCKID_LEN]) -> Self {
        Self { id: *id }
    }
    pub fn data(&self) -> &[u8; BLOCKID_LEN] {
        &self.id
    }
    pub fn from_hex(hex_data: &str) -> Result<Self> {
        Self::from_slice(&hex::decode(hex_data)?)
    }
    pub fn to_hex(&self) -> String {
        hex::encode_upper(self.data())
    }
}

impl std::fmt::Debug for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlockId({})", self.to_hex())
    }
}

impl BinRead for BlockId {
    type Args = ();
    fn read_options<R: Read + Seek>(reader: &mut R, ro: &ReadOptions, _: ()) -> BinResult<BlockId> {
        let blockid = <[u8; BLOCKID_LEN]>::read_options(reader, ro, ())?;
        let blockid = BlockId::from_slice(&blockid)
            .expect("Can't fail because we pass in an array of exactly the right size");
        Ok(blockid)
    }
}

impl BinWrite for BlockId {
    fn write_options<W: Write>(
        &self,
        writer: &mut W,
        wo: &WriterOption,
    ) -> Result<(), std::io::Error> {
        <[u8; BLOCKID_LEN]>::write_options(self.data(), writer, wo)
    }
}

pub struct OptionData(Option<Data>);

impl OptionData {
    fn has_value(&self) -> bool {
        self.0.is_some()
    }

    fn value(&self) -> Result<&[u8]> {
        match &self.0 {
            None => bail!("OptionData doesn't have a value"),
            Some(data) => Ok(data),
        }
    }
}

lazy_static::lazy_static! {
    static ref TOKIO_RUNTIME: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
}

struct RustBlockStore2Bridge(Box<dyn BlockStore>);

impl RustBlockStore2Bridge {
    fn try_create(&self, id: &BlockId, data: &[u8]) -> Result<bool> {
        // TODO Can we avoid a copy at the ffi boundary? i.e. use OptimizedBlockStoreWriter?
        TOKIO_RUNTIME.block_on(self.0.try_create(id, data))
    }
    fn remove(&self, id: &BlockId) -> Result<bool> {
        TOKIO_RUNTIME.block_on(self.0.remove(id))
    }
    fn load(&self, id: &BlockId) -> Result<Box<OptionData>> {
        let loaded = TOKIO_RUNTIME.block_on(self.0.load(id))?;
        Ok(Box::new(OptionData(loaded)))
    }
    fn store(&self, id: &BlockId, data: &[u8]) -> Result<()> {
        // TODO Can we avoid a copy at the ffi boundary? i.e. use OptimizedBlockStoreWriter?
        TOKIO_RUNTIME.block_on(self.0.store(id, data))
    }
    fn num_blocks(&self) -> Result<u64> {
        Ok(TOKIO_RUNTIME.block_on(self.0.num_blocks()).unwrap())
    }
    fn estimate_num_free_bytes(&self) -> Result<u64> {
        self.0.estimate_num_free_bytes()
    }
    fn block_size_from_physical_block_size(&self, block_size: u64) -> u64 {
        // In C++, the convention was to return 0 instead of an error,
        // so let's catch errors and return 0 instead.
        // TODO Is there a better way?
        self.0
            .block_size_from_physical_block_size(block_size)
            .unwrap_or(0)
    }
    fn all_blocks(&self) -> Result<Vec<BlockId>> {
        TOKIO_RUNTIME
            .block_on(async { TryStreamExt::try_collect(self.0.all_blocks().await?).await })
    }
}

fn new_inmemory_blockstore() -> Box<RustBlockStore2Bridge> {
    Box::new(RustBlockStore2Bridge(Box::new(InMemoryBlockStore::new())))
}

fn new_encrypted_inmemory_blockstore() -> Box<RustBlockStore2Bridge> {
    let key =
        EncryptionKey::from_hex("9726ca3703940a918802953d8db5996c5fb25008a20c92cb95aa4b8fe92702d9")
            .unwrap();
    Box::new(RustBlockStore2Bridge(Box::new(EncryptedBlockStore::new(
        InMemoryBlockStore::new(),
        Aes256Gcm::new(key),
    ))))
}

fn new_integrity_inmemory_blockstore(
    integrity_file_path: &str,
) -> Result<Box<RustBlockStore2Bridge>> {
    Ok(Box::new(RustBlockStore2Bridge(Box::new(
        IntegrityBlockStore::new(
            InMemoryBlockStore::new(),
            Path::new(integrity_file_path).to_path_buf(),
            ClientId { id: 1 },
            IntegrityConfig {
                allow_integrity_violations: false,
                missing_block_is_integrity_violation: true,
                on_integrity_violation: Box::new(|| {}),
            },
        )?,
    ))))
}

fn new_ondisk_blockstore(basedir: &str) -> Box<RustBlockStore2Bridge> {
    Box::new(RustBlockStore2Bridge(Box::new(OnDiskBlockStore::new(
        Path::new(basedir).to_path_buf(),
    ))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::binary::{BinaryReadExt, BinaryWriteExt};
    use std::io::Cursor;

    #[test]
    fn serialize_deserialize_blockid() {
        let blockid = BlockId::from_hex("ea92df46054175fe9ec0dec871d3affd").unwrap();
        let mut serialized = Vec::new();
        blockid.serialize_to_stream(&mut serialized).unwrap();
        let deserialized =
            BlockId::deserialize_from_complete_stream(&mut Cursor::new(serialized)).unwrap();
        assert_eq!(blockid, deserialized);
    }
}
