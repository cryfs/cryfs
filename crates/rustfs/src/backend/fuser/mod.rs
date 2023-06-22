//! This module allows running a file system using the [fuser] library.

mod running_filesystem;
pub use running_filesystem::RunningFilesystem;

mod backend_adapter;

mod mount;
pub use mount::{mount, spawn_mount};
