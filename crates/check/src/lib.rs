#![forbid(unsafe_code)]
// TODO #![deny(missing_docs)]

mod args;

mod cli;
pub use cli::{check_filesystem, RecoverCli};

mod checks;
mod console;
mod error;
pub use error::CorruptedError;
mod runner;
mod task_queue;

cryfs_version::assert_cargo_version_equals_git_version!();
