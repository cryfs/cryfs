use crate::hash::{Digest, Hash, HashAlgorithm, HashAlgorithmDef, Salt};

pub struct LibsodiumSha512;
impl HashAlgorithmDef for LibsodiumSha512 {
    const DIGEST_LEN: usize = 64;
    const SALT_LEN: usize = 8;
}
impl HashAlgorithm<{ LibsodiumSha512::DIGEST_LEN }, { LibsodiumSha512::SALT_LEN }>
    for LibsodiumSha512
{
    fn hash(
        data: &[u8],
        salt: Salt<{ LibsodiumSha512::SALT_LEN }>,
    ) -> Hash<{ LibsodiumSha512::DIGEST_LEN }, { LibsodiumSha512::SALT_LEN }> {
        let mut hasher = sodiumoxide::crypto::hash::sha512::State::new();
        hasher.update(salt.get());
        hasher.update(data);
        let digest = Digest::new(hasher.finalize().0);

        Hash { digest, salt }
    }
}
