use super::symmetric::EncryptionKey;
use anyhow::Result;

pub trait KDFParameters: Sized {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(serialized: &[u8]) -> Result<Self>;
}

pub trait PasswordBasedKDF {
    type Settings;
    type Parameters: KDFParameters;

    fn derive_key(
        key_size: usize,
        password: &str,
        kdf_parameters: &Self::Parameters,
    ) -> EncryptionKey;

    /// Generate a new set of KDF parameters based on the given settings.
    /// This can be used to encrypt new data but will be useless for trying to decrypt existing data.
    fn generate_parameters(settings: &Self::Settings) -> Result<Self::Parameters>;
}

pub mod scrypt;
