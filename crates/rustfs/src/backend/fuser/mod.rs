//! This module allows running a file system using the [fuser] library.

mod backend_adapter;

mod mount;
// TODO BackendAdapter is currently only needed for e2e-perf-tests. Can we remove it from the public API?
pub use backend_adapter::BackendAdapter;
pub use fuser::MountOption;
pub use mount::{mount, spawn_mount};

pub type RunningFilesystem = super::RunningFilesystem<fuser::BackgroundSession>;
