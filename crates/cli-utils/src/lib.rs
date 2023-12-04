//! This crate contains some utilities for handling the command line interface.
//! This is shared between the different cryfs executables, e.g. `cryfs` and `cryfs-recover`.

#![forbid(unsafe_code)]
// TODO #![deny(missing_docs)]

mod path;
pub use path::parse_path;

mod args;
pub mod password_provider;
mod version;

mod env;
pub use env::Environment;

mod application;
pub use application::{run, Application};

mod config;
pub use config::print_config;

cryfs_version::assert_cargo_version_equals_git_version!();

pub mod reexports_for_tests {
    pub use anyhow;
    pub use async_trait;
    pub use clap;
    pub use cryfs_version;
}
