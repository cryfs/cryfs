//! Cryptographic hash functions.
//!
//! This module provides cryptographic hash functions used for data integrity verification
//! and other cryptographic operations. All hash operations use a salt to prevent rainbow
//! table attacks.
//!
//! # Available Algorithms
//!
//! - **SHA-512**: 512-bit (64 byte) digest with 8-byte salt
//!
//! # Backend Implementations
//!
//! Multiple backend implementations are available:
//!
//! - [`OpensslSha512`]: Uses OpenSSL's SHA-512 implementation (default, generally fastest)
//! - [`Sha2Sha512`]: Pure Rust implementation from the `sha2` crate
//! - [`LibsodiumSha512`]: Uses libsodium's SHA-512 implementation
//!
//! # Example
//!
//! ```
//! use cryfs_crypto::hash::{Sha512, Salt, HashAlgorithm};
//!
//! // Generate a random salt
//! let salt = Salt::generate_random();
//!
//! // Hash some data
//! let data = b"Hello, CryFS!";
//! let hash = Sha512::hash(data, salt);
//!
//! // Access the digest and salt
//! println!("Digest: {}", hash.digest.to_hex());
//! println!("Salt: {}", hash.salt.to_hex());
//! ```

mod backends;
mod digest;
mod hash;
mod salt;

pub use digest::Digest;
pub use hash::Hash;
pub use salt::Salt;

// TODO Consider hardening by (1) increasing salt size to a full hash block and (2) switching to SHA3

#[cfg(test)]
mod tests;

// TODO Once Rust has const generic expressions stabilized, we should just have one `impl HashAlgorithmDef for Sha512` instead of one per backend.
//      Also, we should use the HashAlgorithmDef as a type parameter for HashAlgorithm instead of repeating the consts.
//      And backends would just do `impl HashAlgorithm<Sha512> for Sha512Backend`.

/// Defines the size constants for a hash algorithm.
///
/// This trait provides the digest length and salt length for a hash algorithm.
/// It is used in conjunction with [`HashAlgorithm`] to define the complete
/// hash algorithm interface.
///
/// # Associated Constants
///
/// - `DIGEST_LEN`: The length of the hash digest in bytes
/// - `SALT_LEN`: The length of the salt in bytes
pub trait HashAlgorithmDef {
    /// The length of the hash digest in bytes.
    const DIGEST_LEN: usize;
    /// The length of the salt in bytes.
    const SALT_LEN: usize;
}

/// A cryptographic hash algorithm.
///
/// This trait defines the interface for computing cryptographic hashes.
/// Implementations combine the salt and data before hashing to prevent
/// rainbow table attacks.
///
/// # Type Parameters
///
/// - `DIGEST_LEN`: The length of the hash digest in bytes
/// - `SALT_LEN`: The length of the salt in bytes
///
/// # Security
///
/// The salt is prepended to the data before hashing. This ensures that
/// the same data with different salts produces different digests.
pub trait HashAlgorithm<const DIGEST_LEN: usize, const SALT_LEN: usize> {
    /// Computes the hash of the given data with the provided salt.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to hash
    /// * `salt` - The salt to use for hashing
    ///
    /// # Returns
    ///
    /// A [`struct@Hash`] containing both the computed digest and the salt used.
    fn hash(data: &[u8], salt: Salt<SALT_LEN>) -> Hash<DIGEST_LEN, SALT_LEN>;
}

pub use backends::{LibsodiumSha512, OpensslSha512, Sha2Sha512};

/// Default SHA-512 implementation using OpenSSL.
///
/// This is the recommended SHA-512 implementation as it is generally
/// the fastest option on most platforms.
pub type Sha512 = backends::OpensslSha512;
