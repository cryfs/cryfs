use anyhow::Result;
use byte_unit::Byte;
use futures::future::BoxFuture;
use futures::stream::BoxStream;
use std::any::Any;
use std::fmt::Debug;

use crate::{
    BlockId, Overhead, RemoveResult, TryCreateResult,
    low_level::interface::{BlockStoreDeleter, BlockStoreReader, BlockStoreWriter, LLBlockStore},
};
use cryfs_utils::async_drop::AsyncDrop;
use cryfs_utils::data::Data;

/// Dyn-compatible version of [LLBlockStore], for use behind `dyn` (see [DynBlockStore]).
///
/// The async methods of [BlockStoreReader], [BlockStoreWriter], [BlockStoreDeleter] and
/// [AsyncDrop] return `impl Future`, which makes those traits and with them [LLBlockStore]
/// not dyn-compatible. This trait mirrors each of those methods with a version returning
/// a boxed future, and [DynLLBlockStore::async_drop_boxed] takes `self: Box<Self>` so it
/// can be called on a `Box<dyn DynLLBlockStore>`. It is implemented for every [LLBlockStore].
pub trait DynLLBlockStore: Debug + Any {
    fn exists_boxed<'a>(&'a self, id: &'a BlockId) -> BoxFuture<'a, Result<bool>>;
    fn load_boxed<'a>(&'a self, id: &'a BlockId) -> BoxFuture<'a, Result<Option<Data>>>;
    fn num_blocks_boxed(&self) -> BoxFuture<'_, Result<u64>>;
    fn estimate_num_free_bytes(&self) -> Result<Byte>;
    fn overhead(&self) -> Overhead;
    fn all_blocks_boxed(&self) -> BoxFuture<'_, Result<BoxStream<'static, Result<BlockId>>>>;

    fn try_create_boxed<'a>(
        &'a self,
        id: &'a BlockId,
        data: &'a [u8],
    ) -> BoxFuture<'a, Result<TryCreateResult>>;
    fn store_boxed<'a>(&'a self, id: &'a BlockId, data: &'a [u8]) -> BoxFuture<'a, Result<()>>;

    fn remove_boxed<'a>(&'a self, id: &'a BlockId) -> BoxFuture<'a, Result<RemoveResult>>;

    /// Same as [AsyncDrop::async_drop_impl], but callable on a `Box<dyn DynLLBlockStore>`.
    fn async_drop_boxed(self: Box<Self>) -> BoxFuture<'static, Result<()>>;
}

impl<B: LLBlockStore + Sync> DynLLBlockStore for B {
    fn exists_boxed<'a>(&'a self, id: &'a BlockId) -> BoxFuture<'a, Result<bool>> {
        Box::pin(self.exists(id))
    }

    fn load_boxed<'a>(&'a self, id: &'a BlockId) -> BoxFuture<'a, Result<Option<Data>>> {
        Box::pin(self.load(id))
    }

    fn num_blocks_boxed(&self) -> BoxFuture<'_, Result<u64>> {
        Box::pin(self.num_blocks())
    }

    fn estimate_num_free_bytes(&self) -> Result<Byte> {
        BlockStoreReader::estimate_num_free_bytes(self)
    }

    fn overhead(&self) -> Overhead {
        BlockStoreReader::overhead(self)
    }

    fn all_blocks_boxed(&self) -> BoxFuture<'_, Result<BoxStream<'static, Result<BlockId>>>> {
        Box::pin(self.all_blocks())
    }

    fn try_create_boxed<'a>(
        &'a self,
        id: &'a BlockId,
        data: &'a [u8],
    ) -> BoxFuture<'a, Result<TryCreateResult>> {
        Box::pin(self.try_create(id, data))
    }

    fn store_boxed<'a>(&'a self, id: &'a BlockId, data: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(self.store(id, data))
    }

    fn remove_boxed<'a>(&'a self, id: &'a BlockId) -> BoxFuture<'a, Result<RemoveResult>> {
        Box::pin(self.remove(id))
    }

    fn async_drop_boxed(self: Box<Self>) -> BoxFuture<'static, Result<()>> {
        Box::pin((*self).async_drop_impl())
    }
}

#[derive(Debug)]
pub struct DynBlockStore(pub Box<dyn DynLLBlockStore + Sync + Send>);

impl BlockStoreReader for DynBlockStore {
    async fn exists(&self, id: &BlockId) -> Result<bool> {
        self.0.exists_boxed(id).await
    }

    async fn load(&self, id: &BlockId) -> Result<Option<Data>> {
        self.0.load_boxed(id).await
    }

    async fn num_blocks(&self) -> Result<u64> {
        self.0.num_blocks_boxed().await
    }

    fn estimate_num_free_bytes(&self) -> Result<Byte> {
        self.0.estimate_num_free_bytes()
    }

    fn overhead(&self) -> Overhead {
        self.0.overhead()
    }

    async fn all_blocks(&self) -> Result<BoxStream<'static, Result<BlockId>>> {
        self.0.all_blocks_boxed().await
    }
}

impl BlockStoreWriter for DynBlockStore {
    async fn try_create(&self, id: &BlockId, data: &[u8]) -> Result<TryCreateResult> {
        self.0.try_create_boxed(id, data).await
    }

    async fn store(&self, id: &BlockId, data: &[u8]) -> Result<()> {
        self.0.store_boxed(id, data).await
    }
}

impl BlockStoreDeleter for DynBlockStore {
    async fn remove(&self, id: &BlockId) -> Result<RemoveResult> {
        self.0.remove_boxed(id).await
    }
}

impl AsyncDrop for DynBlockStore {
    type Error = anyhow::Error;

    async fn async_drop_impl(self) -> Result<(), Self::Error> {
        self.0.async_drop_boxed().await
    }
}

impl LLBlockStore for DynBlockStore {}
