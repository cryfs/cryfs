#[cfg(not(feature = "benchmark"))]
use cryfs_rustfs::RequestInfo;

#[cfg(not(feature = "benchmark"))]
pub fn request_info() -> RequestInfo {
    use cryfs_rustfs::{Gid, Uid};

    RequestInfo {
        unique: 0,
        uid: Uid::from(0),
        gid: Gid::from(0),
        pid: 0,
    }
}
