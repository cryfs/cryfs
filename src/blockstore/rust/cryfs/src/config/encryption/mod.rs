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

// TODO Tests, including backwards compatibility tests that make sure we can read the C++ format
