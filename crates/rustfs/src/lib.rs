// TODO #![deny(missing_docs)]

pub mod object_based_api;

pub mod high_level_api;
pub mod low_level_api;

mod common;
pub use common::{
    AbsolutePath, AbsolutePathBuf, AtimeUpdateBehavior, DirEntry, FsError, FsResult, Gid,
    InodeNumber, Mode, NodeAttrs, NodeKind, NumBytes, OpenFlags, ParsePathError, PathComponent,
    PathComponentBuf, RequestInfo, Statfs, Uid,
};

pub mod backend;

pub use cryfs_utils::data::Data;

// Needed for `use` statements in doctests
// TODO Is there a better way?
pub use common::{Callback, CallbackImpl};

#[cfg(test)]
mod tests;

// TODO Test backends by running a mock filesystem, calling syscalls into it, and making sure the correct AsyncFilesystem functions get called
// TODO Black-box test AsyncFilesystem instances (e.g. passthrough, inmemory) by mounting them and calling file system operations on them
// TODO Test mount/spawn_mount correctly mount
// TODO Test RunningFilesystem::unmount_join correctly unmounts and SIGINT correctly unmounts.
// TODO Test the file system mountpoint is correctly freed after unmounting (e.g. we can re-mount to the same directory)
// TODO pjdfs could be useful for testing POSIX conformity. The fuser crate seems to use the fork here https://github.com/fleetfs/pjdfstest
