use rand::{Rng, rng};
use std::fmt::Debug;

const DIGEST_LEN: usize = 64;
const SALT_LEN: usize = 8;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Digest([u8; DIGEST_LEN]);

impl Digest {
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn from_hex(hex: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(hex)?;
        if bytes.len() != DIGEST_LEN {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut array = [0u8; DIGEST_LEN];
        array.copy_from_slice(&bytes);
        Ok(Self(array))
    }
}

impl Debug for Digest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Digest").field(&hex::encode(self.0)).finish()
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Salt([u8; SALT_LEN]);

impl Salt {
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn from_hex(hex: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(hex)?;
        if bytes.len() != SALT_LEN {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut array = [0u8; SALT_LEN];
        array.copy_from_slice(&bytes);
        Ok(Self(array))
    }

    pub fn generate_random() -> Self {
        Self(rng().random())
    }
}

impl Debug for Salt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Salt").field(&hex::encode(self.0)).finish()
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Hash {
    pub digest: Digest,
    pub salt: Salt,
}

pub fn hash(data: &[u8], salt: Salt) -> Hash {
    let mut salted_data = vec![0; data.len() + salt.0.len()];
    salted_data[..salt.0.len()].copy_from_slice(&salt.0);
    salted_data[salt.0.len()..].copy_from_slice(data);
    let digest = Digest(openssl::sha::sha512(&salted_data));

    Hash { digest, salt }
}

// TODO Test
