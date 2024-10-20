//#![forbid(unsafe_code)]
// TODO #![deny(missing_docs)]

mod background_process;
mod ipc;
mod mounter;
mod runner;

pub use mounter::Mounter;
pub use runner::{CreateOrLoad, MountArgs};

cryfs_version::assert_cargo_version_equals_git_version!();
