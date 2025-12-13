use rand::{Rng, rng};
use std::fmt::Debug;

const DIGEST_LEN: usize = 64;
const SALT_LEN: usize = 8;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Digest([u8; DIGEST_LEN]);

impl Digest {
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

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Salt([u8; SALT_LEN]);

impl Salt {
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

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Hash {
    pub digest: Digest,
    pub salt: Salt,
}

pub fn hash(data: &[u8], salt: Salt) -> Hash {
    let mut salted_data = vec![0; data.len() + salt.0.len()];
    salted_data[..salt.0.len()].copy_from_slice(&salt.0);
    salted_data[salt.0.len()..].copy_from_slice(data);
    let digest = Digest(openssl::sha::sha512(&salted_data));

    Hash { digest, salt }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digest_to_hex_and_from_hex() {
        // Create a test digest
        let original = Digest([42u8; DIGEST_LEN]);

        // Convert to hex and back
        let hex = original.to_hex();
        let restored = Digest::from_hex(&hex).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn test_digest_from_hex_invalid_length() {
        // Too short
        let result = Digest::from_hex("abcd");
        assert!(result.is_err());

        // Too long
        let too_long = "a".repeat(DIGEST_LEN * 2 + 2);
        let result = Digest::from_hex(&too_long);
        assert!(result.is_err());
    }

    #[test]
    fn test_digest_from_hex_invalid_chars() {
        let invalid = "z".repeat(DIGEST_LEN * 2);
        let result = Digest::from_hex(&invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_salt_to_hex_and_from_hex() {
        // Create a test salt
        let original = Salt([123u8; SALT_LEN]);

        // Convert to hex and back
        let hex = original.to_hex();
        let restored = Salt::from_hex(&hex).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn test_salt_from_hex_invalid_length() {
        // Too short
        let result = Salt::from_hex("ab");
        assert!(result.is_err());

        // Too long
        let too_long = "a".repeat(SALT_LEN * 2 + 2);
        let result = Salt::from_hex(&too_long);
        assert!(result.is_err());
    }

    #[test]
    fn test_salt_generate_random() {
        let salt1 = Salt::generate_random();
        let salt2 = Salt::generate_random();

        // Random salts should be different (with very high probability)
        assert_ne!(salt1, salt2);
    }

    #[test]
    fn test_hash_deterministic_with_same_salt() {
        let data = b"test data";
        let salt = Salt([1, 2, 3, 4, 5, 6, 7, 8]);

        let hash1 = hash(data, salt);
        let hash2 = hash(data, salt);

        assert_eq!(hash1.digest, hash2.digest);
        assert_eq!(hash1.salt, hash2.salt);
    }

    #[test]
    fn test_hash_different_with_different_salts() {
        let data = b"test data";
        let salt1 = Salt([1, 2, 3, 4, 5, 6, 7, 8]);
        let salt2 = Salt([8, 7, 6, 5, 4, 3, 2, 1]);

        let hash1 = hash(data, salt1);
        let hash2 = hash(data, salt2);

        assert_ne!(hash1.digest, hash2.digest);
        assert_eq!(hash1.salt, salt1);
        assert_eq!(hash2.salt, salt2);
    }

    #[test]
    fn test_hash_different_with_different_data() {
        let salt = Salt([1, 2, 3, 4, 5, 6, 7, 8]);

        let hash1 = hash(b"data1", salt);
        let hash2 = hash(b"data2", salt);

        assert_ne!(hash1.digest, hash2.digest);
        assert_eq!(hash1.salt, salt);
        assert_eq!(hash2.salt, salt);
    }

    #[test]
    fn test_hash_empty_data() {
        let salt = Salt::generate_random();
        let hash_result = hash(b"", salt);

        assert_eq!(hash_result.salt, salt);
        // Should produce a valid digest even for empty data
        assert_ne!(hash_result.digest.to_hex(), "");
    }

    #[test]
    fn test_digest_debug_format() {
        let digest = Digest([0xab; DIGEST_LEN]);
        let debug_str = format!("{:?}", digest);

        assert!(debug_str.contains("Digest"));
        assert!(debug_str.contains(&digest.to_hex()));
    }

    #[test]
    fn test_salt_debug_format() {
        let salt = Salt([0xcd; SALT_LEN]);
        let debug_str = format!("{:?}", salt);

        assert!(debug_str.contains("Salt"));
        assert!(debug_str.contains(&salt.to_hex()));
    }

    #[test]
    fn test_backwards_compatibility() {
        // This test ensures the hash function output doesn't change between versions
        // Use concrete input and salt values and verify exact output
        let data = b"Hello, CryFS!";
        let salt = Salt([0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]);

        let hash_result = hash(data, salt);

        // Verify the salt is preserved
        assert_eq!(hash_result.salt, salt);

        // Verify the exact digest value (SHA-512 of salt + data)
        let expected_digest = "a3faef145ba1c9b66b8b89f685827e08c465704b1d12242acf45a4e0d4275f1cc3d07a72e1e1804993a15329776b55a2450f123d9e2e0f5c6f108891c977c9a0";
        assert_eq!(hash_result.digest.to_hex(), expected_digest);
    }

    #[test]
    fn test_backwards_compatibility_empty_data() {
        // This test ensures the hash function output doesn't change for empty input
        let data = b"";
        let salt = Salt([0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10]);

        let hash_result = hash(data, salt);

        // Verify the salt is preserved
        assert_eq!(hash_result.salt, salt);

        // Verify the exact digest value (SHA-512 of salt + empty data)
        let expected_digest = "245a64d8d9f7be46dcfabcfb0cbfa48d78077f18f4c2408e0f36517bdbb94f0f675c6c089d68e24862f9d238636a28adeaf022ae23b7db282455da537215d734";
        assert_eq!(hash_result.digest.to_hex(), expected_digest);
    }
}
