//! Cryptographic primitives for CryFS.
//!
//! This crate provides the cryptographic building blocks used by CryFS to encrypt
//! filesystems. It includes symmetric encryption ciphers, cryptographic hash functions,
//! and key derivation functions (KDFs).
//!
//! # Modules
//!
//! - [`symmetric`]: Symmetric encryption ciphers (AES-GCM, XChaCha20-Poly1305)
//! - [`hash`]: Cryptographic hash functions (SHA-512)
//! - [`kdf`]: Key derivation functions for password-based encryption (scrypt)
//!
//! # Backend Selection
//!
//! Each cryptographic primitive has multiple backend implementations available:
//!
//! - **OpenSSL**: Generally the fastest option, uses the system's OpenSSL library
//! - **Pure Rust crates**: Portable implementations that don't require external libraries
//! - **libsodium**: High-quality cryptographic library with constant-time implementations
//!
//! Default type aliases (e.g., [`hash::Sha512`], [`symmetric::Aes256Gcm`]) point to the
//! recommended backend for each algorithm. You can also explicitly choose a specific
//! backend by using the full type name (e.g., [`symmetric::OpensslAes256Gcm`]).
//!
//! # Security Considerations
//!
//! - All encryption uses authenticated encryption (AEAD) to provide both confidentiality
//!   and integrity protection
//! - Key derivation uses memory-hard functions (scrypt) to resist brute-force attacks
//! - Encryption keys are stored in protected memory that is zeroed on drop
//! - Nonces are randomly generated for each encryption operation
//! - This crate uses `#![forbid(unsafe_code)]` to prevent memory safety issues
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
//! let message = b"Hello, CryFS!";
//! let mut plaintext = Data::allocate(
//!     Aes256Gcm::<DefaultNonceSize>::CIPHERTEXT_OVERHEAD_PREFIX,
//!     message.len(),
//!     Aes256Gcm::<DefaultNonceSize>::CIPHERTEXT_OVERHEAD_SUFFIX,
//! );
//! plaintext.as_mut().copy_from_slice(message);
//!
//! let ciphertext = cipher.encrypt(plaintext).expect("encryption succeeded");
//!
//! // Decrypt the data
//! let decrypted = cipher.decrypt(ciphertext).expect("decryption succeeded");
//! assert_eq!(decrypted.as_ref(), b"Hello, CryFS!");
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod hash;
pub mod kdf;
pub mod symmetric;

cryfs_version::assert_cargo_version_equals_git_version!();
