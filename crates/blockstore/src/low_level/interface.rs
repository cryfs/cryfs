use anyhow::Result;
use async_trait::async_trait;
use byte_unit::Byte;
use derive_more::{Display, Error};
use futures::stream::BoxStream;
use std::any::Any;
use std::fmt::Debug;

use crate::{
    utils::{RemoveResult, TryCreateResult},
    BlockId,
};
use cryfs_utils::{async_drop::AsyncDrop, data::Data};

#[derive(Error, Display, Debug)]
#[display("Invalid block size: {message}")]
pub struct InvalidBlockSizeError {
    message: String,
}
impl InvalidBlockSizeError {
    pub fn new(message: String) -> Self {
        Self { message: message }
    }
}

#[async_trait]
pub trait BlockStoreReader {
    // TODO Add test cases for exists(), they're not among the C++ test cases since we added it later
    async fn exists(&self, id: &BlockId) -> Result<bool>;
    async fn load(&self, id: &BlockId) -> Result<Option<Data>>;
    async fn num_blocks(&self) -> Result<u64>;
    fn estimate_num_free_bytes(&self) -> Result<Byte>;
    fn block_size_from_physical_block_size(
        &self,
        block_size: Byte,
    ) -> Result<Byte, InvalidBlockSizeError>;

    async fn all_blocks(&self) -> Result<BoxStream<'static, Result<BlockId>>>;
}

#[async_trait]
pub trait BlockStoreDeleter {
    async fn remove(&self, id: &BlockId) -> Result<RemoveResult>;
}

#[async_trait]
pub trait BlockStoreWriter {
    async fn try_create(&self, id: &BlockId, data: &[u8]) -> Result<TryCreateResult>;
    async fn store(&self, id: &BlockId, data: &[u8]) -> Result<()>;
}

#[async_trait]
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

    async fn try_create_optimized(
        &self,
        id: &BlockId,
        data: Self::BlockData,
    ) -> Result<TryCreateResult>;

    async fn store_optimized(&self, id: &BlockId, data: Self::BlockData) -> Result<()>;
}

#[async_trait]
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

pub trait BlockStore:
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
/// Such BlockData instances can be created with the [create_block_data_wrapper!] macro.
///
/// This not being public is an important part of our safety net.
/// Only things in the blockstore module can create instances of this,
/// so we can make sure the invariants are always kept.
#[macro_use]
pub(crate) mod block_data {
    use cryfs_utils::data::Data;

    pub trait IBlockData: AsRef<[u8]> + AsMut<[u8]> + Clone {
        // TODO Rename to new_unchecked ?
        fn new(data: Data) -> Self;
        fn extract(self) -> Data;
    }

    macro_rules! create_block_data_wrapper {
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
}
