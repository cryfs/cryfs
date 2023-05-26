use anyhow::{ensure, Context, Result};
use binrw::{binrw, until_eof, BinRead, BinWrite, NullString};
use std::io::{Cursor, Read, Seek, Write};

use cryfs_utils::{crypto::symmetric::EncryptionKey, data::Data};

use super::super::{ciphers::lookup_cipher_dyn, cryconfig::CryConfig};
use super::padding::{add_padding, remove_padding, PADDING_OVERHEAD_PREFIX};

const HEADER: &str = "cryfs.config.inner;0";

// Inner config data is grown to this size before encryption to hide its actual size
const CONFIG_SIZE: usize = 900;

#[binrw]
#[brw(little)]
struct InnerConfigLayout {
    header: NullString,
    cipher_name: NullString,
    #[br(parse_with = until_eof)]
    encrypted_config: Vec<u8>,
    // TODO Actually storing these Vecs in an `InnerConfigLayout` object means we have to allocate them during (de)serialization. This could be avoided, maybe by using a `Serializer/Deserializer` system as C++ CryFS cpp-utils had it
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

        let padding_overhead_suffix = CONFIG_SIZE - PADDING_OVERHEAD_PREFIX;
        let mut plaintext = Data::allocate(
            PADDING_OVERHEAD_PREFIX + cipher.ciphertext_overhead_prefix(),
            0,
            CONFIG_SIZE + padding_overhead_suffix + cipher.ciphertext_overhead_suffix(),
        );
        config
            .serialize(plaintext.append_writer::<false>())
            .context("Trying to serialize CryConfig")?;
        ensure!(
            plaintext.len() <= CONFIG_SIZE,
            "Plaintext is too large. We should increase `CONFIG_SIZE`."
        );

        let plaintext = add_padding(plaintext.into(), CONFIG_SIZE)
            .context("Trying to add padding to InnerConfig")?;

        let encrypted_config = cipher
            .encrypt(plaintext)
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
        let plaintext =
            remove_padding(plaintext).context("Trying to remove padding from InnerConfig")?;
        let config = CryConfig::deserialize(&mut Cursor::new(plaintext))?;

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
