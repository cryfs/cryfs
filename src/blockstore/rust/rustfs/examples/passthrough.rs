use async_trait::async_trait;
use cryfs_rustfs::{Device, Dir, DirEntry, Gid, Mode, Node, NodeAttrs, NodeKind, NumBytes, Uid};
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

    async fn load_node(&self, path: &Path) -> std::io::Result<Self::Node> {
        let path = self.apply_basedir(path);
        Ok(PassthroughNode { path })
    }

    async fn load_dir(&self, path: &Path) -> std::io::Result<Self::Dir> {
        let path = self.apply_basedir(path);
        Ok(PassthroughDir { path })
    }
}

struct PassthroughNode {
    path: PathBuf,
}

#[async_trait]
impl Node for PassthroughNode {
    async fn getattr(&self) -> std::io::Result<NodeAttrs> {
        let metadata = tokio::fs::symlink_metadata(&self.path).await?;
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
            atime: metadata.accessed()?,
            mtime: metadata.modified()?,
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
    async fn entries(&self) -> std::io::Result<Vec<DirEntry>> {
        let mut entries = Vec::new();
        let mut dir = tokio::fs::read_dir(&self.path).await?;
        while let Some(entry) = dir.next_entry().await? {
            let name = entry.file_name();
            let name = name.to_string_lossy().into_owned(); // TODO Is to_string_lossy the best way to convert from OsString to String?
            let node_type = entry.file_type().await?;
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
