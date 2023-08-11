use cryfs_rustfs::{FsResult, Gid, Mode, NodeAttrs, Uid};
use std::time::SystemTime;

pub fn setattr(
    attrs: &mut NodeAttrs,
    mode: Option<Mode>,
    uid: Option<Uid>,
    gid: Option<Gid>,
    atime: Option<SystemTime>,
    mtime: Option<SystemTime>,
    ctime: Option<SystemTime>,
) -> FsResult<NodeAttrs> {
    if let Some(mode) = mode {
        attrs.mode = mode;
    }
    if let Some(uid) = uid {
        attrs.uid = uid;
    }
    if let Some(gid) = gid {
        attrs.gid = gid;
    }
    if let Some(atime) = atime {
        attrs.atime = atime;
    }
    if let Some(mtime) = mtime {
        attrs.mtime = mtime;
    }
    if let Some(ctime) = ctime {
        attrs.ctime = ctime;
    }
    Ok(attrs.clone())
}
