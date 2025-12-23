use crate::hash::{Digest, Salt};

/// The result of a hash operation, containing both the digest and the salt used.
///
/// A `Hash` bundles together the computed digest and the salt that was used
/// during hashing. This allows the hash to be verified later by re-hashing
/// with the same salt.
///
/// # Type Parameters
///
/// - `DIGEST_LEN`: The length of the hash digest in bytes
/// - `SALT_LEN`: The length of the salt in bytes
///
/// # Example
///
/// ```
/// use cryfs_crypto::hash::{Sha512, Salt, HashAlgorithm};
///
/// let salt = Salt::generate_random();
/// let hash = Sha512::hash(b"data to hash", salt);
///
/// // Store both digest and salt for later verification
/// let stored_digest = hash.digest;
/// let stored_salt = hash.salt;
///
/// // Later, verify by re-hashing with the same salt
/// let verification_hash = Sha512::hash(b"data to hash", stored_salt);
/// assert_eq!(verification_hash.digest, stored_digest);
/// ```
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Hash<const DIGEST_LEN: usize, const SALT_LEN: usize> {
    /// The computed hash digest.
    pub digest: Digest<DIGEST_LEN>,
    /// The salt that was used during hashing.
    pub salt: Salt<SALT_LEN>,
}

impl<const _DIGEST_LEN: usize, const _SALT_LEN: usize> Hash<_DIGEST_LEN, _SALT_LEN> {
    /// The length of the hash digest in bytes.
    pub const DIGEST_LEN: usize = _DIGEST_LEN;
    /// The length of the salt in bytes.
    pub const SALT_LEN: usize = _SALT_LEN;
}
