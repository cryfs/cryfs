//! Make a temporary cargo project with a given `Cargo.toml` and `main.rs`,
//! and allow building or running it.

#![forbid(unsafe_code)]
// TODO #![deny(missing_docs)]

mod builder;
mod project;

pub use builder::TempProjectBuilder;
pub use project::TempProject;
