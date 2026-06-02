//! This module allows running a file system using the [fuse-mt] library.

mod backend_adapter;

mod mount;
pub use mount::{mount, spawn_mount};

pub use fuser::{Config, MountOption, SessionACL};

// fuse_mt mounts via fuser 0.16 (`fuser_fusemt`), so its session is fuser-0.16's BackgroundSession.
pub type RunningFilesystem = super::RunningFilesystem<fuser_fusemt::BackgroundSession>;
