//! Scrypt key derivation function.
//!
//! Scrypt is a memory-hard password-based key derivation function designed to be
//! expensive to perform hardware attacks (ASICs and GPUs). It is particularly
//! well-suited for deriving encryption keys from passwords.
//!
//! # Parameters
//!
//! Scrypt has three main parameters that control its cost:
//!
//! - **N (log_n)**: CPU/memory cost parameter (as a power of 2). Higher values
//!   require more memory and time.
//! - **r**: Block size parameter. Affects memory and CPU usage.
//! - **p**: Parallelization parameter. Higher values allow more parallelism.
//!
//! # Preset Settings
//!
//! The module provides several preset configurations via [`ScryptSettings`]:
//!
//! - [`ScryptSettings::PARANOID`]: Maximum security (32GB memory, very slow)
//! - [`ScryptSettings::DEFAULT`]: Good balance of security and usability (1GB memory)
//! - [`ScryptSettings::LOW_MEMORY`]: For memory-constrained environments (500MB memory)
//! - [`ScryptSettings::TEST`]: Fast settings for testing only (128KB memory)
//!
//! # Example
//!
//! ```
//! use cryfs_crypto::kdf::scrypt::{Scrypt, ScryptSettings};
//! use cryfs_crypto::kdf::PasswordBasedKDF;
//!
//! // Generate parameters with test settings (fast, for examples only)
//! let params = Scrypt::generate_parameters(&ScryptSettings::TEST).unwrap();
//!
//! // Derive a 32-byte encryption key
//! let key = Scrypt::derive_key(32, "my_secure_password", &params);
//! assert_eq!(key.num_bytes(), 32);
//! ```

mod params;
pub use params::ScryptParams;

mod settings;
pub use settings::ScryptSettings;

pub mod backends;

/// Default scrypt implementation using the pure Rust `scrypt` crate.
pub type Scrypt = backends::scrypt::ScryptScrypt;

#[cfg(test)]
mod tests;
