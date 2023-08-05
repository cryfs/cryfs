use super::{Gid, Uid};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct RequestInfo {
    /// The unique ID assigned to this request by FUSE.
    /// TODO Rename to request_id or id?
    pub unique: u64,
    /// The user ID of the process making the request.
    pub uid: Uid,
    /// The group ID of the process making the request.
    pub gid: Gid,
    /// The process ID of the process making the request.
    /// // TODO Make a Pid type instead of using u32
    pub pid: u32,
}
