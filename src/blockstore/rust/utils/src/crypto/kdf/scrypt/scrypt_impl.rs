// TODO Separate out InfallibleUnwrap from lockable and don't depend on lockable from this crate
use lockable::InfallibleUnwrap;

use super::super::{KDFResult, PasswordBasedKDF};
use super::params::ScryptParams;
use super::settings::ScryptSettings;
use crate::crypto::symmetric::EncryptionKey;

pub struct Scrypt;

impl PasswordBasedKDF for Scrypt {
    type Settings = ScryptSettings;
    type Parameters = ScryptParams;

    fn derive_existing_key(
        key_size: usize,
        password: &str,
        kdf_parameters: &ScryptParams,
    ) -> EncryptionKey {
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

    fn derive_new_key(
        key_size: usize,
        password: &str,
        settings: &ScryptSettings,
    ) -> KDFResult<Self::Parameters> {
        let kdf_parameters =
            ScryptParams::generate(settings).expect("Generating scrypt parameters failed");
        let key = Self::derive_existing_key(key_size, password, &kdf_parameters);
        KDFResult {
            key,
            kdf_parameters,
        }
    }
}

// TODO Tests
// TODO Backwards-compatibility tests that make sure this produces the same keystream as C++ did
// TODO Benchmarks
