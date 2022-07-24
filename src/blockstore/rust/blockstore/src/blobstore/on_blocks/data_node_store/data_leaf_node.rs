use anyhow::{ensure, Result};
use binary_layout::Field;

use super::layout::{node, FORMAT_VERSION_HEADER};
use super::DataNodeStore;
use crate::blockstore::{high_level::Block, low_level::BlockStore};
use crate::data::Data;

pub struct DataLeafNode<B: BlockStore + Send + Sync> {
    block: Block<B>,
}

impl<B: BlockStore + Send + Sync> DataLeafNode<B> {
    pub fn new(block: Block<B>, store: &DataNodeStore<B>) -> Result<Self> {
        let view = super::node::View::new(block.data());
        assert_eq!(
            0, view.depth().read(),
            "Loaded a leaf with depth {}. This doesn't make sense, it should have been loaded as an inner node",
            view.depth().read(),
        );
        let max_bytes_per_leaf = store.max_bytes_per_leaf();
        let size = view.size().read();
        ensure!(
            size > 0,
            "Loaded a leaf that claims to store 0 bytes but the minimum is 1.",
        );
        ensure!(
            size < max_bytes_per_leaf,
            "Loaded a leaf that claims to store {} bytes but the maximum is {}.",
            size,
            max_bytes_per_leaf,
        );
        Ok(Self { block })
    }

    pub fn num_bytes(&self) -> u32 {
        let view = super::node::View::new(self.block.data());
        view.size().read()
    }
}

pub fn serialize_leaf_node<B: BlockStore + Send + Sync>(
    mut data: Data,
    node_store: &DataNodeStore<B>,
) -> Data {
    let size: u32 = u32::try_from(data.len()).unwrap();
    assert!(
        size < node_store.max_bytes_per_leaf(),
        "Tried to create leaf with {} bytes but each leaf can only hold {}",
        size,
        node_store.max_bytes_per_leaf()
    );
    assert!(data.available_prefix_bytes() >= node::data::OFFSET, "Data objects passed to create_new_leaf_node_optimized must have at least {} prefix bytes available, but only had {}", node::data::OFFSET, data.available_prefix_bytes());
    data.grow_region_fail_if_reallocation_necessary(node::data::OFFSET, 0).expect("Not enough prefix bytes available for data object passed to DataNodeStore::create_new_leaf_node_optimized");
    let mut view = node::View::new(&mut data);
    view.format_version_header_mut()
        .write(FORMAT_VERSION_HEADER);
    view.unused_must_be_zero_mut().write(0);
    view.depth_mut().write(0);
    view.size_mut().write(size);
    // view.data is already set correctly because we grew this view from the data input
    data
}
