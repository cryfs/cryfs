use anyhow::Result;
use async_trait::async_trait;
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
/// [LLBlockStore] has [AsyncDrop] as a supertrait, and [AsyncDrop::async_drop_impl]
/// returns `impl Future`, which makes [AsyncDrop] and with it [LLBlockStore] not
/// dyn-compatible. This trait has the same supertraits as [LLBlockStore] except for
/// [AsyncDrop], which it replaces by [DynLLBlockStore::async_drop_boxed], a method taking
/// `self: Box<Self>` and returning a boxed future. It is implemented for every [LLBlockStore].
pub trait DynLLBlockStore:
    BlockStoreReader + BlockStoreWriter + BlockStoreDeleter + Debug + Any
{
    /// Same as [AsyncDrop::async_drop_impl], but callable on a `Box<dyn DynLLBlockStore>`.
    fn async_drop_boxed(self: Box<Self>) -> BoxFuture<'static, Result<()>>;
}

impl<B: LLBlockStore> DynLLBlockStore for B {
    fn async_drop_boxed(self: Box<Self>) -> BoxFuture<'static, Result<()>> {
        Box::pin((*self).async_drop_impl())
    }
}

#[derive(Debug)]
pub struct DynBlockStore(pub Box<dyn DynLLBlockStore + Sync + Send>);

#[async_trait]
impl BlockStoreReader for DynBlockStore {
    async fn exists(&self, id: &BlockId) -> Result<bool> {
        let r = (*self.0).exists(id);
        r.await
    }

    async fn load(&self, id: &BlockId) -> Result<Option<Data>> {
        let r = (*self.0).load(id);
        r.await
    }

    async fn num_blocks(&self) -> Result<u64> {
        let r = (*self.0).num_blocks();
        r.await
    }

    fn estimate_num_free_bytes(&self) -> Result<Byte> {
        (*self.0).estimate_num_free_bytes()
    }

    fn overhead(&self) -> Overhead {
        (*self.0).overhead()
    }

    async fn all_blocks(&self) -> Result<BoxStream<'static, Result<BlockId>>> {
        let r = (*self.0).all_blocks();
        r.await
    }
}

#[async_trait]
impl BlockStoreWriter for DynBlockStore {
    async fn try_create(&self, id: &BlockId, data: &[u8]) -> Result<TryCreateResult> {
        let r = (*self.0).try_create(id, data);
        r.await
    }

    async fn store(&self, id: &BlockId, data: &[u8]) -> Result<()> {
        let r = (*self.0).store(id, data);
        r.await
    }
}

#[async_trait]
impl BlockStoreDeleter for DynBlockStore {
    async fn remove(&self, id: &BlockId) -> Result<RemoveResult> {
        let r = (*self.0).remove(id);
        r.await
    }
}

impl AsyncDrop for DynBlockStore {
    type Error = anyhow::Error;

    async fn async_drop_impl(self) -> Result<(), Self::Error> {
        let r = self.0.async_drop_boxed();
        let r = r.await?;
        Ok(r)
    }
}

impl LLBlockStore for DynBlockStore {}
