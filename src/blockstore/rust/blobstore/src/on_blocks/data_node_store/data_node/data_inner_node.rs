use anyhow::{anyhow, ensure, Result};
use binary_layout::Field;
use std::num::{NonZeroU32, NonZeroU8};

use super::super::layout::{node, NodeLayout, FORMAT_VERSION_HEADER};
use super::DataNode;
use cryfs_blockstore::{Block, BlockId, BlockStore, LockingBlockStore, BLOCKID_LEN};
use cryfs_utils::data::{Data, ZeroedData};

pub(super) const MAX_DEPTH: u8 = 10;

#[derive(Debug)]
pub struct DataInnerNode<B: BlockStore + Send + Sync> {
    block: Block<B>,
}

impl<B: BlockStore + Send + Sync> DataInnerNode<B> {
    pub fn new(block: Block<B>, layout: &NodeLayout) -> Result<Self> {
        // Min block size: enough for header and for inner nodes to have at least two children and form a tree.
        let min_block_size = usize::try_from(node::data::OFFSET + 2 * BLOCKID_LEN).unwrap();
        assert!(layout.block_size_bytes as usize >= min_block_size, "Block doesn't have enough space for header and two children. This should have been checked before calling DataInnerNode::new");

        let view = node::View::new(block.data());
        ensure!(
            view.format_version_header().read() == FORMAT_VERSION_HEADER,
            "Loaded a node with format version {} but the current version is {}",
            view.format_version_header().read(),
            FORMAT_VERSION_HEADER,
        );
        let depth = view.depth().read();
        assert_ne!(
            0, depth,
            "Loaded an inner node with depth 0. This doesn't make sense, it should have been loaded as a leaf node",
        );
        ensure!(
            block.data().len() == layout.block_size_bytes as usize,
            "Loaded block of size {} but expected {}",
            block.data().len(),
            layout.block_size_bytes
        );
        ensure!(
            depth <= MAX_DEPTH,
            "Loaded an inner node with depth {} but the maximum is {}",
            depth,
            MAX_DEPTH,
        );
        let size = view.size().read();
        let max_children_per_inner_node = layout.max_children_per_inner_node();
        ensure!(
            size >= 1,
            "Loaded an inner node that claims to store {} children but the minimum is 1.",
            size,
        );
        ensure!(
            size <= max_children_per_inner_node,
            "Loaded an inner node that claims to store {} children but the maximum is {}.",
            size,
            max_children_per_inner_node,
        );
        Ok(Self { block })
    }

    pub fn depth(&self) -> NonZeroU8 {
        let view = node::View::new(self.block.data());
        NonZeroU8::new(view.depth().read())
            .expect("DataInnerNode class invariant violated: Has depth of zero")
    }

    pub fn block_id(&self) -> &BlockId {
        self.block.block_id()
    }

    pub(super) fn raw_blockdata(&self) -> &Data {
        self.block.data()
    }

    pub(super) fn into_block(self) -> Block<B> {
        self.block
    }

    pub(super) async fn flush(&mut self, blockstore: &LockingBlockStore<B>) -> Result<()> {
        blockstore.flush_block(&mut self.block).await
    }

    pub fn num_children(&self) -> NonZeroU32 {
        let view = node::View::new(self.block.data().as_ref());
        NonZeroU32::new(view.size().read())
            .expect("DataInnerNode class invariant violated: Has only zero children")
    }

    pub fn children<'a>(&'a self) -> impl Iterator<Item = BlockId> + ExactSizeIterator + 'a {
        let view = node::View::new(self.block.data().as_ref());
        let num_children = usize::try_from(view.size().read()).unwrap();
        let children_data = view.into_data().into_slice();
        let children_ids = children_data.chunks_exact(BLOCKID_LEN);
        assert!(
            num_children <= children_ids.len(),
            "Class invariant violated: Tried to load an inner node with {} children but support at most {} per inner node",
            num_children,
            children_ids.len(),
        );
        children_ids
            .take(num_children)
            .map(|id_bytes| BlockId::from_slice(id_bytes).unwrap())
    }

    fn _children_mut_raw<'a>(
        &'a mut self,
    ) -> impl Iterator<Item = &'a mut [u8]> + ExactSizeIterator + 'a {
        let view = node::View::new(self.block.data_mut().as_mut());
        let children_data: &mut [u8] = view.into_data().into_slice();
        children_data.chunks_exact_mut(BLOCKID_LEN)
    }

    pub fn add_child(&mut self, child: &DataNode<B>) -> Result<()> {
        let depth = self.depth().get();
        let view = node::View::new(self.block.data_mut());
        let prev_num_children = view.size().read();
        ensure!(
            child.depth() == depth - 1,
            "Tried to add a child of depth {} to an inner node of depth {}",
            child.depth(),
            self.depth()
        );
        let new_child_entry: &mut [u8] = self
            ._children_mut_raw()
            .skip(usize::try_from(prev_num_children).unwrap())
            .next()
            .ok_or_else(|| anyhow!("Adding more children than we can store"))?;
        new_child_entry.copy_from_slice(child.block_id().data());
        let mut view = node::View::new(self.block.data_mut());
        view.size_mut().write(prev_num_children + 1);
        Ok(())
    }

    pub fn shrink_num_children(&mut self, new_num_children: NonZeroU32) -> Result<()> {
        let mut view = node::View::new(self.block.data_mut().as_mut());
        let old_num_children = view.size().read();
        ensure!(
            new_num_children.get() <= old_num_children,
            "Called DataInnerNode::shrink_num_children({}) for a node with {} children",
            new_num_children,
            view.size().read()
        );
        let free_begin = usize::try_from(new_num_children.get()).unwrap() * BLOCKID_LEN;
        let free_end = usize::try_from(old_num_children).unwrap() * BLOCKID_LEN;
        view.data_mut()[free_begin..free_end].fill(0);
        view.size_mut().write(new_num_children.get());
        Ok(())
    }

    pub fn upcast(self) -> DataNode<B> {
        DataNode::Inner(self)
    }
}

pub fn serialize_inner_node(depth: u8, children: &[BlockId], layout: &NodeLayout) -> Data {
    let data = ZeroedData::new(layout.block_size_bytes.try_into().unwrap());
    initialize_inner_node(depth, children, layout, data)
}

pub fn initialize_inner_node<D>(
    depth: u8,
    children: &[BlockId],
    layout: &NodeLayout,
    dest: ZeroedData<D>,
) -> D
where
    D: AsRef<[u8]> + AsMut<[u8]>,
{
    assert!(
        depth != 0,
        "Inner node cannot have a depth of 0. Is this perhaps a leaf instead?"
    );
    assert!(
        depth <= MAX_DEPTH,
        "Inner node cannot have a depth of {}, the maximum is {}",
        depth,
        MAX_DEPTH,
    );
    assert!(
        children.len() >= 1,
        "Inner node must have at least one child"
    );
    assert!(
        children.len() <= usize::try_from(layout.max_children_per_inner_node()).unwrap(),
        "Inner nodes can only store {} children but tried to store {}",
        layout.max_children_per_inner_node(),
        children.len(),
    );

    let mut view = node::View::new(dest.into_inner());
    view.format_version_header_mut()
        .write(FORMAT_VERSION_HEADER);
    view.unused_mut().write(0);
    view.depth_mut().write(depth);
    view.size_mut()
        .write(u32::try_from(children.len()).unwrap());
    _serialize_children(view.data_mut(), children);
    view.into_storage()
}

fn _serialize_children(dest: &mut [u8], children: &[BlockId]) {
    assert!(dest.len() >= children.len() * BLOCKID_LEN, "Serializing {} children requires {} bytes but tried to serialize into a buffer with {} bytes.", children.len(), children.len() * BLOCKID_LEN, dest.len());
    for (index, child) in children.iter().enumerate() {
        // TODO Some way to avoid this copy by not using &[BlockId] or Vec<BlockId> but our own collection type that already has it aligned correctly?
        dest[(BLOCKID_LEN * index)..(BLOCKID_LEN * (index + 1))].copy_from_slice(child.data());
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::testutils::*;
    use super::*;

    #[allow(non_snake_case)]
    mod new {
        use super::*;

        #[tokio::test]
        async fn whenLoadingFullInnerNodeAtDepth1_thenSucceeds() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let node = new_full_inner_node(nodestore).await;

                    let block = node.into_block();
                    let node = DataInnerNode::new(block, nodestore.layout()).unwrap();

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
                    let node = DataInnerNode::new(block, nodestore.layout()).unwrap();

                    assert_eq!(1, node.num_children().get(),);
                    assert_eq!(2, node.depth().get());
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
                        "Loaded a node with format version 10 but the current version is 0",
                        DataInnerNode::new(block, nodestore.layout())
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
        #[should_panic = "Loaded an inner node with depth 0. This doesn't make sense, it should have been loaded as a leaf node"]
        async fn whenLoadingLeafAsInnerNode_thenFails() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let node = new_full_leaf_node(nodestore).await;

                    let block = node.into_block();

                    let _ = DataInnerNode::new(block, nodestore.layout());
                })
            })
            .await;
        }

        #[tokio::test]
        async fn whenLoadingTooSmallInnerNode_thenFails() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let node = new_full_inner_node(nodestore).await;

                    let mut block = node.into_block();
                    let len = block.data().len();
                    block.data_mut().resize(len - 1);

                    assert_eq!(
                        "Loaded block of size 1023 but expected 1024",
                        DataInnerNode::new(block, nodestore.layout())
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

                    let mut block = node.into_block();
                    let len = block.data().len();
                    block.data_mut().resize(len + 1);

                    assert_eq!(
                        "Loaded block of size 1025 but expected 1024",
                        DataInnerNode::new(block, nodestore.layout())
                            .unwrap_err()
                            .to_string(),
                    );
                })
            })
            .await
        }

        #[tokio::test]
        async fn givenLayoutWithJustLargeEnoughBlockSize_whenLoading_thenSucceeds() {
            const JUST_LARGE_ENOUGH_SIZE: u32 = node::data::OFFSET as u32 + 2 * BLOCKID_LEN as u32;
            with_nodestore_with_blocksize(JUST_LARGE_ENOUGH_SIZE, |nodestore| {
                Box::pin(async move {
                    let node = new_full_inner_node(nodestore).await;

                    let block = node.into_block();

                    let node = DataInnerNode::new(block, nodestore.layout()).unwrap();
                    assert_eq!(2, node.num_children().get());
                })
            })
            .await;
        }

        #[tokio::test]
        #[should_panic = "Tried to create a DataNodeStore with block size 39 (physical: 39) but must be at least 40"]
        async fn givenLayoutWithTooSmallBlockSize_whenLoading_thenFailss() {
            const JUST_LARGE_ENOUGH_SIZE: usize = node::data::OFFSET + 2 * BLOCKID_LEN;
            with_nodestore_with_blocksize(JUST_LARGE_ENOUGH_SIZE as u32 - 1, |nodestore| {
                Box::pin(async move {
                    new_full_inner_node(nodestore).await;
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
                        .write(MAX_DEPTH + 1);

                    assert_eq!(
                        "Loaded an inner node with depth 11 but the maximum is 10",
                        DataInnerNode::new(block, nodestore.layout())
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
                        DataInnerNode::new(block, nodestore.layout())
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
                        DataInnerNode::new(block, nodestore.layout())
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
    }

    mod serialize_inner_node {
        use super::*;

        #[test]
        fn test_serialize_inner_node() {
            let layout = NodeLayout {
                block_size_bytes: PHYSICAL_BLOCK_SIZE_BYTES,
            };
            let blockid1 = BlockId::new_random();
            let blockid2 = BlockId::new_random();
            let children = vec![blockid1, blockid2];
            let serialized = serialize_inner_node(1, &children, &layout);
            let view = node::View::new(serialized.as_ref());
            assert_eq!(view.format_version_header().read(), FORMAT_VERSION_HEADER);
            assert_eq!(view.unused().read(), 0);
            assert_eq!(view.depth().read(), 1);
            assert_eq!(view.size().read(), 2);
            assert_eq!(&view.data()[0..BLOCKID_LEN], blockid1.data());
            assert_eq!(
                &view.data()[BLOCKID_LEN..(BLOCKID_LEN * 2)],
                blockid2.data()
            );
        }
    }

    mod block_id {
        use super::*;

        #[tokio::test]
        async fn loaded_node_returns_correct_key() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let block_id = *new_inner_node(nodestore).await.block_id();

                    let loaded = load_inner_node(nodestore, block_id).await;
                    assert_eq!(block_id, *loaded.block_id());
                })
            })
            .await;
        }
    }

    mod add_child {
        use super::*;

        #[tokio::test]
        async fn from_one_to_two_at_depth_1() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let child1 = new_full_leaf_node(nodestore).await;
                    let mut node = nodestore
                        .create_new_inner_node(1, &[*child1.block_id()])
                        .await
                        .unwrap();
                    assert_eq!(1, node.num_children().get());

                    let child2 = new_full_leaf_node(nodestore).await.upcast();
                    node.add_child(&child2).unwrap();
                    assert_eq!(2, node.num_children().get());
                    assert_eq!(
                        vec![*child1.block_id(), *child2.block_id()],
                        node.children().collect::<Vec<_>>(),
                    );

                    // Still correct after loading
                    let node_id = *node.block_id();
                    drop(node);
                    let node = load_inner_node(nodestore, node_id).await;
                    assert_eq!(2, node.num_children().get());
                    assert_eq!(
                        vec![*child1.block_id(), *child2.block_id()],
                        node.children().collect::<Vec<_>>(),
                    );
                })
            })
            .await;
        }

        #[tokio::test]
        async fn from_almost_full_to_full_at_depth_1() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let left_children = new_full_leaves(
                        nodestore,
                        nodestore.layout().max_children_per_inner_node() - 1,
                    )
                    .await;

                    let mut node = nodestore
                        .create_new_inner_node(1, &left_children)
                        .await
                        .unwrap();
                    assert_eq!(
                        nodestore.layout().max_children_per_inner_node() - 1,
                        node.num_children().get()
                    );

                    let right_child = new_full_leaf_node(nodestore).await.upcast();
                    node.add_child(&right_child).unwrap();
                    let mut leaves = left_children;
                    leaves.push(*right_child.block_id());
                    assert_eq!(
                        nodestore.layout().max_children_per_inner_node(),
                        node.num_children().get()
                    );
                    assert_eq!(leaves, node.children().collect::<Vec<_>>(),);

                    // Still correct after loading
                    let node_id = *node.block_id();
                    drop(node);
                    let node = load_inner_node(nodestore, node_id).await;
                    assert_eq!(
                        nodestore.layout().max_children_per_inner_node(),
                        node.num_children().get()
                    );
                    assert_eq!(leaves, node.children().collect::<Vec<_>>(),);
                })
            })
            .await;
        }

        #[tokio::test]
        async fn from_one_to_two_at_depth_2() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let child1 = new_inner_node(nodestore).await;
                    let mut node = nodestore
                        .create_new_inner_node(2, &[*child1.block_id()])
                        .await
                        .unwrap();
                    assert_eq!(1, node.num_children().get());

                    let child2 = new_inner_node(nodestore).await.upcast();
                    node.add_child(&child2).unwrap();
                    assert_eq!(2, node.num_children().get());
                    assert_eq!(
                        vec![*child1.block_id(), *child2.block_id()],
                        node.children().collect::<Vec<_>>(),
                    );

                    // Still correct after loading
                    let node_id = *node.block_id();
                    drop(node);
                    let node = load_inner_node(nodestore, node_id).await;
                    assert_eq!(2, node.num_children().get());
                    assert_eq!(
                        vec![*child1.block_id(), *child2.block_id()],
                        node.children().collect::<Vec<_>>(),
                    );
                })
            })
            .await;
        }

        #[tokio::test]
        async fn from_almost_full_to_full_at_depth_2() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let left_children = new_inner_nodes(
                        nodestore,
                        nodestore.layout().max_children_per_inner_node() - 1,
                    )
                    .await;

                    let mut node = nodestore
                        .create_new_inner_node(2, &left_children)
                        .await
                        .unwrap();
                    assert_eq!(
                        nodestore.layout().max_children_per_inner_node() - 1,
                        node.num_children().get()
                    );

                    let right_child = new_inner_node(nodestore).await.upcast();
                    node.add_child(&right_child).unwrap();
                    let mut children = left_children;
                    children.push(*right_child.block_id());
                    assert_eq!(
                        nodestore.layout().max_children_per_inner_node(),
                        node.num_children().get()
                    );
                    assert_eq!(children, node.children().collect::<Vec<_>>(),);

                    // Still correct after loading
                    let node_id = *node.block_id();
                    drop(node);
                    let node = load_inner_node(nodestore, node_id).await;
                    assert_eq!(
                        nodestore.layout().max_children_per_inner_node(),
                        node.num_children().get()
                    );
                    assert_eq!(children, node.children().collect::<Vec<_>>(),);
                })
            })
            .await;
        }

        #[tokio::test]
        async fn wrong_depth() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let child1 = new_inner_node(nodestore).await;
                    let mut node = nodestore
                        .create_new_inner_node(2, &[*child1.block_id()])
                        .await
                        .unwrap();
                    assert_eq!(1, node.num_children().get());

                    // Adding child at wrong depth fails
                    let child2 = new_full_leaf_node(nodestore).await.upcast();
                    assert_eq!(
                        "Tried to add a child of depth 0 to an inner node of depth 2",
                        node.add_child(&child2).unwrap_err().to_string(),
                    );

                    // Node is unchanged
                    assert_eq!(1, node.num_children().get());
                    assert_eq!(
                        vec![*child1.block_id()],
                        node.children().collect::<Vec<_>>(),
                    );

                    // Still correct after loading
                    let node_id = *node.block_id();
                    drop(node);
                    let node = load_inner_node(nodestore, node_id).await;
                    assert_eq!(1, node.num_children().get());
                    assert_eq!(
                        vec![*child1.block_id()],
                        node.children().collect::<Vec<_>>(),
                    );
                })
            })
            .await;
        }

        #[tokio::test]
        async fn node_already_full() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let children = new_full_leaves(
                        nodestore,
                        nodestore.layout().max_children_per_inner_node(),
                    )
                    .await;

                    let mut node = nodestore.create_new_inner_node(1, &children).await.unwrap();
                    assert_eq!(
                        nodestore.layout().max_children_per_inner_node(),
                        node.num_children().get()
                    );

                    // Adding child over limit fails
                    let right_child = new_full_leaf_node(nodestore).await.upcast();
                    assert_eq!(
                        "Adding more children than we can store",
                        node.add_child(&right_child).unwrap_err().to_string(),
                    );

                    // Node is unchanged
                    assert_eq!(
                        nodestore.layout().max_children_per_inner_node(),
                        node.num_children().get()
                    );
                    assert_eq!(children, node.children().collect::<Vec<_>>(),);

                    // Still correct after loading
                    let node_id = *node.block_id();
                    drop(node);
                    let node = load_inner_node(nodestore, node_id).await;
                    assert_eq!(
                        nodestore.layout().max_children_per_inner_node(),
                        node.num_children().get()
                    );
                    assert_eq!(children, node.children().collect::<Vec<_>>(),);
                })
            })
            .await;
        }
    }

    mod children_and_num_children {
        use super::*;

        #[tokio::test]
        async fn one_child() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let block_id = *new_full_leaf_node(nodestore).await.block_id();
                    let node = nodestore
                        .create_new_inner_node(1, &[block_id])
                        .await
                        .unwrap();

                    assert_eq!(1, node.num_children().get());
                    assert_eq!(vec![block_id], node.children().collect::<Vec<_>>(),);

                    // Still correct after loading
                    let node_id = *node.block_id();
                    drop(node);
                    let node = load_inner_node(nodestore, node_id).await;
                    assert_eq!(1, node.num_children().get());
                    assert_eq!(vec![block_id], node.children().collect::<Vec<_>>(),);
                })
            })
            .await;
        }

        #[tokio::test]
        async fn two_children() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let block_id1 = *new_full_leaf_node(nodestore).await.block_id();
                    let block_id2 = *new_full_leaf_node(nodestore).await.block_id();
                    let node = nodestore
                        .create_new_inner_node(1, &[block_id1, block_id2])
                        .await
                        .unwrap();

                    assert_eq!(2, node.num_children().get());
                    assert_eq!(
                        vec![block_id1, block_id2],
                        node.children().collect::<Vec<_>>(),
                    );

                    // Still correct after loading
                    let node_id = *node.block_id();
                    drop(node);
                    let node = load_inner_node(nodestore, node_id).await;
                    assert_eq!(2, node.num_children().get());
                    assert_eq!(
                        vec![block_id1, block_id2],
                        node.children().collect::<Vec<_>>(),
                    );
                })
            })
            .await;
        }

        #[tokio::test]
        async fn max_children() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let block_ids = new_full_leaves(
                        nodestore,
                        nodestore.layout().max_children_per_inner_node(),
                    )
                    .await;
                    let node = nodestore
                        .create_new_inner_node(1, &block_ids)
                        .await
                        .unwrap();

                    assert_eq!(
                        nodestore.layout().max_children_per_inner_node(),
                        node.num_children().get()
                    );
                    assert_eq!(block_ids, node.children().collect::<Vec<_>>(),);

                    // Still correct after loading
                    let node_id = *node.block_id();
                    drop(node);
                    let node = load_inner_node(nodestore, node_id).await;
                    assert_eq!(
                        nodestore.layout().max_children_per_inner_node(),
                        node.num_children().get()
                    );
                    assert_eq!(block_ids, node.children().collect::<Vec<_>>(),);
                })
            })
            .await;
        }
    }

    mod physical_block_size {
        use super::*;

        #[tokio::test]
        async fn block_has_correct_size() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    assert_ne!(
                        PHYSICAL_BLOCK_SIZE_BYTES,
                        nodestore.layout().max_bytes_per_leaf()
                    );
                    let node = new_inner_node(nodestore).await;
                    let block = node.into_block();
                    assert_eq!(PHYSICAL_BLOCK_SIZE_BYTES as usize, block.data().len());
                })
            })
            .await;
        }
    }

    mod depth {
        use super::*;

        #[tokio::test]
        async fn depth_1() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let leaf = new_full_leaf_node(nodestore).await;
                    let node = nodestore
                        .create_new_inner_node(1, &[*leaf.block_id()])
                        .await
                        .unwrap();
                    assert_eq!(1, node.depth().get());

                    // And after loading
                    let node_id = *node.block_id();
                    drop(node);
                    let node = load_inner_node(nodestore, node_id).await;
                    assert_eq!(1, node.depth().get());
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
                        .unwrap();
                    assert_eq!(2, node.depth().get());

                    // And after loading
                    let node_id = *node.block_id();
                    drop(node);
                    let node = load_inner_node(nodestore, node_id).await;
                    assert_eq!(2, node.depth().get());
                })
            })
            .await;
        }
    }

    mod into_block {
        use super::*;

        #[tokio::test]
        async fn into_block() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let leaf = new_full_leaf_node(nodestore).await;
                    let node = nodestore
                        .create_new_inner_node(1, &[*leaf.block_id()])
                        .await
                        .unwrap();
                    let block = node.into_block();
                    assert_eq!(
                        leaf.block_id().data(),
                        &block.data()[node::data::OFFSET..(node::data::OFFSET + BLOCKID_LEN)]
                    );
                })
            })
            .await;
        }
    }

    mod upcast {
        use super::*;

        #[tokio::test]
        async fn one_child_depth_1() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let block_id = *new_full_leaf_node(nodestore).await.block_id();
                    let node = nodestore
                        .create_new_inner_node(1, &[block_id])
                        .await
                        .unwrap();

                    let DataNode::Inner(node) = node.upcast() else {
                        panic!("Should have upcast as inner node");
                    };

                    assert_eq!(1, node.num_children().get());
                    assert_eq!(vec![block_id], node.children().collect::<Vec<_>>(),);
                })
            })
            .await;
        }

        #[tokio::test]
        async fn two_children_depth_1() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let block_id1 = *new_full_leaf_node(nodestore).await.block_id();
                    let block_id2 = *new_full_leaf_node(nodestore).await.block_id();
                    let node = nodestore
                        .create_new_inner_node(1, &[block_id1, block_id2])
                        .await
                        .unwrap();

                    let DataNode::Inner(node) = node.upcast() else {
                        panic!("Should have upcast as inner node");
                    };

                    assert_eq!(2, node.num_children().get());
                    assert_eq!(
                        vec![block_id1, block_id2],
                        node.children().collect::<Vec<_>>(),
                    );
                })
            })
            .await;
        }

        #[tokio::test]
        async fn max_children_depth_1() {
            with_nodestore(|nodestore| {
                Box::pin(async move {
                    let block_ids = new_full_leaves(
                        nodestore,
                        nodestore.layout().max_children_per_inner_node(),
                    )
                    .await;
                    let node = nodestore
                        .create_new_inner_node(1, &block_ids)
                        .await
                        .unwrap();

                    let DataNode::Inner(node) = node.upcast() else {
                        panic!("Should have upcast as inner node");
                    };

                    assert_eq!(
                        nodestore.layout().max_children_per_inner_node(),
                        node.num_children().get()
                    );
                    assert_eq!(block_ids, node.children().collect::<Vec<_>>(),);
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
                        .unwrap();

                    let DataNode::Inner(node) = node.upcast() else {
                        panic!("Should have upcast as inner node");
                    };
                    assert_eq!(2, node.depth().get());
                })
            })
            .await;
        }
    }

    // TODO Test
    //  - flush
    //  - shrink_num_children
}
