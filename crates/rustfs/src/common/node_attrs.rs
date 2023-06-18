use std::time::SystemTime;

use super::{Gid, Mode, NumBytes, Uid};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeAttrs {
    pub nlink: u32,
    pub mode: Mode,
    pub uid: Uid,
    pub gid: Gid,
    pub num_bytes: NumBytes,

    /// `num_blocks` is the number of 512B blocks allocated for this node.
    /// This is only needed for special cases like files with holes in them.
    /// Otherwise, `num_blocks == num_bytes / 512` is correct and if you're ok
    /// with that default, you can leave this field as `None`.
    pub num_blocks: Option<u64>,

    pub atime: SystemTime,
    pub mtime: SystemTime,
    pub ctime: SystemTime,
}
