use anyhow::{anyhow, Result};
use binary_layout::{define_layout, Field};
use std::num::NonZeroU64;

use cryfs_blockstore::BLOCKID_LEN;

pub const FORMAT_VERSION_HEADER: u16 = 0;

define_layout!(node, LittleEndian, {
    format_version_header: u16,

    // Not currently used, only used for alignment.
    unused: u8,

    // Leaf nodes have a depth of 0. Each layer above has a depth of one higher than the level directly below.
    depth: u8,

    // Leaf nodes store number of data byes here. Inner nodes store number of children.
    size: u32,

    // Data. Leaf nodes just store bytes here. Inner nodes store a list of child block ids.
    data: [u8],
});

#[derive(Debug, Clone, Copy)]
pub struct NodeLayout {
    pub block_size_bytes: u32,
}

impl NodeLayout {
    #[cfg(test)]
    pub const fn header_len() -> usize {
        node::data::OFFSET
    }

    pub fn max_bytes_per_leaf(&self) -> u32 {
        self.block_size_bytes - u32::try_from(node::data::OFFSET).unwrap()
    }

    pub fn max_children_per_inner_node(&self) -> u32 {
        let datasize = self.max_bytes_per_leaf();
        datasize / u32::try_from(BLOCKID_LEN).unwrap()
    }

    pub fn num_leaves_per_full_subtree(&self, depth: u8) -> Result<NonZeroU64> {
        Ok(NonZeroU64::new(
            u64::from(self.max_children_per_inner_node())
                .checked_pow(u32::from(depth))
                .ok_or_else(|| {
                    anyhow!(
                        "Overflow in max_children_per_inner_node^(depth-1): {}^({}-1)",
                        self.max_children_per_inner_node(),
                        depth,
                    )
                })?,
        )
        .expect("non_zero^x can never be zero"))
    }
}
