use anyhow::{ensure, Result};

use super::{
    layout::{node, NodeLayout, FORMAT_VERSION_HEADER},
    DataNodeStore,
};
use crate::blockstore::{high_level::Block, low_level::BlockStore, BlockId};
use crate::data::{Data, ZeroedData};

mod data_inner_node;
pub use data_inner_node::{serialize_inner_node, DataInnerNode};

mod data_leaf_node;
pub use data_leaf_node::{serialize_leaf_node_optimized, DataLeafNode};

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

    pub(super) fn raw_blockdata(&self) -> &Data {
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

    pub(super) fn as_block_mut(&mut self) -> &mut Block<B> {
        match self {
            Self::Leaf(leaf) => leaf.as_block_mut(),
            Self::Inner(inner) => inner.as_block_mut(),
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
