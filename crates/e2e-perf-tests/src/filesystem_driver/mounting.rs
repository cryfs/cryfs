use anyhow::Result;
use async_trait::async_trait;
use nix::sys::{stat::UtimensatFlags, time::TimeSpec};
use std::{
    io::SeekFrom,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, SystemTime},
};
use tempdir::TempDir;
use tokio::{
    fs::{self, File, OpenOptions},
    io::{AsyncReadExt as _, AsyncSeekExt as _, AsyncWriteExt as _},
};

use super::FilesystemDriver;
use cryfs_blobstore::{BlobStoreOnBlocks, TrackingBlobStore};
use cryfs_blockstore::{
    DynBlockStore, HLSharedBlockStore, HLTrackingBlockStore, LockingBlockStore,
};
use cryfs_filesystem::filesystem::CryDevice;
use cryfs_rustfs::{
    AbsolutePath, AbsolutePathBuf, FsError, FsResult, Gid, Mode, NodeAttrs, NodeKind, NumBytes,
    PathComponent, PathComponentBuf, Statfs, Uid,
    backend::{BackgroundSession, RunningFilesystem},
};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};
use std::os::unix::io::AsRawFd;

trait MountingBackend {
    type Session: BackgroundSession + Send + 'static;
    async fn spawn_mount(
        device: AsyncDropGuard<
            CryDevice<
                AsyncDropArc<
                    TrackingBlobStore<
                        BlobStoreOnBlocks<
                            HLSharedBlockStore<
                                HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>,
                            >,
                        >,
                    >,
                >,
            >,
        >,
        mountdir: &Path,
    ) -> FsResult<RunningFilesystem<Self::Session>>;
}
struct FuserBackend;
impl MountingBackend for FuserBackend {
    type Session = fuser::BackgroundSession;
    async fn spawn_mount(
        device: AsyncDropGuard<
            CryDevice<
                AsyncDropArc<
                    TrackingBlobStore<
                        BlobStoreOnBlocks<
                            HLSharedBlockStore<
                                HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>,
                            >,
                        >,
                    >,
                >,
            >,
        >,
        mountdir: &Path,
    ) -> FsResult<RunningFilesystem<Self::Session>> {
        let runtime = tokio::runtime::Handle::current();
        cryfs_rustfs::backend::fuser::spawn_mount(|_uid, _gid| device, mountdir, runtime, &[])
            .await
            .map_err(|error| FsError::InternalError {
                error: error.into(),
            })
    }
}
struct FusemtBackend;
impl MountingBackend for FusemtBackend {
    type Session = fuse_mt_fuser::BackgroundSession;
    async fn spawn_mount(
        device: AsyncDropGuard<
            CryDevice<
                AsyncDropArc<
                    TrackingBlobStore<
                        BlobStoreOnBlocks<
                            HLSharedBlockStore<
                                HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>,
                            >,
                        >,
                    >,
                >,
            >,
        >,
        mountdir: &Path,
    ) -> FsResult<RunningFilesystem<Self::Session>> {
        let runtime = tokio::runtime::Handle::current();
        cryfs_rustfs::backend::fuse_mt::spawn_mount(|_uid, _gid| device, mountdir, runtime, &[])
            .await
            .map_err(|error| FsError::InternalError {
                error: error.into(),
            })
    }
}

pub type FuserMountingFilesystemDriver = MountingFilesystemDriver<FuserBackend>;
pub type FusemtMountingFilesystemDriver = MountingFilesystemDriver<FusemtBackend>;

enum MaybeMounted<BS>
where
    BS: BackgroundSession + Send + 'static,
{
    Invalid, // temporary state, should never be persisted
    Mounted {
        fs: RunningFilesystem<BS>,
    },
    NotMounted {
        device: AsyncDropGuard<
            CryDevice<
                AsyncDropArc<
                    TrackingBlobStore<
                        BlobStoreOnBlocks<
                            HLSharedBlockStore<
                                HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>,
                            >,
                        >,
                    >,
                >,
            >,
        >,
    },
}

/// A FilesystemDriver implementation that mounts the filesystem using rustfs::mount into a tempdir
/// and then performs operations using syscalls on the tempdir.
///
/// This isn't that useful for counting operations, since the operating system may come in and run
/// its own operations on the filesystem, but it is useful for benchmarking.
pub struct MountingFilesystemDriver<B>
where
    B: MountingBackend,
{
    // The temporary directory where our filesystem is mounted
    mountdir: TempDir,
    // Keep a reference to the running filesystem to keep it mounted
    fs: Mutex<MaybeMounted<B::Session>>,
}

impl<B> std::fmt::Debug for MountingFilesystemDriver<B>
where
    B: MountingBackend,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MountingFilesystemDriver")
    }
}

impl<B> MountingFilesystemDriver<B>
where
    B: MountingBackend,
{
    fn real_path_for_node(&self, node: &Option<AbsolutePathBuf>) -> PathBuf {
        let mut path = self.mountdir.path().to_owned();
        if let Some(node) = node {
            path = path.join(node);
        }
        path
    }

    fn real_path_and_node_from_parent_and_name(
        &self,
        parent: Option<AbsolutePathBuf>,
        name: &PathComponent,
    ) -> (PathBuf, <Self as FilesystemDriver>::NodeHandle) {
        let node = parent.unwrap_or_else(AbsolutePathBuf::root).push(name);
        let real_path = self.real_path_for_node(&Some(node.clone()));
        (real_path, node)
    }
}

impl<B> FilesystemDriver for MountingFilesystemDriver<B>
where
    B: MountingBackend,
{
    type NodeHandle = AbsolutePathBuf;

    type FileHandle = File;

    async fn new(
        device: AsyncDropGuard<
            CryDevice<
                AsyncDropArc<
                    TrackingBlobStore<
                        BlobStoreOnBlocks<
                            HLSharedBlockStore<
                                HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>,
                            >,
                        >,
                    >,
                >,
            >,
        >,
    ) -> AsyncDropGuard<Self> {
        let mountdir = TempDir::new("cryfs-syscall-driver").unwrap();
        let fs = Mutex::new(MaybeMounted::NotMounted { device });
        AsyncDropGuard::new(Self { mountdir, fs })
    }

    async fn init(&self) -> FsResult<()> {
        let mut fs = self.fs.lock().unwrap();
        let MaybeMounted::NotMounted { device } =
            std::mem::replace(&mut *fs, MaybeMounted::Invalid)
        else {
            panic!("Filesystem is already mounted");
        };

        let new_fs = B::spawn_mount(device, self.mountdir.path()).await?;

        *fs = MaybeMounted::Mounted { fs: new_fs };
        Ok(())
    }

    async fn destroy(&self) {
        let mut fs = self.fs.lock().unwrap();
        let MaybeMounted::Mounted { fs } = std::mem::replace(&mut *fs, MaybeMounted::Invalid)
        else {
            panic!("Filesystem is not mounted");
        };
        fs.unmount_join();
    }

    async fn mkdir(
        &self,
        parent: Option<Self::NodeHandle>,
        name: &PathComponent,
    ) -> FsResult<Self::NodeHandle> {
        let (real_path, node) = self.real_path_and_node_from_parent_and_name(parent, name);

        fs::create_dir(&real_path)
            .await
            .map_err(|error| FsError::InternalError {
                error: error.into(),
            })?;

        Ok(node)
    }

    async fn mkdir_recursive(&self, path: &AbsolutePath) -> FsResult<Self::NodeHandle> {
        let real_path = self.real_path_for_node(&Some(path.to_owned()));

        // Create all directories in path
        fs::create_dir_all(&real_path)
            .await
            .map_err(|error| FsError::InternalError {
                error: error.into(),
            })?;

        Ok(path.to_owned())
    }

    async fn create_file(
        &self,
        parent: Option<Self::NodeHandle>,
        name: &PathComponent,
    ) -> FsResult<Self::NodeHandle> {
        let (new_file_node, fh) = self.create_and_open_file(parent, name).await?;
        // Close the file handle
        std::mem::drop(fh);

        Ok(new_file_node)
    }

    async fn create_and_open_file(
        &self,
        parent: Option<Self::NodeHandle>,
        name: &PathComponent,
    ) -> FsResult<(Self::NodeHandle, File)> {
        let (real_path, node) = self.real_path_and_node_from_parent_and_name(parent, name);

        let fh = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&real_path)
            .await
            .map_err(|error| FsError::InternalError {
                error: error.into(),
            })?;

        Ok((node, fh))
    }

    async fn create_symlink(
        &self,
        parent: Option<Self::NodeHandle>,
        name: &PathComponent,
        target: &AbsolutePath,
    ) -> FsResult<Self::NodeHandle> {
        let (real_path, node) = self.real_path_and_node_from_parent_and_name(parent, name);

        fs::symlink(target.as_str(), &real_path)
            .await
            .map_err(|error| FsError::InternalError {
                error: error.into(),
            })?;

        Ok(node)
    }

    async fn unlink(&self, parent: Option<Self::NodeHandle>, name: &PathComponent) -> FsResult<()> {
        let (real_path, _node) = self.real_path_and_node_from_parent_and_name(parent, name);

        fs::remove_file(&real_path)
            .await
            .map_err(|error| FsError::InternalError {
                error: error.into(),
            })?;

        Ok(())
    }

    async fn rmdir(&self, parent: Option<Self::NodeHandle>, name: &PathComponent) -> FsResult<()> {
        let (real_path, _node) = self.real_path_and_node_from_parent_and_name(parent, name);

        fs::remove_dir(&real_path)
            .await
            .map_err(|error| FsError::InternalError {
                error: error.into(),
            })?;

        Ok(())
    }

    async fn lookup(
        &self,
        parent: Option<Self::NodeHandle>,
        name: &PathComponent,
    ) -> FsResult<Self::NodeHandle> {
        let (real_path, node) = self.real_path_and_node_from_parent_and_name(parent, name);

        // Check if path exists
        match fs::try_exists(&real_path).await {
            Ok(true) => {}
            Ok(false) => return Err(FsError::NodeDoesNotExist),
            Err(error) => {
                return Err(FsError::InternalError {
                    error: error.into(),
                });
            }
        }

        Ok(node)
    }

    async fn getattr(&self, node: Option<Self::NodeHandle>) -> FsResult<NodeAttrs> {
        let real_path = self.real_path_for_node(&node);

        let metadata =
            fs::symlink_metadata(&real_path)
                .await
                .map_err(|error| FsError::InternalError {
                    error: error.into(),
                })?;

        Ok(metadata_to_node_attrs(metadata))
    }

    async fn fgetattr(&self, _node: Self::NodeHandle, open_file: &fs::File) -> FsResult<NodeAttrs> {
        let metadata = open_file
            .metadata()
            .await
            .map_err(|error| FsError::InternalError {
                error: error.into(),
            })?;

        Ok(metadata_to_node_attrs(metadata))
    }

    async fn chmod(&self, node: Option<Self::NodeHandle>, mode: Mode) -> FsResult<()> {
        let real_path = self.real_path_for_node(&node);

        asyncify(move || {
            let real_path = std::ffi::CString::new(real_path.to_str().unwrap()).unwrap();
            let mode = u32::from(mode);
            let result = unsafe { libc::chmod(real_path.as_ptr(), mode) };
            if 0 == result {
                Ok(())
            } else {
                Err(std::io::Error::last_os_error())
            }
        })
        .await
        .map_err(|error| FsError::InternalError {
            error: error.into(),
        })
    }

    async fn fchmod(
        &self,
        _node: Self::NodeHandle,
        open_file: &fs::File,
        mode: Mode,
    ) -> FsResult<()> {
        let raw_fd = open_file.as_raw_fd();
        asyncify(move || {
            let mode = u32::from(mode);
            nix::sys::stat::fchmod(raw_fd, nix::sys::stat::Mode::from_bits(mode).unwrap())
                .map_err(|error| std::io::Error::from_raw_os_error(error as i32))
        })
        .await
        .map_err(|error| FsError::InternalError {
            error: error.into(),
        })
    }

    async fn chown(
        &self,
        node: Option<Self::NodeHandle>,
        uid: Option<Uid>,
        gid: Option<Gid>,
    ) -> FsResult<()> {
        let real_path = self.real_path_for_node(&node);

        let uid = uid.map(u32::from);
        let gid = gid.map(u32::from);

        asyncify(move || std::os::unix::fs::lchown(&real_path, uid, gid))
            .await
            .map_err(|error| FsError::InternalError {
                error: error.into(),
            })
    }

    async fn fchown(
        &self,
        _node: Self::NodeHandle,
        open_file: &fs::File,
        uid: Option<Uid>,
        gid: Option<Gid>,
    ) -> FsResult<()> {
        let uid = uid.map(u32::from);
        let gid = gid.map(u32::from);
        let open_file = open_file.try_clone().await.unwrap();

        asyncify(move || std::os::unix::fs::fchown(open_file, uid, gid))
            .await
            .map_err(|error| FsError::InternalError {
                error: error.into(),
            })
    }

    async fn truncate(&self, node: Option<Self::NodeHandle>, size: NumBytes) -> FsResult<()> {
        let real_path = self.real_path_for_node(&node);
        let size = i64::try_from(u64::from(size)).unwrap();
        asyncify(move || {
            nix::unistd::truncate(&real_path, size)
                .map_err(|error| std::io::Error::from_raw_os_error(error as i32))
        })
        .await
        .map_err(|error| FsError::InternalError {
            error: error.into(),
        })
    }

    async fn ftruncate(
        &self,
        _node: Self::NodeHandle,
        open_file: &fs::File,
        size: NumBytes,
    ) -> FsResult<()> {
        let size = i64::try_from(u64::from(size)).unwrap();
        let open_file = open_file.try_clone().await.unwrap();
        asyncify(move || {
            nix::unistd::ftruncate(open_file, size)
                .map_err(|error| std::io::Error::from_raw_os_error(error as i32))
        })
        .await
        .map_err(|error| FsError::InternalError {
            error: error.into(),
        })
    }

    async fn utimens(
        &self,
        node: Option<Self::NodeHandle>,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
    ) -> FsResult<()> {
        let real_path = self.real_path_for_node(&node);
        let atime = atime.map(to_timespec).unwrap_or(TimeSpec::UTIME_OMIT);
        let mtime = mtime.map(to_timespec).unwrap_or(TimeSpec::UTIME_OMIT);

        asyncify(move || {
            nix::sys::stat::utimensat(
                None,
                &real_path,
                &atime,
                &mtime,
                UtimensatFlags::NoFollowSymlink,
            )
            .map_err(|error| std::io::Error::from_raw_os_error(error as i32))
        })
        .await
        .map_err(|error| FsError::InternalError {
            error: error.into(),
        })
    }

    async fn futimens(
        &self,
        _node: Self::NodeHandle,
        open_file: &fs::File,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
    ) -> FsResult<()> {
        let atime = atime.map(to_timespec).unwrap_or(TimeSpec::UTIME_OMIT);
        let mtime = mtime.map(to_timespec).unwrap_or(TimeSpec::UTIME_OMIT);
        let open_file = open_file.as_raw_fd();

        asyncify(move || {
            nix::sys::stat::futimens(open_file, &atime, &mtime)
                .map_err(|error| std::io::Error::from_raw_os_error(error as i32))
        })
        .await
        .map_err(|error| FsError::InternalError {
            error: error.into(),
        })
    }

    async fn readlink(&self, node: Self::NodeHandle) -> FsResult<AbsolutePathBuf> {
        let real_path = self.real_path_for_node(&Some(node));

        let target = fs::read_link(&real_path)
            .await
            .map_err(|error| FsError::InternalError {
                error: error.into(),
            })?;

        let target = AbsolutePathBuf::try_from_string(target.to_str().unwrap().to_string())
            .map_err(|_| FsError::InvalidPath)?;

        Ok(target)
    }

    async fn open(&self, node: Self::NodeHandle) -> FsResult<fs::File> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(false)
            .truncate(false)
            .open(&node)
            .await
            .map_err(|error| FsError::InternalError {
                error: error.into(),
            })
    }

    async fn release(&self, _node: Self::NodeHandle, open_file: fs::File) -> FsResult<()> {
        std::mem::drop(open_file);
        Ok(())
    }

    async fn statfs(&self) -> FsResult<Statfs> {
        let real_path = self.real_path_for_node(&Some(AbsolutePathBuf::root()));
        let result = asyncify(move || {
            nix::sys::statvfs::statvfs(&real_path)
                .map_err(|error| std::io::Error::from_raw_os_error(error as i32))
        })
        .await
        .map_err(|error| FsError::InternalError {
            error: error.into(),
        })?;

        Ok(Statfs {
            max_filename_length: u32::try_from(result.name_max()).unwrap(),
            blocksize: u32::try_from(result.block_size()).unwrap(),
            num_total_blocks: result.blocks(),
            num_free_blocks: result.blocks_free(),
            num_available_blocks: result.blocks_available(),
            num_total_inodes: result.files(),
            num_free_inodes: result.files_free(),
        })
    }

    async fn rename(
        &self,
        old_parent: Option<Self::NodeHandle>,
        old_name: &PathComponent,
        new_parent: Option<Self::NodeHandle>,
        new_name: &PathComponent,
    ) -> FsResult<()> {
        let (real_old_path, _old_node) =
            self.real_path_and_node_from_parent_and_name(old_parent, old_name);
        let (real_new_path, _new_node) =
            self.real_path_and_node_from_parent_and_name(new_parent, new_name);

        fs::rename(&real_old_path, &real_new_path)
            .await
            .map_err(|error| FsError::InternalError {
                error: error.into(),
            })?;

        Ok(())
    }

    async fn readdir(
        &self,
        node: Option<Self::NodeHandle>,
    ) -> FsResult<Vec<(PathComponentBuf, NodeKind)>> {
        let real_path = self.real_path_for_node(&node);

        let mut entries =
            fs::read_dir(&real_path)
                .await
                .map_err(|error| FsError::InternalError {
                    error: error.into(),
                })?;

        let mut result = Vec::new();
        while let Some(entry) =
            entries
                .next_entry()
                .await
                .map_err(|error| FsError::InternalError {
                    error: error.into(),
                })?
        {
            let path_component = PathComponentBuf::try_from_string(
                entry
                    .file_name()
                    .into_string()
                    .map_err(|_| FsError::InvalidPath)?,
            )
            .map_err(|_| FsError::InvalidPath)?;
            let kind = entry
                .file_type()
                .await
                .map_err(|error| FsError::InternalError {
                    error: error.into(),
                })?;
            let kind = if kind.is_dir() {
                NodeKind::Dir
            } else if kind.is_file() {
                NodeKind::File
            } else if kind.is_symlink() {
                NodeKind::Symlink
            } else {
                panic!("Unknown file type");
            };
            result.push((path_component, kind));
        }

        Ok(result)
    }

    async fn read(
        &self,
        _node: Self::NodeHandle,
        open_file: &mut fs::File,
        offset: NumBytes,
        size: NumBytes,
    ) -> FsResult<Vec<u8>> {
        open_file
            .seek(SeekFrom::Start(u64::from(offset)))
            .await
            .map_err(|error| FsError::InternalError {
                error: error.into(),
            })?;

        let mut buf = vec![0; usize::try_from(u64::from(size)).unwrap()];
        open_file
            .read(&mut buf)
            .await
            .map_err(|error| FsError::InternalError {
                error: error.into(),
            })?;

        Ok(buf)
    }

    async fn write(
        &self,
        _node: Self::NodeHandle,
        open_file: &mut fs::File,
        offset: NumBytes,
        data: Vec<u8>,
    ) -> FsResult<()> {
        open_file
            .seek(SeekFrom::Start(u64::from(offset)))
            .await
            .map_err(|error| FsError::InternalError {
                error: error.into(),
            })?;

        open_file
            .write_all(&data)
            .await
            .map_err(|error| FsError::InternalError {
                error: error.into(),
            })?;

        Ok(())
    }

    async fn flush(&self, _node: Self::NodeHandle, open_file: &mut fs::File) -> FsResult<()> {
        // Technically, fuse `flush` is closing the file, and flushing the stream isn't really the same thing.
        // But since we only use this for performance tests, we can probably keep it this way.
        open_file
            .flush()
            .await
            .map_err(|error| FsError::InternalError {
                error: error.into(),
            })?;
        Ok(())
    }

    async fn fsync(
        &self,
        _node: Self::NodeHandle,
        open_file: &mut fs::File,
        datasync: bool,
    ) -> FsResult<()> {
        if datasync {
            open_file
                .sync_data()
                .await
                .map_err(|error| FsError::InternalError {
                    error: error.into(),
                })?;
        } else {
            open_file
                .sync_all()
                .await
                .map_err(|error| FsError::InternalError {
                    error: error.into(),
                })?;
        }
        Ok(())
    }
}

#[async_trait]
impl<B> AsyncDrop for MountingFilesystemDriver<B>
where
    B: MountingBackend,
{
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<()> {
        // The filesystem will be unmounted when RunningFilesystem is dropped
        Ok(())
    }
}

fn metadata_to_node_attrs(metadata: std::fs::Metadata) -> NodeAttrs {
    let mode = Mode::from(metadata.permissions().mode());
    if metadata.is_dir() {
        assert!(metadata.file_type().is_dir());
    } else if metadata.is_file() {
        assert!(metadata.file_type().is_file());
    } else if metadata.file_type().is_symlink() {
        assert!(metadata.file_type().is_symlink());
    }

    NodeAttrs {
        mode,
        nlink: metadata.nlink() as u32,
        uid: Uid::from(metadata.uid()),
        gid: Gid::from(metadata.gid()),
        num_bytes: NumBytes::from(metadata.len()),
        num_blocks: None,
        atime: SystemTime::UNIX_EPOCH + Duration::from_secs(metadata.atime() as u64),
        mtime: SystemTime::UNIX_EPOCH + Duration::from_secs(metadata.mtime() as u64),
        ctime: SystemTime::UNIX_EPOCH + Duration::from_secs(metadata.ctime() as u64),
    }
}

// Copied from [tokio::fs::asyncify]
async fn asyncify<F, T>(f: F) -> std::io::Result<T>
where
    F: FnOnce() -> std::io::Result<T> + Send + 'static,
    T: Send + 'static,
{
    match tokio::task::spawn_blocking(f).await {
        Ok(res) => res,
        Err(_) => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "background task failed",
        )),
    }
}

fn to_timespec(time: SystemTime) -> TimeSpec {
    let duration = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0));
    TimeSpec::from_duration(duration)
}
