use anyhow::Result;
use async_trait::async_trait;
use byte_unit::Byte;
use futures::stream::BoxStream;
#[cfg(test)]
use futures::stream::TryStreamExt;
#[cfg(test)]
use std::collections::HashSet;
use std::fmt::Debug;

use crate::{
    RemoveResult,
    implementations::on_blocks::data_node_store::{DataNode, DataNodeStore},
};
use cryfs_blockstore::{BlockId, BlockStore, InvalidBlockSizeError};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard},
    data::Data,
};

use super::{
    traversal::{self, LoadNodeError},
    tree::DataTree,
};

#[derive(Debug)]
pub struct DataTreeStore<B: BlockStore + AsyncDrop + Debug + Send + Sync> {
    node_store: AsyncDropGuard<AsyncDropArc<DataNodeStore<B>>>,
}

impl<B: BlockStore + AsyncDrop + Debug + Send + Sync> DataTreeStore<B> {
    pub async fn new(
        block_store: AsyncDropGuard<B>,
        physical_block_size: Byte,
    ) -> Result<AsyncDropGuard<Self>, InvalidBlockSizeError> {
        Ok(AsyncDropGuard::new(Self {
            node_store: AsyncDropArc::new(
                DataNodeStore::new(block_store, physical_block_size).await?,
            ),
        }))
    }
}

impl<B: BlockStore<Block: Send + Sync> + AsyncDrop + Debug + Send + Sync> DataTreeStore<B> {
    pub async fn load_tree(
        &self,
        root_node_id: BlockId,
    ) -> Result<Option<AsyncDropGuard<DataTree<B>>>> {
        Ok(self
            .node_store
            .load(root_node_id)
            .await?
            .map(|root_node| DataTree::new(root_node, AsyncDropArc::clone(&self.node_store))))
    }

    pub async fn create_tree(&self) -> Result<AsyncDropGuard<DataTree<B>>> {
        let new_leaf = self
            .node_store
            .create_new_leaf_node(&Data::from(vec![]))
            .await?;
        Ok(DataTree::new(
            new_leaf.upcast(),
            AsyncDropArc::clone(&self.node_store),
        ))
    }

    pub async fn try_create_tree(
        &self,
        id: BlockId,
    ) -> Result<Option<AsyncDropGuard<DataTree<B>>>> {
        let new_leaf = self
            .node_store
            .try_create_new_leaf_node(id, &Data::from(vec![]))
            .await?;
        Ok(new_leaf.map(|new_leaf| {
            DataTree::new(new_leaf.upcast(), AsyncDropArc::clone(&self.node_store))
        }))
    }

    pub async fn remove_tree_by_id(&self, root_node_id: BlockId) -> Result<RemoveResult> {
        match self.load_tree(root_node_id).await? {
            Some(tree) => {
                DataTree::remove(tree).await?;
                Ok(RemoveResult::SuccessfullyRemoved)
            }
            None => Ok(RemoveResult::NotRemovedBecauseItDoesntExist),
        }
    }

    pub async fn num_nodes(&self) -> Result<u64> {
        self.node_store.num_nodes().await
    }

    pub fn estimate_space_for_num_blocks_left(&self) -> Result<u64> {
        self.node_store.estimate_space_for_num_blocks_left()
    }

    pub fn logical_block_size_bytes(&self) -> Byte {
        self.node_store.logical_block_size_bytes()
    }

    // TODO Test
    pub async fn load_block_depth(&self, id: &BlockId) -> Result<Option<u8>> {
        Ok(self.node_store.load(*id).await?.map(|node| node.depth()))
    }

    pub fn into_inner_node_store(
        this: AsyncDropGuard<Self>,
    ) -> AsyncDropGuard<AsyncDropArc<DataNodeStore<B>>> {
        this.unsafe_into_inner_dont_drop().node_store
    }

    pub async fn load_all_nodes_in_subtree_of_id(
        &self,
        subtree_root_id: BlockId,
    ) -> BoxStream<'_, Result<DataNode<B>, LoadNodeError>> {
        traversal::load_all_nodes_in_subtree_of_id(&self.node_store, subtree_root_id).await
    }

    pub async fn flush_tree_if_cached(&self, root_node_id: BlockId) -> Result<()> {
        // TODO We could be smarter here and remember an in-memory map of all the not-yet-flushed blobs and which
        //      blocks they modified, and then only load and flush those here. Maybe even automatically clear it when
        //      things get flushed or pruned from caches naturally.
        //      But we don't have that yet. So the best we can do to get correct behavior here is to load the entire tree.
        //      and flush it.
        if let Some(mut tree) = self.load_tree(root_node_id).await? {
            tree.flush().await?;
        }
        Ok(())
    }

    #[cfg(test)]
    // This needs to load all blocks, so it's not very efficient. Only use it for tests.
    pub async fn all_tree_roots(&self) -> Result<Vec<BlockId>> {
        let all_nodes: Vec<BlockId> = self.node_store.all_nodes().await?.try_collect().await?;
        let mut potential_roots: HashSet<BlockId> = all_nodes.iter().copied().collect();

        for node_id in all_nodes {
            match self.node_store.load(node_id).await? {
                Some(DataNode::Leaf(_)) | None => { /* do nothing */ }
                Some(DataNode::Inner(inner)) => {
                    for child_id in inner.children() {
                        potential_roots.remove(&child_id);
                    }
                }
            }
        }

        Ok(potential_roots.into_iter().collect())
    }

    #[cfg(any(test, feature = "testutils"))]
    pub async fn clear_cache_slow(&self) -> Result<()> {
        self.node_store.clear_cache_slow().await
    }

    #[cfg(any(test, feature = "testutils"))]
    pub async fn clear_unloaded_blocks_from_cache(&self) -> Result<()> {
        self.node_store.clear_unloaded_blocks_from_cache().await
    }
}

#[async_trait]
impl<B: BlockStore<Block: Send + Sync> + AsyncDrop + Debug + Send + Sync> AsyncDrop
    for DataTreeStore<B>
{
    type Error = <B as AsyncDrop>::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.node_store.async_drop().await
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::super::testutils::*;
    use super::*;
    use anyhow::anyhow;
    use cryfs_blockstore::{InMemoryBlockStore, LockingBlockStore, MockBlockStore, Overhead};

    fn make_mock_block_store() -> AsyncDropGuard<MockBlockStore> {
        let mut blockstore = AsyncDropGuard::new(MockBlockStore::new());
        blockstore
            .expect_async_drop_impl()
            .times(1)
            .returning(move || Box::pin(async { Ok(()) }));
        blockstore
    }

    mod new {
        use super::*;

        #[tokio::test]
        async fn invalid_block_size() {
            assert_eq!(
                "Invalid block size: Tried to create a DataNodeStore with block size 10 (physical: 10) but must be at least 40",
                DataTreeStore::new(
                    LockingBlockStore::new(InMemoryBlockStore::new()),
                    Byte::from_u64(10)
                )
                .await
                .unwrap_err()
                .to_string(),
            );
        }

        #[tokio::test]
        async fn valid_block_size() {
            let mut store = DataTreeStore::new(
                LockingBlockStore::new(InMemoryBlockStore::new()),
                Byte::from_u64(40),
            )
            .await
            .unwrap();
            store.async_drop().await.unwrap();
        }

        #[tokio::test]
        async fn calculation_throws_error() {
            let mut blockstore = make_mock_block_store();
            blockstore
                .expect_overhead()
                .times(1)
                .returning(move || Overhead::new(Byte::from_u64(100_000)));
            assert_eq!(
                "Invalid block size: Physical block size 32768 is smaller than overhead 100000",
                DataTreeStore::new(
                    LockingBlockStore::new(blockstore),
                    Byte::from_u64_with_unit(32, byte_unit::Unit::KiB).unwrap()
                )
                .await
                .unwrap_err()
                .to_string()
            );
        }
    }

    mod load_tree {
        use super::*;

        #[tokio::test]
        async fn not_existing() {
            with_treestore(|store| {
                Box::pin(async move {
                    let tree = store
                        .load_tree(BlockId::from_hex("d86afd0489d7c3046c446e8ec1a049fe").unwrap())
                        .await
                        .unwrap();
                    assert!(tree.is_none());
                })
            })
            .await;
        }

        #[tokio::test]
        async fn existing_one_leaf_node() {
            with_treestore(|store| {
                Box::pin(async move {
                    let root_id = {
                        let mut created = store.create_tree().await.unwrap();
                        let root_id = *created.root_node_id();
                        created.async_drop().await.unwrap();
                        root_id
                    };
                    let mut tree = store.load_tree(root_id).await.unwrap().unwrap();
                    assert_eq!(root_id, *tree.root_node_id());
                    tree.async_drop().await.unwrap();
                })
            })
            .await;
        }

        #[tokio::test]
        async fn existing_multiple_leaf_nodes() {
            with_treestore(|store| {
                Box::pin(async move {
                    let root_id = {
                        let mut tree = store.create_tree().await.unwrap();
                        tree.resize_num_bytes(10 * PHYSICAL_BLOCK_SIZE.as_u64())
                            .await
                            .unwrap();
                        let root_id = *tree.root_node_id();
                        tree.async_drop().await.unwrap();
                        root_id
                    };
                    let mut tree = store.load_tree(root_id).await.unwrap().unwrap();
                    assert_eq!(root_id, *tree.root_node_id());
                    tree.async_drop().await.unwrap();
                })
            })
            .await;
        }
    }

    mod create_tree {
        use super::*;

        #[tokio::test]
        async fn loadable_after_creation() {
            with_treestore(|store| {
                Box::pin(async move {
                    let root_id = {
                        let mut created = store.create_tree().await.unwrap();
                        let root_id = *created.root_node_id();
                        created.async_drop().await.unwrap();
                        root_id
                    };
                    let mut tree = store.load_tree(root_id).await.unwrap().unwrap();
                    assert_eq!(root_id, *tree.root_node_id());
                    tree.async_drop().await.unwrap();
                })
            })
            .await;
        }

        #[tokio::test]
        async fn is_just_one_empty_leaf_node() {
            with_treestore_and_nodestore(|treestore, nodestore| {
                Box::pin(async move {
                    let mut tree = treestore.create_tree().await.unwrap();
                    assert_eq!(tree.num_nodes().await.unwrap(), 1);
                    assert_eq!(tree.num_bytes().await.unwrap(), 0);
                    tree.flush().await.unwrap();

                    let DataNode::Leaf(node) =
                        nodestore.load(*tree.root_node_id()).await.unwrap().unwrap()
                    else {
                        panic!("Expected inner node");
                    };
                    assert_eq!(0, node.num_bytes());
                    tree.async_drop().await.unwrap();
                })
            })
            .await;
        }
    }

    mod try_create_tree {
        use super::*;

        #[tokio::test]
        async fn loadable_after_creation() {
            with_treestore(|store| {
                Box::pin(async move {
                    let root_id = BlockId::from_hex("d86afd0489d7c3046c446e8ec1a049fe").unwrap();
                    let mut tree = store.try_create_tree(root_id).await.unwrap().unwrap();
                    assert_eq!(root_id, *tree.root_node_id());
                    tree.async_drop().await.unwrap();

                    let mut tree = store.load_tree(root_id).await.unwrap().unwrap();
                    assert_eq!(root_id, *tree.root_node_id());
                    tree.async_drop().await.unwrap();
                })
            })
            .await;
        }

        #[tokio::test]
        async fn is_just_one_empty_leaf_node() {
            with_treestore_and_nodestore(|treestore, nodestore| {
                Box::pin(async move {
                    let root_id = BlockId::from_hex("d86afd0489d7c3046c446e8ec1a049fe").unwrap();
                    let mut tree = treestore.try_create_tree(root_id).await.unwrap().unwrap();
                    assert_eq!(tree.num_nodes().await.unwrap(), 1);
                    assert_eq!(tree.num_bytes().await.unwrap(), 0);
                    tree.flush().await.unwrap();

                    let DataNode::Leaf(node) =
                        nodestore.load(*tree.root_node_id()).await.unwrap().unwrap()
                    else {
                        panic!("Expected inner node");
                    };
                    assert_eq!(0, node.num_bytes());

                    tree.async_drop().await.unwrap();
                })
            })
            .await;
        }

        #[tokio::test]
        async fn with_already_existing_id() {
            with_treestore(|store| {
                Box::pin(async move {
                    let root_id = BlockId::from_hex("d86afd0489d7c3046c446e8ec1a049fe").unwrap();
                    let mut tree = store.try_create_tree(root_id).await.unwrap().unwrap();
                    assert_eq!(root_id, *tree.root_node_id());
                    tree.async_drop().await.unwrap();

                    assert!(store.try_create_tree(root_id).await.unwrap().is_none());
                })
            })
            .await;
        }
    }

    mod remove_tree_by_id {
        use super::*;

        #[tokio::test]
        async fn givenEmptyTreeStore_whenRemovingNonExistingEntry_thenFails() {
            with_treestore(move |store| {
                Box::pin(async move {
                    assert_eq!(
                        RemoveResult::NotRemovedBecauseItDoesntExist,
                        store
                            .remove_tree_by_id(
                                BlockId::from_hex("3674b8dc1c3c1c41e331a1ebd4949087").unwrap()
                            )
                            .await
                            .unwrap()
                    );
                })
            })
            .await
        }

        #[tokio::test]
        async fn givenOtherwiseEmptyTreeStore_whenRemovingExistingOneNodeTree_thenCannotBeLoadedAnymore()
         {
            with_treestore(move |store| {
                Box::pin(async move {
                    let root_id = create_one_leaf_tree_return_id(&store).await;
                    let tree = store.load_tree(root_id).await.unwrap();
                    assert!(tree.is_some());
                    tree.unwrap().async_drop().await.unwrap();

                    assert_eq!(
                        RemoveResult::SuccessfullyRemoved,
                        store.remove_tree_by_id(root_id).await.unwrap()
                    );
                    assert!(store.load_tree(root_id).await.unwrap().is_none());
                })
            })
            .await
        }

        #[tokio::test]
        async fn givenOtherwiseEmptyTreeStore_whenRemovingExistingMultiNodeTree_thenCannotBeLoadedAnymore()
         {
            with_treestore(move |store| {
                Box::pin(async move {
                    const NUM_LEAVES: u64 = 10;
                    let root_id = create_multi_leaf_tree_return_id(&store, NUM_LEAVES).await;
                    let tree = store.load_tree(root_id).await.unwrap();
                    assert!(tree.is_some());
                    tree.unwrap().async_drop().await.unwrap();

                    assert_eq!(
                        RemoveResult::SuccessfullyRemoved,
                        store.remove_tree_by_id(root_id).await.unwrap()
                    );
                    assert!(store.load_tree(root_id).await.unwrap().is_none());
                })
            })
            .await
        }

        #[tokio::test]
        async fn givenOtherwiseEmptyTreeStore_whenRemovingExistingMultiNodeTree_thenDeletesAllNodesOfThisTree()
         {
            with_treestore_and_nodestore(move |treestore, nodestore| {
                Box::pin(async move {
                    const NUM_LEAVES: u64 = 10;
                    let root_id = create_multi_leaf_tree_return_id(&treestore, NUM_LEAVES).await;
                    treestore.clear_cache_slow().await.unwrap();
                    assert_eq!(NUM_LEAVES + 3, nodestore.num_nodes().await.unwrap());

                    assert_eq!(
                        RemoveResult::SuccessfullyRemoved,
                        treestore.remove_tree_by_id(root_id).await.unwrap()
                    );
                    treestore.clear_cache_slow().await.unwrap();
                    assert_eq!(0, nodestore.num_nodes().await.unwrap());
                })
            })
            .await
        }

        #[tokio::test]
        async fn givenTreeStoreWithOtherTrees_whenRemovingNonExistingEntry_thenFails() {
            with_treestore(move |store| {
                Box::pin(async move {
                    let _other_tree = TreeFixture::create_tree_with_data_and_id(
                        &store,
                        BlockId::from_hex("41e331a31c3c1c1ebd4949087674b8dc").unwrap(),
                        10 * store.logical_block_size_bytes().as_u64() as usize,
                        0,
                    )
                    .await;

                    assert_eq!(
                        RemoveResult::NotRemovedBecauseItDoesntExist,
                        store
                            .remove_tree_by_id(
                                BlockId::from_hex("3674b8dc1c3c1c41e331a1ebd4949087").unwrap()
                            )
                            .await
                            .unwrap()
                    );
                })
            })
            .await
        }

        #[tokio::test]
        async fn givenTreeStoreWithOtherTrees_whenRemovingExistingOneNodeTree_thenCannotBeLoadedAnymore()
         {
            with_treestore(move |store| {
                Box::pin(async move {
                    let _other_tree = TreeFixture::create_tree_with_data(
                        &store,
                        10 * store.logical_block_size_bytes().as_u64() as usize,
                        0,
                    )
                    .await;
                    let root_id = create_one_leaf_tree_return_id(&store).await;
                    let tree = store.load_tree(root_id).await.unwrap();
                    assert!(tree.is_some());
                    tree.unwrap().async_drop().await.unwrap();

                    assert_eq!(
                        RemoveResult::SuccessfullyRemoved,
                        store.remove_tree_by_id(root_id).await.unwrap()
                    );
                    assert!(store.load_tree(root_id).await.unwrap().is_none());
                })
            })
            .await
        }

        #[tokio::test]
        async fn givenTreeStoreWithOtherTrees_whenRemovingExistingMultiNodeTree_thenCannotBeLoadedAnymore()
         {
            with_treestore(move |store| {
                Box::pin(async move {
                    const NUM_LEAVES: u64 = 10;

                    let _other_tree = TreeFixture::create_tree_with_data(
                        &store,
                        NUM_LEAVES as usize * store.logical_block_size_bytes().as_u64() as usize,
                        0,
                    )
                    .await;

                    let root_id = create_multi_leaf_tree_return_id(&store, NUM_LEAVES).await;
                    let tree = store.load_tree(root_id).await.unwrap();
                    assert!(tree.is_some());
                    tree.unwrap().async_drop().await.unwrap();

                    assert_eq!(
                        RemoveResult::SuccessfullyRemoved,
                        store.remove_tree_by_id(root_id).await.unwrap()
                    );
                    assert!(store.load_tree(root_id).await.unwrap().is_none());
                })
            })
            .await
        }

        #[tokio::test]
        async fn givenTreeStoreWithOtherTrees_whenRemovingExistingMultiNodeTree_thenDeletesAllNodesOfThisTree()
         {
            with_treestore_and_nodestore(move |treestore, nodestore| {
                Box::pin(async move {
                    const NUM_LEAVES: u64 = 10;

                    let _other_tree = TreeFixture::create_tree_with_data(
                        &treestore,
                        NUM_LEAVES as usize
                            * treestore.logical_block_size_bytes().as_u64() as usize,
                        0,
                    )
                    .await;

                    let root_id = create_multi_leaf_tree_return_id(&treestore, NUM_LEAVES).await;
                    treestore.clear_cache_slow().await.unwrap();
                    assert_eq!(2 * NUM_LEAVES + 6, nodestore.num_nodes().await.unwrap());

                    assert_eq!(
                        RemoveResult::SuccessfullyRemoved,
                        treestore.remove_tree_by_id(root_id).await.unwrap()
                    );
                    treestore.clear_cache_slow().await.unwrap();
                    assert_eq!(NUM_LEAVES + 3, nodestore.num_nodes().await.unwrap());
                })
            })
            .await
        }

        #[tokio::test]
        async fn givenTreeStoreWithOtherTrees_whenRemovingExistingMultiNodeTree_thenDoesntDeleteOtherTrees()
         {
            with_treestore_and_nodestore(move |treestore, nodestore| {
                Box::pin(async move {
                    const NUM_LEAVES: u64 = 10;

                    let other_tree = TreeFixture::create_tree_with_data(
                        &treestore,
                        NUM_LEAVES as usize
                            * treestore.logical_block_size_bytes().as_u64() as usize,
                        0,
                    )
                    .await;

                    let root_id = create_multi_leaf_tree_return_id(&treestore, NUM_LEAVES).await;
                    treestore.clear_cache_slow().await.unwrap();
                    assert_eq!(2 * NUM_LEAVES + 6, nodestore.num_nodes().await.unwrap());

                    assert_eq!(
                        RemoveResult::SuccessfullyRemoved,
                        treestore.remove_tree_by_id(root_id).await.unwrap()
                    );
                    treestore.clear_cache_slow().await.unwrap();
                    assert_eq!(NUM_LEAVES + 3, nodestore.num_nodes().await.unwrap());

                    other_tree.assert_data_is_still_intact(&treestore).await;
                })
            })
            .await
        }
    }

    mod num_nodes {
        use super::*;

        #[tokio::test]
        async fn empty() {
            with_treestore(move |store| {
                Box::pin(async move {
                    assert_eq!(0, store.num_nodes().await.unwrap());
                })
            })
            .await
        }

        #[tokio::test]
        async fn after_adding_trees() {
            with_treestore(move |store| {
                Box::pin(async move {
                    assert_eq!(0, store.num_nodes().await.unwrap());
                    create_one_leaf_tree_return_id(&store).await;
                    assert_eq!(1, store.num_nodes().await.unwrap());
                    create_multi_leaf_tree_return_id(&store, 10).await;
                    assert_eq!(14, store.num_nodes().await.unwrap());
                })
            })
            .await
        }

        #[tokio::test]
        async fn after_removing_trees() {
            with_treestore(move |store| {
                Box::pin(async move {
                    let tree1 = create_multi_leaf_tree_return_id(&store, 10).await;
                    let tree2 = create_one_leaf_tree_return_id(&store).await;
                    let tree3 = create_multi_leaf_tree_return_id(&store, 20).await;
                    assert_eq!(38, store.num_nodes().await.unwrap());
                    assert_eq!(
                        RemoveResult::SuccessfullyRemoved,
                        store.remove_tree_by_id(tree1).await.unwrap()
                    );
                    assert_eq!(25, store.num_nodes().await.unwrap());
                    assert_eq!(
                        RemoveResult::SuccessfullyRemoved,
                        store.remove_tree_by_id(tree2).await.unwrap()
                    );
                    assert_eq!(24, store.num_nodes().await.unwrap());
                    assert_eq!(
                        RemoveResult::SuccessfullyRemoved,
                        store.remove_tree_by_id(tree3).await.unwrap()
                    );
                    assert_eq!(0, store.num_nodes().await.unwrap());
                })
            })
            .await
        }
    }

    mod estimate_space_for_num_blocks_left {
        use super::*;

        #[tokio::test]
        async fn no_space_left() {
            let mut blockstore = make_mock_block_store();
            blockstore
                .expect_overhead()
                .times(1)
                .returning(|| Overhead::new(Byte::from_u64(0)));
            blockstore
                .expect_estimate_num_free_bytes()
                .returning(|| Ok(Byte::from_u64(0)));
            let mut treestore =
                DataTreeStore::new(LockingBlockStore::new(blockstore), Byte::from_u64(100))
                    .await
                    .unwrap();

            assert_eq!(0, treestore.estimate_space_for_num_blocks_left().unwrap());

            treestore.async_drop().await.unwrap();
        }

        #[tokio::test]
        async fn almost_enough_space_for_one_block() {
            let mut blockstore = make_mock_block_store();
            blockstore
                .expect_overhead()
                .times(1)
                .returning(|| Overhead::new(Byte::from_u64(0)));
            blockstore
                .expect_estimate_num_free_bytes()
                .returning(|| Ok(Byte::from_u64(99)));
            let mut treestore =
                DataTreeStore::new(LockingBlockStore::new(blockstore), Byte::from_u64(100))
                    .await
                    .unwrap();

            assert_eq!(0, treestore.estimate_space_for_num_blocks_left().unwrap());

            treestore.async_drop().await.unwrap();
        }

        #[tokio::test]
        async fn just_enough_space_for_one_block() {
            let mut blockstore = make_mock_block_store();
            blockstore
                .expect_overhead()
                .times(1)
                .returning(|| Overhead::new(Byte::from_u64(0)));
            blockstore
                .expect_estimate_num_free_bytes()
                .returning(|| Ok(Byte::from_u64(100)));
            let mut treestore =
                DataTreeStore::new(LockingBlockStore::new(blockstore), Byte::from_u64(100))
                    .await
                    .unwrap();

            assert_eq!(1, treestore.estimate_space_for_num_blocks_left().unwrap());

            treestore.async_drop().await.unwrap();
        }

        #[tokio::test]
        async fn enough_space_for_100_blocks() {
            let mut blockstore = make_mock_block_store();
            blockstore
                .expect_overhead()
                .times(1)
                .returning(|| Overhead::new(Byte::from_u64(0)));
            blockstore
                .expect_estimate_num_free_bytes()
                .returning(|| Ok(Byte::from_u64(32 * 1024 * 10240 + 123)));
            let mut treestore = DataTreeStore::new(
                LockingBlockStore::new(blockstore),
                Byte::from_u64(32 * 1024),
            )
            .await
            .unwrap();

            assert_eq!(
                10240,
                treestore.estimate_space_for_num_blocks_left().unwrap()
            );

            treestore.async_drop().await.unwrap();
        }

        #[tokio::test]
        async fn calculation_is_based_on_physical_block_size_not_block_size() {
            let mut blockstore = make_mock_block_store();
            blockstore
                .expect_overhead()
                .times(1)
                .returning(|| Overhead::new(Byte::from_u64(15 * 1024)));
            blockstore
                .expect_estimate_num_free_bytes()
                .returning(|| Ok(Byte::from_u64(32 * 1024 * 10240 + 123)));
            let mut treestore = DataTreeStore::new(
                LockingBlockStore::new(blockstore),
                Byte::from_u64(32 * 1024),
            )
            .await
            .unwrap();

            assert_eq!(
                10240,
                treestore.estimate_space_for_num_blocks_left().unwrap()
            );

            treestore.async_drop().await.unwrap();
        }

        #[tokio::test]
        async fn calculation_throws_error() {
            let mut blockstore = make_mock_block_store();
            blockstore
                .expect_overhead()
                .times(1)
                .returning(|| Overhead::new(Byte::from_u64(0)));
            blockstore
                .expect_estimate_num_free_bytes()
                .returning(|| Err(anyhow!("some error")));
            let mut treestore = DataTreeStore::new(
                LockingBlockStore::new(blockstore),
                Byte::from_u64(32 * 1024),
            )
            .await
            .unwrap();

            assert_eq!(
                "some error",
                treestore
                    .estimate_space_for_num_blocks_left()
                    .unwrap_err()
                    .to_string(),
            );

            treestore.async_drop().await.unwrap();
        }
    }

    mod logical_block_size_bytes {
        use super::*;

        #[tokio::test]
        async fn test() {
            let mut blockstore = make_mock_block_store();
            blockstore
                .expect_overhead()
                .times(1)
                .returning(|| Overhead::new(Byte::from_u64(100)));
            let mut treestore = DataTreeStore::new(
                LockingBlockStore::new(blockstore),
                Byte::from_u64(32 * 1024 * 10),
            )
            .await
            .unwrap();

            assert_eq!(
                super::super::super::super::data_node_store::NodeLayout {
                    block_size: Byte::from_u64(32 * 1024 * 10 - 100),
                }
                .max_bytes_per_leaf() as u64,
                treestore.logical_block_size_bytes().as_u64()
            );

            treestore.async_drop().await.unwrap();
        }
    }
}
