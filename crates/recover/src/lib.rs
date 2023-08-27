#![forbid(unsafe_code)]
// TODO #![deny(missing_docs)]

mod cli;
pub use cli::Cli;

cryfs_version::assert_cargo_version_equals_git_version!();
