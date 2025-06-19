use anyhow::Result;
use async_trait::async_trait;
use byte_unit::Byte;
use futures::stream::BoxStream;
use std::fmt::Debug;

use cryfs_blockstore::{BLOCKID_LEN, BlockId};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    data::Data,
};

use crate::{BlobId, RemoveResult};

pub const BLOBID_LEN: usize = BLOCKID_LEN;

#[async_trait]
pub trait Blob: Sized + Debug + AsyncDrop {
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

    async fn remove(this: AsyncDropGuard<Self>) -> Result<()>;

    fn all_blocks(&self) -> Result<BoxStream<'_, Result<BlockId>>>;
}

#[async_trait]
pub trait BlobStore {
    // TODO Remove Send+Sync bound
    type ConcreteBlob: Blob + Debug + Send + Sync;

    async fn create(&self) -> Result<AsyncDropGuard<Self::ConcreteBlob>>;
    async fn try_create(&self, id: &BlobId) -> Result<Option<AsyncDropGuard<Self::ConcreteBlob>>>;
    async fn load(&self, id: &BlobId) -> Result<Option<AsyncDropGuard<Self::ConcreteBlob>>>;
    async fn remove_by_id(&self, id: &BlobId) -> Result<RemoveResult>;
    async fn num_nodes(&self) -> Result<u64>;
    fn estimate_space_for_num_blocks_left(&self) -> Result<u64>;
    // logical means "space we can use" as opposed to "space it takes on the disk" (i.e. logical is without headers, checksums, ...)
    fn logical_block_size_bytes(&self) -> Byte;

    #[cfg(any(test, feature = "testutils"))]
    async fn clear_cache_slow(&self) -> Result<()>;

    #[cfg(test)]
    async fn all_blobs(&self) -> Result<Vec<BlobId>>;
}
