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
        let mut salted_data = vec![0; data.len() + salt.get().len()];
        salted_data[..salt.get().len()].copy_from_slice(salt.get());
        salted_data[salt.get().len()..].copy_from_slice(data);
        let digest = Digest::new(sodiumoxide::crypto::hash::sha512::hash(&salted_data).0);

        Hash { digest, salt }
    }
}
