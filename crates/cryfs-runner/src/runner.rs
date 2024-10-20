use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use cryfs_blobstore::{BlobId, BlobStoreOnBlocks};
use cryfs_blockstore::{
    AllowIntegrityViolations, BlockStore, ClientId, IntegrityConfig, InvalidBlockSizeError,
    LockingBlockStore, MissingBlockIsIntegrityViolation, OnDiskBlockStore,
};
use cryfs_cli_utils::{
    setup_blockstore_stack, BlockstoreCallback, CliError, CliErrorKind, CliResultExt,
    CliResultExtFn,
};
use cryfs_filesystem::{config::CryConfig, filesystem::CryDevice, localstate::LocalStateDir};
use cryfs_rustfs::backend::fuser;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CreateOrLoad {
    CreateNewFilesystem,
    LoadExistingFilesystem,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MountArgs {
    pub basedir: PathBuf,
    pub mountdir: PathBuf,
    pub config: CryConfig,
    pub allow_integrity_violations: AllowIntegrityViolations,
    pub create_or_load: CreateOrLoad,
    pub my_client_id: ClientId,
    pub local_state_dir: LocalStateDir,
}

/// On error: will return the error
/// On success: will call on_successfully_mounted and then block until the filesystem is unmounted, then return Ok.
pub async fn mount_filesystem(
    mount_args: MountArgs,
    on_successfully_mounted: impl FnOnce() + Send + Sync,
) -> Result<(), CliError> {
    let missing_block_is_integrity_violation =
        if mount_args.config.missingBlockIsIntegrityViolation() {
            MissingBlockIsIntegrityViolation::IsAViolation
        } else {
            MissingBlockIsIntegrityViolation::IsNotAViolation
        };
    setup_blockstore_stack(
        OnDiskBlockStore::new(mount_args.basedir.to_owned()),
        &mount_args.config,
        mount_args.my_client_id,
        &mount_args.local_state_dir,
        IntegrityConfig {
            allow_integrity_violations: mount_args.allow_integrity_violations,
            missing_block_is_integrity_violation,
            on_integrity_violation: Box::new(|err| {
                // TODO
            }),
        },
        FilesystemRunner {
            mountdir: &mount_args.mountdir,
            config: &mount_args.config,
            create_or_load: mount_args.create_or_load,
            on_successfully_mounted,
        },
    )
    .await??;

    Ok(())
}

struct FilesystemRunner<'m, 'c, OnSuccessfullyMounted: FnOnce()> {
    pub mountdir: &'m Path,
    pub config: &'c CryConfig,
    pub create_or_load: CreateOrLoad,
    pub on_successfully_mounted: OnSuccessfullyMounted,
}

impl<'m, 'c, OnSuccessfullyMounted: FnOnce()> BlockstoreCallback
    for FilesystemRunner<'m, 'c, OnSuccessfullyMounted>
{
    type Result = Result<(), CliError>;

    async fn callback<B: BlockStore + Send + Sync + AsyncDrop + 'static>(
        self,
        blockstore: AsyncDropGuard<LockingBlockStore<B>>,
    ) -> Self::Result {
        // TODO No unwrap. Should we instead change blocksize_bytes in the config file struct?
        let mut blobstore = BlobStoreOnBlocks::new(
            blockstore,
            u32::try_from(self.config.blocksize_bytes).unwrap(),
        )
        .await
        .map_cli_error(|_: &InvalidBlockSizeError| CliErrorKind::UnspecifiedError)?;

        let root_blob_id = BlobId::from_hex(&self.config.root_blob);
        let root_blob_id = match root_blob_id {
            Ok(root_blob_id) => root_blob_id,
            Err(e) => {
                if let Err(err) = blobstore.async_drop().await {
                    log::error!("Error while dropping blockstore: {:?}", err);
                }
                return Err(e)
                    .context("Error parsing root blob id")
                    .map_cli_error(CliErrorKind::InvalidFilesystem);
            }
        };

        let device = match self.create_or_load {
            CreateOrLoad::CreateNewFilesystem => {
                CryDevice::create_new_filesystem(blobstore, root_blob_id)
                    .await
                    .map_cli_error(CliErrorKind::UnspecifiedError)?
            }
            CreateOrLoad::LoadExistingFilesystem => {
                CryDevice::load_filesystem(blobstore, root_blob_id)
            }
        };

        let fs = |_uid, _gid| device;
        fuser::mount(
            fs,
            self.mountdir,
            tokio::runtime::Handle::current(),
            self.on_successfully_mounted,
        )
        .map_cli_error(|_| CliErrorKind::UnspecifiedError)?;
        Ok(())
    }
}

// TODO Tests
