use async_trait::async_trait;

use super::error::FsResult;
use crate::utils::NodeKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntry {
    pub name: String,
    pub kind: NodeKind,
}

#[async_trait]
pub trait Dir {
    async fn entries(&self) -> FsResult<Vec<DirEntry>>;
}
