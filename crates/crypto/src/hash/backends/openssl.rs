use crate::hash::{Digest, Hash, HashAlgorithm, HashAlgorithmDef, Salt};

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
        let mut salted_data = vec![0; data.len() + salt.get().len()];
        salted_data[..salt.get().len()].copy_from_slice(salt.get());
        salted_data[salt.get().len()..].copy_from_slice(data);
        let digest = Digest::new(openssl::sha::sha512(&salted_data));

        Hash { digest, salt }
    }
}
