use sha2::Digest as _;

use crate::hash::{Digest, Hash, HashAlgorithm, HashAlgorithmDef, Salt};

pub struct Sha2Sha512;
impl HashAlgorithmDef for Sha2Sha512 {
    const DIGEST_LEN: usize = 64;
    const SALT_LEN: usize = 8;
}
impl HashAlgorithm<{ Sha2Sha512::DIGEST_LEN }, { Sha2Sha512::SALT_LEN }> for Sha2Sha512 {
    fn hash(
        data: &[u8],
        salt: Salt<{ Sha2Sha512::SALT_LEN }>,
    ) -> Hash<{ Sha2Sha512::DIGEST_LEN }, { Sha2Sha512::SALT_LEN }> {
        let mut hasher = sha2::Sha512::new();
        hasher.update(salt.get());
        hasher.update(data);
        let digest = Digest::new(hasher.finalize().into());

        Hash { digest, salt }
    }
}
