mod backends;
mod digest;
mod hash;
mod salt;

pub use digest::Digest;
pub use hash::Hash;
pub use salt::Salt;

#[cfg(test)]
mod tests;

pub trait HashAlgorithmDef {
    const DIGEST_LEN: usize;
    const SALT_LEN: usize;
}

pub trait HashAlgorithm<const DIGEST_LEN: usize, const SALT_LEN: usize> {
    fn hash(data: &[u8], salt: Salt<SALT_LEN>) -> Hash<DIGEST_LEN, SALT_LEN>;
}

pub type Sha512 = backends::OpensslSha512;
