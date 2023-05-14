use async_trait::async_trait;
use cryfs_rustfs::{FsResult, Gid, Mode, Node, NodeAttrs, Uid};
use std::time::SystemTime;

use super::dir::InMemoryDir;
use super::file::InMemoryFile;
use super::symlink::InMemorySymlink;

// TODO We should update ctime whenever metadata changes
// TODO We should update atime and mtime correctly

pub trait IsInMemoryNode {
    fn metadata(&self) -> NodeAttrs;
    fn update_metadata(&self, callback: impl FnOnce(&mut NodeAttrs));
}

pub enum InMemoryNode {
    File(InMemoryFile),
    Dir(InMemoryDir),
    Symlink(InMemorySymlink),
}

impl InMemoryNode {
    fn update_metadata(&self, callback: impl FnOnce(&mut NodeAttrs)) {
        match self {
            InMemoryNode::File(file) => file.update_metadata(callback),
            InMemoryNode::Dir(dir) => dir.update_metadata(callback),
            InMemoryNode::Symlink(symlink) => symlink.update_metadata(callback),
        }
    }
}

#[async_trait]
impl Node for InMemoryNode {
    async fn getattr(&self) -> FsResult<NodeAttrs> {
        match self {
            InMemoryNode::File(file) => Ok(file.metadata()),
            InMemoryNode::Dir(dir) => Ok(dir.metadata()),
            InMemoryNode::Symlink(symlink) => Ok(symlink.metadata()),
        }
    }

    async fn chmod(&self, mode: Mode) -> FsResult<()> {
        self.update_metadata(|metadata| {
            metadata.mode = Mode::from(mode);
        });
        Ok(())
    }

    async fn chown(&self, uid: Option<Uid>, gid: Option<Gid>) -> FsResult<()> {
        self.update_metadata(|metadata| {
            if let Some(uid) = uid {
                metadata.uid = uid;
            }
            if let Some(gid) = gid {
                metadata.gid = gid;
            }
        });
        Ok(())
    }

    async fn utimens(
        &self,
        last_access: Option<SystemTime>,
        last_modification: Option<SystemTime>,
    ) -> FsResult<()> {
        self.update_metadata(|metadata| {
            if let Some(last_access) = last_access {
                metadata.atime = last_access;
            }
            if let Some(last_modification) = last_modification {
                metadata.mtime = last_modification;
            }
        });
        Ok(())
    }
}
