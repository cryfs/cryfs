use derive_more::{Add, AddAssign, Sum};

#[derive(Debug, Add, AddAssign, Sum, Clone, Copy, PartialEq, Eq)]
pub struct BlobStoreActionCounts {
    pub blob_num_bytes: u64,
    pub blob_resize: u64,
    pub blob_read_all: u64,
    pub blob_read: u64,
    pub blob_try_read: u64,
    pub blob_write: u64,
    pub blob_flush: u64,
    pub blob_num_nodes: u64,
    pub blob_remove: u64,
    pub blob_all_blocks: u64,
    pub store_create: u64,
    pub store_try_create: u64,
    pub store_load: u64,
    pub store_remove_by_id: u64,
    pub store_load_block_depth: u64,
    pub store_num_nodes: u64,
    pub store_estimate_space_for_num_blocks_left: u64,
    pub store_virtual_block_size_bytes: u64,
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
