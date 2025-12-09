use derive_more::Debug;
use std::time::SystemTime;
use time::OffsetDateTime;

use super::{Gid, Mode, NumBytes, Uid};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeAttrs {
    pub nlink: u32,
    #[debug("{mode}")]
    pub mode: Mode,
    #[debug("{uid}")]
    pub uid: Uid,
    #[debug("{gid}")]
    pub gid: Gid,
    #[debug("{num_bytes}")]
    pub num_bytes: NumBytes,

    /// `num_blocks` is the number of 512B blocks allocated for this node.
    /// This is only needed for special cases like files with holes in them.
    /// Otherwise, `num_blocks == num_bytes / 512` is correct and if you're ok
    /// with that default, you can leave this field as `None`.
    pub num_blocks: Option<u64>,

    #[debug("{}", format_datetime(*atime))]
    pub atime: SystemTime,
    #[debug("{}", format_datetime(*mtime))]
    pub mtime: SystemTime,
    #[debug("{}", format_datetime(*ctime))]
    pub ctime: SystemTime,
}

fn format_datetime(time: SystemTime) -> String {
    let datetime: OffsetDateTime = time.into();
    datetime
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| format!("[invalid] {:?}", time))
}
