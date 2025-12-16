use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FilesystemId([u8; 16]);

impl FilesystemId {
    pub fn new_random() -> Self {
        Self(rand::random())
    }

    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    pub fn to_bytes(&self) -> [u8; 16] {
        self.0
    }

    pub fn from_hex(hex: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(hex)?;
        if bytes.len() != 16 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut array = [0u8; 16];
        array.copy_from_slice(&bytes);
        Ok(Self(array))
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl Debug for FilesystemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FilesystemId")
            .field(&hex::encode(self.0))
            .finish()
    }
}
