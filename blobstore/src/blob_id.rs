use anyhow::Result;
use binrw::{BinRead, BinWrite};

use cryfs_blockstore::{BlockId, BLOCKID_LEN};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, BinRead, BinWrite)]
pub struct BlobId {
    pub(super) root: BlockId,
}

impl BlobId {
    pub fn new_random() -> Self {
        Self {
            root: BlockId::new_random(),
        }
    }

    #[inline]
    pub fn from_slice(id_data: &[u8]) -> Result<Self> {
        Ok(Self {
            root: BlockId::from_slice(id_data)?,
        })
    }

    #[inline]
    pub fn from_array(id: &[u8; BLOCKID_LEN]) -> Self {
        Self {
            root: BlockId::from_array(id),
        }
    }

    #[inline]
    pub fn data(&self) -> &[u8; BLOCKID_LEN] {
        self.root.data()
    }

    pub fn from_hex(hex_data: &str) -> Result<Self> {
        Ok(Self {
            root: BlockId::from_hex(hex_data)?,
        })
    }

    pub fn to_hex(&self) -> String {
        self.root.to_hex()
    }
}

impl std::fmt::Debug for BlobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlobId({})", self.root.to_hex())
    }
}
