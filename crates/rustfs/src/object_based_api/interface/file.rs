use std::fmt::Debug;

use crate::common::{FsResult, OpenInFlags};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

pub trait File: AsyncDrop + Debug + Sized {
    type Device: super::Device;

    fn into_open(
        this: AsyncDropGuard<Self>,
        flags: OpenInFlags,
    ) -> impl Future<Output = FsResult<AsyncDropGuard<<Self::Device as super::Device>::OpenFile>>> + Send;
}
