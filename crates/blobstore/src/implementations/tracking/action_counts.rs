use derive_more::{Add, AddAssign, Sum};
use std::fmt::Debug;

#[derive(Add, AddAssign, Sum, Clone, Copy, PartialEq, Eq)]
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
    pub store_num_nodes: u32,
    pub store_estimate_space_for_num_blocks_left: u32,
    pub store_logical_block_size_bytes: u32,
    pub store_flush_if_cached: u32,
}

impl Debug for BlobStoreActionCounts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ds = f.debug_struct("BlobStoreActionCounts");
        let mut print_field = |name, value: u32| {
            if value != 0 {
                ds.field(name, &value);
            }
        };
        print_field("blob_num_bytes", self.blob_num_bytes);
        print_field("blob_resize", self.blob_resize);
        print_field("blob_read_all", self.blob_read_all);
        print_field("blob_read", self.blob_read);
        print_field("blob_try_read", self.blob_try_read);
        print_field("blob_write", self.blob_write);
        print_field("blob_flush", self.blob_flush);
        print_field("blob_num_nodes", self.blob_num_nodes);
        print_field("blob_remove", self.blob_remove);
        print_field("blob_all_blocks", self.blob_all_blocks);
        print_field("store_create", self.store_create);
        print_field("store_try_create", self.store_try_create);
        print_field("store_load", self.store_load);
        print_field("store_remove_by_id", self.store_remove_by_id);
        print_field("store_num_nodes", self.store_num_nodes);
        print_field(
            "store_estimate_space_for_num_blocks_left",
            self.store_estimate_space_for_num_blocks_left,
        );
        print_field(
            "store_logical_block_size_bytes",
            self.store_logical_block_size_bytes,
        );
        print_field("store_flush_if_cached", self.store_flush_if_cached);
        ds.finish()
    }
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
        store_num_nodes: 0,
        store_estimate_space_for_num_blocks_left: 0,
        store_logical_block_size_bytes: 0,
        store_flush_if_cached: 0,
    };
}
