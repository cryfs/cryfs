use async_trait::async_trait;

use super::error::FsResult;
use super::node::NodeAttrs;
use crate::utils::{Gid, Mode, NodeKind, Uid};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntry {
    pub name: String,
    pub kind: NodeKind,
}

#[async_trait]
pub trait Dir {
    type Device: super::Device;

    async fn entries(&self) -> FsResult<Vec<DirEntry>>;

    async fn create_child_dir(
        &self,
        name: &str,
        mode: Mode,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<NodeAttrs>;
    async fn remove_child_dir(&self, name: &str) -> FsResult<()>;

    async fn create_child_symlink(
        &self,
        name: &str,
        target: &Path,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<NodeAttrs>;
    async fn remove_child_file_or_symlink(&self, name: &str) -> FsResult<()>;

    async fn create_and_open_file(
        &self,
        name: &str,
        mode: Mode,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<(NodeAttrs, <Self::Device as super::Device>::OpenFile)>;

    async fn rename_child(&self, old_name: &str, new_path: &Path) -> FsResult<()>;
}
