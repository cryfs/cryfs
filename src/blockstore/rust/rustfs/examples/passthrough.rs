use async_trait::async_trait;
use cryfs_rustfs::{
    Data, Device, Dir, DirEntry, File, FsError, FsResult, Gid, Mode, Node, NodeAttrs, NodeKind,
    NumBytes, OpenFile, OpenFlags, Statfs, Symlink, Uid,
};
use std::fs::Metadata;
use std::os::fd::AsRawFd;
use std::os::linux::fs::MetadataExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// TODO Go through all API calls we're doing (e.g. std::fs, tokio::fs, nix::) and make sure we're using the API correctly
//      and handle errors that can happen.

struct PassthroughDevice {
    basedir: PathBuf,
}

impl PassthroughDevice {
    fn apply_basedir(&self, path: &Path) -> PathBuf {
        assert!(path.is_absolute());
        let path = path.strip_prefix("/").unwrap();
        assert!(!path.is_absolute());
        let node_path = self.basedir.join(path);
        // Assert node_path doesn't escape the basedir
        // TODO Assert is probably a bad choice here. What should we do instead? Return an error?
        assert!(
            node_path.starts_with(&self.basedir),
            "Path {} escaped basedir {}",
            node_path.display(),
            self.basedir.display()
        );
        node_path
    }
}

#[async_trait]
impl Device for PassthroughDevice {
    type Node = PassthroughNode;
    type Dir = PassthroughDir;
    type Symlink = PassthroughSymlink;
    type File = PassthroughFile;
    type OpenFile = PassthroughOpenFile;

    async fn load_node(&self, path: &Path) -> FsResult<Self::Node> {
        let path = self.apply_basedir(path);
        Ok(PassthroughNode { path })
    }

    async fn load_dir(&self, path: &Path) -> FsResult<Self::Dir> {
        let path = self.apply_basedir(path);
        Ok(PassthroughDir { path })
    }

    async fn load_symlink(&self, path: &Path) -> FsResult<Self::Symlink> {
        let path = self.apply_basedir(path);
        Ok(PassthroughSymlink { path })
    }

    async fn load_file(&self, path: &Path) -> FsResult<Self::File> {
        let path = self.apply_basedir(path);
        Ok(PassthroughFile { path })
    }

    async fn statfs(&self) -> FsResult<Statfs> {
        let path = self.basedir.clone();
        tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                // TODO Make this platform independent
                let stat = nix::sys::statfs::statfs(&path).map_error()?;
                Ok(convert_statfs(stat))
            })
            .await
            .map_err(|_: tokio::task::JoinError| FsError::UnknownError)?
    }
}

struct PassthroughNode {
    path: PathBuf,
}

#[async_trait]
impl Node for PassthroughNode {
    async fn getattr(&self) -> FsResult<NodeAttrs> {
        let metadata = tokio::fs::symlink_metadata(&self.path).await.map_error()?;
        convert_metadata(metadata)
    }

    async fn chmod(&self, mode: Mode) -> FsResult<()> {
        let path = self.path.join(&self.path);
        let permissions = std::fs::Permissions::from_mode(mode.into());
        tokio::fs::set_permissions(path, permissions)
            .await
            .map_error()?;
        Ok(())
    }

    async fn chown(&self, uid: Option<Uid>, gid: Option<Gid>) -> FsResult<()> {
        let path = self.path.join(&self.path);
        let uid = uid.map(|uid| nix::unistd::Uid::from_raw(uid.into()));
        let gid = gid.map(|gid| nix::unistd::Gid::from_raw(gid.into()));
        let _: () = tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                // TODO Make this platform independent
                nix::unistd::fchownat(
                    None,
                    &path,
                    uid,
                    gid,
                    nix::unistd::FchownatFlags::NoFollowSymlink,
                )
                .map_error()?;
                Ok(())
            })
            .await
            .map_err(|_: tokio::task::JoinError| FsError::UnknownError)??;
        Ok(())
    }

    async fn utimens(
        &self,
        last_access: Option<SystemTime>,
        last_modification: Option<SystemTime>,
    ) -> FsResult<()> {
        let path = self.path.join(&self.path);
        tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                let (atime, mtime) = match (last_access, last_modification) {
                    (Some(atime), Some(mtime)) => {
                        // Both atime and mtime are being overwritten, no need to load previous values
                        (atime, mtime)
                    }
                    (atime, mtime) => {
                        // Either atime or mtime are not being overwritten, we need to load the previous values first.
                        let metadata = path.metadata().map_error()?;
                        let atime = match atime {
                            Some(atime) => atime,
                            None => metadata.accessed().map_error()?,
                        };
                        let mtime = match mtime {
                            Some(mtime) => mtime,
                            None => metadata.modified().map_error()?,
                        };
                        (atime, mtime)
                    }
                };
                nix::sys::stat::utimensat(
                    None,
                    &path,
                    &convert_timespec(atime),
                    &convert_timespec(mtime),
                    nix::sys::stat::UtimensatFlags::NoFollowSymlink,
                )
                .map_error()?;
                Ok(())
            })
            .await
            .map_err(|_: tokio::task::JoinError| FsError::UnknownError)??;
        Ok(())
    }
}

struct PassthroughDir {
    path: PathBuf,
}

#[async_trait]
impl Dir for PassthroughDir {
    type Device = PassthroughDevice;

    async fn entries(&self) -> FsResult<Vec<DirEntry>> {
        let mut entries = Vec::new();
        let mut dir = tokio::fs::read_dir(&self.path).await.map_error()?;
        while let Some(entry) = dir.next_entry().await.map_error()? {
            let name = entry.file_name();
            let name = name.to_string_lossy().into_owned(); // TODO Is to_string_lossy the best way to convert from OsString to String?
            let node_type = entry.file_type().await.map_error()?;
            let kind = if node_type.is_file() {
                NodeKind::File
            } else if node_type.is_dir() {
                NodeKind::Dir
            } else if node_type.is_symlink() {
                NodeKind::Symlink
            } else {
                panic!(
                    "Unknown node type in {path:?} : {entry:?}",
                    path = self.path,
                );
            };
            entries.push(DirEntry { name, kind });
        }
        Ok(entries)
    }

    async fn create_child_dir(
        &self,
        name: &str,
        mode: Mode,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<NodeAttrs> {
        let path = self.path.join(name);
        let path_clone = path.clone();
        let _: () = tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                // TODO Make this platform independent
                // TODO Don't use unwrap
                nix::unistd::mkdir(
                    &path_clone,
                    nix::sys::stat::Mode::from_bits(mode.into()).unwrap(),
                )
                .map_error()?;
                nix::unistd::chown(
                    &path_clone,
                    Some(nix::unistd::Uid::from_raw(uid.into())),
                    Some(nix::unistd::Gid::from_raw(gid.into())),
                )
                .map_error()?;
                Ok(())
            })
            .await
            .map_err(|_: tokio::task::JoinError| FsError::UnknownError)??;
        // TODO Return value directly without another call but make sure it returns the same value
        PassthroughNode { path }.getattr().await
    }

    async fn remove_child_dir(&self, name: &str) -> FsResult<()> {
        let path = self.path.join(name);
        tokio::fs::remove_dir(path).await.map_error()?;
        Ok(())
    }

    async fn create_child_symlink(
        &self,
        name: &str,
        target: &Path,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<NodeAttrs> {
        let path = self.path.join(name);
        let path_clone = path.clone();
        let target = target.to_owned();
        let _: () = tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                // TODO Make this platform independent
                std::os::unix::fs::symlink(&target, &path_clone).map_error()?;
                nix::unistd::fchownat(
                    None,
                    &path_clone,
                    Some(nix::unistd::Uid::from_raw(uid.into())),
                    Some(nix::unistd::Gid::from_raw(gid.into())),
                    nix::unistd::FchownatFlags::NoFollowSymlink,
                )
                .map_error()?;
                Ok(())
            })
            .await
            .map_err(|_: tokio::task::JoinError| FsError::UnknownError)??;
        // TODO Return value directly without another call but make sure it returns the same value
        PassthroughNode { path }.getattr().await
    }

    async fn remove_child_file_or_symlink(&self, name: &str) -> FsResult<()> {
        let path = self.path.join(name);
        tokio::fs::remove_file(path).await.map_error()?;
        Ok(())
    }

    async fn create_and_open_file(
        &self,
        name: &str,
        mode: Mode,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<(NodeAttrs, PassthroughOpenFile)> {
        let path = self.path.join(name);
        tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                let open_file = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(mode.into())
                    .open(&path)
                    .map_error()?;
                // TODO Can we compute the Metadata without asking the underlying file system? We just created the file after all.
                let metadata = open_file.metadata().map_error()?;
                nix::unistd::fchownat(
                    None,
                    &path,
                    Some(nix::unistd::Uid::from_raw(uid.into())),
                    Some(nix::unistd::Gid::from_raw(gid.into())),
                    nix::unistd::FchownatFlags::NoFollowSymlink,
                )
                .map_error()?;
                Ok((
                    convert_metadata(metadata)?,
                    PassthroughOpenFile {
                        open_file: tokio::fs::File::from_std(open_file),
                    },
                ))
            })
            .await
            .map_err(|_: tokio::task::JoinError| FsError::UnknownError)?
    }

    async fn rename_child(&self, old_name: &str, new_path: &Path) -> FsResult<()> {
        let old_path = self.path.join(old_name);
        tokio::fs::rename(old_path, new_path).await.map_error()?;
        Ok(())
    }
}

struct PassthroughSymlink {
    path: PathBuf,
}

#[async_trait]
impl Symlink for PassthroughSymlink {
    async fn target(&self) -> FsResult<PathBuf> {
        let target = tokio::fs::read_link(&self.path).await.map_error()?;
        Ok(target)
    }
}

struct PassthroughFile {
    path: PathBuf,
}

#[async_trait]
impl File for PassthroughFile {
    type Device = PassthroughDevice;

    async fn open(&self, openflags: OpenFlags) -> FsResult<PassthroughOpenFile> {
        let mut options = tokio::fs::OpenOptions::new();
        match openflags {
            OpenFlags::Read => options.read(true),
            OpenFlags::Write => options.write(true),
            OpenFlags::ReadWrite => options.read(true).write(true),
        };
        let open_file = options.open(&self.path).await.map_error()?;
        Ok(PassthroughOpenFile { open_file })
    }

    async fn truncate(&self, new_size: NumBytes) -> FsResult<()> {
        let path = self.path.clone();
        tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                nix::unistd::truncate(&path, u64::from(new_size) as libc::off_t).map_error()?;
                Ok(())
            })
            .await
            .map_err(|_: tokio::task::JoinError| FsError::UnknownError)?
    }
}

struct PassthroughOpenFile {
    open_file: tokio::fs::File,
}

#[async_trait]
impl OpenFile for PassthroughOpenFile {
    async fn getattr(&self) -> FsResult<NodeAttrs> {
        let metadata = self.open_file.metadata().await.map_error()?;
        convert_metadata(metadata)
    }

    async fn chmod(&self, mode: Mode) -> FsResult<()> {
        let permissions = std::fs::Permissions::from_mode(mode.into());
        self.open_file
            .set_permissions(permissions)
            .await
            .map_error()?;
        Ok(())
    }

    async fn chown(&self, uid: Option<Uid>, gid: Option<Gid>) -> FsResult<()> {
        let uid = uid.map(|uid| nix::unistd::Uid::from_raw(uid.into()));
        let gid = gid.map(|gid| nix::unistd::Gid::from_raw(gid.into()));
        // TODO Can we do this without duplicating the file descriptor?
        let open_file = self.open_file.try_clone().await.map_error()?;

        tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                nix::unistd::fchown(open_file.as_raw_fd(), uid, gid).map_error()?;
                Ok(())
            })
            .await
            .map_err(|_: tokio::task::JoinError| FsError::UnknownError)??;
        Ok(())
    }

    async fn truncate(&self, new_size: NumBytes) -> FsResult<()> {
        self.open_file.set_len(new_size.into()).await.map_error()?;
        Ok(())
    }

    async fn utimens(
        &self,
        last_access: Option<SystemTime>,
        last_modification: Option<SystemTime>,
    ) -> FsResult<()> {
        // TODO Can we do this without duplicating the file descriptor?
        let open_file = self
            .open_file
            .try_clone()
            .await
            .map_error()?
            .into_std()
            .await;
        tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                let (atime, mtime) = match (last_access, last_modification) {
                    (Some(atime), Some(mtime)) => {
                        // Both atime and mtime are being overwritten, no need to load previous values
                        (atime, mtime)
                    }
                    (atime, mtime) => {
                        // Either atime or mtime are not being overwritten, we need to load the previous values first.
                        let metadata = open_file.metadata().map_error()?;
                        let atime = match atime {
                            Some(atime) => atime,
                            None => metadata.accessed().map_error()?,
                        };
                        let mtime = match mtime {
                            Some(mtime) => mtime,
                            None => metadata.modified().map_error()?,
                        };
                        (atime, mtime)
                    }
                };
                nix::sys::stat::futimens(
                    open_file.as_raw_fd(),
                    &convert_timespec(atime),
                    &convert_timespec(mtime),
                )
                .map_error()?;
                Ok(())
            })
            .await
            .map_err(|_: tokio::task::JoinError| FsError::UnknownError)??;
        Ok(())
    }

    async fn read(&self, offset: NumBytes, size: NumBytes) -> FsResult<Data> {
        let mut buffer: Data = vec![0; usize::try_from(u64::from(size)).unwrap()].into();
        // TODO Is this possible without duplicating the file descriptor?
        let open_file = self.open_file.try_clone().await.map_error()?;
        tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                // Using `pread` instead of `read` because `read` requires a call to `seek` first
                // and there could be a race condition if multiple tasks read from the same file
                // and overwrite each other's seek position.
                let res = nix::sys::uio::pread(
                    open_file.as_raw_fd(),
                    &mut buffer,
                    i64::try_from(u64::from(offset)).unwrap(),
                )
                .map_error();
                match res {
                    Ok(num_read) => {
                        buffer.shrink_to_subregion(0..num_read);
                        Ok(buffer)
                    }
                    Err(err) => Err(err),
                }
            })
            .await
            .expect("Error in spawn_blocking task")
    }

    async fn write(&self, offset: NumBytes, data: Data) -> FsResult<()> {
        // TODO Is this possible without duplicating the file descriptor?
        let open_file = self.open_file.try_clone().await.map_error()?;
        tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                // Using `pwrite` instead of `write` because `write` requires a call to `seek` first
                // and there could be a race condition if multiple tasks write to the same file
                // and overwrite each other's seek position.
                let num_written = nix::sys::uio::pwrite(
                    open_file.as_raw_fd(),
                    data.as_ref(),
                    i64::try_from(u64::from(offset)).unwrap(),
                )
                .map_error()?;
                // TODO Should we handle the case where not all data was written gracefully by retrying to write the rest? The pwrite manpage says it's not an error if not all data gets written.
                assert_eq!(data.len(), num_written, "pwrite did not write all data");
                Ok(())
            })
            .await
            .map_err(|_: tokio::task::JoinError| FsError::UnknownError)??;
        Ok(())
    }

    async fn flush(&self) -> FsResult<()> {
        // flush strictly speaking isn't a request to sync dirty data,
        // but it's a good place to do it because it's usually triggered
        // by a `close` syscall and errors that happen here are reported
        // back as errors by the `close` syscall.
        self.open_file.sync_all().await.map_error()?;
        Ok(())
    }

    async fn fsync(&self, datasync: bool) -> FsResult<()> {
        if datasync {
            // sync data and metadata
            self.open_file.sync_all().await.map_error()?;
        } else {
            // only sync data, not metadata
            self.open_file.sync_data().await.map_error()?;
        }
        Ok(())
    }
}

trait IoResultExt<T> {
    fn map_error(self) -> FsResult<T>;
}
impl<T> IoResultExt<T> for std::io::Result<T> {
    fn map_error(self) -> FsResult<T> {
        self.map_err(|err| match err.raw_os_error() {
            Some(error_code) => FsError::Custom { error_code },
            None => FsError::UnknownError,
        })
    }
}
trait NixResultExt<T> {
    fn map_error(self) -> FsResult<T>;
}
impl<T> NixResultExt<T> for nix::Result<T> {
    fn map_error(self) -> FsResult<T> {
        self.map_err(|errno| {
            let error = std::io::Error::from(errno);
            match error.raw_os_error() {
                Some(error_code) => FsError::Custom { error_code },
                None => FsError::UnknownError,
            }
        })
    }
}

fn convert_metadata(metadata: Metadata) -> FsResult<NodeAttrs> {
    Ok(NodeAttrs {
        // TODO Make nlink platform independent
        // TODO No unwrap
        nlink: u32::try_from(metadata.st_nlink()).unwrap(),
        // TODO Make mode, uid, gid, blocks platform independent
        mode: metadata.st_mode().into(),
        uid: metadata.st_uid().into(),
        gid: metadata.st_gid().into(),
        num_bytes: NumBytes::from(metadata.len()),
        blocks: metadata.st_blocks(),
        atime: metadata.accessed().map_error()?,
        mtime: metadata.modified().map_error()?,
        // TODO No unwrap in ctime
        // TODO Make ctime platform independent (currently it requires the linux field st_ctime)
        // TODO Is st_ctime_nsec actually the total number of nsec or only the sub-second part?
        ctime: UNIX_EPOCH + Duration::from_nanos(u64::try_from(metadata.st_ctime_nsec()).unwrap()),
    })
}

fn convert_timespec(time: SystemTime) -> nix::sys::time::TimeSpec {
    time.duration_since(UNIX_EPOCH)
        // TODO No unwrap.expect
        .expect("Time is before unix epoch")
        .into()
}

fn convert_statfs(stat: nix::sys::statfs::Statfs) -> Statfs {
    Statfs {
        // TODO Don't use unwrap
        max_filename_length: u32::try_from(stat.maximum_name_length()).unwrap(),
        blocksize: u32::try_from(stat.block_size()).unwrap(),
        num_total_blocks: stat.blocks(),
        num_free_blocks: stat.blocks_free(),
        num_available_blocks: stat.blocks_available(),
        num_total_inodes: stat.files(),
        num_free_inodes: stat.files_free(),
    }
}

const USAGE: &str = "Usage: passthroughfs [basedir] [mountdir]";

fn main() {
    // TODO Use clap for argument parsing

    env_logger::init();

    let mut args = std::env::args();
    let _executable = args.next().unwrap();
    let basedir = args.next().expect(USAGE);
    let mountdir = args.next().expect(USAGE);
    assert!(args.next().is_none(), "{}", USAGE);

    let device = PassthroughDevice {
        basedir: basedir.into(),
    };

    cryfs_rustfs::fuse_mt::mount(device, mountdir).unwrap();
}
