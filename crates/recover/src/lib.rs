#![forbid(unsafe_code)]
// TODO #![deny(missing_docs)]

mod args;

mod cli;
pub use cli::RecoverCli;

cryfs_version::assert_cargo_version_equals_git_version!();