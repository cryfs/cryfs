use anyhow::{ensure, Result};
use binary_layout::Field;

use super::super::layout::{node, NodeLayout, FORMAT_VERSION_HEADER};
use crate::blockstore::{high_level::Block, low_level::BlockStore, BlockId};
use crate::data::Data;

pub struct DataLeafNode<B: BlockStore + Send + Sync> {
    block: Block<B>,
}

impl<B: BlockStore + Send + Sync> DataLeafNode<B> {
    pub fn new(block: Block<B>, layout: &NodeLayout) -> Result<Self> {
        let view = node::View::new(block.data());
        assert_eq!(
            0, view.depth().read(),
            "Loaded a leaf with depth {}. This doesn't make sense, it should have been loaded as an inner node",
            view.depth().read(),
        );
        assert!(block.data().len() > node::data::OFFSET, "Block doesn't have enough space for header. This should have been checked before calling DataLeafNode::new");
        let max_bytes_per_leaf = layout.max_bytes_per_leaf();
        let size = view.size().read();
        ensure!(
            size <= max_bytes_per_leaf,
            "Loaded a leaf that claims to store {} bytes but the maximum is {}.",
            size,
            max_bytes_per_leaf,
        );
        Ok(Self { block })
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

    pub fn num_bytes(&self) -> u32 {
        let view = node::View::new(self.block.data());
        view.size().read()
    }

    pub fn max_bytes_per_leaf(&self) -> u32 {
        NodeLayout {
            block_size_bytes: u32::try_from(self.block.data().len()).unwrap(),
        }
        .max_bytes_per_leaf()
    }

    pub fn resize(&mut self, new_num_bytes: u32) {
        assert!(
            new_num_bytes <= self.max_bytes_per_leaf(),
            "Trying to resize to {} bytes which is larger than the maximal size of {}",
            new_num_bytes,
            self.max_bytes_per_leaf()
        );
        let mut view = node::View::new(self.block.data_mut());
        let old_num_bytes = view.size().read();
        if new_num_bytes < old_num_bytes {
            let newly_unused_data_region = &mut view.data_mut()
                [usize::try_from(new_num_bytes).unwrap()..usize::try_from(old_num_bytes).unwrap()];
            newly_unused_data_region.fill(0);
        }
        view.size_mut().write(new_num_bytes);
    }

    pub fn data(&self) -> &[u8] {
        let view = node::View::new(self.block.data().as_ref());
        view.into_data().into_slice()
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        let view = node::View::new(self.block.data_mut().as_mut());
        view.into_data().into_slice()
    }
}

pub fn serialize_leaf_node_optimized(mut data: Data, layout: &NodeLayout) -> Data {
    let size: u32 = u32::try_from(data.len()).unwrap();
    assert!(
        size <= layout.max_bytes_per_leaf(),
        "Tried to create leaf with {} bytes but each leaf can only hold {}",
        size,
        layout.max_bytes_per_leaf()
    );
    assert!(data.available_prefix_bytes() >= node::data::OFFSET, "Data objects passed to serialize_leaf_node must have at least {} prefix bytes available, but only had {}", node::data::OFFSET, data.available_prefix_bytes());
    data.grow_region_fail_if_reallocation_necessary(node::data::OFFSET, 0)
        .expect("Not enough prefix bytes available for data object passed to serialize_leaf_node");
    let mut view = node::View::new(&mut data);
    view.format_version_header_mut()
        .write(FORMAT_VERSION_HEADER);
    view.unused_must_be_zero_mut().write(0);
    view.depth_mut().write(0);
    view.size_mut().write(size);
    // view.data is already set correctly because we grew this view from the data input
    data
}
