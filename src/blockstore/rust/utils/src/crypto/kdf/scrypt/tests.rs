use crate::crypto::kdf::{
    scrypt::{ScryptParams, ScryptSettings},
    KDFParameters, PasswordBasedKDF,
};

#[generic_tests::define]
mod generic {
    use super::*;

    #[test]
    fn generated_key_is_reproductible_448<S>()
    where
        S: PasswordBasedKDF<Settings = ScryptSettings, Parameters = ScryptParams>,
    {
        let params = S::generate_parameters(&ScryptSettings::TEST).unwrap();
        let derived_key = S::derive_key(56, "mypassword", &params);

        let params = ScryptParams::deserialize(&params.serialize()).unwrap();
        let rederived_key = S::derive_key(56, "mypassword", &params);

        assert_eq!(derived_key.to_hex(), rederived_key.to_hex());
    }

    #[test]
    fn backwards_compatibility_448<S>()
    where
        S: PasswordBasedKDF<Settings = ScryptSettings, Parameters = ScryptParams>,
    {
        let kdf_parameters = ScryptParams::deserialize(&hex::decode("00040000000000000100000002000000E429AFB0500BD5D172089598B76E6B9ED6D0DDAF3B08F99AA05357F96F4F7823").unwrap()).unwrap();
        let rederived_key = S::derive_key(56, "mypassword", &kdf_parameters);
        assert_eq!("70416B4E1569E2335442F7FE740E6A8ADC149514B7B6D7838A996AE0E2125F743341E72FF9F44C91A9675EAE459C0C0126FDB6CE220436E0", rederived_key.to_hex());
    }

    #[test]
    fn generated_key_is_reproductible_256<S>()
    where
        S: PasswordBasedKDF<Settings = ScryptSettings, Parameters = ScryptParams>,
    {
        let params = S::generate_parameters(&ScryptSettings::TEST).unwrap();
        let derived_key = S::derive_key(32, "mypassword", &params);

        let params = ScryptParams::deserialize(&params.serialize()).unwrap();
        let rederived_key = S::derive_key(32, "mypassword", &params);

        assert_eq!(derived_key.to_hex(), rederived_key.to_hex());
    }

    #[test]
    fn backwards_compatibility_256<S>()
    where
        S: PasswordBasedKDF<Settings = ScryptSettings, Parameters = ScryptParams>,
    {
        let kdf_parameters = ScryptParams::deserialize(&hex::decode("000400000000000001000000020000007D65C035E0C4250003A24ED11ABD41F6101DEEC104F6875EE1B808A6683535BD").unwrap()).unwrap();
        let rederived_key = S::derive_key(32, "mypassword", &kdf_parameters);
        assert_eq!(
            "A423A0176F99A3197722D4B8686110FC2E2C04FF5E37AE43A7241097598F599D",
            rederived_key.to_hex()
        );
    }

    #[test]
    fn generated_key_is_reproductible_128<S>()
    where
        S: PasswordBasedKDF<Settings = ScryptSettings, Parameters = ScryptParams>,
    {
        let params = S::generate_parameters(&ScryptSettings::TEST).unwrap();
        let derived_key = S::derive_key(16, "mypassword", &params);

        let params = ScryptParams::deserialize(&params.serialize()).unwrap();
        let rederived_key = S::derive_key(16, "mypassword", &params);

        assert_eq!(derived_key.to_hex(), rederived_key.to_hex());
    }

    #[test]
    fn backwards_compatibility_128<S>()
    where
        S: PasswordBasedKDF<Settings = ScryptSettings, Parameters = ScryptParams>,
    {
        let kdf_parameters = ScryptParams::deserialize(&hex::decode("000400000000000001000000020000008514339A7F583D80C9865C9EA01B698EE8AEAF99AE5F7AE79C8817D2E73D553D").unwrap()).unwrap();
        let rederived_key = S::derive_key(16, "mypassword", &kdf_parameters);
        assert_eq!("2EF2F0A4EC335C961D4BE58BFB722F75", rederived_key.to_hex());
    }

    #[test]
    fn backwards_compatibility_default_settings<S>()
    where
        S: PasswordBasedKDF<Settings = ScryptSettings, Parameters = ScryptParams>,
    {
        // These kdf_parameters were created with [ScryptSettings::DEFAULT]. This test case will take a bit longer to run.
        let kdf_parameters = ScryptParams::deserialize(&hex::decode("00001000000000000400000008000000D04ACF9519113E1F4E4D7FB39EFBF257CD71CF8536A468B546C2F5A65C6B622C").unwrap()).unwrap();
        let rederived_key = S::derive_key(32, "mypassword", &kdf_parameters);
        assert_eq!(
            "AB70B1923F3EB9EB8A75C15FD665AC3494C5EBAB80323D864135DBB2911ECF59",
            rederived_key.to_hex()
        );
    }

    #[test]
    fn different_passwords_result_in_different_keys<S>()
    where
        S: PasswordBasedKDF<Settings = ScryptSettings, Parameters = ScryptParams>,
    {
        let params = S::generate_parameters(&ScryptSettings::TEST).unwrap();
        let derived_key_1 = S::derive_key(16, "mypassword", &params);
        let derived_key_2 = S::derive_key(16, "mypassword2", &params);

        assert_ne!(derived_key_1.to_hex(), derived_key_2.to_hex());
    }

    #[instantiate_tests(<crate::crypto::kdf::scrypt::Scrypt>)]
    mod scrypt_default {}

    #[instantiate_tests(<crate::crypto::kdf::scrypt::backends::scrypt::Scrypt>)]
    mod scrypt {}

    #[instantiate_tests(<crate::crypto::kdf::scrypt::backends::openssl::ScryptOpenssl>)]
    mod scrypt_openssl {}
}
