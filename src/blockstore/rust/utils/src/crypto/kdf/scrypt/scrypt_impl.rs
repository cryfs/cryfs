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

// TODO Benchmarks

#[cfg(test)]
mod tests {
    use super::*;

    use crate::crypto::kdf::KDFParameters;

    #[test]
    fn generated_key_is_reproductible_448() {
        let params = Scrypt::generate_parameters(&ScryptSettings::TEST).unwrap();
        let derived_key = Scrypt::derive_key(56, "mypassword", &params);

        let params = ScryptParams::deserialize(&params.serialize()).unwrap();
        let rederived_key = Scrypt::derive_key(56, "mypassword", &params);

        assert_eq!(derived_key.to_hex(), rederived_key.to_hex());
    }

    #[test]
    fn backwards_compatibility_448() {
        let kdf_parameters = ScryptParams::deserialize(&hex::decode("00040000000000000100000002000000E429AFB0500BD5D172089598B76E6B9ED6D0DDAF3B08F99AA05357F96F4F7823").unwrap()).unwrap();
        let rederived_key = Scrypt::derive_key(56, "mypassword", &kdf_parameters);
        assert_eq!("70416B4E1569E2335442F7FE740E6A8ADC149514B7B6D7838A996AE0E2125F743341E72FF9F44C91A9675EAE459C0C0126FDB6CE220436E0", rederived_key.to_hex());
    }

    #[test]
    fn generated_key_is_reproductible_256() {
        let params = Scrypt::generate_parameters(&ScryptSettings::TEST).unwrap();
        let derived_key = Scrypt::derive_key(32, "mypassword", &params);

        let params = ScryptParams::deserialize(&params.serialize()).unwrap();
        let rederived_key = Scrypt::derive_key(32, "mypassword", &params);

        assert_eq!(derived_key.to_hex(), rederived_key.to_hex());
    }

    #[test]
    fn backwards_compatibility_256() {
        let kdf_parameters = ScryptParams::deserialize(&hex::decode("000400000000000001000000020000007D65C035E0C4250003A24ED11ABD41F6101DEEC104F6875EE1B808A6683535BD").unwrap()).unwrap();
        let rederived_key = Scrypt::derive_key(32, "mypassword", &kdf_parameters);
        assert_eq!(
            "A423A0176F99A3197722D4B8686110FC2E2C04FF5E37AE43A7241097598F599D",
            rederived_key.to_hex()
        );
    }

    #[test]
    fn generated_key_is_reproductible_128() {
        let params = Scrypt::generate_parameters(&ScryptSettings::TEST).unwrap();
        let derived_key = Scrypt::derive_key(16, "mypassword", &params);

        let params = ScryptParams::deserialize(&params.serialize()).unwrap();
        let rederived_key = Scrypt::derive_key(16, "mypassword", &params);

        assert_eq!(derived_key.to_hex(), rederived_key.to_hex());
    }

    #[test]
    fn backwards_compatibility_128() {
        let kdf_parameters = ScryptParams::deserialize(&hex::decode("000400000000000001000000020000008514339A7F583D80C9865C9EA01B698EE8AEAF99AE5F7AE79C8817D2E73D553D").unwrap()).unwrap();
        let rederived_key = Scrypt::derive_key(16, "mypassword", &kdf_parameters);
        assert_eq!("2EF2F0A4EC335C961D4BE58BFB722F75", rederived_key.to_hex());
    }

    #[test]
    fn backwards_compatibility_default_settings() {
        // These kdf_parameters were created with [ScryptSettings::DEFAULT]. This test case will take a bit longer to run.
        let kdf_parameters = ScryptParams::deserialize(&hex::decode("00001000000000000400000008000000D04ACF9519113E1F4E4D7FB39EFBF257CD71CF8536A468B546C2F5A65C6B622C").unwrap()).unwrap();
        let rederived_key = Scrypt::derive_key(32, "mypassword", &kdf_parameters);
        assert_eq!(
            "AB70B1923F3EB9EB8A75C15FD665AC3494C5EBAB80323D864135DBB2911ECF59",
            rederived_key.to_hex()
        );
    }

    #[test]
    fn different_passwords_result_in_different_keys() {
        let params = Scrypt::generate_parameters(&ScryptSettings::TEST).unwrap();
        let derived_key_1 = Scrypt::derive_key(16, "mypassword", &params);
        let derived_key_2 = Scrypt::derive_key(16, "mypassword2", &params);

        assert_ne!(derived_key_1.to_hex(), derived_key_2.to_hex());
    }
}
