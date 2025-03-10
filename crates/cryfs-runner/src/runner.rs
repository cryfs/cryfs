use anyhow::{Context, Result};
use cryfs_blobstore::{BlobId, BlobStoreOnBlocks};
use cryfs_blockstore::{
    AllowIntegrityViolations, BlockStore, ClientId, IntegrityConfig, InvalidBlockSizeError,
    LockingBlockStore, MissingBlockIsIntegrityViolation, OnDiskBlockStore,
};
use cryfs_cli_utils::{
    BlockstoreCallback, CliError, CliErrorKind, CliResultExt, CliResultExtFn,
    setup_blockstore_stack,
};
use cryfs_filesystem::{config::CryConfig, filesystem::CryDevice, localstate::LocalStateDir};
use cryfs_rustfs::AtimeUpdateBehavior;
use cryfs_rustfs::backend::fuser::{self, MountOption};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use crate::unmount_trigger::{TriggerReason, UnmountTrigger};

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
    pub fuse_options: Box<[FuseOption]>,
    pub atime_behavior: AtimeUpdateBehavior,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FuseOption {
    AllowOther,
    AllowRoot,
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
    let unmount_trigger = UnmountTrigger::new();
    let unmount_trigger_clone = unmount_trigger.clone();
    let trigger_reason = Arc::clone(unmount_trigger.trigger_reason());
    setup_blockstore_stack(
        OnDiskBlockStore::new(mount_args.basedir.to_owned()),
        &mount_args.config,
        mount_args.my_client_id,
        &mount_args.local_state_dir,
        IntegrityConfig {
            allow_integrity_violations: mount_args.allow_integrity_violations,
            missing_block_is_integrity_violation,
            on_integrity_violation: Box::new(move |err| {
                unmount_trigger_clone.trigger_now(TriggerReason::IntegrityViolation(err.clone()));
            }),
        },
        FilesystemRunner {
            basedir: &mount_args.basedir,
            mountdir: &mount_args.mountdir,
            config: &mount_args.config,
            create_or_load: mount_args.create_or_load,
            on_successfully_mounted,
            unmount_trigger,
            unmount_idle: mount_args.unmount_idle,
            fuse_options: mount_args.fuse_options,
        },
    )
    .await??;

    let trigger_reason = trigger_reason.lock().unwrap().clone();
    match trigger_reason {
        None => {
            // Regular unmount, not triggered by unmount idle or an integrity violation
            Ok(())
        }
        Some(TriggerReason::UnmountIdle) => Ok(()),
        Some(TriggerReason::IntegrityViolation(err)) => Err(CliError {
            error: err.into(),
            kind: CliErrorKind::IntegrityViolation,
        }),
    }
}

struct FilesystemRunner<'b, 'm, 'c, OnSuccessfullyMounted: FnOnce()> {
    pub basedir: &'b Path,
    pub mountdir: &'m Path,
    pub config: &'c CryConfig,
    pub create_or_load: CreateOrLoad,
    pub on_successfully_mounted: OnSuccessfullyMounted,
    pub unmount_trigger: UnmountTrigger,
    pub unmount_idle: Option<Duration>,
    pub fuse_options: Box<[FuseOption]>,
}

impl<'b, 'm, 'c, OnSuccessfullyMounted: FnOnce()> BlockstoreCallback
    for FilesystemRunner<'b, 'm, 'c, OnSuccessfullyMounted>
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

        let mut device = match self.create_or_load {
            CreateOrLoad::CreateNewFilesystem => {
                CryDevice::create_new_filesystem(blobstore, root_blob_id)
                    .await
                    .map_cli_error(CliErrorKind::UnspecifiedError)?
            }
            CreateOrLoad::LoadExistingFilesystem => {
                CryDevice::load_filesystem(blobstore, root_blob_id)
            }
        };
        match device
            .sanity_check()
            .await
            .map_cli_error(CliErrorKind::InvalidFilesystem)
        {
            Ok(()) => {}
            Err(e) => {
                device.async_drop().await.unwrap();
                return Err(e);
            }
        }

        // TODO Test unmounting after idle works correctly
        if let Some(unmount_idle) = self.unmount_idle {
            self.unmount_trigger
                .trigger_after_idle_timeout(device.last_access_time(), unmount_idle);
        }

        let fs = |_uid, _gid| device;
        let mount_options = [
            MountOption::FSName(format!("cryfs@{}", self.basedir.display())),
            MountOption::Subtype("cryfs".to_string()),
        ]
        .into_iter()
        .chain(self.fuse_options.iter().map(|o| match o {
            FuseOption::AllowOther => MountOption::AllowOther,
            FuseOption::AllowRoot => MountOption::AllowRoot,
        }))
        .collect::<Box<[_]>>();
        fuser::mount(
            fs,
            self.mountdir,
            tokio::runtime::Handle::current(),
            Some(self.unmount_trigger.waiter()),
            &mount_options,
            self.on_successfully_mounted,
        )
        .await
        .map_cli_error(|_| CliErrorKind::UnspecifiedError)?;
        Ok(())
    }
}

// TODO Tests
