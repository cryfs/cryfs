use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::time::Duration;
use std::{
    path::{Path, PathBuf},
    sync::atomic::Ordering,
};
use tokio_util::sync::CancellationToken;

use cryfs_blobstore::{BlobId, BlobStore, BlobStoreOnBlocks};
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
    pub unmount_idle: Option<Duration>,
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
            unmount_idle: mount_args.unmount_idle,
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
    pub unmount_idle: Option<Duration>,
}

impl<'m, 'c, OnSuccessfullyMounted: FnOnce()> BlockstoreCallback
    for FilesystemRunner<'m, 'c, OnSuccessfullyMounted>
{
    type Result = Result<(), CliError>;

    async fn callback<B: BlockStore + Send + Sync + AsyncDrop + 'static>(
        self,
        blockstore: AsyncDropGuard<LockingBlockStore<B>>,
    ) -> Self::Result {
        let mut blobstore = BlobStoreOnBlocks::new(blockstore, self.config.blocksize)
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

        // TODO Test unmounting after idle works correctly
        let unmount_trigger = self.unmount_idle.map(|unmount_idle| {
            make_unmount_trigger(
                &device,
                // TODO Pass in config of how long to wait before unmounting and only set this up if requested on the CLI
                unmount_idle,
            )
        });

        let fs = |_uid, _gid| device;
        fuser::mount(
            fs,
            self.mountdir,
            tokio::runtime::Handle::current(),
            unmount_trigger,
            self.on_successfully_mounted,
        )
        .map_cli_error(|_| CliErrorKind::UnspecifiedError)?;
        Ok(())
    }
}

/// Make a trigger that will cancel after the filesystem is inactive for a certain amount of time.
fn make_unmount_trigger<B>(
    device: &CryDevice<B>,
    unmount_after_idle_for: Duration,
) -> CancellationToken
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
{
    let last_filesystem_access_time = device.last_access_time();

    let unmount_trigger = CancellationToken::new();
    let unmount_trigger_clone = unmount_trigger.clone();
    tokio::task::spawn(async move {
        loop {
            if last_filesystem_access_time
                .load(Ordering::Relaxed)
                .elapsed()
                > unmount_after_idle_for
            {
                unmount_trigger_clone.cancel();
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });
    unmount_trigger
}

// TODO Tests
