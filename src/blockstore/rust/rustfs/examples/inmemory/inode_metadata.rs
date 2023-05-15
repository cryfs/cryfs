use cryfs_rustfs::{Gid, Mode, NodeAttrs, Uid};
use std::time::SystemTime;

pub fn chmod(attrs: &mut NodeAttrs, mode: Mode) {
    attrs.mode = mode;
}

pub fn chown(attrs: &mut NodeAttrs, uid: Option<Uid>, gid: Option<Gid>) {
    if let Some(uid) = uid {
        attrs.uid = uid;
    }
    if let Some(gid) = gid {
        attrs.gid = gid;
    }
}

pub fn utimens(
    attrs: &mut NodeAttrs,
    last_access: Option<SystemTime>,
    last_modification: Option<SystemTime>,
) {
    if let Some(last_access) = last_access {
        attrs.atime = last_access;
    }
    if let Some(last_modification) = last_modification {
        attrs.mtime = last_modification;
    }
}
