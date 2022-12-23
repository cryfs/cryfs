use anyhow::{ensure, Result};

use super::super::{
    layout::{node, NodeLayout, FORMAT_VERSION_HEADER},
    DataNodeStore,
};
use super::{
    data_inner_node::{self, DataInnerNode},
    data_leaf_node::DataLeafNode,
};
use cryfs_blockstore::{Block, BlockId, BlockStore, LockingBlockStore};
use cryfs_utils::data::{Data, ZeroedData};

#[derive(Debug)]
pub enum DataNode<B: BlockStore + Send + Sync> {
    Inner(DataInnerNode<B>),
    Leaf(DataLeafNode<B>),
}

impl<B: BlockStore + Send + Sync> DataNode<B> {
    pub fn parse(block: Block<B>, layout: &NodeLayout) -> Result<Self> {
        ensure!(
            usize::try_from(layout.block_size_bytes).unwrap() == block.data().len(),
            "Expected to load block of size {} but loaded block {:?} had size {}",
            layout.block_size_bytes,
            block.block_id(),
            block.data().len(),
        );
        let node_view = node::View::new(block.data());
        let format_version_header = node_view.format_version_header().read();
        ensure!(
            FORMAT_VERSION_HEADER == format_version_header,
            "Loaded a node {:?} with format_version_header == {}. This is not a supported format.",
            block.block_id(),
            format_version_header
        );
        let depth = node_view.depth().read();
        if depth == 0 {
            Ok(DataNode::Leaf(DataLeafNode::new(block, layout)?))
        } else {
            Ok(DataNode::Inner(DataInnerNode::new(block, layout)?))
        }
    }

    pub fn depth(&self) -> u8 {
        match self {
            Self::Leaf(_) => 0,
            Self::Inner(inner) => inner.depth().get(),
        }
    }

    pub fn block_id(&self) -> &BlockId {
        match self {
            Self::Leaf(leaf) => leaf.block_id(),
            Self::Inner(inner) => inner.block_id(),
        }
    }

    // TODO No pub(crate) but rather pub(super::super)?
    pub(crate) fn raw_blockdata(&self) -> &Data {
        match self {
            Self::Leaf(leaf) => leaf.raw_blockdata(),
            Self::Inner(inner) => inner.raw_blockdata(),
        }
    }

    pub async fn remove(self, node_store: &DataNodeStore<B>) -> Result<()> {
        self._into_block().remove(&node_store.block_store).await
    }

    fn _into_block(self) -> Block<B> {
        match self {
            Self::Leaf(leaf) => leaf.into_block(),
            Self::Inner(inner) => inner.into_block(),
        }
    }

    pub(crate) async fn flush(&mut self, block_store: &LockingBlockStore<B>) -> Result<()> {
        match self {
            Self::Leaf(leaf) => leaf.flush(block_store).await,
            Self::Inner(inner) => inner.flush(block_store).await,
        }
    }

    pub fn convert_to_new_inner_node(
        self,
        first_child: DataNode<B>,
        layout: &NodeLayout,
    ) -> DataInnerNode<B> {
        let mut block = self._into_block();
        let block_data: ZeroedData<&mut Data> = ZeroedData::fill_with_zeroes(block.data_mut());
        data_inner_node::initialize_inner_node(
            first_child.depth() + 1,
            &[*first_child.block_id()],
            layout,
            block_data,
        );
        DataInnerNode::new(block, layout)
            .expect("Newly created inner node shouldn't violate any invariants")
    }

    pub fn overwrite_node_with(
        self,
        source: &DataNode<B>,
        layout: &NodeLayout,
    ) -> Result<DataNode<B>> {
        let mut block = self._into_block();
        let dest_data = block.data_mut();
        let source_data = source.raw_blockdata();
        assert_eq!(
            usize::try_from(layout.block_size_bytes).unwrap(),
            source_data.len(),
            "Source block has {} bytes but the layout expects {}",
            source_data.len(),
            layout.block_size_bytes
        );
        assert_eq!(
            usize::try_from(layout.block_size_bytes).unwrap(),
            dest_data.len(),
            "Destination block has {} bytes but the layout expects {}",
            dest_data.len(),
            layout.block_size_bytes
        );
        dest_data.copy_from_slice(source_data);
        // TODO DataNode::parse() is checking invariants again but we don't need to do that - violating invariants wouldn't have been able to create the source object.
        DataNode::parse(block, layout)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::testutils::*;
    use super::*;
    use binary_layout::Field;
    use cryfs_blockstore::BLOCKID_LEN;

    #[allow(non_snake_case)]
    mod parse {
        use super::*;

        #[tokio::test]
        async fn whenLoadingFullInnerNodeAtDepth1_thenSucceeds() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let node = new_full_inner_node(nodestore).await;

                    let block = node.into_block();
                    let DataNode::Inner(node) = DataNode::parse(block, nodestore.layout()).unwrap() else {
                        panic!("Expected inner node");
                    };

                    assert_eq!(
                        nodestore.layout().max_children_per_inner_node(),
                        node.num_children().get(),
                    );
                    assert_eq!(1, node.depth().get());
                })
            })
            .await;
        }

        #[tokio::test]
        async fn whenLoadingNodeWithOneChildAtDepth2_thenSucceeds() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let child = new_full_inner_node(nodestore).await;
                    let node = nodestore
                        .create_new_inner_node(2, &[*child.block_id()])
                        .await
                        .unwrap();

                    let block = node.into_block();
                    let DataNode::Inner(node) = DataNode::parse(block, nodestore.layout()).unwrap() else {
                        panic!("Expected inner node");
                    };

                    assert_eq!(1, node.num_children().get(),);
                    assert_eq!(2, node.depth().get());
                })
            })
            .await;
        }

        #[tokio::test]
        async fn whenLoadingFullLeafNode_thenSucceeds() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let node = new_full_leaf_node(nodestore).await;

                    let block = node.into_block();
                    let DataNode::Leaf(node) = DataNode::parse(block, nodestore.layout()).unwrap() else {
                        panic!("Expected leaf node");
                    };

                    assert_eq!(
                        nodestore.layout().max_bytes_per_leaf() as usize,
                        node.data().len(),
                    );
                })
            })
            .await;
        }

        #[tokio::test]
        async fn whenLoadingEmptyLeafNode_thenSucceeds() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let node = new_empty_leaf_node(nodestore).await;

                    let block = node.into_block();
                    let DataNode::Leaf(node) = DataNode::parse(block, nodestore.layout()).unwrap() else {
                        panic!("Expected leaf node");
                    };

                    assert_eq!(0, node.data().len());
                })
            })
            .await;
        }

        #[tokio::test]
        async fn whenLoadingInnerNodeWithWrongFormatVersion_thenFails() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let node = new_full_inner_node(nodestore).await;
                    let node_id = *node.block_id();

                    let mut block = node.into_block();
                    node::View::new(block.data_mut())
                        .format_version_header_mut()
                        .write(10);

                    assert_eq!(
                        format!("Loaded a node BlockId({}) with format_version_header == 10. This is not a supported format.", node_id.to_hex()),
                        DataNode::parse(block, nodestore.layout())
                            .unwrap_err()
                            .to_string(),
                    );

                    // Still fails when loading
                    assert_eq!(
                        format!("Loaded a node BlockId({}) with format_version_header == 10. This is not a supported format.", node_id.to_hex()),
                        nodestore.load(node_id).await.unwrap_err().to_string(),
                    );
                })
            })
            .await;
        }

        #[tokio::test]
        async fn whenLoadingLeafWithWrongFormatVersion_thenFails() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let node = new_full_leaf_node(nodestore).await;
                    let node_id = *node.block_id();

                    let mut block = node.into_block();
                    node::View::new(block.data_mut())
                        .format_version_header_mut()
                        .write(10);

                    assert_eq!(
                        format!("Loaded a node BlockId({}) with format_version_header == 10. This is not a supported format.", node_id.to_hex()),
                        DataNode::parse(block, nodestore.layout())
                            .unwrap_err()
                            .to_string(),
                    );

                    // Still fails when loading
                    assert_eq!(
                        format!("Loaded a node BlockId({}) with format_version_header == 10. This is not a supported format.", node_id.to_hex()),
                        nodestore.load(node_id).await.unwrap_err().to_string(),
                    );
                })
            })
            .await;
        }

        #[tokio::test]
        async fn whenLoadingTooSmallInnerNode_thenFails() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let node = new_full_inner_node(nodestore).await;
                    let node_id = *node.block_id();

                    let mut block = node.into_block();
                    let len = block.data().len();
                    block.data_mut().resize(len - 1);

                    assert_eq!(
                        format!("Expected to load block of size 1024 but loaded block BlockId({}) had size 1023", node_id.to_hex()),
                        DataNode::parse(block, nodestore.layout())
                            .unwrap_err()
                            .to_string(),
                    );
                })
            })
            .await
        }

        #[tokio::test]
        async fn whenLoadingTooLargeInnerNode_thenFails() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let node = new_full_inner_node(nodestore).await;
                    let node_id = *node.block_id();

                    let mut block = node.into_block();
                    let len = block.data().len();
                    block.data_mut().resize(len + 1);

                    assert_eq!(
                        format!("Expected to load block of size 1024 but loaded block BlockId({}) had size 1025", node_id.to_hex()),
                        DataNode::parse(block, nodestore.layout())
                            .unwrap_err()
                            .to_string(),
                    );
                })
            })
            .await
        }

        #[tokio::test]
        async fn whenLoadingTooSmallLeafNode_thenFails() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let node = new_empty_leaf_node(nodestore).await;
                    let node_id = *node.block_id();

                    let mut block = node.into_block();
                    let len = block.data().len();
                    block.data_mut().resize(len - 1);

                    assert_eq!(
                        format!("Expected to load block of size 1024 but loaded block BlockId({}) had size 1023", node_id.to_hex()),
                        DataNode::parse(block, nodestore.layout())
                            .unwrap_err()
                            .to_string(),
                    );
                })
            })
            .await
        }

        #[tokio::test]
        async fn whenLoadingTooLargeLeafNode_thenFails() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let node = new_empty_leaf_node(nodestore).await;
                    let node_id = *node.block_id();

                    let mut block = node.into_block();
                    let len = block.data().len();
                    block.data_mut().resize(len + 1);

                    assert_eq!(
                        format!("Expected to load block of size 1024 but loaded block BlockId({}) had size 1025", node_id.to_hex()),
                        DataNode::parse(block, nodestore.layout())
                            .unwrap_err()
                            .to_string(),
                    );
                })
            })
            .await
        }

        #[tokio::test]
        async fn givenLayoutWithJustLargeEnoughBlockSize_whenLoadingInnerNode_thenSucceeds() {
            const JUST_LARGE_ENOUGH_SIZE: u32 = node::data::OFFSET as u32 + 2 * BLOCKID_LEN as u32;
            with_nodestore_with_blocksize(JUST_LARGE_ENOUGH_SIZE, |nodestore| {
                Box::pin(async move {
                    let node = new_full_inner_node(nodestore).await;

                    let block = node.into_block();

                    let DataNode::Inner(node) = DataNode::parse(block, nodestore.layout()).unwrap() else {
                        panic!("Expected an inner node");
                    };
                    assert_eq!(2, node.num_children().get());
                })
            })
            .await;
        }

        #[tokio::test]
        async fn givenLayoutWithJustLargeEnoughBlockSize_whenLoadingLeafNode_thenSucceeds() {
            const JUST_LARGE_ENOUGH_SIZE: u32 = node::data::OFFSET as u32 + 2 * BLOCKID_LEN as u32;
            with_nodestore_with_blocksize(JUST_LARGE_ENOUGH_SIZE, |nodestore| {
                Box::pin(async move {
                    let node = new_full_leaf_node(nodestore).await;

                    let block = node.into_block();

                    let DataNode::Leaf(node) = DataNode::parse(block, nodestore.layout()).unwrap() else {
                        panic!("Expected a leaf node");
                    };
                    assert_eq!(
                        nodestore.layout().max_bytes_per_leaf() as usize,
                        node.data().len()
                    );
                })
            })
            .await;
        }

        #[tokio::test]
        #[should_panic = "Tried to create a DataNodeStore with block size 39 (physical: 39) but must be at least 40"]
        async fn givenLayoutWithTooSmallBlockSize_whenLoadingInnerNode_thenFailss() {
            const JUST_LARGE_ENOUGH_SIZE: usize = node::data::OFFSET + 2 * BLOCKID_LEN;
            with_nodestore_with_blocksize(JUST_LARGE_ENOUGH_SIZE as u32 - 1, |nodestore| {
                Box::pin(async move {
                    new_full_inner_node(nodestore).await;
                })
            })
            .await;
        }

        #[tokio::test]
        #[should_panic = "Tried to create a DataNodeStore with block size 39 (physical: 39) but must be at least 40"]
        async fn givenLayoutWithTooSmallBlockSize_whenLoadingLeafNode_thenFailss() {
            const JUST_LARGE_ENOUGH_SIZE: usize = node::data::OFFSET + 2 * BLOCKID_LEN;
            with_nodestore_with_blocksize(JUST_LARGE_ENOUGH_SIZE as u32 - 1, |nodestore| {
                Box::pin(async move {
                    new_full_leaf_node(nodestore).await;
                })
            })
            .await;
        }

        #[tokio::test]
        async fn whenLoadingTooDeepInnerNode_thenFails() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let node = new_full_inner_node(nodestore).await;
                    let node_id = *node.block_id();

                    let mut block = node.into_block();
                    node::View::new(block.data_mut())
                        .depth_mut()
                        .write(super::super::data_inner_node::MAX_DEPTH + 1);

                    assert_eq!(
                        "Loaded an inner node with depth 11 but the maximum is 10",
                        DataNode::parse(block, nodestore.layout())
                            .unwrap_err()
                            .to_string(),
                    );

                    // Still fails when loading
                    assert_eq!(
                        "Loaded an inner node with depth 11 but the maximum is 10",
                        nodestore.load(node_id).await.unwrap_err().to_string(),
                    );
                })
            })
            .await;
        }

        #[tokio::test]
        async fn whenLoadingInnerNodeWithTooFewChildren_thenFails() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let node = new_full_inner_node(nodestore).await;
                    let node_id = *node.block_id();

                    let mut block = node.into_block();
                    node::View::new(block.data_mut()).size_mut().write(0);

                    assert_eq!(
                        "Loaded an inner node that claims to store 0 children but the minimum is 1.",
                        DataNode::parse(block, nodestore.layout())
                            .unwrap_err()
                            .to_string(),
                    );

                    // Still fails when loading
                    assert_eq!(
                        "Loaded an inner node that claims to store 0 children but the minimum is 1.",
                        nodestore.load(node_id).await.unwrap_err().to_string(),
                    );
                })
            })
            .await;
        }

        #[tokio::test]
        async fn whenLoadingInnerNodeWithTooManyChildren_thenFails() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let node = new_full_inner_node(nodestore).await;
                    let node_id = *node.block_id();

                    let mut block = node.into_block();
                    node::View::new(block.data_mut()).size_mut().write(nodestore.layout().max_children_per_inner_node() + 1);

                    assert_eq!(
                        "Loaded an inner node that claims to store 64 children but the maximum is 63.",
                        DataNode::parse(block, nodestore.layout())
                            .unwrap_err()
                            .to_string(),
                    );

                    // Still fails when loading
                    assert_eq!(
                        "Loaded an inner node that claims to store 64 children but the maximum is 63.",
                        nodestore.load(node_id).await.unwrap_err().to_string(),
                    );
                })
            })
            .await;
        }

        #[tokio::test]
        async fn whenLoadingLeafNodeWithTooMuchData_thenFails() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let node = new_full_leaf_node(nodestore).await;
                    let node_id = *node.block_id();

                    let mut block = node.into_block();
                    node::View::new(block.data_mut())
                        .size_mut()
                        .write(nodestore.layout().max_bytes_per_leaf() + 1);

                    assert_eq!(
                        "Loaded a leaf that claims to store 1017 bytes but the maximum is 1016.",
                        DataNode::parse(block, nodestore.layout())
                            .unwrap_err()
                            .to_string(),
                    );

                    // Still fails when loading
                    assert_eq!(
                        "Loaded a leaf that claims to store 1017 bytes but the maximum is 1016.",
                        nodestore.load(node_id).await.unwrap_err().to_string(),
                    );
                })
            })
            .await;
        }
    }

    mod block_id {
        use super::*;

        #[tokio::test]
        async fn loaded_inner_node_returns_correct_key() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let block_id = *new_inner_node(nodestore).await.block_id();

                    let loaded = load_node(nodestore, block_id).await;
                    assert_eq!(block_id, *loaded.block_id());
                })
            })
            .await;
        }

        #[tokio::test]
        async fn loaded_leaf_node_returns_correct_key() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let block_id = *new_full_leaf_node(nodestore).await.block_id();

                    let loaded = load_node(nodestore, block_id).await;
                    assert_eq!(block_id, *loaded.block_id());
                })
            })
            .await;
        }
    }

    mod convert_to_new_inner_node {
        use super::*;

        #[tokio::test]
        async fn converts_leaf_node_to_inner_node_depth_1() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let source_node = nodestore
                        .create_new_leaf_node(&full_leaf_data(1))
                        .await
                        .unwrap()
                        .upcast();
                    let first_child = new_full_leaf_node(nodestore).await.upcast();
                    let first_child_id = *first_child.block_id();

                    let inner_node =
                        source_node.convert_to_new_inner_node(first_child, nodestore.layout());

                    assert_eq!(1, inner_node.depth().get());
                    assert_eq!(1, inner_node.num_children().get());
                    assert_eq!(first_child_id, inner_node.children().next().unwrap());
                    // Assert the unused region gets zeroed out
                    let used_space = node::data::OFFSET + BLOCKID_LEN;
                    assert_eq!(
                        &vec![0; PHYSICAL_BLOCK_SIZE_BYTES as usize - used_space],
                        &inner_node.raw_blockdata()[used_space..]
                    );
                })
            })
            .await;
        }

        #[tokio::test]
        async fn converts_leaf_node_to_inner_node_depth_2() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let source_node = nodestore
                        .create_new_leaf_node(&full_leaf_data(1))
                        .await
                        .unwrap()
                        .upcast();
                    let first_child = new_inner_node(nodestore).await.upcast();
                    let first_child_id = *first_child.block_id();

                    let inner_node =
                        source_node.convert_to_new_inner_node(first_child, nodestore.layout());

                    assert_eq!(2, inner_node.depth().get());
                    assert_eq!(1, inner_node.num_children().get());
                    assert_eq!(first_child_id, inner_node.children().next().unwrap());
                    // Assert the unused region gets zeroed out
                    let used_space = node::data::OFFSET + BLOCKID_LEN;
                    assert_eq!(
                        &vec![0; PHYSICAL_BLOCK_SIZE_BYTES as usize - used_space],
                        &inner_node.raw_blockdata()[used_space..]
                    );
                })
            })
            .await;
        }

        #[tokio::test]
        async fn converts_inner_node_to_inner_node_depth_1() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let source_node = new_full_inner_node(nodestore).await.upcast();
                    let first_child = new_full_leaf_node(nodestore).await.upcast();
                    let first_child_id = *first_child.block_id();

                    let inner_node =
                        source_node.convert_to_new_inner_node(first_child, nodestore.layout());

                    assert_eq!(1, inner_node.depth().get());
                    assert_eq!(1, inner_node.num_children().get());
                    assert_eq!(first_child_id, inner_node.children().next().unwrap());
                    // Assert the unused region gets zeroed out
                    let used_space = node::data::OFFSET + BLOCKID_LEN;
                    assert_eq!(
                        &vec![0; PHYSICAL_BLOCK_SIZE_BYTES as usize - used_space],
                        &inner_node.raw_blockdata()[used_space..]
                    );
                })
            })
            .await;
        }

        #[tokio::test]
        async fn converts_inner_node_to_inner_node_depth_2() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let source_node = new_full_inner_node(nodestore).await.upcast();
                    let first_child = new_inner_node(nodestore).await.upcast();
                    let first_child_id = *first_child.block_id();

                    let inner_node =
                        source_node.convert_to_new_inner_node(first_child, nodestore.layout());

                    assert_eq!(2, inner_node.depth().get());
                    assert_eq!(1, inner_node.num_children().get());
                    assert_eq!(first_child_id, inner_node.children().next().unwrap());
                    // Assert the unused region gets zeroed out
                    let used_space = node::data::OFFSET + BLOCKID_LEN;
                    assert_eq!(
                        &vec![0; PHYSICAL_BLOCK_SIZE_BYTES as usize - used_space],
                        &inner_node.raw_blockdata()[used_space..]
                    );
                })
            })
            .await;
        }
    }

    #[allow(non_snake_case)]
    mod remove {
        use super::*;

        #[tokio::test]
        async fn givenOtherwiseEmptyNodeStore_whenRemovingExistingLeaf_withRemovingAfterCreating_thenCannotBeLoadedAnymore(
        ) {
            with_nodestore(move |nodestore| {
                Box::pin(async move {
                    let leaf = new_full_leaf_node(nodestore).await.upcast();
                    let node_id = *leaf.block_id();

                    leaf.remove(nodestore).await.unwrap();
                    assert!(nodestore.load(node_id).await.unwrap().is_none());
                })
            })
            .await
        }

        #[tokio::test]
        async fn givenOtherwiseEmptyNodeStore_whenRemovingExistingLeaf_withRemovingAfterLoading_thenCannotBeLoadedAnymore(
        ) {
            with_nodestore(move |nodestore| {
                Box::pin(async move {
                    let node_id = *new_full_leaf_node(nodestore).await.block_id();
                    let leaf = nodestore.load(node_id).await.unwrap().unwrap();

                    leaf.remove(nodestore).await.unwrap();
                    assert!(nodestore.load(node_id).await.unwrap().is_none());
                })
            })
            .await
        }

        #[tokio::test]
        async fn givenOtherwiseEmptyNodeStore_whenRemovingExistingInnerNode_withRemovingAfterCreating_thenCannotBeLoadedAnymore(
        ) {
            with_nodestore(move |nodestore| {
                Box::pin(async move {
                    let leaf = new_full_leaf_node(nodestore).await.upcast();
                    let node = nodestore
                        .create_new_inner_node(1, &[*leaf.block_id()])
                        .await
                        .unwrap()
                        .upcast();
                    let node_id = *node.block_id();

                    leaf.remove(nodestore).await.unwrap();
                    node.remove(nodestore).await.unwrap();

                    assert!(nodestore.load(node_id).await.unwrap().is_none());
                })
            })
            .await
        }

        #[tokio::test]
        async fn givenOtherwiseEmptyNodeStore_whenRemovingExistingInnerNode_withRemovingAfterLoading_thenCannotBeLoadedAnymore(
        ) {
            with_nodestore(move |nodestore| {
                Box::pin(async move {
                    let leaf = new_full_leaf_node(nodestore).await.upcast();
                    let node_id = *nodestore
                        .create_new_inner_node(1, &[*leaf.block_id()])
                        .await
                        .unwrap()
                        .block_id();
                    let node = nodestore.load(node_id).await.unwrap().unwrap();

                    leaf.remove(nodestore).await.unwrap();
                    node.remove(nodestore).await.unwrap();

                    assert!(nodestore.load(node_id).await.unwrap().is_none());
                })
            })
            .await
        }

        #[tokio::test]
        async fn givenNodeStoreWithOtherEntries_whenRemovingExistingLeafNode_withRemovingAfterCreating_thenCannotBeLoadedAnymore(
        ) {
            with_nodestore(move |nodestore| {
                Box::pin(async move {
                    new_full_inner_node(nodestore).await;

                    let node = new_full_leaf_node(nodestore).await.upcast();
                    let node_id = *node.block_id();

                    node.remove(nodestore).await.unwrap();

                    assert!(nodestore.load(node_id).await.unwrap().is_none());
                })
            })
            .await
        }

        #[tokio::test]
        async fn givenNodeStoreWithOtherEntries_whenRemovingExistingLeafNode_withRemovingAfterLoading_thenCannotBeLoadedAnymore(
        ) {
            with_nodestore(move |nodestore| {
                Box::pin(async move {
                    new_full_inner_node(nodestore).await;

                    let node_id = *new_full_leaf_node(nodestore).await.block_id();
                    let node = nodestore.load(node_id).await.unwrap().unwrap();

                    node.remove(nodestore).await.unwrap();

                    assert!(nodestore.load(node_id).await.unwrap().is_none());
                })
            })
            .await
        }

        #[tokio::test]
        async fn givenNodeStoreWithOtherEntries_whenRemovingExistingLeafNode_withRemovingAfterCreating_thenDoesntDeleteOtherNodes(
        ) {
            with_nodestore(move |nodestore| {
                Box::pin(async move {
                    let full_inner = *new_full_inner_node(nodestore).await.block_id();

                    let node = new_full_leaf_node(nodestore).await.upcast();

                    node.remove(nodestore).await.unwrap();

                    assert_full_inner_node_is_valid(nodestore, full_inner).await;
                })
            })
            .await
        }

        #[tokio::test]
        async fn givenNodeStoreWithOtherEntries_whenRemovingExistingLeafNode_withRemovingAfterLoading_thenDoesntDeleteOtherNodes(
        ) {
            with_nodestore(move |nodestore| {
                Box::pin(async move {
                    let full_inner = *new_full_inner_node(nodestore).await.block_id();

                    let node_id = *new_full_leaf_node(nodestore).await.block_id();
                    let node = nodestore.load(node_id).await.unwrap().unwrap();

                    node.remove(nodestore).await.unwrap();

                    assert_full_inner_node_is_valid(nodestore, full_inner).await;
                })
            })
            .await
        }

        #[tokio::test]
        async fn givenNodeStoreWithOtherEntries_whenRemovingExistingInnerNode_withRemovingAfterCreating_thenCannotBeLoadedAnymore(
        ) {
            with_nodestore(move |nodestore| {
                Box::pin(async move {
                    new_full_inner_node(nodestore).await;

                    let leaf = new_full_leaf_node(nodestore).await.upcast();
                    let node = nodestore
                        .create_new_inner_node(1, &[*leaf.block_id()])
                        .await
                        .unwrap()
                        .upcast();
                    let node_id = *node.block_id();

                    leaf.remove(nodestore).await.unwrap();
                    node.remove(nodestore).await.unwrap();

                    assert!(nodestore.load(node_id).await.unwrap().is_none());
                })
            })
            .await
        }

        #[tokio::test]
        async fn givenNodeStoreWithOtherEntries_whenRemovingExistingInnerNode_withRemovingAfterLoading_thenCannotBeLoadedAnymore(
        ) {
            with_nodestore(move |nodestore| {
                Box::pin(async move {
                    new_full_inner_node(nodestore).await;

                    let leaf = new_full_leaf_node(nodestore).await.upcast();
                    let node_id = *nodestore
                        .create_new_inner_node(1, &[*leaf.block_id()])
                        .await
                        .unwrap()
                        .block_id();
                    let node = nodestore.load(node_id).await.unwrap().unwrap();

                    leaf.remove(nodestore).await.unwrap();
                    node.remove(nodestore).await.unwrap();

                    assert!(nodestore.load(node_id).await.unwrap().is_none());
                })
            })
            .await
        }

        #[tokio::test]
        async fn givenNodeStoreWithOtherEntries_whenRemovingExistingInnerNode_withRemovingAfterCreating_thenDoesntDeleteOtherEntries(
        ) {
            with_nodestore(move |nodestore| {
                Box::pin(async move {
                    let full_inner = *new_full_inner_node(nodestore).await.block_id();

                    let leaf = new_full_leaf_node(nodestore).await.upcast();
                    let node = nodestore
                        .create_new_inner_node(1, &[*leaf.block_id()])
                        .await
                        .unwrap()
                        .upcast();

                    leaf.remove(nodestore).await.unwrap();
                    node.remove(nodestore).await.unwrap();

                    assert_full_inner_node_is_valid(nodestore, full_inner).await;
                })
            })
            .await
        }

        #[tokio::test]
        async fn givenNodeStoreWithOtherEntries_whenRemovingExistingInnerNode_withRemovingAfterLoading_thenDoesntDeleteOtherEntries(
        ) {
            with_nodestore(move |nodestore| {
                Box::pin(async move {
                    let full_inner = *new_full_inner_node(nodestore).await.block_id();

                    let leaf = new_full_leaf_node(nodestore).await.upcast();
                    let node_id = *nodestore
                        .create_new_inner_node(1, &[*leaf.block_id()])
                        .await
                        .unwrap()
                        .block_id();
                    let node = nodestore.load(node_id).await.unwrap().unwrap();

                    leaf.remove(nodestore).await.unwrap();
                    node.remove(nodestore).await.unwrap();

                    assert_full_inner_node_is_valid(nodestore, full_inner).await;
                })
            })
            .await
        }
    }

    mod depth {
        use super::*;

        #[tokio::test]
        async fn depth_0() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let node = new_full_leaf_node(nodestore).await.upcast();
                    assert_eq!(0, node.depth());

                    // And after loading
                    let node_id = *node.block_id();
                    drop(node);
                    let node = load_node(nodestore, node_id).await;
                    assert_eq!(0, node.depth());
                })
            })
            .await;
        }

        #[tokio::test]
        async fn depth_1() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let leaf = new_full_leaf_node(nodestore).await;
                    let node = nodestore
                        .create_new_inner_node(1, &[*leaf.block_id()])
                        .await
                        .unwrap()
                        .upcast();
                    assert_eq!(1, node.depth());

                    // And after loading
                    let node_id = *node.block_id();
                    drop(node);
                    let node = load_node(nodestore, node_id).await;
                    assert_eq!(1, node.depth());
                })
            })
            .await;
        }

        #[tokio::test]
        async fn depth_2() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let leaf = new_full_inner_node(nodestore).await;
                    let node = nodestore
                        .create_new_inner_node(2, &[*leaf.block_id()])
                        .await
                        .unwrap()
                        .upcast();
                    assert_eq!(2, node.depth());

                    // And after loading
                    let node_id = *node.block_id();
                    drop(node);
                    let node = load_node(nodestore, node_id).await;
                    assert_eq!(2, node.depth());
                })
            })
            .await;
        }
    }

    // TODO Test
    //  - overwrite_node_with
}
