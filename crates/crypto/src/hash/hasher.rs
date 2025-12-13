use crate::hash::{Digest, Hash, Salt};

pub fn hash(data: &[u8], salt: Salt) -> Hash {
    let mut salted_data = vec![0; data.len() + salt.get().len()];
    salted_data[..salt.get().len()].copy_from_slice(salt.get());
    salted_data[salt.get().len()..].copy_from_slice(data);
    let digest = Digest::new(openssl::sha::sha512(&salted_data));

    Hash { digest, salt }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_deterministic_with_same_salt() {
        let data = b"test data";
        let salt = Salt::new([1, 2, 3, 4, 5, 6, 7, 8]);

        let hash1 = hash(data, salt);
        let hash2 = hash(data, salt);

        assert_eq!(hash1.digest, hash2.digest);
        assert_eq!(hash1.salt, hash2.salt);
    }

    #[test]
    fn test_hash_different_with_different_salts() {
        let data = b"test data";
        let salt1 = Salt::new([1, 2, 3, 4, 5, 6, 7, 8]);
        let salt2 = Salt::new([8, 7, 6, 5, 4, 3, 2, 1]);

        let hash1 = hash(data, salt1);
        let hash2 = hash(data, salt2);

        assert_ne!(hash1.digest, hash2.digest);
        assert_eq!(hash1.salt, salt1);
        assert_eq!(hash2.salt, salt2);
    }

    #[test]
    fn test_hash_different_with_different_data() {
        let salt = Salt::new([1, 2, 3, 4, 5, 6, 7, 8]);

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
    fn test_backwards_compatibility() {
        // This test ensures the hash function output doesn't change between versions
        // Use concrete input and salt values and verify exact output
        let data = b"Hello, CryFS!";
        let salt = Salt::new([0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]);

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
        let salt = Salt::new([0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10]);

        let hash_result = hash(data, salt);

        // Verify the salt is preserved
        assert_eq!(hash_result.salt, salt);

        // Verify the exact digest value (SHA-512 of salt + empty data)
        let expected_digest = "245a64d8d9f7be46dcfabcfb0cbfa48d78077f18f4c2408e0f36517bdbb94f0f675c6c089d68e24862f9d238636a28adeaf022ae23b7db282455da537215d734";
        assert_eq!(hash_result.digest.to_hex(), expected_digest);
    }
}
