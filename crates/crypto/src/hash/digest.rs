use std::fmt::Debug;

pub const DIGEST_LEN: usize = 64;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Digest([u8; DIGEST_LEN]);

impl Digest {
    pub fn new(bytes: [u8; DIGEST_LEN]) -> Self {
        Self(bytes)
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn from_hex(hex: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(hex)?;
        if bytes.len() != DIGEST_LEN {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut array = [0u8; DIGEST_LEN];
        array.copy_from_slice(&bytes);
        Ok(Self(array))
    }
}

impl Debug for Digest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Digest").field(&hex::encode(self.0)).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_hex_and_from_hex() {
        // Create a test digest
        let original = Digest([42u8; DIGEST_LEN]);

        // Convert to hex and back
        let hex = original.to_hex();
        let restored = Digest::from_hex(&hex).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn test_from_hex_invalid_length() {
        // Too short
        let result = Digest::from_hex("abcd");
        assert!(result.is_err());

        // Too long
        let too_long = "a".repeat(DIGEST_LEN * 2 + 2);
        let result = Digest::from_hex(&too_long);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_hex_invalid_chars() {
        let invalid = "z".repeat(DIGEST_LEN * 2);
        let result = Digest::from_hex(&invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_debug_format() {
        let digest = Digest::new([0xab; DIGEST_LEN]);
        let debug_str = format!("{:?}", digest);

        assert!(debug_str.contains("Digest"));
        assert!(debug_str.contains(&digest.to_hex()));
    }
}
