use anyhow::{Context, Result};
use cryfs_blobstore::{BlobId, BlobStore, BlobStoreOnBlocks};
use cryfs_blockstore::{
    AllowIntegrityViolations, ClientId, IntegrityConfig, InvalidBlockSizeError, LLBlockStore,
    LockingBlockStore, MissingBlockIsIntegrityViolation, OnDiskBlockStore,
};
use cryfs_cli_utils::{
    BlockstoreCallback, CliError, CliErrorKind, CliResultExt, CliResultExtFn,
    setup_blockstore_stack,
};
use cryfs_config::{config::CryConfig, localstate::LocalStateDir};
use cryfs_filesystem::filesystem::CryDevice;
use cryfs_rustfs::AtimeUpdateBehavior;
use cryfs_rustfs::object_based_api::{Config, MountOption, RustfsBackend, SessionACL};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use super::unmount_trigger::{TriggerReason, UnmountTrigger};

// Run with the fuser backend. This can be switched to fuse-mt if desired.
type Backend = cryfs_rustfs::object_based_api::RustfsFuserBackend;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CreateOrLoad {
    CreateNewFilesystem,
    LoadExistingFilesystem,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MountArgs {
    pub vaultdir: PathBuf,
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
/// On success: will call `on_successfully_mounted` and then block until the filesystem is unmounted, then return Ok.
///
/// `on_successfully_mounted` runs once, right after the mount is established. If
/// it returns `Err` (e.g. the daemon failed to notify a parent CLI that has
/// since exited), the filesystem is unmounted again instead of served — we
/// don't keep a mount alive that nobody is waiting on.
pub(crate) async fn mount_filesystem(
    mount_args: MountArgs,
    on_successfully_mounted: impl FnOnce() -> Result<(), anyhow::Error> + Send + Sync,
) -> Result<(), CliError> {
    let missing_block_is_integrity_violation =
        if mount_args.config.missing_block_is_integrity_violation() {
            MissingBlockIsIntegrityViolation::IsAViolation
        } else {
            MissingBlockIsIntegrityViolation::IsNotAViolation
        };
    let unmount_trigger = UnmountTrigger::new();
    let unmount_trigger_clone = unmount_trigger.clone();
    let trigger_reason = Arc::clone(unmount_trigger.trigger_reason());

    // Wrap the caller's notification so a failure unmounts the filesystem
    // rather than leaving it mounted with no one waiting on it.
    let on_successfully_mounted =
        notify_or_unmount(on_successfully_mounted, unmount_trigger.clone());
    setup_blockstore_stack(
        OnDiskBlockStore::new(mount_args.vaultdir.to_owned()),
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
            vaultdir: &mount_args.vaultdir,
            mountdir: &mount_args.mountdir,
            config: &mount_args.config,
            create_or_load: mount_args.create_or_load,
            on_successfully_mounted,
            unmount_trigger,
            unmount_idle: mount_args.unmount_idle,
            fuse_options: mount_args.fuse_options,
            atime_behavior: mount_args.atime_behavior,
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
        // Intentional teardown because the caller vanished; there is no one to
        // report an error to, so this is not a failure.
        Some(TriggerReason::NotificationFailed) => Ok(()),
        Some(TriggerReason::IntegrityViolation(err)) => Err(CliError {
            error: Arc::new(err.into()),
            kind: CliErrorKind::IntegrityViolation,
        }),
    }
}

/// Wrap a mount-success notification so that if it fails — e.g. the parent CLI
/// that requested a background mount has since exited — the filesystem is
/// unmounted via `unmount_trigger` instead of being served with no one waiting
/// on it.
///
/// Triggering the (caller-owned) trigger here is safe even though the returned
/// closure runs *before* the mount wires up its trigger listener: the token is
/// level-triggered, so an already-cancelled state is observed as soon as the
/// listener attaches.
fn notify_or_unmount(
    on_successfully_mounted: impl FnOnce() -> Result<(), anyhow::Error> + Send + Sync,
    unmount_trigger: UnmountTrigger,
) -> impl FnOnce() + Send + Sync {
    move || {
        if let Err(err) = on_successfully_mounted() {
            log::warn!(
                "Filesystem mounted but notifying the caller failed ({err:#}); unmounting again"
            );
            unmount_trigger.trigger_now(TriggerReason::NotificationFailed);
        }
    }
}

struct FilesystemRunner<'v, 'm, 'c, OnSuccessfullyMounted: FnOnce()> {
    pub vaultdir: &'v Path,
    pub mountdir: &'m Path,
    pub config: &'c CryConfig,
    pub create_or_load: CreateOrLoad,
    pub on_successfully_mounted: OnSuccessfullyMounted,
    pub unmount_trigger: UnmountTrigger,
    pub unmount_idle: Option<Duration>,
    pub fuse_options: Box<[FuseOption]>,
    pub atime_behavior: AtimeUpdateBehavior,
}

impl<'v, 'm, 'c, OnSuccessfullyMounted: FnOnce()> BlockstoreCallback
    for FilesystemRunner<'v, 'm, 'c, OnSuccessfullyMounted>
{
    type Result = Result<(), CliError>;

    async fn callback<B: LLBlockStore + Send + Sync + AsyncDrop + 'static>(
        self,
        blockstore: AsyncDropGuard<LockingBlockStore<B>>,
    ) -> Self::Result {
        let blobstore = BlobStoreOnBlocks::new(blockstore, self.config.blocksize)
            .await
            .map_cli_error(|_: &InvalidBlockSizeError| CliErrorKind::UnspecifiedError)?;

        let device = make_device(
            blobstore,
            self.config,
            self.create_or_load,
            self.atime_behavior,
        )
        .await?;

        // TODO Test unmounting after idle works correctly
        if let Some(unmount_idle) = self.unmount_idle {
            self.unmount_trigger
                .trigger_after_idle_timeout(device.last_access_time(), unmount_idle);
        }

        let fs = |_uid, _gid| device;
        let fuse_atime_option = match self.atime_behavior {
            AtimeUpdateBehavior::Relatime
            | AtimeUpdateBehavior::Strictatime
            | AtimeUpdateBehavior::NodiratimeRelatime
            | AtimeUpdateBehavior::NodiratimeStrictatime => MountOption::Atime,
            AtimeUpdateBehavior::Noatime => MountOption::NoAtime,
        };
        // TODO How to set Config.n_threads and Config.clone_fd ? Would multiple threads be faster?
        let mut config = Config::default();
        config.mount_options = vec![
            MountOption::FSName(format!("cryfs@{}", self.vaultdir.display())),
            MountOption::Subtype("cryfs".to_string()),
            // let the kernel handle permission checking based on permission flags instead of calling the `access()` function of the fuse filesystem
            MountOption::DefaultPermissions,
            fuse_atime_option,
            // TODO What other MountOptions should we set (or let the user set on the command line)?
        ];
        // `SessionACL` is single-valued in fuser 0.17, unlike the old independent `AllowOther`/`AllowRoot`
        // mount options (where both could be set at once). Collapse order-independently to the most
        // permissive requested: `allow_other` (any user, which already subsumes root) wins over
        // `allow_root`; otherwise owner-only (FUSE's default).
        let allow_other = self
            .fuse_options
            .iter()
            .any(|o| matches!(o, FuseOption::AllowOther));
        let allow_root = self
            .fuse_options
            .iter()
            .any(|o| matches!(o, FuseOption::AllowRoot));
        config.acl = if allow_other {
            SessionACL::All
        } else if allow_root {
            SessionACL::RootAndOwner
        } else {
            SessionACL::Owner
        };
        Backend::mount(
            fs,
            self.mountdir,
            tokio::runtime::Handle::current(),
            Some(self.unmount_trigger.waiter()),
            &config,
            self.on_successfully_mounted,
        )
        .await
        .map_cli_error(|_| CliErrorKind::UnspecifiedError)?;
        Ok(())
    }
}

async fn make_device<B>(
    mut blobstore: AsyncDropGuard<B>,
    config: &CryConfig,
    create_or_load: CreateOrLoad,
    atime_behavior: AtimeUpdateBehavior,
) -> Result<AsyncDropGuard<CryDevice<B>>, CliError>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    B::ConcreteBlob: AsyncDrop<Error = anyhow::Error>,
{
    let root_blob_id = BlobId::from_hex(&config.root_blob);
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

    let mut device = match create_or_load {
        CreateOrLoad::CreateNewFilesystem => {
            CryDevice::create_new_filesystem(blobstore, root_blob_id, atime_behavior)
                .await
                .map_cli_error(CliErrorKind::UnspecifiedError)?
        }
        CreateOrLoad::LoadExistingFilesystem => {
            CryDevice::load_filesystem(blobstore, root_blob_id, atime_behavior)
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

    Ok(device)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notify_or_unmount_triggers_unmount_when_notification_fails() {
        // If the success notification fails (e.g. the parent CLI vanished), the
        // wrapper must fire the unmount trigger with NotificationFailed so the
        // mount is torn down instead of served.
        let trigger = UnmountTrigger::new();
        let wrapped = notify_or_unmount(|| Err(anyhow::anyhow!("parent gone")), trigger.clone());
        wrapped();

        assert!(
            trigger.waiter().is_cancelled(),
            "a failed notification must trigger an unmount"
        );
        assert!(
            matches!(
                *trigger.trigger_reason().lock().unwrap(),
                Some(TriggerReason::NotificationFailed)
            ),
            "the unmount reason must be NotificationFailed"
        );
    }

    #[test]
    fn notify_or_unmount_does_not_unmount_on_success() {
        let trigger = UnmountTrigger::new();
        let wrapped = notify_or_unmount(|| Ok(()), trigger.clone());
        wrapped();

        assert!(
            !trigger.waiter().is_cancelled(),
            "a successful notification must not trigger an unmount"
        );
        assert!(trigger.trigger_reason().lock().unwrap().is_none());
    }
}
