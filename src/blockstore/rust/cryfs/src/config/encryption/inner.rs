use anyhow::{ensure, Context, Result};
use binrw::{binrw, until_eof, BinRead, BinWrite, NullString};
use std::io::{Read, Seek, Write};

use cryfs_utils::crypto::symmetric::{lookup_cipher_dyn, EncryptionKey};

use super::super::cryconfig::CryConfig;

const HEADER: &str = "cryfs.config.inner;0";

// Inner config data is grown to this size before encryption to hide its actual size
// TODO Actually do this, using the RandomPadding implementation from C++
const CONFIG_SIZE: usize = 900;

#[binrw]
#[brw(little)]
struct InnerConfigLayout {
    header: NullString,
    cipher_name: NullString,
    #[br(parse_with = until_eof)]
    encrypted_config: Vec<u8>,
}

/// Wraps a [CryConfig] instance and encrypts it, then prepends the used cipher name and a fixed header.
/// This is meant to be then placed into an [super::OuterConfig] instance for further encryption.
///
/// Common usage patterns are:
/// * When loading a cryfs config file, call first [InnerConfig::deserialize] to get an [InnerConfig] instance,
///   then call [InnerConfig::decrypt] to get the contained [CryConfig] object.
/// * When storing a cryfs config file, call [InnerConfig::encrypt] to get an [InnerConfig] instance,
///   then call [InnerConfig::serialize] to get the serialized representation of the [InnerConfig] instance.
pub struct InnerConfig {
    cipher_name: String,
    encrypted_config: Vec<u8>,
}

impl InnerConfig {
    /// Take a [CryConfig] and encrypt it into an [InnerConfig] instance
    pub fn encrypt(
        config: CryConfig,
        inner_key: impl FnOnce(usize) -> EncryptionKey,
    ) -> Result<InnerConfig> {
        let cipher_name = config.cipher.clone();
        let cipher = lookup_cipher_dyn(&cipher_name, inner_key)
            .with_context(|| format!("Trying to look up cipher {}", config.cipher))?;

        let plaintext = config.serialize().into_bytes();
        let encrypted_config = cipher
            .encrypt(plaintext.into())
            .context("Trying to Cipher::encrypt InnerConfig")?;

        Ok(Self {
            cipher_name,
            encrypted_config: encrypted_config.into_vec(),
        })
    }

    /// Decrypt an [InnerConfig] instance to get the contained [CryConfig] object
    pub fn decrypt(self, inner_key: impl FnOnce(usize) -> EncryptionKey) -> Result<CryConfig> {
        let cipher = lookup_cipher_dyn(&self.cipher_name, inner_key)
            .with_context(|| format!("Trying to look up cipher {}", self.cipher_name))?;

        let plaintext = cipher
            .decrypt(self.encrypted_config.into())
            .context("Trying to Cipher::decrypt InnerConfig")?;
        let plaintext = String::from_utf8(plaintext.into_vec())
            .context("Trying to convert decrypted InnerConfig to UTF-8")?;
        let config = CryConfig::deserialize(&plaintext)?;

        ensure!(
            config.cipher == self.cipher_name,
            "Cipher name in CryConfig does not match cipher in InnerConfig",
        );

        Ok(config)
    }

    pub fn deserialize(source: &mut (impl Read + Seek)) -> Result<Self> {
        let layout = InnerConfigLayout::read(source)?;
        let read_header: String = layout
            .header
            .try_into()
            .context("Header is not valid UTF-8")?;
        ensure!(
            read_header == HEADER,
            "Invalid header in outer config. Expected '{HEADER}', got '{read_header}'",
        );
        let cipher_name = layout
            .cipher_name
            .try_into()
            .context("Cipher name is not valid UTF-8")?;
        Ok(Self {
            cipher_name,
            encrypted_config: layout.encrypted_config,
        })
    }

    pub fn serialize(self, dest: &mut (impl Write + Seek)) -> Result<()> {
        let layout = InnerConfigLayout {
            header: HEADER.into(),
            cipher_name: self.cipher_name.into(),
            encrypted_config: self.encrypted_config,
        };
        layout.write(dest)?;
        Ok(())
    }
}
