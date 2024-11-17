use anyhow::Result;
use byte_unit::Byte;
use std::path::Path;

use cryfs_filesystem::config::Console;
use cryfs_utils::crypto::kdf::scrypt::ScryptSettings;
use cryfs_version::{Version, VersionInfo};

pub struct FixtureCreationConsole;

impl Console for FixtureCreationConsole {
    fn ask_migrate_filesystem(
        &self,
        _current_filesystem_format_version: &Version,
        _new_filesystem_format_version: &Version,
        _cryfs_version: &VersionInfo,
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

    fn ask_blocksize_bytes_for_new_filesystem(&self) -> Result<Byte> {
        Ok(Byte::from_u64(104))
    }

    fn ask_create_basedir(&self, path: &Path) -> Result<bool> {
        panic!("unused")
    }

    fn ask_create_mountdir(&self, path: &Path) -> Result<bool> {
        panic!("unused")
    }
}
