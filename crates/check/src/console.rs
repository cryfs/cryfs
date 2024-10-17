use anyhow::Result;
use std::path::Path;

use cryfs_filesystem::config::Console;
use cryfs_utils::crypto::kdf::scrypt::ScryptSettings;
use cryfs_version::{Version, VersionInfo};

// TODO What to do in these cases?

pub struct RecoverConsole;

impl Console for RecoverConsole {
    fn ask_migrate_filesystem(
        &self,
        _current_filesystem_format_version: &Version,
        _new_filesystem_format_version: &Version,
        _cryfs_version: &VersionInfo,
    ) -> Result<bool> {
        todo!()
    }

    fn ask_allow_replaced_filesystem(&self) -> Result<bool> {
        todo!()
    }

    fn ask_disable_single_client_mode(&self) -> Result<bool> {
        todo!()
    }

    fn ask_single_client_mode_for_new_filesystem(&self) -> Result<bool> {
        todo!()
    }

    /// We're in the process of creating a new file system and need to ask the user for the scrypt settings to use
    fn ask_scrypt_settings_for_new_filesystem(&self) -> Result<ScryptSettings> {
        todo!()
    }

    fn ask_cipher_for_new_filesystem(&self) -> Result<String> {
        todo!()
    }

    fn ask_blocksize_bytes_for_new_filesystem(&self) -> Result<u64> {
        todo!()
    }

    fn ask_create_basedir(&self, _path: &Path) -> Result<bool> {
        todo!()
    }

    fn ask_create_mountdir(&self, _path: &Path) -> Result<bool> {
        todo!()
    }
}
