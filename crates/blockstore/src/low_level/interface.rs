use anyhow::Result;
use byte_unit::Byte;
use futures::stream::BoxStream;
use std::any::Any;
use std::fmt::Debug;

use crate::{
    BlockId, Overhead,
    utils::{RemoveResult, TryCreateResult},
};
use cryfs_utils::{async_drop::AsyncDrop, data::Data};

pub trait BlockStoreReader {
    // TODO Add test cases for exists(), they're not among the C++ test cases since we added it later
    fn exists(&self, id: &BlockId) -> impl Future<Output = Result<bool>> + Send;
    fn load(&self, id: &BlockId) -> impl Future<Output = Result<Option<Data>>> + Send;
    fn num_blocks(&self) -> impl Future<Output = Result<u64>> + Send;
    fn estimate_num_free_bytes(&self) -> Result<Byte>;
    fn overhead(&self) -> Overhead;

    fn all_blocks(
        &self,
    ) -> impl Future<Output = Result<BoxStream<'static, Result<BlockId>>>> + Send;
}

pub trait BlockStoreDeleter {
    fn remove(&self, id: &BlockId) -> impl Future<Output = Result<RemoveResult>> + Send;
}

pub trait BlockStoreWriter {
    fn try_create(
        &self,
        id: &BlockId,
        data: &[u8],
    ) -> impl Future<Output = Result<TryCreateResult>> + Send;
    fn store(&self, id: &BlockId, data: &[u8]) -> impl Future<Output = Result<()>> + Send;
}

pub trait OptimizedBlockStoreWriter {
    /// In-memory representation of the data of a block. This can be allocated using [OptimizedBlockStoreWriter::allocate]
    /// and then can be passed to [OptimizedBlockStoreWriter::try_create_optimized] or [OptimizedBlockStoreWriter::store_optimized].
    ///
    /// The reason we use this class and don't use just [cryfs_utils::data::Data] or `&[u8]` is for optimizations purposes.
    /// Some blockstores prepend header to the data before storing and require the block data to be set up in a way
    /// that makes sure that data can be prepended without having to copy the block data.
    type BlockData: block_data::IBlockData + Send;

    /// Allocates an in-memory representation of a data block that can be written to
    /// and that can then be passed to [OptimizedBlockStoreWriter::try_create_optimized] or [OptimizedBlockStoreWriter::store_optimized].
    fn allocate(size: usize) -> Self::BlockData;

    fn try_create_optimized(
        &self,
        id: &BlockId,
        data: Self::BlockData,
    ) -> impl Future<Output = Result<TryCreateResult>> + Send;

    fn store_optimized(
        &self,
        id: &BlockId,
        data: Self::BlockData,
    ) -> impl Future<Output = Result<()>> + Send;
}

impl<B: OptimizedBlockStoreWriter + Sync> BlockStoreWriter for B {
    async fn try_create(&self, id: &BlockId, data: &[u8]) -> Result<TryCreateResult> {
        let mut block_data = Self::allocate(data.len());
        assert_eq!(block_data.as_ref().len(), data.len());
        block_data.as_mut().copy_from_slice(data);
        self.try_create_optimized(id, block_data).await
    }

    async fn store(&self, id: &BlockId, data: &[u8]) -> Result<()> {
        let mut block_data = Self::allocate(data.len());
        assert_eq!(block_data.as_ref().len(), data.len());
        block_data.as_mut().copy_from_slice(data);
        self.store_optimized(id, block_data).await
    }
}

pub trait LLBlockStore:
    BlockStoreReader
    + BlockStoreWriter
    + BlockStoreDeleter
    + AsyncDrop<Error = anyhow::Error>
    + Debug
    + Any
{
}

/// BlockData instances wrap a [Data] instance and guarantee the upholding of an
/// important invariant for [OptimizedBlockStoreWriter], namely that the data stored
/// has enough prefix bytes available and can be grown during the writing process
/// to e.g. add a block header without requiring the block data to be copied.
/// Such BlockData instances can be created with the [block_data::create_block_data_wrapper!] macro.
///
/// This not being public is an important part of our safety net.
/// Only things in the blockstore module can create instances of this,
/// so we can make sure the invariants are always kept.
pub(crate) mod block_data {
    use cryfs_utils::data::Data;

    pub trait IBlockData: AsRef<[u8]> + AsMut<[u8]> + Clone {
        // TODO Rename to new_unchecked ?
        fn new(data: Data) -> Self;
        fn extract(self) -> Data;
    }

    macro_rules! create_block_data_wrapper_ {
        ($name: ident) => {
            #[derive(Clone)]
            pub struct $name(Data);

            impl AsRef<[u8]> for BlockData {
                fn as_ref(&self) -> &[u8] {
                    self.0.as_ref()
                }
            }

            impl AsMut<[u8]> for BlockData {
                fn as_mut(&mut self) -> &mut [u8] {
                    self.0.as_mut()
                }
            }

            impl $crate::low_level::interface::block_data::IBlockData for $name {
                fn new(data: Data) -> Self {
                    Self(data)
                }

                fn extract(self) -> Data {
                    self.0
                }
            }
        };
    }
    pub(crate) use create_block_data_wrapper_ as create_block_data_wrapper;
}
