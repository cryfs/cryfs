use async_trait::async_trait;
use cryfs_rustfs::{FsResult, NodeAttrs, Symlink};
use std::path::PathBuf;
use std::sync::Mutex;

use super::node::IsInMemoryNode;

pub struct InMemorySymlink {
    metadata: Mutex<NodeAttrs>,
    target: PathBuf,
}

impl InMemorySymlink {
    pub fn new(target: PathBuf, metadata: NodeAttrs) -> Self {
        Self {
            metadata: Mutex::new(metadata),
            target,
        }
    }
}

#[async_trait]
impl Symlink for InMemorySymlink {
    async fn target(&self) -> FsResult<PathBuf> {
        Ok(self.target.clone())
    }
}

impl IsInMemoryNode for InMemorySymlink {
    fn metadata(&self) -> NodeAttrs {
        *self.metadata.lock().unwrap()
    }

    fn update_metadata(&self, callback: impl FnOnce(&mut NodeAttrs)) {
        let mut metadata = self.metadata.lock().unwrap();
        callback(&mut metadata);
    }
}
