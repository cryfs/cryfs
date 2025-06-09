//! This crate contains performance tests and benchmarks for CryFS.
//!
//! # Performance tests
//! These tests run filesystem operations and check the number of blobs and blocks that are read or written by each file system operation.
//! This is meant to ensure that we don't accidentally run more operations than necessary and don't regress performance over time.
//!
//! To run these, just run
//! $ cargo test
//!
//! # Benchmarks
//! This crate also allows running these tests as [criterion] benchmarks, measuring the time they take.
//!
//! To run these, just run
//! $ cargo bench --feature benchmark
//!
//! The `benchmark` feature is necessary so we generate benchmark code instead of test code.
//!
//! # Implementation details
//! Performance tests are pretty fast and are
//! - implemented on top of an [InMemoryBlockStore]
//! - executed by directly calling the [rustfs] filesystem API. No need to mount it or use fuse
//!
//! Benchmarks aim to be more realistic and are
//! - implemented on top of a [TempDirBlockStore] (i.e. blocks are stored on the real file system)
//! - executed by mounting CryFS using [fuse] or [fuse_mt] and executing filesystem operations through real OS syscalls
//!
//! A lot of the code in this crate is annotated by either
//! - #[cfg(feature = "benchmark")]      // Code only necessary for benchmarks
//! - #[cfg(not(feature = "benchmark"))] // Code only necessary for perf tests

#![cfg(any(test, feature = "benchmark"))]

mod filesystem_driver;
mod filesystem_fixture;
pub mod operations;
pub mod perf_test_macro;
mod test_driver;

// TODO For some reason, pressing CTRL+C during a benchmark run exits the program but keeps benchmarks running in the background? What's going on there?

cryfs_version::assert_cargo_version_equals_git_version!();
