use async_trait::async_trait;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};
use std::fmt::Debug;

use crate::common::FsResult;

#[async_trait]
pub trait Symlink: AsyncDrop + Debug + Sized {
    type Device: super::Device;

    fn into_node(
        this: AsyncDropGuard<Self>,
    ) -> AsyncDropGuard<<Self::Device as super::Device>::Node>;

    // TODO Use a custom wrapper type for the target path, a type that allows paths to be either absolute or relative.
    //      We're using String instead of PathBuf today because String enforces utf-8 but would be better to have our own
    //      type that enforces more invariants (e.g. no null bytes, see [crate::AbsolutePath] for some invariant candidates).
    async fn target(&self) -> FsResult<String>;
}
