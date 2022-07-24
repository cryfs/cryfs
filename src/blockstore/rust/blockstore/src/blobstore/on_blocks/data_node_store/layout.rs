use binary_layout::define_layout;

pub const FORMAT_VERSION_HEADER: u16 = 0;

define_layout!(node, LittleEndian, {
    format_version_header: u16,

    // Not currently used, only used for alignment.
    unused_must_be_zero: u8,

    // Leaf nodes have a depth of 0. Each layer above has a depth of one higher than the level directly below.
    depth: u8,

    // Leaf nodes store number of data byes here. Inner nodes store number of children.
    size: u32,

    // Data. Leaf nodes just store bytes here. Inner nodes store a list of child block ids.
    data: [u8],
});
