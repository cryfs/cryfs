use crate::common::{Gid, RequestInfo, Uid};

pub fn assert_request_info_is_correct(req: &RequestInfo) {
    assert_eq!(Uid::from(users::get_current_uid()), req.uid);
    assert_eq!(Gid::from(users::get_effective_gid()), req.gid);
    // TODO This doesn't seem to work?
    //assert_eq!(std::process::id, req.pid);
}
