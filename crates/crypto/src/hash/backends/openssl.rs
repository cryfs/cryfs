//! SHA-512 implementation using OpenSSL.

use crate::hash::{Digest, Hash, HashAlgorithm, HashAlgorithmDef, Salt};

/// SHA-512 hash algorithm using OpenSSL.
///
/// This implementation uses the system's OpenSSL library and is generally
/// the fastest option on most platforms. It benefits from hardware acceleration
/// when available.
///
/// This is the default SHA-512 implementation used by [`crate::hash::Sha512`].
///
/// # Constants
///
/// - Digest length: 64 bytes (512 bits)
/// - Salt length: 8 bytes
pub struct OpensslSha512;
impl HashAlgorithmDef for OpensslSha512 {
    const DIGEST_LEN: usize = 64;
    const SALT_LEN: usize = 8;
}
impl HashAlgorithm<{ OpensslSha512::DIGEST_LEN }, { OpensslSha512::SALT_LEN }> for OpensslSha512 {
    fn hash(
        data: &[u8],
        salt: Salt<{ OpensslSha512::SALT_LEN }>,
    ) -> Hash<{ OpensslSha512::DIGEST_LEN }, { OpensslSha512::SALT_LEN }> {
        let mut hasher = openssl::sha::Sha512::new();
        hasher.update(salt.get());
        hasher.update(data);
        let digest = Digest::new(hasher.finish());

        Hash { digest, salt }
    }
}
