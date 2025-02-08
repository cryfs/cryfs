use anyhow::Result;
use byte_unit::Byte;
use std::path::Path;

use cryfs_utils::crypto::kdf::scrypt::ScryptSettings;
use cryfs_version::{Version, VersionInfo};

/// Interface for cryfs to interact with the user, e.g. ask questions and get answers on the terminal
pub trait Console {
    // TODO Only some of these should be in crate/cryfs. Most questions are specific to crates/cryfs-cli. Can we split this trait?

    /// We're in the process of opening a filesystem from an earlier version of CryFS.
    /// Ask the user whether they want to migrate the filesystem to the current version.
    /// TODO The C++ message here was "This filesystem is for CryFS " + config.Version() + " (or a later version with the same storage format). You're running a CryFS version using storage format " + CryConfig::FilesystemFormatVersion + ". It is recommended to create a new filesystem with CryFS 0.10 and copy your files into it. If you don't want to do that, we can also attempt to migrate the existing filesystem, but that can take a long time, you won't be getting some of the performance advantages of the 0.10 release series, and if the migration fails, your data may be lost. If you decide to continue, please make sure you have a backup of your data. Do you want to attempt a migration now?"
    fn ask_migrate_filesystem(
        &self,
        // TODO Pass in a version struct instead of strings
        current_filesystem_format_version: &Version,
        new_filesystem_format_version: &Version,
        cryfs_version: &VersionInfo,
    ) -> Result<bool>;

    /// We're in the process of opening a filesystem but the encryption key is different than the last time
    /// we opened this file system. Maybe an attacker replaced the whole file system.
    /// Ask the user whether they want to continue.
    fn ask_allow_changed_encryption_key(&self) -> Result<bool>;

    /// We're in the process of opening a filesystem but the filesystem id is different than the last time
    /// we opened a file system from this basedir. Maybe an attacker replaced the whole file system.
    /// Ask the user whether they want to continue.
    fn ask_allow_replaced_filesystem(&self) -> Result<bool>;

    /// We're in the process of opening a filesystem that has an exclusive_client_id set
    /// and is therefore in single client mode. Ask the user whether they want to disable
    /// single client mode.
    /// TODO The C++ message here was "This filesystem is setup to treat missing blocks as integrity violations and therefore only works in single-client mode. You are trying to access it from a different client.\nDo you want to disable this integrity feature and stop treating missing blocks as integrity violations?\nChoosing yes will not affect the confidentiality of your data, but in future you might not notice if an attacker deletes one of your files."
    fn ask_disable_single_client_mode(&self) -> Result<bool>;

    // We're in the process of creating a new file system and need to ask the user whether they want to use single client mode
    // TODO The C++ message here was "Most integrity checks are enabled by default. However, by default CryFS does not treat missing blocks as integrity violations.\nThat is, if CryFS finds a block missing, it will assume that this is due to a synchronization delay and not because an attacker deleted the block.\nIf you are in a single-client setting, you can let it treat missing blocks as integrity violations, which will ensure that you notice if an attacker deletes one of your files.\nHowever, in this case, you will not be able to use the file system with other devices anymore.\nDo you want to treat missing blocks as integrity violations?
    fn ask_single_client_mode_for_new_filesystem(&self) -> Result<bool>;

    /// We're in the process of creating a new file system and need to ask the user for the scrypt settings to use
    fn ask_scrypt_settings_for_new_filesystem(&self) -> Result<ScryptSettings>;

    /// We're in the process of creating a new file system and need to ask the user for the cipher to use
    /// TODO Probably it's better to have an enum for all the ciphers we support and return that here instead of returning a string. This would also need us to change `ciphers.rs` and the lookup code probably.
    fn ask_cipher_for_new_filesystem(&self) -> Result<String>;

    /// We're in the process of creating a new file system and need to ask the user for the block size to use
    fn ask_blocksize_bytes_for_new_filesystem(&self) -> Result<Byte>;

    /// We've tried to load a file system but the basedir doesn't exist. Ask whether we should create it.
    fn ask_create_basedir(&self, path: &Path) -> Result<bool>;

    /// We've tried to mount a file system but the mountdir doesn't exist. Ask whether we should create it.
    fn ask_create_mountdir(&self, path: &Path) -> Result<bool>;
}
