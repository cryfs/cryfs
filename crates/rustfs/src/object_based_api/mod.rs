mod interface;
pub use interface::{Device, Dir, File, Node, OpenFile, Symlink};

mod utils;
#[cfg(feature = "fuser")]
pub use utils::FUSE_ROOT_ID;

// TODO Remove pub(crate)
#[cfg(feature = "fuse_mt")]
pub(crate) mod high_level_adapter;
#[cfg(feature = "fuser")]
pub(crate) mod low_level_adapter;

// TODO ObjectBasedFsAdapter, ObjectBasedFsAdapterLL and FUSE_ROOT_ID are currently only needed for e2e-perf-tests. Can we remove it from the public API?
#[cfg(feature = "fuse_mt")]
pub use high_level_adapter::ObjectBasedFsAdapter;
#[cfg(feature = "fuser")]
pub use low_level_adapter::ObjectBasedFsAdapterLL;
