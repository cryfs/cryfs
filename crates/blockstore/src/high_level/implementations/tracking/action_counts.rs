use derive_more::{Add, AddAssign, Sum};

#[derive(Debug, Add, AddAssign, Sum, PartialEq, Eq, Clone, Copy)]
pub struct ActionCounts {
    pub store_load: u32,
    pub store_try_create: u32,
    pub store_overwrite: u32,
    pub store_remove_by_id: u32,
    pub store_remove: u32,
    pub store_num_blocks: u32,
    pub store_estimate_num_free_bytes: u32,
    pub store_block_size_from_physical_block_size: u32,
    pub store_all_blocks: u32,
    pub store_create: u32,
    pub store_flush_block: u32,
    pub blob_data: u32,
    pub blob_data_mut: u32,
    pub blob_resize: u32,
}

impl ActionCounts {
    pub const ZERO: Self = Self {
        store_load: 0,
        store_try_create: 0,
        store_overwrite: 0,
        store_remove_by_id: 0,
        store_remove: 0,
        store_num_blocks: 0,
        store_estimate_num_free_bytes: 0,
        store_block_size_from_physical_block_size: 0,
        store_all_blocks: 0,
        store_create: 0,
        store_flush_block: 0,
        blob_data: 0,
        blob_data_mut: 0,
        blob_resize: 0,
    };
}
