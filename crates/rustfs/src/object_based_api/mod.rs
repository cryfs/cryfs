mod interface;
pub use interface::{Device, Dir, File, Node, OpenFile, Symlink};

mod utils;

// TODO Remove pub(crate)
pub(crate) mod high_level_adapter;
pub(crate) mod low_level_adapter;

// TODO ObjectBasedFsAdapter, ObjectBasedFsAdapterLL and FUSE_ROOT_ID are currently only needed for e2e-perf-tests. Can we remove it from the public API?
pub use high_level_adapter::ObjectBasedFsAdapter;
pub use low_level_adapter::{FUSE_ROOT_ID, ObjectBasedFsAdapterLL};
