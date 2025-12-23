//! Scrypt parameter storage and serialization.

use anyhow::{Result, ensure};
use binrw::{BinRead, BinWrite, binrw, helpers::until_eof};
use rand::{RngCore, rng};
use std::fmt::Debug;
use std::io::Cursor;

use super::super::KDFParameters;
use super::ScryptSettings;

/// Parameters for the scrypt key derivation function.
///
/// `ScryptParams` contains all the information needed to derive the same key
/// from a password: the cost parameters (N, r, p) and the random salt. These
/// parameters must be stored alongside encrypted data to enable decryption.
///
/// # Serialization Format
///
/// The parameters are serialized in little-endian binary format:
/// - `n` (8 bytes): CPU/memory cost as `2^log_n`
/// - `r` (4 bytes): Block size parameter
/// - `p` (4 bytes): Parallelization parameter
/// - `salt` (remaining bytes): Random salt
///
/// # Example
///
/// ```
/// use cryfs_crypto::kdf::scrypt::{ScryptParams, ScryptSettings};
/// use cryfs_crypto::kdf::KDFParameters;
///
/// // Generate new parameters
/// let params = ScryptParams::generate(&ScryptSettings::TEST).unwrap();
///
/// // Serialize for storage
/// let serialized = params.serialize();
///
/// // Later, deserialize to derive the same key
/// let restored = ScryptParams::deserialize(&serialized).unwrap();
/// assert_eq!(params.log_n(), restored.log_n());
/// ```
#[derive(Clone, PartialEq, Eq)]
#[binrw]
#[brw(little)]
pub struct ScryptParams {
    #[br(try_map = |x: u64| parse_log_n(x))]
    #[bw(map = |x: &u8| write_log_n(*x))]
    log_n: u8,

    r: u32,

    p: u32,

    #[br(parse_with = until_eof)]
    salt: Vec<u8>,
}

impl ScryptParams {
    /// Generates new scrypt parameters with a random salt.
    ///
    /// # Arguments
    ///
    /// * `settings` - The cost settings (log_n, r, p, salt_len)
    ///
    /// # Returns
    ///
    /// New parameters with a randomly generated salt, or an error if
    /// the settings are invalid (e.g., log_n >= 64).
    pub fn generate(settings: &ScryptSettings) -> Result<Self> {
        ensure!(
            settings.log_n < 64,
            "Scrypt parameter log_n is {} but must be smaller than 64",
            settings.log_n,
        );
        let mut salt = vec![0; settings.salt_len];
        rng().fill_bytes(&mut salt);
        Ok(Self {
            log_n: settings.log_n,
            r: settings.r,
            p: settings.p,
            salt,
        })
    }

    /// Returns the CPU/memory cost parameter as log2(N).
    ///
    /// The actual N value used by scrypt is `2^log_n`.
    pub fn log_n(&self) -> u8 {
        self.log_n
    }

    /// Returns the block size parameter (r).
    pub fn r(&self) -> u32 {
        self.r
    }

    /// Returns the parallelization parameter (p).
    pub fn p(&self) -> u32 {
        self.p
    }

    /// Returns the random salt used for key derivation.
    pub fn salt(&self) -> &[u8] {
        &self.salt
    }
}

fn write_log_n(log_n: u8) -> u64 {
    assert!(
        log_n < 64,
        "Scrypt parameter log_n is {log_n} but must be smaller than 64",
    );
    1 << log_n
}

fn parse_log_n(n: u64) -> Result<u8> {
    let log_n: u8 =
        u8::try_from(n.ilog2()).expect("log2(64 bit value) cannot be larger than u8::MAX");
    ensure!(
        write_log_n(log_n) == n,
        "Scrypt parameter n={n} must be a power of 2 but isn't"
    );
    Ok(log_n)
}

impl KDFParameters for ScryptParams {
    fn serialize(&self) -> Vec<u8> {
        let mut result = Cursor::new(vec![]);
        self.write(&mut result)
            .expect("Writing can't fail because our serializer shouldn't throw anywhere");
        result.into_inner()
    }

    fn deserialize(serialized: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(serialized);
        let result = Self::read(&mut cursor)?;
        Ok(result)
    }
}

impl Debug for ScryptParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScryptParams")
            .field("log_n", &self.log_n)
            .field("r", &self.r)
            .field("p", &self.p)
            .field("salt", &hex::encode(&self.salt))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate() {
        let params = ScryptParams::generate(&ScryptSettings {
            log_n: 10,
            r: 8,
            p: 1,
            salt_len: 53,
        })
        .unwrap();
        assert_eq!(params.log_n, 10);
        assert_eq!(params.r, 8);
        assert_eq!(params.p, 1);
        assert_eq!(params.salt.len(), 53);
    }

    #[test]
    fn serialize() {
        let params = ScryptParams::generate(&ScryptSettings {
            log_n: 10,
            r: 8,
            p: 1,
            salt_len: 53,
        })
        .unwrap();
        let serialized = params.serialize();
        let deserialized = ScryptParams::deserialize(&serialized).unwrap();
        assert_eq!(deserialized.log_n, 10);
        assert_eq!(deserialized.r, 8);
        assert_eq!(deserialized.p, 1);
        assert_eq!(deserialized.salt, params.salt);
    }
}
