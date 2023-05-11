use async_trait::async_trait;
use cryfs_rustfs::{
    Device, Dir, DirEntry, FsError, FsResult, Gid, Mode, Node, NodeAttrs, NodeKind, NumBytes, Uid,
};
use std::os::linux::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

    async fn load_node(&self, path: &Path) -> FsResult<Self::Node> {
        let path = self.apply_basedir(path);
        Ok(PassthroughNode { path })
    }

    async fn load_dir(&self, path: &Path) -> FsResult<Self::Dir> {
        let path = self.apply_basedir(path);
        Ok(PassthroughDir { path })
    }
}

struct PassthroughNode {
    path: PathBuf,
}

#[async_trait]
impl Node for PassthroughNode {
    async fn getattr(&self) -> FsResult<NodeAttrs> {
        let metadata = tokio::fs::symlink_metadata(&self.path).await.map_error()?;
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
            ctime: UNIX_EPOCH
                + Duration::from_nanos(u64::try_from(metadata.st_ctime_nsec()).unwrap()),
        })
    }
}

struct PassthroughDir {
    path: PathBuf,
}

#[async_trait]
impl Dir for PassthroughDir {
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
                // TODO Don't use UnknownError
                .map_err(|_| FsError::UnknownError)?;
                nix::unistd::chown(
                    &path_clone,
                    Some(nix::unistd::Uid::from_raw(uid.into())),
                    Some(nix::unistd::Gid::from_raw(gid.into())),
                )
                // TODO Don't use UnknownError
                .map_err(|_| FsError::UnknownError)?;
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
