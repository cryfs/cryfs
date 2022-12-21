use anyhow::{anyhow, bail, ensure, Result};
use async_trait::async_trait;
use binary_layout::Field;

pub use crate::RemoveResult;
use cryfs_blockstore::{BlockId, BlockStore, LockingBlockStore, BLOCKID_LEN};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    data::Data,
};

mod layout;
use layout::node;
pub use layout::NodeLayout;

mod data_node;
pub use data_node::{DataInnerNode, DataLeafNode, DataNode};

#[cfg(test)]
mod testutils;

#[derive(Debug)]
pub struct DataNodeStore<B: BlockStore + Send + Sync> {
    block_store: AsyncDropGuard<LockingBlockStore<B>>,
    layout: NodeLayout,
}

impl<B: BlockStore + Send + Sync> DataNodeStore<B> {
    pub fn new(
        block_store: AsyncDropGuard<LockingBlockStore<B>>,
        physical_block_size_bytes: u32,
    ) -> Result<AsyncDropGuard<Self>> {
        let block_size_bytes = u32::try_from(
            block_store
                .block_size_from_physical_block_size(u64::from(physical_block_size_bytes))?,
        )
        .unwrap();
        // Min block size: enough for header and for inner nodes to have at least two children and form a tree.
        let min_block_size = u32::try_from(node::data::OFFSET + 2 * BLOCKID_LEN).unwrap();
        ensure!(
            block_size_bytes >= min_block_size,
            "Tried to create a DataNodeStore with block size {} (physical: {}) but must be at least {}",
            block_size_bytes,
            physical_block_size_bytes,
            min_block_size,
        );
        Ok(AsyncDropGuard::new(Self {
            block_store,
            layout: NodeLayout { block_size_bytes },
        }))
    }

    pub fn layout(&self) -> &NodeLayout {
        &self.layout
    }

    pub async fn load(&self, block_id: BlockId) -> Result<Option<DataNode<B>>> {
        match self.block_store.load(block_id).await? {
            None => Ok(None),
            Some(block) => DataNode::parse(block, &self.layout).map(Some),
        }
    }

    fn _allocate_data_for_leaf_node(&self) -> Data {
        let mut data = Data::from(vec![
            0;
            usize::try_from(self.layout.block_size_bytes).unwrap()
        ]);
        data.shrink_to_subregion(node::data::OFFSET..);
        assert_eq!(
            usize::try_from(self.layout.max_bytes_per_leaf()).unwrap(),
            data.len()
        );
        data
    }

    pub async fn create_new_leaf_node(&self, data: &Data) -> Result<DataLeafNode<B>> {
        let mut leaf_data = self._allocate_data_for_leaf_node();
        leaf_data[..data.len()].copy_from_slice(data); // TODO Avoid copy_from_slice and instead rename this function to create_new_leaf_node_optimized
        let block_data = data_node::serialize_leaf_node_optimized(
            leaf_data,
            u32::try_from(data.len()).unwrap(),
            &self.layout,
        );
        // TODO Use create_optimized instead of create?
        let blockid = self.block_store.create(&block_data).await?;
        // TODO Avoid extra load here. Do our callers actually need this object? If no, just return the block id. If yes, maybe change block store API to return the block?
        match self.load(blockid).await? {
            None => bail!("We just created this block, it must exist"),
            Some(DataNode::Inner(_)) => {
                bail!("We just created a leaf node but then it got loaded as an inner node")
            }
            Some(DataNode::Leaf(node)) => Ok(node),
        }
    }

    pub async fn create_new_inner_node(
        &self,
        depth: u8,
        children: &[BlockId],
    ) -> Result<DataInnerNode<B>> {
        let block_data = data_node::serialize_inner_node(depth, children, &self.layout);
        // TODO Use create_optimized instead of create?
        let blockid = self.block_store.create(&block_data).await?;
        // TODO Avoid extra load here. Do our callers actually need this object? If no, just return the block id. If yes, maybe change block store API to return the block?
        match self.load(blockid).await? {
            None => bail!("We just created this block, it must exist"),
            Some(DataNode::Leaf(_)) => {
                bail!("We just created a inner node but then it got loaded as a leaf node")
            }
            Some(DataNode::Inner(node)) => Ok(node),
        }
    }

    pub async fn create_new_node_as_copy_from(&self, source: &DataNode<B>) -> Result<DataNode<B>> {
        let source_data = source.raw_blockdata();
        assert_eq!(usize::try_from(self.layout.block_size_bytes).unwrap(), source_data.len(), "Source node has wrong layout and has {} bytes. We expected {} bytes. Is it from the same DataNodeStore?", source_data.len(), self.layout.block_size_bytes);
        // TODO Use create_optimized instead of create?
        let blockid = self.block_store.create(source_data).await?;
        // TODO Avoid extra load here. Do our callers actually need this object? If no, just return the block id. If yes, maybe change block store API to return the block?
        self.load(blockid)
            .await?
            .ok_or_else(|| anyhow!("We just created {:?} but now couldn't find it", blockid))
    }

    pub async fn overwrite_leaf_node(&self, block_id: &BlockId, data: &[u8]) -> Result<()> {
        let mut data_obj = self._allocate_data_for_leaf_node();
        // TODO Make an overwrite_leaf_node_optimized version that requires that enough prefix bytes are already available in the data input and that doesn't require us to copy_from_slice here?
        (&mut data_obj.as_mut()[..data.len()]).copy_from_slice(data);
        let block_data = data_node::serialize_leaf_node_optimized(
            data_obj,
            u32::try_from(data.len()).unwrap(),
            &self.layout,
        );
        // TODO Use store_optimized instead of store?
        self.block_store.overwrite(block_id, &block_data).await
    }

    pub async fn remove_by_id(&self, block_id: &BlockId) -> Result<RemoveResult> {
        self.block_store.remove(block_id).await
    }

    pub async fn num_nodes(&self) -> Result<u64> {
        self.block_store.num_blocks().await
    }

    pub fn estimate_space_for_num_blocks_left(&self) -> Result<u64> {
        Ok(self.block_store.estimate_num_free_bytes()? / u64::from(self.layout.block_size_bytes))
    }

    pub fn virtual_block_size_bytes(&self) -> u32 {
        self.layout.block_size_bytes
    }

    pub async fn flush_node(&self, node: &mut DataNode<B>) -> Result<()> {
        self.block_store.flush_block(node.as_block_mut()).await
    }
}

#[async_trait]
impl<B: BlockStore + Send + Sync> AsyncDrop for DataNodeStore<B> {
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.block_store.async_drop().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cryfs_blockstore::InMemoryBlockStore;
    use testutils::*;

    mod create_new_leaf_node {
        use super::*;

        async fn test(nodestore: &DataNodeStore<InMemoryBlockStore>, data: Data) {
            let node = nodestore.create_new_leaf_node(&data).await.unwrap();
            assert_eq!(data.as_ref(), node.data());

            // and it's still correct after loading
            let block_id = *node.block_id();
            drop(node);
            let node = load_leaf_node(nodestore, block_id).await;
            assert_eq!(data.as_ref(), node.data());
        }

        #[tokio::test]
        async fn empty() {
            with_nodestore(move |nodestore| {
                Box::pin(async move { test(nodestore, Data::empty()).await })
            })
            .await
        }

        #[tokio::test]
        async fn some_data() {
            with_nodestore(move |nodestore| {
                Box::pin(async move { test(nodestore, half_full_leaf_data(1)).await })
            })
            .await
        }

        #[tokio::test]
        async fn full() {
            with_nodestore(move |nodestore| {
                Box::pin(async move { test(nodestore, full_leaf_data(1)).await })
            })
            .await
        }
    }

    mod create_new_inner_node {
        use futures::future;

        use super::*;

        async fn test(
            nodestore: &DataNodeStore<InMemoryBlockStore>,
            depth: u8,
            children: &[BlockId],
        ) {
            let node = nodestore
                .create_new_inner_node(depth, children)
                .await
                .unwrap();
            assert_eq!(depth, node.depth().get());
            assert_eq!(children.len(), node.num_children().get() as usize);
            assert_eq!(children, &node.children().collect::<Vec<_>>());

            // and it's still correct after loading
            let block_id = *node.block_id();
            drop(node);
            let node = load_inner_node(nodestore, block_id).await;
            assert_eq!(depth, node.depth().get());
            assert_eq!(children.len(), node.num_children().get() as usize);
            assert_eq!(children, &node.children().collect::<Vec<_>>());
        }

        #[tokio::test]
        async fn one_child_leaf() {
            with_nodestore(move |nodestore| {
                Box::pin(async move {
                    let child = *new_leaf_node(nodestore).await.block_id();
                    test(nodestore, 1, &[child]).await
                })
            })
            .await
        }

        #[tokio::test]
        async fn two_children_leaves() {
            with_nodestore(move |nodestore| {
                Box::pin(async move {
                    let child1 = *new_leaf_node(nodestore).await.block_id();
                    let child2 = *new_leaf_node(nodestore).await.block_id();
                    test(nodestore, 1, &[child1, child2]).await
                })
            })
            .await
        }

        #[tokio::test]
        async fn max_children_leaves() {
            with_nodestore(move |nodestore| {
                Box::pin(async move {
                    let children = future::join_all(
                        (0..nodestore.layout().max_children_per_inner_node())
                            .map(|_| async { *new_leaf_node(nodestore).await.block_id() })
                            .collect::<Vec<_>>(),
                    )
                    .await;
                    test(nodestore, 1, &children).await
                })
            })
            .await
        }

        #[tokio::test]
        async fn one_child_inner() {
            with_nodestore(move |nodestore| {
                Box::pin(async move {
                    let child = *new_inner_node(nodestore).await.block_id();
                    test(nodestore, 1, &[child]).await
                })
            })
            .await
        }

        #[tokio::test]
        async fn two_children_inner() {
            with_nodestore(move |nodestore| {
                Box::pin(async move {
                    let child1 = *new_inner_node(nodestore).await.block_id();
                    let child2 = *new_inner_node(nodestore).await.block_id();
                    test(nodestore, 1, &[child1, child2]).await
                })
            })
            .await
        }

        #[tokio::test]
        async fn max_children_inner() {
            with_nodestore(move |nodestore| {
                Box::pin(async move {
                    let children = future::join_all(
                        (0..nodestore.layout().max_children_per_inner_node())
                            .map(|_| async { *new_inner_node(nodestore).await.block_id() })
                            .collect::<Vec<_>>(),
                    )
                    .await;
                    test(nodestore, 1, &children).await
                })
            })
            .await
        }
    }
}
// TODO Tests
