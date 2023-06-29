use cryfs_rustfs::{FsResult, NodeAttrs, NumBytes};
use std::fs::Metadata;
use std::os::unix::fs::MetadataExt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::errors::IoResultExt;

pub fn convert_metadata(metadata: Metadata) -> FsResult<NodeAttrs> {
    Ok(NodeAttrs {
        // TODO Make nlink platform independent
        // TODO No unwrap
        nlink: u32::try_from(metadata.nlink()).unwrap(),
        // TODO Make mode, uid, gid, blocks platform independent
        mode: metadata.mode().into(),
        uid: metadata.uid().into(),
        gid: metadata.gid().into(),
        num_bytes: NumBytes::from(metadata.len()),
        num_blocks: Some(metadata.blocks()),
        atime: metadata.accessed().map_error()?,
        mtime: metadata.modified().map_error()?,
        // TODO No unwrap in ctime
        // TODO Make ctime platform independent (currently it requires the linux field st_ctime)
        // TODO Is st_ctime_nsec actually the total number of nsec or only the sub-second part?
        ctime: UNIX_EPOCH + Duration::from_nanos(u64::try_from(metadata.ctime_nsec()).unwrap()),
    })
}

pub fn convert_timespec(time: SystemTime) -> nix::sys::time::TimeSpec {
    time.duration_since(UNIX_EPOCH)
        // TODO No unwrap.expect
        .expect("Time is before unix epoch")
        .into()
}
