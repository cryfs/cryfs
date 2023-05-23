use super::symmetric::EncryptionKey;
use anyhow::Result;

pub trait KDFParameters: Sized {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(serialized: &[u8]) -> Result<Self>;
}

pub struct KDFResult<P: KDFParameters> {
    key: EncryptionKey,
    kdf_parameters: P,
}

pub trait PasswordBasedKDF {
    type Settings;
    type Parameters: KDFParameters;

    fn derive_existing_key(
        key_size: usize,
        password: &str,
        kdf_parameters: &Self::Parameters,
    ) -> EncryptionKey;

    fn derive_new_key(
        key_size: usize,
        password: &str,
        settings: &Self::Settings,
    ) -> KDFResult<Self::Parameters>;
}

pub mod scrypt;
