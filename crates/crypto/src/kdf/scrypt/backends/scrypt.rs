//! Pure Rust scrypt implementation using the `scrypt` crate.

use anyhow::Result;
// TODO Separate out InfallibleUnwrap from lockable and don't depend on lockable from this crate
use lockable::InfallibleUnwrap;

use crate::kdf::PasswordBasedKDF;
use crate::kdf::scrypt::params::ScryptParams;
use crate::kdf::scrypt::settings::ScryptSettings;
use crate::symmetric::EncryptionKey;

// TODO Some scrypt parameter settings that could have been created by cryfs 1.0 can't be loaded in cryfs 2.0.
//      See https://github.com/RustCrypto/password-hashes/issues/866

/// Scrypt implementation using the pure Rust `scrypt` crate.
///
/// This is the default scrypt implementation and provides good portability
/// without requiring external libraries.
pub struct ScryptScrypt;

impl PasswordBasedKDF for ScryptScrypt {
    type Settings = ScryptSettings;
    type Parameters = ScryptParams;

    fn derive_key(key_size: usize, password: &str, kdf_parameters: &ScryptParams) -> EncryptionKey {
        let params = scrypt::Params::new(
            kdf_parameters.log_n(),
            kdf_parameters.r(),
            kdf_parameters.p(),
        );
        let params = match params {
            Ok(params) => params,
            Err(_) => panic!("Invalid scrypt parameters: {kdf_parameters:?}"),
        };
        EncryptionKey::new(key_size, |key_data| {
            scrypt::scrypt(
                password.as_bytes(),
                kdf_parameters.salt(),
                &params,
                key_data,
            )
            .expect("Error in scrypt");
            Ok(())
        })
        .infallible_unwrap()
    }

    fn generate_parameters(settings: &ScryptSettings) -> Result<ScryptParams> {
        ScryptParams::generate(settings)
    }
}
