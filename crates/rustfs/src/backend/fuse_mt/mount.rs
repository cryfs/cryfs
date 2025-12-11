use fuse_mt::FuseMT;
use fuser::MountOption;
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
    mount_options: &[MountOption],
    on_successfully_mounted: impl FnOnce(),
) -> std::io::Result<()>
where
    Fs: AsyncFilesystem + AsyncDrop<Error = FsError> + Debug + Send + Sync + 'static,
{
    let fs = spawn_mount(fs, mountpoint, runtime, mount_options).await?;
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
    mount_options: &[MountOption],
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
    let session = fuser::spawn_mount2(fs, mountpoint, mount_options);
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
