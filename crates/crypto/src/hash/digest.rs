use std::fmt::Debug;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Digest<const DIGEST_LEN: usize>([u8; DIGEST_LEN]);

impl<const DIGEST_LEN: usize> Digest<DIGEST_LEN> {
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

impl<const DIGEST_LEN: usize> Debug for Digest<DIGEST_LEN> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Digest").field(&hex::encode(self.0)).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST_LEN: usize = 64;

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
        let result = Digest::<DIGEST_LEN>::from_hex("abcd");
        assert!(result.is_err());

        // Too long
        let too_long = "a".repeat(DIGEST_LEN * 2 + 2);
        let result = Digest::<DIGEST_LEN>::from_hex(&too_long);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_hex_invalid_chars() {
        let invalid = "z".repeat(DIGEST_LEN * 2);
        let result = Digest::<DIGEST_LEN>::from_hex(&invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_debug_format() {
        let digest = Digest::new([0xab; DIGEST_LEN]);
        let debug_str = format!("{:?}", digest);

        assert!(debug_str.contains("Digest"));
        assert!(debug_str.contains(&digest.to_hex()));
    }

    #[test]
    fn test_hex_format() {
        // Test with specific byte pattern to verify exact hex encoding
        let bytes = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54,
            0x32, 0x10, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb,
            0xcc, 0xdd, 0xee, 0xff, 0x0f, 0x1e, 0x2d, 0x3c, 0x4b, 0x5a, 0x69, 0x78, 0x87, 0x96,
            0xa5, 0xb4, 0xc3, 0xd2, 0xe1, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
            0x0e, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10,
        ];
        let digest = Digest::new(bytes);

        // Verify to_hex produces the expected hex string
        let hex = digest.to_hex();
        assert_eq!(
            hex,
            "0123456789abcdeffedcba987654321000112233445566778899aabbccddeeff0f1e2d3c4b5a69788796a5b4c3d2e1f0123456789abcdef00edcba9876543210"
        );

        // Verify from_hex can parse it back
        let restored = Digest::from_hex(&hex).unwrap();
        assert_eq!(digest, restored);
    }
}
