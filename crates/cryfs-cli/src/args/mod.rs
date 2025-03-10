mod cryfs_args;
mod fuse_option;
mod mount_args;

pub use cryfs_args::CryfsArgs;
pub use fuse_option::{AtimeOption, FuseOption};
pub use mount_args::MountArgs;

// TODO Evaluate `clap_mangen` as a potential automatic manpage generator
// TODO Evaluate `clap_complete` as a potenatial shell completion generator
// TODO Tests for each cli argument
// TODO Tests
