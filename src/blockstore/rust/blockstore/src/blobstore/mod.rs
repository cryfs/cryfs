use anyhow::Result;
use async_trait::async_trait;

use crate::blockstore::BlockId;
use crate::data::Data;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlobId {
    root: BlockId,
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
    // TODO Can we make size take &self instead of &mut self?
    async fn size(&mut self) -> Result<u64>;

    // fn read_all(&self) -> Data;
    // fn read(&self, target: &mut [u8], offset: u64, size: u64) -> Result<()>;
    // fn try_read(&self, target: &mut [u8], offset: u64, size: u64) -> Result<()>;
    // fn write(&self, source: &[u8], offset: u64, size: u64) -> Result<()>;

    // fn flush(&self) -> Result<()>;
    // fn num_nodes(&self) -> Result<()>;
}

#[async_trait]
pub trait BlobStore {
    type ConcreteBlob: Blob;

    // fn create(&self) -> Result<Self::ConcreteBlob>;
    fn load(&self, id: &BlobId) -> Result<Option<Self::ConcreteBlob>>;
    // fn remove_by_id(&self, id: &BlobId) -> Result<RemoveResult>;
    // // fn remove_blob(&self, blob: Self::ConcreteBlob) -> Result<()>;
    // fn num_blocks(&self) -> Result<u64>;
    // fn estimate_space_for_num_blocks_left() -> Result<u64>;
    // //virtual means "space we can use" as opposed to "space it takes on the disk" (i.e. virtual is without headers, checksums, ...)
    // fn virtual_blocksize_bytes() -> Result<u64>;
}

pub mod on_blocks;
