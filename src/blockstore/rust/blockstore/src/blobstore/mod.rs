use anyhow::Result;
use async_trait::async_trait;

use crate::blockstore::{BlockId, BLOCKID_LEN};
use crate::data::Data;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlobId {
    root: BlockId,
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

#[derive(Debug, PartialEq, Eq)]
#[must_use]
pub enum TryCreateResult {
    SuccessfullyCreated,
    NotCreatedBecauseBlockIdAlreadyExists,
}

#[derive(Debug, PartialEq, Eq)]
#[must_use]
pub enum RemoveResult {
    SuccessfullyRemoved,
    NotRemovedBecauseItDoesntExist,
}

#[async_trait]
pub trait Blob {
    fn id(&self) -> BlobId;
    // TODO Can we make size take &self instead of &mut self? Same for other read-only functions?
    async fn num_bytes(&mut self) -> Result<u64>;
    async fn resize(&mut self, new_num_bytes: u64) -> Result<()>;

    async fn read_all(&mut self) -> Result<Data>;
    async fn read(&mut self, target: &mut [u8], offset: u64) -> Result<()>;
    async fn try_read(&mut self, target: &mut [u8], offset: u64) -> Result<usize>;
    async fn write(&mut self, source: &[u8], offset: u64) -> Result<()>;

    async fn flush(&mut self) -> Result<()>;
    async fn num_nodes(&mut self) -> Result<u64>;

    async fn remove(self) -> Result<()>;
}

#[async_trait]
pub trait BlobStore {
    type ConcreteBlob: Blob;

    async fn create(&self) -> Result<Self::ConcreteBlob>;
    async fn load(&self, id: &BlobId) -> Result<Option<Self::ConcreteBlob>>;
    async fn remove_by_id(&self, id: &BlobId) -> Result<RemoveResult>;
    async fn num_nodes(&self) -> Result<u64>;
    fn estimate_space_for_num_blocks_left(&self) -> Result<u64>;
    // //virtual means "space we can use" as opposed to "space it takes on the disk" (i.e. virtual is without headers, checksums, ...)
    fn virtual_block_size_bytes(&self) -> Result<u64>;
}

pub mod on_blocks;
