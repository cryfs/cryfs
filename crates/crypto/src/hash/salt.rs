use derive_more::From;
use rand::{Rng as _, rng};
use std::fmt::Debug;

/// A cryptographic salt for hash operations.
///
/// A salt is random data that is used as an additional input to a hash function.
/// Using a unique salt for each hash operation prevents rainbow table attacks
/// and ensures that identical data produces different hashes.
///
/// # Type Parameters
///
/// - `SALT_LEN`: The length of the salt in bytes
///
/// # Security
///
/// - Salts should be generated randomly using [`Salt::generate_random()`]
/// - Each hash operation should use a unique salt
/// - Salts do not need to be kept secret (unlike keys)
///
/// # Example
///
/// ```
/// use cryfs_crypto::hash::Salt;
///
/// // Generate a random 8-byte salt
/// let salt = Salt::<8>::generate_random();
///
/// // Convert to hex for storage
/// let hex_str = salt.to_hex();
///
/// // Parse back from hex
/// let restored = Salt::<8>::from_hex(&hex_str).unwrap();
/// assert_eq!(salt, restored);
/// ```
#[derive(Clone, Copy, Eq, PartialEq, From)]
pub struct Salt<const SALT_LEN: usize>([u8; SALT_LEN]);

impl<const SALT_LEN: usize> Salt<SALT_LEN> {
    /// Creates a new salt from raw bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The raw salt bytes
    #[inline]
    pub fn new(bytes: [u8; SALT_LEN]) -> Self {
        Self(bytes)
    }

    /// Returns a reference to the raw salt bytes.
    #[inline]
    pub fn get(&self) -> &[u8; SALT_LEN] {
        &self.0
    }

    /// Encodes the salt as a lowercase hexadecimal string.
    ///
    /// # Returns
    ///
    /// A string containing the hex-encoded salt. The length will be
    /// `SALT_LEN * 2` characters.
    #[inline]
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Parses a salt from a hexadecimal string.
    ///
    /// # Arguments
    ///
    /// * `hex` - A hex string representing the salt (case-insensitive)
    ///
    /// # Returns
    ///
    /// - `Ok(Salt)` if parsing succeeds
    /// - `Err(FromHexError)` if the string is not valid hex or has wrong length
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The string contains non-hexadecimal characters
    /// - The string length is not exactly `SALT_LEN * 2`
    pub fn from_hex(hex: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(hex)?;
        if bytes.len() != SALT_LEN {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut array = [0u8; SALT_LEN];
        array.copy_from_slice(&bytes);
        Ok(Self(array))
    }

    /// Generates a cryptographically random salt.
    ///
    /// This method uses a cryptographically secure random number generator
    /// to produce a unique salt. Each call returns a different value.
    ///
    /// # Example
    ///
    /// ```
    /// use cryfs_crypto::hash::Salt;
    ///
    /// let salt1 = Salt::<8>::generate_random();
    /// let salt2 = Salt::<8>::generate_random();
    /// // Salts will be different (with overwhelming probability)
    /// assert_ne!(salt1, salt2);
    /// ```
    #[inline]
    pub fn generate_random() -> Self {
        Self(rng().random())
    }
}

impl<const SALT_LEN: usize> Debug for Salt<SALT_LEN> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Salt").field(&hex::encode(self.0)).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SALT_LEN: usize = 8;

    #[test]
    fn test_to_hex_and_from_hex() {
        // Create a test salt
        let original = Salt([123u8; SALT_LEN]);

        // Convert to hex and back
        let hex = original.to_hex();
        let restored = Salt::from_hex(&hex).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn test_from_hex_invalid_length() {
        // Too short
        let result = Salt::<SALT_LEN>::from_hex("ab");
        assert!(result.is_err());

        // Too long
        let too_long = "a".repeat(SALT_LEN * 2 + 2);
        let result = Salt::<SALT_LEN>::from_hex(&too_long);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_random() {
        let salt1 = Salt::<SALT_LEN>::generate_random();
        let salt2 = Salt::<SALT_LEN>::generate_random();

        // Random salts should be different (with very high probability)
        assert_ne!(salt1, salt2);
    }

    #[test]
    fn test_debug_format() {
        let salt = Salt::new([0xcd; SALT_LEN]);
        let debug_str = format!("{:?}", salt);

        assert!(debug_str.contains("Salt"));
        assert!(debug_str.contains(&salt.to_hex()));
    }

    #[test]
    fn test_hex_format() {
        // Test with specific byte pattern to verify exact hex encoding
        let bytes = [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
        let salt = Salt::new(bytes);

        // Verify to_hex produces the expected hex string
        let hex = salt.to_hex();
        assert_eq!(hex, "0123456789abcdef");

        // Verify from_hex can parse it back
        let restored = Salt::from_hex(&hex).unwrap();
        assert_eq!(salt, restored);
    }
}
