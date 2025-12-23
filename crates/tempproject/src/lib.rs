//! Create temporary Cargo projects for testing.
//!
//! This crate provides utilities for creating, building, and running temporary
//! Cargo projects. It is useful for testing CLI tools, build systems, or any
//! code that needs to interact with compiled Rust binaries.
//!
//! # Features
//!
//! - **Builder pattern**: Use [`TempProjectBuilder`] to configure your project's
//!   `Cargo.toml` and `main.rs` contents
//! - **Build caching**: Build results are cached, so calling [`TempProject::run_debug`]
//!   multiple times only compiles once
//! - **Debug and release modes**: Build and run in either debug or release mode
//! - **Error handling**: Failed builds return [`ProcessError`] with exit code,
//!   stdout, and stderr for debugging
//!
//! # Example
//!
//! ```no_run
//! use tempproject::TempProjectBuilder;
//!
//! // Create a temporary project
//! let project = TempProjectBuilder::new()?
//!     .cargo(r#"
//!         [package]
//!         name = "example"
//!         version = "0.1.0"
//!         edition = "2021"
//!     "#)
//!     .main(r#"
//!         fn main() {
//!             println!("Hello from temporary project!");
//!         }
//!     "#)
//!     .build()?;
//!
//! // Build and run the project in debug mode
//! let mut cmd = project.run_debug()?;
//! cmd.assert().success().stdout(predicates::str::contains("Hello"));
//!
//! // The project directory is automatically cleaned up when `project` is dropped
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! # Error Handling
//!
//! When a build fails, [`ProcessError`] provides detailed information:
//!
//! ```no_run
//! use tempproject::TempProjectBuilder;
//!
//! let project = TempProjectBuilder::new()?
//!     .cargo(r#"
//!         [package]
//!         name = "broken"
//!         version = "0.1.0"
//!     "#)
//!     .main("fn main() { undefined_function(); }")
//!     .build()?;
//!
//! match project.build_debug() {
//!     Ok(path) => println!("Built successfully: {:?}", path),
//!     Err(err) => {
//!         println!("Build failed with exit code: {:?}", err.exit_code);
//!         println!("Stderr: {:?}", err.stderr);
//!     }
//! }
//! # Ok::<(), anyhow::Error>(())
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod builder;
mod project;

pub use builder::TempProjectBuilder;
pub use project::{ProcessError, TempProject};
