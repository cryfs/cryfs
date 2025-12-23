//! OpenSSL-based scrypt implementation.

use anyhow::Result;
// TODO Separate out InfallibleUnwrap from lockable and don't depend on lockable from this crate
use lockable::InfallibleUnwrap;

use crate::kdf::{
    PasswordBasedKDF,
    scrypt::{ScryptParams, ScryptSettings},
};
use crate::symmetric::EncryptionKey;

/// Scrypt implementation using OpenSSL.
///
/// This implementation uses OpenSSL's scrypt function, which may provide
/// better performance on some platforms due to optimized assembly code.
pub struct ScryptOpenssl;

impl PasswordBasedKDF for ScryptOpenssl {
    type Settings = ScryptSettings;
    type Parameters = ScryptParams;

    fn derive_key(key_size: usize, password: &str, kdf_parameters: &ScryptParams) -> EncryptionKey {
        // TODO Is this implementation parallelized? Our other backend is but not sure about this one.
        let log_n = kdf_parameters.log_n();
        assert!(
            log_n < 64,
            "Scrypt parameter log_n is {} but must be smaller than 64",
            log_n
        );
        let n = 1u64 << log_n;
        let r = u64::from(kdf_parameters.r());
        let p = u64::from(kdf_parameters.p());
        // TODO What does MAXMEM do exactly? Would setting it to a lower value allow it to work on lower end hardware without crashing? Or would it just move the crash to a different code location?
        const MAXMEM: u64 = u64::MAX;
        EncryptionKey::new(key_size, |key_data| {
            openssl::pkcs5::scrypt(
                password.as_bytes(),
                kdf_parameters.salt(),
                n,
                r,
                p,
                MAXMEM,
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
