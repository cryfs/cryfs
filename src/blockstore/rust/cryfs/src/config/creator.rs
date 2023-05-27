use rand::{thread_rng, RngCore};
use thiserror::Error;
// TODO Separate InfallibleUnwrap from lockable crate and remove lockable crate from our dependencies
use lockable::InfallibleUnwrap;

use super::ciphers::{lookup_cipher_sync, SyncCipherCallback};
use super::console::Console;
use super::loader::{CommandLineFlags, CRYFS_VERSION};
use super::CryConfig;
use crate::config::{FilesystemId, FILESYSTEM_FORMAT_VERSION};
use crate::localstate::{FilesystemMetadata, LocalStateDir};
use cryfs_blobstore::BlobId;
use cryfs_blockstore::ClientId;
use cryfs_utils::crypto::symmetric::{CipherDef, EncryptionKey};

#[derive(Error, Debug)]
pub enum ConfigCreateError {
    #[error("The cipher '{cipher_name}' is not supported")]
    CipherNotSupported { cipher_name: String },

    #[error("Error checking the local state of the file system: {0}")]
    LocalStateError(anyhow::Error),
}

pub struct ConfigCreateResult {
    pub config: CryConfig,
    pub my_client_id: ClientId,
}

pub fn create(
    console: &impl Console,
    command_line_flags: &CommandLineFlags,
    local_state_dir: &LocalStateDir,
) -> Result<ConfigCreateResult, ConfigCreateError> {
    let cipher_name = console.ask_cipher_for_new_filesystem();
    let enc_key = _generate_encryption_key(&cipher_name)?;
    let filesystem_id = FilesystemId::new_random();
    let local_state =
        FilesystemMetadata::load_or_generate(&local_state_dir, &filesystem_id, &enc_key, console)
            .map_err(ConfigCreateError::LocalStateError)?;
    let my_client_id = *local_state.my_client_id();
    let exclusive_client_id =
        _generate_exclusive_client_id(my_client_id, command_line_flags, console)
            .map(|id| id.id.get());
    let config = CryConfig {
        root_blob: _generate_root_blob_id().to_hex(),
        enc_key: enc_key.to_hex(),
        cipher: cipher_name,
        format_version: FILESYSTEM_FORMAT_VERSION.to_string(),
        created_with_version: CRYFS_VERSION.to_string(),
        last_opened_with_version: CRYFS_VERSION.to_string(),
        // TODO Check block size is valid (i.e. large enough)
        blocksize_bytes: console.ask_blocksize_bytes_for_new_filesystem(),
        filesystem_id,
        exclusive_client_id,
    };

    Ok(ConfigCreateResult {
        config,
        my_client_id,
    })
}

fn _generate_encryption_key(cipher_name: &str) -> Result<EncryptionKey, ConfigCreateError> {
    struct CreateKeyCallback;
    impl SyncCipherCallback for CreateKeyCallback {
        type Result = EncryptionKey;
        fn callback<C: CipherDef + Send + Sync + 'static>(self) -> Self::Result {
            EncryptionKey::new(C::KEY_SIZE, |data| {
                // TODO Which rng should we use?
                thread_rng().fill_bytes(data);
                Ok(())
            })
            .infallible_unwrap()
        }
    }
    lookup_cipher_sync(cipher_name, CreateKeyCallback).map_err(|_| {
        ConfigCreateError::CipherNotSupported {
            cipher_name: cipher_name.to_string(),
        }
    })
}

fn _generate_root_blob_id() -> BlobId {
    BlobId::new_random()
}

fn _generate_exclusive_client_id(
    my_client_id: ClientId,
    command_line_flags: &CommandLineFlags,
    console: &impl Console,
) -> Option<ClientId> {
    let single_client_mode = command_line_flags
        .missing_block_is_integrity_violation
        .unwrap_or_else(|| console.ask_single_client_mode_for_new_filesystem());
    if single_client_mode {
        Some(my_client_id)
    } else {
        None
    }
}

// TODO Tests
