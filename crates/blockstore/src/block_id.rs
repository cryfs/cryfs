use anyhow::Result;
use binrw::{BinRead, BinResult, BinWrite, Endian};
use rand::{thread_rng, Rng};
use std::io::{Read, Seek, Write};

pub const BLOCKID_LEN: usize = 16;

// TODO We could optimize the Hash implementation since BlockId is always random. Just take the first x bytes as the hash.

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

    pub const fn zero() -> Self {
        Self {
            id: [0; BLOCKID_LEN],
        }
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
        hex::encode(self.data())
    }

    pub fn to_hex_upper(&self) -> String {
        hex::encode_upper(self.data())
    }
}

impl std::fmt::Display for BlockId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl std::fmt::Debug for BlockId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlockId({self})")
    }
}

impl BinRead for BlockId {
    type Args<'a> = ();
    fn read_options<R: Read + Seek>(reader: &mut R, endian: Endian, _: ()) -> BinResult<BlockId> {
        let blockid = <[u8; BLOCKID_LEN]>::read_options(reader, endian, ())?;
        let blockid = BlockId::from_slice(&blockid)
            .expect("Can't fail because we pass in an array of exactly the right size");
        Ok(blockid)
    }
}

impl BinWrite for BlockId {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: (),
    ) -> Result<(), binrw::Error> {
        <[u8; BLOCKID_LEN]>::write_options(self.data(), writer, endian, args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cryfs_utils::binary::{BinaryReadExt, BinaryWriteExt};
    use std::io::Cursor;

    #[test]
    fn serialize_deserialize_blockid() {
        let blockid = BlockId::from_hex("ea92df46054175fe9ec0dec871d3affd").unwrap();
        let mut writer = Cursor::new(Vec::new());
        blockid.serialize_to_stream(&mut writer).unwrap();
        let serialized = writer.into_inner();
        let deserialized =
            BlockId::deserialize_from_complete_stream(&mut Cursor::new(serialized)).unwrap();
        assert_eq!(blockid, deserialized);
    }

    #[test]
    fn test_display() {
        const HEX: &str = "0070e99ada93ef706935f4693039c900";
        let blob_id = BlockId::from_hex(HEX).unwrap();
        assert_eq!(HEX, format!("{blob_id}"));
    }

    #[test]
    fn test_debug() {
        const HEX: &str = "0070e99ada93ef706935f4693039c900";
        let blob_id = BlockId::from_hex(HEX).unwrap();
        assert_eq!(
            "BlockId(0070e99ada93ef706935f4693039c900)",
            format!("{:?}", blob_id),
        );
    }

    // TODO Other tests
}
