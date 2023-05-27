use std::path::Path;
use thiserror::Error;

use super::CryConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Access {
    ReadOnly,
    ReadWrite,
}

#[derive(Error, Debug)]
pub enum ConfigLoaderError {}

// pub fn load_or_create(
//     filename: &Path,
//     allow_filesystem_upgrade: bool,
//     allow_replaced_filesystem: bool,
// ) -> Result<CryConfig, ConfigLoaderError> {
//     if filename.exists() {
//         return _load_config(
//             filename,
//             allow_filesystem_upgrade,
//             allow_replaced_filesystem,
//             Access::ReadWrite,
//         );
//     } else {
//         return _create_config(filename, allow_replaced_filesystem);
//     }
// }

// fn _load_config(
//     filename: &Pat,
//     allow_filesystem_upgrade: bool,
//     allow_replaced_filesystem: bool,
//     access: Access,
// ) -> Result<CryConfig, ConfigLoaderError> {
//     let config: CryConfigFile = CryConfigFile::load(filename, access)?;
//     _check_version(&config.config, allow_filesystem_upgrade)?;
//     if config.version != CryConfig::FILESYSTEM_FORMAT_VERSION {
//         config.version = CryConfig::FILESYSTEM_FORMAT_VERSION;
//         if access == Access::ReadWrite {
//             config.save();
//         }
//     }
//     if config.last_opened_with_version != gitversion::VersionString() {
//         config.last_opened_with_version = gitversion::VersionString();
//         if access == Access::ReadWrite {
//             config.save();
//         }
//     }
//     _check_cipher(&config.config)?;
//     let local_state = LocalStateMetadata::load_or_generate(
//         _local_state_dir.for_filesystem_id(config.config.filesystem_id),
//         cpputils::Data::from_string(config.config.enc_key),
//         allow_replaced_filesystem,
//     );
//     let my_client_id = local_state.my_client_id();
//     _check_missing_blocks_are_integrity_violations(&config, my_client_id)?;
//     return Ok(config);
// }
