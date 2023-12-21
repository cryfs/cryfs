use anyhow::Result;

use cryfs_cryfs::config::Console;
use cryfs_utils::crypto::kdf::scrypt::ScryptSettings;
use cryfs_version::{Version, VersionInfo};

pub struct FixtureCreationConsole;

impl Console for FixtureCreationConsole {
    fn ask_migrate_filesystem(
        &self,
        current_filesystem_format_version: &Version,
        new_filesystem_format_version: &Version,
        cryfs_version: &VersionInfo,
    ) -> Result<bool> {
        panic!("unused")
    }

    fn ask_allow_replaced_filesystem(&self) -> Result<bool> {
        panic!("unused")
    }

    fn ask_disable_single_client_mode(&self) -> Result<bool> {
        panic!("unused")
    }

    fn ask_single_client_mode_for_new_filesystem(&self) -> Result<bool> {
        panic!("unused")
    }

    fn ask_scrypt_settings_for_new_filesystem(&self) -> Result<ScryptSettings> {
        Ok(ScryptSettings::TEST)
    }

    fn ask_cipher_for_new_filesystem(&self) -> Result<String> {
        Ok("aes-256-gcm".to_owned())
    }

    fn ask_blocksize_bytes_for_new_filesystem(&self) -> Result<u64> {
        Ok(104)
    }
}
