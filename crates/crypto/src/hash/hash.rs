use crate::hash::{Digest, Salt};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Hash {
    pub digest: Digest,
    pub salt: Salt,
}
