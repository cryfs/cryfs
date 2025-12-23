//! Symmetric encryption ciphers.
//!
//! This module provides symmetric encryption ciphers using authenticated encryption
//! with associated data (AEAD). All ciphers provide both confidentiality and integrity
//! protection.
//!
//! # Available Ciphers
//!
//! ## AES-GCM (Advanced Encryption Standard in Galois/Counter Mode)
//!
//! - [`Aes256Gcm`]: 256-bit AES-GCM (recommended for most use cases)
//! - [`Aes128Gcm`]: 128-bit AES-GCM
//!
//! ## XChaCha20-Poly1305
//!
//! - [`XChaCha20Poly1305`]: Modern cipher with 192-bit nonce (recommended for high-volume encryption)
//!
//! # Backend Implementations
//!
//! Multiple backend implementations are available for each cipher:
//!
//! - **OpenSSL**: Generally fastest, uses hardware acceleration when available
//! - **Pure Rust crates**: Portable, no external dependencies
//! - **libsodium**: High-quality implementations, hardware accelerated
//!
//! # Ciphertext Format
//!
//! All ciphers produce ciphertext in the format:
//!
//! ```text
//! [nonce][encrypted_data][authentication_tag]
//! ```
//!
//! The overhead is stored as prefix (nonce) and suffix (tag) bytes.
//!
//! # Example
//!
//! ```
//! use cryfs_crypto::symmetric::{Aes256Gcm, Cipher, CipherDef, EncryptionKey, DefaultNonceSize};
//! use cryfs_utils::data::Data;
//!
//! // Generate a random encryption key
//! let key = EncryptionKey::generate_random::<{Aes256Gcm::<DefaultNonceSize>::KEY_SIZE}>();
//!
//! // Create a cipher instance
//! let cipher = Aes256Gcm::<DefaultNonceSize>::new(key).expect("valid key size");
//!
//! // Prepare plaintext with pre-allocated space for ciphertext overhead
//! let message = b"Secret message";
//! let mut plaintext = Data::allocate(
//!     Aes256Gcm::<DefaultNonceSize>::CIPHERTEXT_OVERHEAD_PREFIX,
//!     message.len(),
//!     Aes256Gcm::<DefaultNonceSize>::CIPHERTEXT_OVERHEAD_SUFFIX,
//! );
//! plaintext.as_mut().copy_from_slice(message);
//!
//! // Encrypt and decrypt
//! let ciphertext = cipher.encrypt(plaintext).expect("encryption succeeded");
//! let decrypted = cipher.decrypt(ciphertext).expect("decryption succeeded");
//! assert_eq!(decrypted.as_ref(), b"Secret message");
//! ```

use anyhow::Result;
use derive_more::{Display, Error};
use static_assertions::const_assert;

use cryfs_utils::data::Data;

/// A symmetric encryption cipher.
///
/// This trait defines the interface for symmetric encryption operations.
/// All implementations use authenticated encryption (AEAD), which provides
/// both confidentiality and integrity protection.
///
/// # Ciphertext Overhead
///
/// Ciphertext is larger than plaintext due to:
/// - **Prefix**: The nonce/IV (returned by [`ciphertext_overhead_prefix`](Cipher::ciphertext_overhead_prefix))
/// - **Suffix**: The authentication tag (returned by [`ciphertext_overhead_suffix`](Cipher::ciphertext_overhead_suffix))
///
/// # Security
///
/// - Nonces are randomly generated for each encryption
/// - Authentication tags are verified during decryption
/// - Decryption fails if the ciphertext was tampered with
pub trait Cipher {
    // TODO Can we make this API safer? It requires the data block passed in to have at least CIPHERTEXT_OVERHEAD prefix bytes available.

    /// Encrypts the given plaintext.
    ///
    /// The `data` parameter must have sufficient prefix and suffix space
    /// pre-allocated for the ciphertext overhead. Use `Data::allocate()` or
    /// `Data::grow_region()` to allocate this space.
    ///
    /// # Arguments
    ///
    /// * `data` - The plaintext to encrypt, with pre-allocated space for overhead
    ///
    /// # Returns
    ///
    /// The ciphertext containing `[nonce][encrypted_data][auth_tag]`
    ///
    /// # Errors
    ///
    /// Returns an error if encryption fails (rare, typically indicates system issues)
    fn encrypt(&self, data: Data) -> Result<Data>;

    /// Decrypts the given ciphertext.
    ///
    /// Verifies the authentication tag and decrypts the data. If the ciphertext
    /// has been tampered with, decryption will fail.
    ///
    /// # Arguments
    ///
    /// * `data` - The ciphertext to decrypt
    ///
    /// # Returns
    ///
    /// The decrypted plaintext
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The ciphertext is too small (missing nonce or tag)
    /// - The authentication tag verification fails (tampering detected)
    /// - The key is incorrect
    fn decrypt(&self, data: Data) -> Result<Data>;

    /// Returns the number of prefix bytes added to ciphertext (the nonce size).
    fn ciphertext_overhead_prefix(&self) -> usize;

    /// Returns the number of suffix bytes added to ciphertext (the auth tag size).
    fn ciphertext_overhead_suffix(&self) -> usize;
}

/// Error returned when creating a cipher with an invalid key size.
#[derive(Error, Display, Debug)]
#[display("Expected key size of {expected} bytes, but got {got} bytes")]
pub struct InvalidKeySizeError {
    /// The expected key size in bytes.
    pub expected: usize,
    /// The actual key size provided.
    pub got: usize,
}

/// A symmetric cipher with associated type-level constants.
///
/// This trait extends [`Cipher`] with compile-time constants for key size
/// and ciphertext overhead, enabling type-safe cipher selection and
/// compile-time verification of key sizes.
///
/// # Associated Constants
///
/// - `KEY_SIZE`: The required encryption key size in bytes
/// - `CIPHERTEXT_OVERHEAD_PREFIX`: Nonce size in bytes
/// - `CIPHERTEXT_OVERHEAD_SUFFIX`: Authentication tag size in bytes
pub trait CipherDef: Cipher + Sized {
    /// Creates a new cipher instance with the given encryption key.
    ///
    /// # Arguments
    ///
    /// * `key` - The encryption key (must be exactly `KEY_SIZE` bytes)
    ///
    /// # Returns
    ///
    /// A new cipher instance, or an error if the key size is invalid
    fn new(key: EncryptionKey) -> Result<Self, InvalidKeySizeError>;

    /// The required key size in bytes.
    const KEY_SIZE: usize;

    /// The number of prefix bytes added to ciphertext (nonce size).
    const CIPHERTEXT_OVERHEAD_PREFIX: usize;

    /// The number of suffix bytes added to ciphertext (authentication tag size).
    const CIPHERTEXT_OVERHEAD_SUFFIX: usize;
}

// TODO https://github.com/shadowsocks/crypto2 looks pretty fast, maybe we can use them for faster implementations?
// TODO Ring also looks pretty fast, see https://kerkour.com/rust-symmetric-encryption-aead-benchmark

mod backends;
mod key;

#[cfg(test)]
mod cipher_tests;

pub use key::EncryptionKey;

// export ciphers
mod aesgcm;
pub use aesgcm::{
    AeadAes128Gcm, AeadAes256Gcm, Aes128Gcm, Aes256Gcm, LibsodiumAes256GcmNonce12,
    OpensslAes128Gcm, OpensslAes256Gcm,
};
// TODO Is there an openssl implementation for XChaCha20Poly1305?
mod xchacha20poly1305;
pub use xchacha20poly1305::{AeadXChaCha20Poly1305, LibsodiumXChaCha20Poly1305, XChaCha20Poly1305};

// TODO Does DefaultNonceSize need to be public? This is weird. Or at least rename it to AesDefaultNonceSize
pub use self::aesgcm::DefaultNonceSize;

/// The maximum key size in bytes supported by any cipher in this module.
///
/// This constant is used by the scrypt key derivation function to always generate
/// keys of this size, even if the cipher requires fewer bytes. This ensures
/// backwards compatibility when changing ciphers.
///
/// Currently set to 56 bytes to accommodate all supported ciphers.
pub const MAX_KEY_SIZE: usize = 56;
const_assert!(AeadAes128Gcm::<DefaultNonceSize>::KEY_SIZE <= MAX_KEY_SIZE);
const_assert!(AeadAes256Gcm::<DefaultNonceSize>::KEY_SIZE <= MAX_KEY_SIZE);
const_assert!(Aes128Gcm::<DefaultNonceSize>::KEY_SIZE <= MAX_KEY_SIZE);
const_assert!(Aes256Gcm::<DefaultNonceSize>::KEY_SIZE <= MAX_KEY_SIZE);
const_assert!(LibsodiumAes256GcmNonce12::KEY_SIZE <= MAX_KEY_SIZE);
const_assert!(OpensslAes128Gcm::<DefaultNonceSize>::KEY_SIZE <= MAX_KEY_SIZE);
const_assert!(OpensslAes256Gcm::<DefaultNonceSize>::KEY_SIZE <= MAX_KEY_SIZE);
const_assert!(AeadXChaCha20Poly1305::KEY_SIZE <= MAX_KEY_SIZE);
const_assert!(LibsodiumXChaCha20Poly1305::KEY_SIZE <= MAX_KEY_SIZE);
const_assert!(XChaCha20Poly1305::KEY_SIZE <= MAX_KEY_SIZE);
