//! This module allows running a file system using the [fuse-mt] library.

mod backend_adapter;

mod mount;
pub use fuse_mt_fuser::MountOption;
pub use mount::{mount, spawn_mount};

pub type RunningFilesystem = super::RunningFilesystem<fuse_mt_fuser::BackgroundSession>;
