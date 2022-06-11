use anyhow::Result;
use binread::{BinRead, BinResult, ReadOptions};
use binwrite::{BinWrite, WriterOption};
use rand::{thread_rng, Rng};
use std::io::{Read, Seek, Write};

pub const BLOCKID_LEN: usize = 16;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockId {
    id: [u8; BLOCKID_LEN],
}

impl BlockId {
    pub fn new_random() -> Self {
        let mut result = Self {
            id: [0; BLOCKID_LEN],
        };
        let mut rng = thread_rng();
        rng.fill(&mut result.id);
        result
    }

    #[inline]
    pub fn from_slice(id_data: &[u8]) -> Result<Self> {
        Ok(Self::from_array(id_data.try_into()?))
    }

    #[inline]
    pub fn from_array(id: &[u8; BLOCKID_LEN]) -> Self {
        Self { id: *id }
    }

    #[inline]
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
