use derive_more::{Add, AddAssign, Sum};
use std::fmt::Debug;

#[derive(Add, AddAssign, Sum, PartialEq, Eq, Clone, Copy)]
pub struct ActionCounts {
    pub store_load: u32,
    pub store_try_create: u32,
    pub store_overwrite: u32,
    pub store_remove_by_id: u32,
    pub store_remove: u32,
    pub store_num_blocks: u32,
    pub store_estimate_num_free_bytes: u32,
    pub store_overhead: u32,
    pub store_all_blocks: u32,
    pub store_create: u32,
    pub store_flush_block: u32,
    pub blob_data: u32,
    pub blob_data_mut: u32,
    pub blob_resize: u32,
}

impl Debug for ActionCounts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ds = f.debug_struct("ActionCounts");
        let mut print_field = |name, value: u32| {
            if value != 0 {
                ds.field(name, &value);
            }
        };
        print_field("store_load", self.store_load);
        print_field("store_try_create", self.store_try_create);
        print_field("store_overwrite", self.store_overwrite);
        print_field("store_remove_by_id", self.store_remove_by_id);
        print_field("store_remove", self.store_remove);
        print_field("store_num_blocks", self.store_num_blocks);
        print_field("store_estimate_num_free_bytes", self.store_estimate_num_free_bytes);
        print_field("store_overhead", self.store_overhead);
        print_field("store_all_blocks", self.store_all_blocks);
        print_field("store_create", self.store_create);
        print_field("store_flush_block", self.store_flush_block);
        print_field("blob_data", self.blob_data);
        print_field("blob_data_mut", self.blob_data_mut);
        print_field("blob_resize", self.blob_resize);
        ds.finish()
    }
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
        store_overhead: 0,
        store_all_blocks: 0,
        store_create: 0,
        store_flush_block: 0,
        blob_data: 0,
        blob_data_mut: 0,
        blob_resize: 0,
    };
}
