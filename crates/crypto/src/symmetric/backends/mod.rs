//! Backend implementations for symmetric ciphers.
//!
//! This module provides multiple backend implementations for each cipher type,
//! allowing users to choose based on performance, portability, or security requirements.
//!
//! # Available Backends
//!
//! - [`aead`]: Pure Rust implementations using the `aead` crate ecosystem
//! - [`openssl`]: OpenSSL-based implementations (generally fastest)
//! - [`libsodium`]: libsodium-based implementations (hardware accelerated)
//! - [`cipher`]: Additional cipher implementations (placeholder for future use)

pub mod aead;
pub mod cipher;
pub mod libsodium;
pub mod openssl;

// TODO Add cargo features for these backends and choose one for CryFS prod builds
