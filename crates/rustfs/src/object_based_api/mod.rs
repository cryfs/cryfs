mod interface;
pub use interface::{Device, Dir, File, Node, OpenFile, Symlink};

// TODO Remove pub(crate)
pub(crate) mod adapter;
