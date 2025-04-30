use anyhow::Result;
use async_trait::async_trait;
use byte_unit::Byte;
use futures::stream::BoxStream;
use std::fmt::Debug;

use cryfs_blockstore::{BLOCKID_LEN, BlockId};
use cryfs_utils::data::Data;

use crate::{BlobId, RemoveResult};

pub const BLOBID_LEN: usize = BLOCKID_LEN;

#[async_trait]
pub trait Blob: Sized + Debug {
    fn id(&self) -> BlobId;
    // TODO Can we make size take &self instead of &mut self? Same for other read-only functions?
    async fn num_bytes(&mut self) -> Result<u64>;
    async fn resize(&mut self, new_num_bytes: u64) -> Result<()>;

    async fn read_all(&mut self) -> Result<Data>;
    async fn read(&mut self, target: &mut [u8], offset: u64) -> Result<()>;
    async fn try_read(&mut self, target: &mut [u8], offset: u64) -> Result<usize>;
    async fn write(&mut self, source: &[u8], offset: u64) -> Result<()>;

    async fn flush(&mut self) -> Result<()>;

    // TODO `num_nodes` and `all_blocks` is a leaky abstraction because it gives away that we use blocks. Remove these.
    async fn num_nodes(&mut self) -> Result<u64>;

    async fn remove(self) -> Result<()>;

    fn all_blocks(&self) -> Result<BoxStream<'_, Result<BlockId>>>;
}

#[async_trait]
pub trait BlobStore {
    // TODO Remove Send+Sync bound
    type ConcreteBlob<'a>: Blob + Debug + Send + Sync
    where
        Self: 'a;

    async fn create(&self) -> Result<Self::ConcreteBlob<'_>>;
    async fn try_create(&self, id: &BlobId) -> Result<Option<Self::ConcreteBlob<'_>>>;
    async fn load(&self, id: &BlobId) -> Result<Option<Self::ConcreteBlob<'_>>>;
    async fn remove_by_id(&self, id: &BlobId) -> Result<RemoveResult>;
    async fn num_nodes(&self) -> Result<u64>;
    fn estimate_space_for_num_blocks_left(&self) -> Result<u64>;
    // virtual means "space we can use" as opposed to "space it takes on the disk" (i.e. virtual is without headers, checksums, ...)
    fn virtual_block_size_bytes(&self) -> Byte;

    // TODO load_block_depth is only needed for our c++ bindings of the stats tool. Remove them.
    async fn load_block_depth(&self, _id: &cryfs_blockstore::BlockId) -> Result<Option<u8>>;

    #[cfg(any(test, feature = "testutils"))]
    async fn clear_cache_slow(&self) -> Result<()>;

    #[cfg(test)]
    async fn all_blobs(&self) -> Result<Vec<BlobId>>;
}
