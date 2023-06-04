use std::fmt::Debug;

use super::AsyncFilesystem;
use crate::common::FsError;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

pub trait IntoFs<Fs>
where
    Fs: AsyncFilesystem + AsyncDrop<Error = FsError> + Debug,
{
    fn into_fs(self) -> AsyncDropGuard<Fs>;
}

impl<Fs> IntoFs<Fs> for AsyncDropGuard<Fs>
where
    Fs: AsyncFilesystem + AsyncDrop<Error = FsError> + Debug,
{
    fn into_fs(self) -> Self {
        self
    }
}
