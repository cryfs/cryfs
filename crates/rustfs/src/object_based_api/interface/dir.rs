use async_trait::async_trait;

use crate::common::{DirEntry, FsResult, Gid, Mode, NodeAttrs, PathComponent, Uid};
use cryfs_utils::async_drop::AsyncDropGuard;

#[async_trait]
pub trait Dir {
    type Device: super::Device;

    fn as_node(&self) -> AsyncDropGuard<<Self::Device as super::Device>::Node>;

    async fn entries(&self) -> FsResult<Vec<DirEntry>>;

    // If the child doesn't exist, it's ok to either immediately fail with [FsError::NodeDoesNotExist]
    // or to return a [Node] object that throws [FsError::NodeDoesNotExist] when any of its members that
    // require existence are called.
    async fn lookup_child(
        &self,
        name: &PathComponent,
    ) -> FsResult<AsyncDropGuard<<Self::Device as super::Device>::Node>>;

    async fn create_child_dir(
        &self,
        name: &PathComponent,
        mode: Mode,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<NodeAttrs>;

    async fn remove_child_dir(&self, name: &PathComponent) -> FsResult<()>;

    async fn create_child_symlink(
        &self,
        name: &PathComponent,
        // TODO Use custom type for target that can wrap an absolute-or-relative path
        target: &str,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<NodeAttrs>;

    async fn remove_child_file_or_symlink(&self, name: &PathComponent) -> FsResult<()>;

    async fn create_and_open_file(
        &self,
        name: &PathComponent,
        mode: Mode,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<(
        NodeAttrs,
        AsyncDropGuard<<Self::Device as super::Device>::OpenFile>,
    )>;
}
