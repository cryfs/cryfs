//! Key derivation functions (KDFs) for password-based encryption.
//!
//! This module provides key derivation functions that convert passwords into
//! cryptographic keys. KDFs are essential for password-based encryption as they:
//!
//! - Derive keys of the required length from passwords of any length
//! - Add computational cost to slow down brute-force attacks
//! - Use salt to prevent rainbow table attacks
//!
//! # Available KDFs
//!
//! - [`scrypt`]: Memory-hard KDF recommended for password hashing
//!
//! # Example
//!
//! ```
//! use cryfs_crypto::kdf::scrypt::{Scrypt, ScryptSettings};
//! use cryfs_crypto::kdf::PasswordBasedKDF;
//!
//! // Generate parameters for a new key derivation
//! let params = Scrypt::generate_parameters(&ScryptSettings::TEST).unwrap();
//!
//! // Derive a 32-byte key from a password
//! let key = Scrypt::derive_key(32, "my_password", &params);
//!
//! // The same password and parameters always produce the same key
//! let key2 = Scrypt::derive_key(32, "my_password", &params);
//! assert_eq!(key, key2);
//! ```

use super::symmetric::EncryptionKey;
use anyhow::Result;
use std::fmt::Debug;

/// Serializable parameters for a key derivation function.
///
/// KDF parameters contain the salt and algorithm-specific settings needed
/// to reproduce the same derived key. They must be stored alongside encrypted
/// data to allow decryption.
pub trait KDFParameters: Sized + Debug {
    /// Serializes the parameters to a byte vector.
    ///
    /// The serialized format must be deterministic and compatible with
    /// [`deserialize`](KDFParameters::deserialize).
    fn serialize(&self) -> Vec<u8>;

    /// Deserializes parameters from a byte slice.
    ///
    /// # Errors
    ///
    /// Returns an error if the serialized data is invalid or corrupted.
    fn deserialize(serialized: &[u8]) -> Result<Self>;
}

/// A password-based key derivation function.
///
/// This trait defines the interface for deriving cryptographic keys from passwords.
/// Implementations should be memory-hard and computationally expensive to resist
/// brute-force attacks.
///
/// # Security
///
/// - Use appropriate settings for your security requirements (see [`scrypt::ScryptSettings`])
/// - Store parameters alongside encrypted data for decryption
/// - Never reuse parameters across different passwords
pub trait PasswordBasedKDF {
    /// Configuration for generating new parameters (e.g., memory cost)
    type Settings;
    /// The actual parameters used for key derivation (includes salt)
    type Parameters: KDFParameters;

    /// Derives an encryption key from a password.
    ///
    /// # Arguments
    ///
    /// * `key_size` - The desired key size in bytes
    /// * `password` - The password to derive from
    /// * `kdf_parameters` - The KDF parameters (including salt)
    ///
    /// # Returns
    ///
    /// An encryption key of the requested size. The same password and parameters
    /// always produce the same key.
    fn derive_key(
        key_size: usize,
        password: &str,
        kdf_parameters: &Self::Parameters,
    ) -> EncryptionKey;

    /// Generates a new set of KDF parameters based on the given settings.
    ///
    /// This generates a random salt and creates parameters suitable for
    /// encrypting new data. The generated parameters cannot be used to
    /// decrypt existing data encrypted with different parameters.
    ///
    /// # Arguments
    ///
    /// * `settings` - The KDF settings (memory cost, iteration count, etc.)
    ///
    /// # Returns
    ///
    /// New KDF parameters with a random salt, or an error if generation fails.
    fn generate_parameters(settings: &Self::Settings) -> Result<Self::Parameters>;
}

pub mod scrypt;
