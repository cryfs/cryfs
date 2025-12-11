use anyhow::Result;
// TODO Separate out InfallibleUnwrap from lockable and don't depend on lockable from this crate
use lockable::InfallibleUnwrap;

use crate::crypto::kdf::PasswordBasedKDF;
use crate::crypto::kdf::scrypt::params::ScryptParams;
use crate::crypto::kdf::scrypt::settings::ScryptSettings;
use crate::crypto::symmetric::EncryptionKey;

pub struct ScryptScrypt;

impl PasswordBasedKDF for ScryptScrypt {
    type Settings = ScryptSettings;
    type Parameters = ScryptParams;

    fn derive_key(key_size: usize, password: &str, kdf_parameters: &ScryptParams) -> EncryptionKey {
        let params = scrypt::Params::new(
            kdf_parameters.log_n(),
            kdf_parameters.r(),
            kdf_parameters.p(),
        )
        .expect("Invalid scrypt parameters");
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
