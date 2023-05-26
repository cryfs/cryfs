use anyhow::{Context, Result};
use std::io::{Read, Seek, Write};

use crate::config::CryConfig;
use cryfs_utils::crypto::{
    kdf::{KDFParameters, PasswordBasedKDF},
    symmetric::{CipherDef, EncryptionKey},
};
use inner::InnerConfig;
use outer::{OuterCipher, OuterConfig};

// TODO Add a module level comment explaining our encryption scheme and why it is split into inner/outer
// TODO Make sure we don't derive the key twice when we load a config file and then store modifications to it

mod inner;
mod outer;
mod padding;

pub fn encrypt<KDF: PasswordBasedKDF>(
    config: CryConfig,
    password: &str,
    kdf_settings: &KDF::Settings,
    dest: &mut (impl Write + Seek),
) -> Result<()> {
    log::info!("Deriving key from password...");

    let kdf_parameters =
        KDF::generate_parameters(kdf_settings).context("Trying to generate KDF parameters")?;
    let (outer_key, inner_key) =
        generate_keys::<KDF>(&kdf_parameters, password).context("Trying to generate keys")?;

    log::info!("Deriving key from password...done");
    log::info!("Encrypting config file...");

    let inner_config =
        InnerConfig::encrypt(config, inner_key).context("Trying to encrypt InnerConfig")?;
    let outer_config = OuterConfig::encrypt(inner_config, kdf_parameters, outer_key)
        .context("Trying to encrypt OuterConfig")?;

    outer_config
        .serialize(dest)
        .context("Trying to serialize outer config")?;

    log::info!("Encrypting config file...done");

    Ok(())
}

pub fn decrypt<KDF: PasswordBasedKDF>(
    source: &mut (impl Read + Seek),
    // TODO Here and throughout the whole function stack, protect password similar to how we protect `EncryptionKey` (mprotect, etc.)
    //      Maybe we also have to protect CryConfig or at least make sure that the key is never unprotected on its way from/to the file into/from the key member of the CryConfig instance
    password: &str,
) -> Result<CryConfig> {
    let outer_config =
        OuterConfig::deserialize(source).context("Trying to deserialize outer config")?;

    log::info!("Deriving key from password...");

    let kdf_parameters = KDF::Parameters::deserialize(outer_config.kdf_parameters())
        .context("Trying to deserialize KDF parameters")?;

    println!("KDF: {kdf_parameters:?}");

    let (outer_key, inner_key) =
        generate_keys::<KDF>(&kdf_parameters, password).context("Trying to generate keys")?;

    log::info!("Deriving key from password...done");
    log::info!("Decrypting config file...");

    let inner_config = outer_config
        .decrypt(outer_key)
        .context("Trying to decrypt outer config")?;
    let config = inner_config
        .decrypt(inner_key)
        .context("Trying to decrypt inner config")?;

    log::info!("Decrypting config file...done");

    Ok(config)
}

fn generate_keys<KDF: PasswordBasedKDF>(
    kdf_parameters: &KDF::Parameters,
    password: &str,
) -> Result<(EncryptionKey, impl Fn(usize) -> EncryptionKey)> {
    const OUTER_KEY_SIZE: usize = OuterCipher::KEY_SIZE;
    const INNER_MAX_KEY_SIZE: usize = cryfs_utils::crypto::symmetric::MAX_KEY_SIZE;
    const COMBINED_KEY_SIZE: usize = OUTER_KEY_SIZE + INNER_MAX_KEY_SIZE;

    let combined_key = KDF::derive_key(COMBINED_KEY_SIZE, password, kdf_parameters);
    let outer_key = combined_key.take_bytes(OUTER_KEY_SIZE);
    let inner_key = move |key_num_bytes| {
        combined_key
            .skip_bytes(OUTER_KEY_SIZE)
            .take_bytes(key_num_bytes)
    };

    Ok((outer_key, inner_key))
}

// TODO More Tests, here and in submodules

#[cfg(test)]
mod tests {
    use crate::config::CryConfig;
    use crate::config::FilesystemId;
    use cryfs_utils::crypto::kdf::scrypt::{Scrypt, ScryptSettings};
    use std::io::Cursor;

    #[test]
    fn test_backwards_compatibility() {
        // Test that we can still read config files created by the C++ version of cryfs
        const CONFIG: &str = "63727966732e636f6e6669673b313b73637279707400300000000000000000001000000000000400000008000000631df2901386bb17475b3dbc776a3646aa9cb4bbf95faa7496d53f3ca6fa1e5e20841c0c51a6c6fdc524f0fc3c405dc83b44cbad86fa8985f8de2f6e720152af8340292df37ed4a2487fa79bc8a73b923acabfa7e6dbd3809c762cd89a2341f36663c11b60615cbdb07aefb9da8b32d269f776b73f82cdec0ead40eee3bb7fd17db390b7a4e69ae0439fffd2ebc5d64fb769ae3ca9bf42c74e90066ca83c761ea6cb3726ce09a5888e22d0244f8f88db42f9f4bc34e2e825a83edb1afe307dd629d50c92fd47add93394fbed1b3547bb9b6afd7aa21c7dea45c8365015e0462174a4fb47bdcecaf49d7ea479fe433c63cb6183561833fce69faf46177c394ad5349e7f16d2ca8883d086429c9be0ad0171742a8e84b4db4ed34d630f641a44edbec67c3649d538e737eb56300608bb5ae2ba8f30cb383788fef95b7f8aed26c8e05522e4383a9e63ab1cc15ecf26e97d63370e16421077fcaa59d52a9fc694850884169ba799057dd1f1fb165f7faba52350146da4f56022fc8220bc3bcd9518f7e136f62314983817e5eea3182cf37e2d718ca79d097281cd241fe75a3e31a0580baf74500adb58ccb6e1c47f33d7597e76c3feb4a97d478f3217a76db2d25e89998031185ce94d27bc366b55279b07bf9d538fb31a4f87a0dfb22a834b686f7a2122eb6ca3f61bb5bbb53f0b8cdab7ac2adb782db32c5cd119c8025de4f9aaf02f4aa3814970c5a590abde487e37422740b76299591b09f1ec872173ac1a6f367bcbdd929b6f2557ecfc86656ba79b670c95c52c2fe52a51df342d0b20894c0bfb9e11e440aafdf9cf6a45cacf3f927cde4392a8356e27c30f14c865c6cf7d975b76645385fc6f02b81c285ffbed7a81a873dea8bb6fc84b37d1252b12a8df278f894a314b5d6d4c75c7bff860c3c087794cb782aa233ef6d1d22301de984f33bbb3c02ced1a6a38ac207b24e3a279fcfdf1bea7fab683003926e53f1efb03eb721bc26ff209607ea473a162d6cca3681e6b95f4c823d6359d778608caf117f6cd57c45a2c2c62023b231d1dd3ecde6feafa1d00c2007cadbaaef248d6d7ada2222b304cfeffbb86e7e908886a71a05c9d15bc00a6006c86d96e61ee45824350e5ffd1b15b4505eb65a76f163aed2fcca997b5d186719b860faaa8818bd0abc7a493b0953b1d222576ee3c6d9339896fc74db19e20b52deabcf1429894f9e369e51745c764cb3ef59f42f486b787926b0fce413c1dcffea4283772417b0cf3c67a1ec2252f20e9396e993256fddb6ded721399ee8d16d533ef99cc3f774f9e0583cac179e2d83d0156e65168c8667dc3b03fdc4b9e65e3af2dd522137c1af94911f9727e38760f373ea1e334184c62b6cdf1ea5e8ad16ee98b0f36c2662e0427f6ecc9995b12fef283d4b27ac85061170d2c42a9112056b8e4db4259fb26a3f872b4cf5b5add0275826c35e104c397cd7d87122b94871bcfe36b2835a219a5fbd4af4b0543f986cdd1db9d9627f8c337082d9e84ed58486f92426e8d9811bc";
        let config = super::decrypt::<Scrypt>(
            &mut Cursor::new(&hex::decode(CONFIG).unwrap()),
            "mypassword",
        )
        .unwrap();
        assert_eq!(
            config,
            CryConfig {
                root_blob: "B7847BAA5663DE6A3155A8017B5A8AC2".to_string(),
                enc_key: "F8294D3955FF8CC06B787D71DE64168DFC4C994046FBABB936B2CFE1629F6772"
                    .to_string(),
                cipher: "xchacha20-poly1305".to_string(),
                version: "0.10".to_string(),
                created_with_version: "0.11.2".to_string(),
                last_opened_with_version: "0.11.3".to_string(),
                blocksize_bytes: 16384,
                filesystem_id: FilesystemId::from_hex("ABDCB364DB327ED401F22E99EB37E78F").unwrap(),
                exclusive_client_id: None,
            }
        );
    }

    #[test]
    fn test_encrypt_decrypt() {
        // Test that we can encrypt and then decrypt a config file and get the same result
        let config = CryConfig {
            root_blob: "6A3155A8017B5A8AC2B7847BAA5663DE".to_string(),
            enc_key: "6B787D71DE64168DFC4C994046FBABB936B2CFE1629F6772F8294D3955FF8CC0".to_string(),
            cipher: "aes-256-gcm".to_string(),
            version: "0.10".to_string(),
            created_with_version: "0.10.2".to_string(),
            last_opened_with_version: "0.11.1".to_string(),
            blocksize_bytes: 16384,
            filesystem_id: FilesystemId::from_hex("B364DB327ED401F22E99EB37E78FABDC").unwrap(),
            exclusive_client_id: None,
        };
        let mut encrypted = vec![];
        super::encrypt::<Scrypt>(
            config.clone(),
            "some_password",
            &ScryptSettings::TEST,
            &mut Cursor::new(&mut encrypted),
        )
        .unwrap();
        let decrypted_config =
            super::decrypt::<Scrypt>(&mut Cursor::new(&encrypted), "some_password").unwrap();
        assert_eq!(config, decrypted_config,);
    }
}
