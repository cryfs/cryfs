//! Backend implementations for scrypt key derivation.
//!
//! This module provides multiple scrypt implementations:
//!
//! - [`scrypt::ScryptScrypt`]: Pure Rust implementation from the `scrypt` crate
//! - [`openssl::ScryptOpenssl`]: OpenSSL-based implementation

// TODO Add cargo features for these backends and choose one for CryFS prod builds

pub mod openssl;
pub mod scrypt;
