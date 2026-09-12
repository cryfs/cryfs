use std::fmt::Debug;
use std::path::Path;
use tokio_util::sync::CancellationToken;

use super::{RunningFilesystem, backend_adapter::BackendAdapter};
use crate::common::FsError;
use crate::low_level_api::AsyncFilesystemLL;
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
    Fs: AsyncFilesystemLL + AsyncDrop<Error = FsError> + Debug + Send + Sync + 'static,
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
    Fs: AsyncFilesystemLL + AsyncDrop<Error = FsError> + Debug + Send + Sync + 'static,
{
    let backend = BackendAdapter::new(fs, runtime);

    // We need to keep a handle to the internal arc because we need to manually async drop it if fuser::spawn_mount2 fails.
    // This is because usually, the internal Arc is dropped in BackendAdapter::destroy() but if fuser::spawn_mount2 fails,
    // it will not call destroy().
    let backend_internal_arc = backend.internal_arc();

    let session = fuser::spawn_mount2(backend, mountpoint, config);
    let session = match session {
        Ok(session) => {
            std::mem::drop(backend_internal_arc);
            session
        }
        Err(e) => {
            // Keep the write lock (`fs_lock`) until we're done so that no operation runs concurrently with the drop.
            let mut fs_lock = backend_internal_arc.write().await;
            let fs = fs_lock
                .take()
                .expect("spawn_mount2 failed without calling destroy(), so the file system must still be alive");
            fs.destroy().await;
            fs.async_drop().await.unwrap();
            return Err(e);
        }
    };

    Ok(RunningFilesystem::new(session))
}
