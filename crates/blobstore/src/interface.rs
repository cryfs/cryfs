use anyhow::Result;
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

pub trait Blob: Sized + Debug + AsyncDrop {
    fn id(&self) -> BlobId;
    // TODO Can we make size take &self instead of &mut self? Same for other read-only functions?
    fn num_bytes(&mut self) -> impl Future<Output = Result<u64>> + Send;
    fn resize(&mut self, new_num_bytes: u64) -> impl Future<Output = Result<()>> + Send;

    fn read_all(&mut self) -> impl Future<Output = Result<Data>> + Send;
    fn read(&mut self, target: &mut [u8], offset: u64) -> impl Future<Output = Result<()>> + Send;
    fn try_read(
        &mut self,
        target: &mut [u8],
        offset: u64,
    ) -> impl Future<Output = Result<usize>> + Send;
    fn write(&mut self, source: &[u8], offset: u64) -> impl Future<Output = Result<()>> + Send;

    fn flush(&mut self) -> impl Future<Output = Result<()>> + Send;

    // TODO `num_nodes` and `all_blocks` is a leaky abstraction because it gives away that we use blocks. Remove these.
    fn num_nodes(&mut self) -> impl Future<Output = Result<u64>> + Send;

    fn remove(this: AsyncDropGuard<Self>) -> impl Future<Output = Result<()>> + Send;

    fn all_blocks(&self) -> Result<BoxStream<'_, Result<BlockId>>>;
}

pub trait BlobStore {
    // TODO Remove Send+Sync bound
    type ConcreteBlob: Blob + Debug + Send + Sync;

    fn create(&self) -> impl Future<Output = Result<AsyncDropGuard<Self::ConcreteBlob>>> + Send;
    fn try_create(
        &self,
        id: &BlobId,
    ) -> impl Future<Output = Result<Option<AsyncDropGuard<Self::ConcreteBlob>>>> + Send;
    fn load(
        &self,
        id: &BlobId,
    ) -> impl Future<Output = Result<Option<AsyncDropGuard<Self::ConcreteBlob>>>> + Send;
    fn remove_by_id(&self, id: &BlobId) -> impl Future<Output = Result<RemoveResult>> + Send;
    fn num_nodes(&self) -> impl Future<Output = Result<u64>> + Send;
    fn estimate_space_for_num_blocks_left(&self) -> Result<u64>;
    // logical means "space we can use" as opposed to "space it takes on the disk" (i.e. logical is without headers, checksums, ...)
    fn logical_block_size_bytes(&self) -> Byte;

    fn flush_if_cached(&self, blob_id: BlobId) -> impl Future<Output = Result<()>> + Send;

    #[cfg(any(test, feature = "testutils"))]
    fn clear_cache_slow(&self) -> impl Future<Output = Result<()>> + Send;
    #[cfg(any(test, feature = "testutils"))]
    fn clear_unloaded_blocks_from_cache(&self) -> impl Future<Output = Result<()>> + Send;

    #[cfg(test)]
    fn all_blobs(&self) -> impl Future<Output = Result<Vec<BlobId>>> + Send;
}
