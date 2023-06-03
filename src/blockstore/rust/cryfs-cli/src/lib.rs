mod args;
mod env;

mod cli;
pub use cli::Cli;

mod console;
mod password_provider;

mod runner;

cryfs_version::assert_cargo_version_equals_git_version!();
