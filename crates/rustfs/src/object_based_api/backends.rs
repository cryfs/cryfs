use std::fmt::Debug;
use std::path::Path;

use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};
use tokio_util::sync::CancellationToken;

use crate::FsError;

pub use fuser::MountOption;

use crate::{
    Gid, Uid,
    backend::{BackgroundSession, RunningFilesystem},
    object_based_api::Device,
};

/// Abstracts over the backend that can run a [Device] based file system.
pub trait RustfsBackend: Send + Sync {
    type BackgroundSession: BackgroundSession + Send + 'static;

    async fn mount<Fs>(
        fs: impl FnOnce(Uid, Gid) -> AsyncDropGuard<Fs> + Send + Sync + 'static,
        mountpoint: impl AsRef<Path>,
        runtime: tokio::runtime::Handle,
        unmount_trigger: Option<CancellationToken>,
        mount_options: &[MountOption],
        on_successfully_mounted: impl FnOnce(),
    ) -> std::io::Result<()>
    where
        Fs: Device + AsyncDrop<Error = FsError> + Send + Sync + Debug + 'static,
        for<'a> Fs::File<'a>: Send,
        Fs::OpenFile: Send + Sync;

    async fn spawn_mount<Fs>(
        fs: impl FnOnce(Uid, Gid) -> AsyncDropGuard<Fs> + Send + Sync + 'static,
        mountpoint: impl AsRef<Path>,
        runtime: tokio::runtime::Handle,
        mount_options: &[MountOption],
    ) -> std::io::Result<RunningFilesystem<Self::BackgroundSession>>
    where
        Fs: Device + AsyncDrop<Error = FsError> + Send + Sync + Debug + 'static,
        for<'a> Fs::File<'a>: Send,
        Fs::OpenFile: Send + Sync;
}

#[cfg(feature = "fuser")]
pub struct RustfsFuserBackend;
#[cfg(feature = "fuser")]
impl RustfsBackend for RustfsFuserBackend {
    type BackgroundSession = fuser::BackgroundSession;

    async fn mount<Fs>(
        fs: impl FnOnce(Uid, Gid) -> AsyncDropGuard<Fs> + Send + Sync + 'static,
        mountpoint: impl AsRef<Path>,
        runtime: tokio::runtime::Handle,
        unmount_trigger: Option<CancellationToken>,
        mount_options: &[MountOption],
        on_successfully_mounted: impl FnOnce(),
    ) -> std::io::Result<()>
    where
        Fs: Device + AsyncDrop<Error = FsError> + Send + Sync + Debug + 'static,
        for<'a> Fs::File<'a>: Send,
        Fs::OpenFile: Send + Sync,
    {
        use crate::object_based_api::ObjectBasedFsAdapterLL;

        crate::backend::fuser::mount(
            ObjectBasedFsAdapterLL::new(fs),
            mountpoint,
            runtime,
            unmount_trigger,
            mount_options,
            on_successfully_mounted,
        )
        .await
    }

    async fn spawn_mount<Fs>(
        fs: impl FnOnce(Uid, Gid) -> AsyncDropGuard<Fs> + Send + Sync + 'static,
        mountpoint: impl AsRef<Path>,
        runtime: tokio::runtime::Handle,
        mount_options: &[MountOption],
    ) -> std::io::Result<RunningFilesystem<Self::BackgroundSession>>
    where
        Fs: Device + AsyncDrop<Error = FsError> + Send + Sync + Debug + 'static,
        for<'a> Fs::File<'a>: Send,
        Fs::OpenFile: Send + Sync,
    {
        use crate::object_based_api::ObjectBasedFsAdapterLL;

        crate::backend::fuser::spawn_mount(
            ObjectBasedFsAdapterLL::new(fs),
            mountpoint,
            runtime,
            mount_options,
        )
        .await
    }
}

#[cfg(feature = "fuse_mt")]
pub struct RustfsFusemtBackend;
#[cfg(feature = "fuse_mt")]
impl RustfsBackend for RustfsFusemtBackend {
    type BackgroundSession = fuser::BackgroundSession;

    async fn mount<Fs>(
        fs: impl FnOnce(Uid, Gid) -> AsyncDropGuard<Fs> + Send + Sync + 'static,
        mountpoint: impl AsRef<Path>,
        runtime: tokio::runtime::Handle,
        unmount_trigger: Option<CancellationToken>,
        mount_options: &[MountOption],
        on_successfully_mounted: impl FnOnce(),
    ) -> std::io::Result<()>
    where
        Fs: Device + AsyncDrop<Error = FsError> + Send + Sync + Debug + 'static,
        for<'a> Fs::File<'a>: Send,
        Fs::OpenFile: Send + Sync,
    {
        use crate::object_based_api::ObjectBasedFsAdapter;

        crate::backend::fuse_mt::mount(
            ObjectBasedFsAdapter::new(fs),
            mountpoint,
            runtime,
            unmount_trigger,
            mount_options,
            on_successfully_mounted,
        )
        .await
    }

    async fn spawn_mount<Fs>(
        fs: impl FnOnce(Uid, Gid) -> AsyncDropGuard<Fs> + Send + Sync + 'static,
        mountpoint: impl AsRef<Path>,
        runtime: tokio::runtime::Handle,
        mount_options: &[MountOption],
    ) -> std::io::Result<RunningFilesystem<Self::BackgroundSession>>
    where
        Fs: Device + AsyncDrop<Error = FsError> + Send + Sync + Debug + 'static,
        for<'a> Fs::File<'a>: Send,
        Fs::OpenFile: Send + Sync,
    {
        use crate::object_based_api::ObjectBasedFsAdapter;

        crate::backend::fuse_mt::spawn_mount(
            ObjectBasedFsAdapter::new(fs),
            mountpoint,
            runtime,
            mount_options,
        )
        .await
    }
}
