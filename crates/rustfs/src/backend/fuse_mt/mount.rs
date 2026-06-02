use fuse_mt::FuseMT;
use std::fmt::Debug;
use std::num::NonZeroUsize;
use std::path::Path;
use tokio_util::sync::CancellationToken;

use super::{RunningFilesystem, backend_adapter::BackendAdapter};
use crate::common::FsError;
use crate::high_level_api::AsyncFilesystem;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

pub async fn mount<Fs>(
    fs: AsyncDropGuard<Fs>,
    mountpoint: impl AsRef<Path>,
    runtime: tokio::runtime::Handle,
    unmount_trigger: Option<CancellationToken>,
    config: &fuser::Config,
    on_successfully_mounted: impl FnOnce(),
) -> std::io::Result<()>
where
    Fs: AsyncFilesystem + AsyncDrop<Error = FsError> + Debug + Send + Sync + 'static,
{
    let fs = spawn_mount(fs, mountpoint, runtime, config).await?;
    on_successfully_mounted();

    if let Some(unmount_trigger) = unmount_trigger {
        fs.unmount_on_trigger(unmount_trigger);
    }

    fs.block_until_unmounted();
    Ok(())
}

pub async fn spawn_mount<Fs>(
    fs: AsyncDropGuard<Fs>,
    mountpoint: impl AsRef<Path>,
    runtime: tokio::runtime::Handle,
    config: &fuser::Config,
) -> std::io::Result<RunningFilesystem>
where
    Fs: AsyncFilesystem + AsyncDrop<Error = FsError> + Debug + Send + Sync + 'static,
{
    let backend = BackendAdapter::new(fs, runtime);

    // We need to keep a handle to the internal arc because we need to manually async drop it if fuser::spawn_mount2 fails.
    // This is because usually, the internal Arc is dropped in BackendAdapter::destroy() but if fuser::spawn_mount2 fails,
    // it will not call destroy().
    let backend_internal_arc = backend.internal_arc();

    let fs = FuseMT::new(backend, num_threads());

    // TODO Fuse args (e.g. filesystem name)
    // `FuseMT` implements fuser 0.16's `Filesystem`, so we mount via the 0.16-pinned `fuser_fusemt`,
    // translating the shared (fuser 0.17) `Config` into fuser-0.16 mount options.
    let mount_options = config_to_fuser16_options(config);
    let session = fuser_fusemt::spawn_mount2(fs, mountpoint, &mount_options);
    let session = match session {
        Ok(session) => {
            std::mem::drop(backend_internal_arc);
            session
        }
        Err(e) => {
            let mut backend_internal_arc = backend_internal_arc.write().await;
            backend_internal_arc.destroy().await;
            backend_internal_arc.async_drop().await.unwrap();
            return Err(e);
        }
    };

    Ok(RunningFilesystem::new(session))
}

fn num_threads() -> usize {
    std::thread::available_parallelism()
        .unwrap_or_else(|err| {
            log::warn!("Could not determine number of cpu cores. Falling back to a parallelism factor of 2. Error: {err:?}");
            NonZeroUsize::new(2).unwrap()
        })
        .get()
}

/// The `fuse_mt` crate is still on fuser 0.16, whose `spawn_mount2` takes `&[MountOption]` (not the
/// structured `Config` introduced in 0.17) and whose `MountOption` still carries the `AllowOther` /
/// `AllowRoot` variants that 0.17 moved into `Config::acl`. Translate the shared (fuser 0.17) `Config`
/// the `RustfsBackend` trait passes us into the fuser-0.16 mount options `fuser_fusemt` expects.
fn config_to_fuser16_options(config: &fuser::Config) -> Vec<fuser_fusemt::MountOption> {
    let mut options: Vec<fuser_fusemt::MountOption> = config
        .mount_options
        .iter()
        .map(convert_mount_option)
        .collect();
    match config.acl {
        fuser::SessionACL::All => options.push(fuser_fusemt::MountOption::AllowOther),
        fuser::SessionACL::RootAndOwner => options.push(fuser_fusemt::MountOption::AllowRoot),
        fuser::SessionACL::Owner => {}
    }
    options
}

fn convert_mount_option(option: &fuser::MountOption) -> fuser_fusemt::MountOption {
    // fuser 0.17's `MountOption` variants are a subset of 0.16's (0.16 also has AllowOther/AllowRoot,
    // handled separately via `Config::acl`), so this is an exhaustive 1:1 mapping.
    use fuser::MountOption as New;
    use fuser_fusemt::MountOption as Old;
    match option {
        New::FSName(name) => Old::FSName(name.clone()),
        New::Subtype(subtype) => Old::Subtype(subtype.clone()),
        New::CUSTOM(custom) => Old::CUSTOM(custom.clone()),
        New::AutoUnmount => Old::AutoUnmount,
        New::DefaultPermissions => Old::DefaultPermissions,
        New::Dev => Old::Dev,
        New::NoDev => Old::NoDev,
        New::Suid => Old::Suid,
        New::NoSuid => Old::NoSuid,
        New::RO => Old::RO,
        New::RW => Old::RW,
        New::Exec => Old::Exec,
        New::NoExec => Old::NoExec,
        New::Atime => Old::Atime,
        New::NoAtime => Old::NoAtime,
        New::DirSync => Old::DirSync,
        New::Sync => Old::Sync,
        New::Async => Old::Async,
    }
}
