use derive_more::{Add, AddAssign, Sum};

#[derive(Debug, Add, AddAssign, Sum, Clone, Copy, PartialEq, Eq)]
pub struct BlobStoreActionCounts {
    pub blob_num_bytes: u32,
    pub blob_resize: u32,
    pub blob_read_all: u32,
    pub blob_read: u32,
    pub blob_try_read: u32,
    pub blob_write: u32,
    pub blob_flush: u32,
    pub blob_num_nodes: u32,
    pub blob_remove: u32,
    pub blob_all_blocks: u32,
    pub store_create: u32,
    pub store_try_create: u32,
    pub store_load: u32,
    pub store_remove_by_id: u32,
    pub store_load_block_depth: u32,
    pub store_num_nodes: u32,
    pub store_estimate_space_for_num_blocks_left: u32,
    pub store_virtual_block_size_bytes: u32,
}

impl BlobStoreActionCounts {
    pub const ZERO: Self = Self {
        blob_num_bytes: 0,
        blob_resize: 0,
        blob_read_all: 0,
        blob_read: 0,
        blob_try_read: 0,
        blob_write: 0,
        blob_flush: 0,
        blob_num_nodes: 0,
        blob_remove: 0,
        blob_all_blocks: 0,
        store_create: 0,
        store_try_create: 0,
        store_load: 0,
        store_remove_by_id: 0,
        store_load_block_depth: 0,
        store_num_nodes: 0,
        store_estimate_space_for_num_blocks_left: 0,
        store_virtual_block_size_bytes: 0,
    };
}
