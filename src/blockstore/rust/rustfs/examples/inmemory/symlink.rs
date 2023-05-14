use async_trait::async_trait;
use cryfs_rustfs::{FsResult, Gid, Mode, NodeAttrs, NumBytes, Symlink, Uid};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;

use super::node::IsInMemoryNode;

pub struct InMemorySymlink {
    metadata: Mutex<NodeAttrs>,
    target: PathBuf,
}

impl InMemorySymlink {
    pub fn new(target: PathBuf, uid: Uid, gid: Gid) -> Self {
        Self {
            metadata: Mutex::new(NodeAttrs {
                // TODO What are the right symlink attributes here?
                nlink: 1,
                mode: Mode::default()
                    .add_symlink_flag()
                    .add_user_read_flag()
                    .add_user_write_flag()
                    .add_user_exec_flag()
                    .add_group_read_flag()
                    .add_group_write_flag()
                    .add_group_exec_flag()
                    .add_other_read_flag()
                    .add_other_write_flag()
                    .add_other_exec_flag(),
                uid,
                gid,
                num_bytes: NumBytes::from(0),
                blocks: 1,
                atime: SystemTime::now(),
                mtime: SystemTime::now(),
                ctime: SystemTime::now(),
            }),
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
