use std::fmt::Debug;

use super::AsyncFilesystemLL;
use crate::common::FsError;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};

pub trait IntoFsLL<Fs>
where
    Fs: AsyncFilesystemLL + AsyncDrop<Error = FsError> + Debug,
{
    fn into_fs(self) -> AsyncDropGuard<Fs>;
}

impl<Fs> IntoFsLL<Fs> for AsyncDropGuard<Fs>
where
    Fs: AsyncFilesystemLL + AsyncDrop<Error = FsError> + Debug,
{
    fn into_fs(self) -> Self {
        self
    }
}
