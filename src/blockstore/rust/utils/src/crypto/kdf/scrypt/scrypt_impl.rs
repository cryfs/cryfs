// TODO Separate out InfallibleUnwrap from lockable and don't depend on lockable from this crate
use anyhow::Result;
use lockable::InfallibleUnwrap;

use super::super::PasswordBasedKDF;
use super::params::ScryptParams;
use super::settings::ScryptSettings;
use crate::crypto::symmetric::EncryptionKey;

pub struct Scrypt;

impl PasswordBasedKDF for Scrypt {
    type Settings = ScryptSettings;
    type Parameters = ScryptParams;

    fn derive_key(key_size: usize, password: &str, kdf_parameters: &ScryptParams) -> EncryptionKey {
        EncryptionKey::new(key_size, |key_data| {
            Ok(scrypt::scrypt(
                password.as_bytes(),
                kdf_parameters.salt(),
                &kdf_parameters.params(),
                key_data,
            )
            .expect("Error in scrypt"))
        })
        .infallible_unwrap()
    }

    fn generate_parameters(settings: &ScryptSettings) -> Result<ScryptParams> {
        ScryptParams::generate(settings)
    }
}

// TODO Tests
// TODO Backwards-compatibility tests that make sure this produces the same keystream as C++ did
// TODO Benchmarks
