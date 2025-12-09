//! This module allows running a file system using the [fuser] library.

mod backend_adapter;

mod mount;
pub use mount::{mount, spawn_mount};

pub use fuser::MountOption;

pub type RunningFilesystem = super::RunningFilesystem<fuser::BackgroundSession>;
