// //! Implements blockstore traits for AsyncDropGuard<BlockStore>.
// //! This makes it easier to wrap BlockStores that use AsyncDrop
// //! into other BlockStores without those having to use something like Deref<Target=BlockStore>

// // TODO Actually, using the deref approach may be better since it avoids the dynamic boxing done by async_trait

// // TODO Can we remove this now that we have BlockStore : AsyncDrop ?

// use anyhow::Result;
// use async_trait::async_trait;
// use futures::stream::Stream;
// use std::fmt::Debug;
// use std::ops::Deref;
// use std::pin::Pin;

// use super::{
//     BlockId, BlockStore, BlockStoreDeleter, BlockStoreReader, OptimizedBlockStoreWriter,
//     RemoveResult, TryCreateResult,
// };
// use crate::data::Data;
// use crate::utils::async_drop::{AsyncDrop, AsyncDropGuard};

// #[async_trait]
// impl<B: BlockStoreReader + Debug + AsyncDrop + Sync> BlockStoreReader for AsyncDropGuard<B> {
//     async fn exists(&self, id: &BlockId) -> Result<bool> {
//         self.deref().exists(id).await
//     }

//     async fn load(&self, id: &BlockId) -> Result<Option<Data>> {
//         self.deref().load(id).await
//     }

//     async fn num_blocks(&self) -> Result<u64> {
//         self.deref().num_blocks().await
//     }

//     fn estimate_num_free_bytes(&self) -> Result<u64> {
//         self.deref().estimate_num_free_bytes()
//     }

//     fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64> {
//         self.deref().block_size_from_physical_block_size(block_size)
//     }

//     async fn all_blocks(&self) -> Result<Pin<Box<dyn Stream<Item = Result<BlockId>> + Send>>> {
//         self.deref().all_blocks().await
//     }
// }

// #[async_trait]
// impl<B: BlockStoreDeleter + Debug + AsyncDrop + Sync> BlockStoreDeleter for AsyncDropGuard<B> {
//     async fn remove(&self, id: &BlockId) -> Result<RemoveResult> {
//         self.deref().remove(id).await
//     }
// }

// // TODO This doesn't work for B that only implement BlockStoreWriter but not OptimizedBlockStoreWriter
// #[async_trait]
// impl<B: OptimizedBlockStoreWriter + Debug + AsyncDrop + Sync> OptimizedBlockStoreWriter
//     for AsyncDropGuard<B>
// {
//     type BlockData = B::BlockData;

//     fn allocate(size: usize) -> Self::BlockData {
//         B::allocate(size)
//     }

//     async fn try_create_optimized(
//         &self,
//         id: &BlockId,
//         data: Self::BlockData,
//     ) -> Result<TryCreateResult> {
//         self.deref().try_create_optimized(id, data).await
//     }

//     async fn store_optimized(&self, id: &BlockId, data: Self::BlockData) -> Result<()> {
//         self.deref().store_optimized(id, data).await
//     }
// }

// impl<B: BlockStore + OptimizedBlockStoreWriter + Debug + Sync + AsyncDrop> BlockStore
//     for AsyncDropGuard<B>
// {
// }
