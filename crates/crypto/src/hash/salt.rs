use derive_more::From;
use rand::{Rng as _, rng};
use std::fmt::Debug;

pub const SALT_LEN: usize = 8;

#[derive(Clone, Copy, Eq, PartialEq, From)]
pub struct Salt([u8; SALT_LEN]);

impl Salt {
    pub fn new(bytes: [u8; SALT_LEN]) -> Self {
        Self(bytes)
    }

    pub fn get(&self) -> &[u8; SALT_LEN] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn from_hex(hex: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(hex)?;
        if bytes.len() != SALT_LEN {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut array = [0u8; SALT_LEN];
        array.copy_from_slice(&bytes);
        Ok(Self(array))
    }

    pub fn generate_random() -> Self {
        Self(rng().random())
    }
}

impl Debug for Salt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Salt").field(&hex::encode(self.0)).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let result = Salt::from_hex("ab");
        assert!(result.is_err());

        // Too long
        let too_long = "a".repeat(SALT_LEN * 2 + 2);
        let result = Salt::from_hex(&too_long);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_random() {
        let salt1 = Salt::generate_random();
        let salt2 = Salt::generate_random();

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
