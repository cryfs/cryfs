mod backends;
mod digest;
mod hash;
mod salt;

pub use digest::Digest;
pub use hash::Hash;
pub use salt::Salt;

// TODO Consider hardening by (1) increasing salt size to a full hash block and (2) switching to SHA3

#[cfg(test)]
mod tests;

pub trait HashAlgorithmDef {
    const DIGEST_LEN: usize;
    const SALT_LEN: usize;
}

pub trait HashAlgorithm<const DIGEST_LEN: usize, const SALT_LEN: usize> {
    fn hash(data: &[u8], salt: Salt<SALT_LEN>) -> Hash<DIGEST_LEN, SALT_LEN>;
}

pub use backends::{LibsodiumSha512, OpensslSha512, Sha2Sha512};

pub type Sha512 = backends::OpensslSha512;
