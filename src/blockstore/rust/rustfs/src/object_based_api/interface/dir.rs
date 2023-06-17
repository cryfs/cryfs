use async_trait::async_trait;

use crate::common::{DirEntry, FsResult, Gid, Mode, NodeAttrs, PathComponent, Uid};
use cryfs_utils::async_drop::AsyncDropGuard;

#[async_trait]
pub trait Dir {
    type Device: super::Device;

    async fn entries(&self) -> FsResult<Vec<DirEntry>>;

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
