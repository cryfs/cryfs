use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

use cryfs_version::Version;

use super::filesystem_id::FilesystemId;

pub const FILESYSTEM_FORMAT_VERSION: Version = konst::unwrap_ctx!(Version::parse_const("0.10"));

/// Configuration for a CryFS file system. This is stored in the cryfs.config file.
/// // TODO Do we need this to be clone?
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CryConfig {
    /// Blob ID of the root directory
    /// // TODO Store this as a BlobId struct instead of String
    pub root_blob: String,

    /// Encryption Key used for encrypting the blocks of the file system
    /// TODO Protect enc_key with mlock, etc. (see cryfs_utils::crypto::symmetric::key::EncryptionKey)
    ///      We can probably change the type of this member to `EncryptionKey`, but need to be careful
    ///      that (de)serialization still keeps it protected.
    pub enc_key: String,

    /// Cipher used for encrypting the blocks of the file system
    /// TODO Store cipher as an enum instead of String
    pub cipher: String,

    /// Current version of the format of this file system
    /// TODO Store version numbers as a struct instead of String
    pub format_version: String,

    /// Original version of the format of this file system.
    /// This may differ from [CryConfig::format_version] if the file system was migrated
    pub created_with_version: String,

    /// Version of the last CryFS instance that opened this file system
    pub last_opened_with_version: String,

    /// Size of the on-disk (i.e. post-encryption) blocks in bytes
    pub blocksize_bytes: u64,

    /// Unique ID of the file system
    pub filesystem_id: FilesystemId,

    /// If the exclusive client Id is set, then additional integrity measures (i.e. treating missing blocks as integrity violations) are enabled.
    /// Because this only works in a single-client setting, only this one client Id is allowed to access the file system.
    /// TODO Store this as an instance of `ClientId` instead of `u32`
    pub exclusive_client_id: Option<u32>,
}

impl CryConfig {
    pub fn serialize(self, writer: impl Write) -> Result<(), serde_json::Error> {
        super::serialization::serialize(self, writer)
    }

    pub fn deserialize(
        reader: impl Read,
    ) -> Result<Self, super::serialization::DeserializationError> {
        super::serialization::deserialize(reader)
    }

    pub fn missingBlockIsIntegrityViolation(&self) -> bool {
        return self.exclusive_client_id.is_some();
    }
}
