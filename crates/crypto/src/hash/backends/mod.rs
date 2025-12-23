//! Backend implementations for hash algorithms.
//!
//! This module provides multiple backend implementations for each hash algorithm,
//! allowing users to choose based on performance or portability requirements.
//!
//! # Available Backends
//!
//! - [`OpensslSha512`]: SHA-512 using OpenSSL (generally fastest)
//! - [`Sha2Sha512`]: SHA-512 using pure Rust `sha2` crate (most portable)
//! - [`LibsodiumSha512`]: SHA-512 using libsodium

mod openssl;
pub use openssl::OpensslSha512;

mod sha2;
pub use sha2::Sha2Sha512;

mod libsodium;
pub use libsodium::LibsodiumSha512;
