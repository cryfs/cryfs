use fuse_mt::FuseMT;
use std::fmt::Debug;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::{Arc, Mutex};

use super::{RunningFilesystem, backend_adapter::BackendAdapter};
use crate::common::FsError;
use crate::high_level_api::{AsyncFilesystem, IntoFs};
use cryfs_utils::async_drop::AsyncDrop;

// TODO fuser backend has two more arguments that are missing here: mount_options, on_successfully_mounted

pub async fn mount<Fs>(
    fs: impl IntoFs<Fs>,
    mountpoint: impl AsRef<Path>,
    runtime: tokio::runtime::Handle,
) -> std::io::Result<()>
where
    Fs: AsyncFilesystem + AsyncDrop<Error = FsError> + Debug + Send + Sync + 'static,
{
    let fs = spawn_mount(fs, mountpoint, runtime).await?;
    fs.block_until_unmounted();
    Ok(())
}

pub async fn spawn_mount<Fs>(
    fs: impl IntoFs<Fs>,
    mountpoint: impl AsRef<Path>,
    runtime: tokio::runtime::Handle,
) -> std::io::Result<RunningFilesystem>
where
    Fs: AsyncFilesystem + AsyncDrop<Error = FsError> + Debug + Send + Sync + 'static,
{
    let backend = BackendAdapter::new(fs.into_fs(), runtime);

    // We need to keep a handle to the internal arc because we need to manually async drop it if fuser::spawn_mount2 fails.
    // This is because usually, the internal Arc is dropped in BackendAdapter::destroy() but if fuser::spawn_mount2 fails,
    // it will not call destroy().
    let backend_internal_arc = backend.internal_arc();

    let fs = FuseMT::new(backend, num_threads());

    // TODO Fuse args (e.g. filesystem name)
    let session = fuse_mt::spawn_mount(fs, mountpoint, &[]);
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
    let session = Arc::new(Mutex::new(Some(session)));

    Ok(RunningFilesystem::new(session))
}

fn num_threads() -> usize {
    std::thread::available_parallelism()
        .unwrap_or(NonZeroUsize::new(2).unwrap())
        .get()
}
