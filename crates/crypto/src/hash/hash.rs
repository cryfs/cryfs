use crate::hash::{Digest, Salt};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Hash<const DIGEST_LEN: usize, const SALT_LEN: usize> {
    pub digest: Digest<DIGEST_LEN>,
    pub salt: Salt<SALT_LEN>,
}

impl<const _DIGEST_LEN: usize, const _SALT_LEN: usize> Hash<_DIGEST_LEN, _SALT_LEN> {
    pub const DIGEST_LEN: usize = _DIGEST_LEN;
    pub const SALT_LEN: usize = _SALT_LEN;
}
