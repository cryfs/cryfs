use cryfs_rustfs::{FsResult, NodeAttrs, NumBytes};
use std::fs::Metadata;
use std::os::linux::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::errors::IoResultExt;

pub fn apply_basedir(basedir: &Path, path: &Path) -> PathBuf {
    assert!(path.is_absolute());
    let path = path.strip_prefix("/").unwrap();
    assert!(!path.is_absolute());
    let node_path = basedir.join(path);
    // Assert node_path doesn't escape the basedir
    // TODO Assert is probably a bad choice here. What should we do instead? Return an error?
    assert!(
        node_path.starts_with(&basedir),
        "Path {} escaped basedir {}",
        node_path.display(),
        basedir.display()
    );
    node_path
}

pub fn convert_metadata(metadata: Metadata) -> FsResult<NodeAttrs> {
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

pub fn convert_timespec(time: SystemTime) -> nix::sys::time::TimeSpec {
    time.duration_since(UNIX_EPOCH)
        // TODO No unwrap.expect
        .expect("Time is before unix epoch")
        .into()
}
