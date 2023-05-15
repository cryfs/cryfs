use async_trait::async_trait;
use cryfs_rustfs::{FsResult, Gid, Mode, Node, NodeAttrs, Uid};
use std::time::SystemTime;

use super::dir::InMemoryDirRef;
use super::file::InMemoryFileRef;
use super::symlink::InMemorySymlinkRef;

// TODO We should update ctime whenever metadata changes
// TODO We should update atime and mtime correctly when things change

pub enum InMemoryNodeRef {
    File(InMemoryFileRef),
    Dir(InMemoryDirRef),
    Symlink(InMemorySymlinkRef),
}

impl InMemoryNodeRef {
    pub fn clone_ref(&self) -> InMemoryNodeRef {
        match self {
            InMemoryNodeRef::File(file) => InMemoryNodeRef::File(file.clone_ref()),
            InMemoryNodeRef::Dir(dir) => InMemoryNodeRef::Dir(dir.clone_ref()),
            InMemoryNodeRef::Symlink(symlink) => InMemoryNodeRef::Symlink(symlink.clone_ref()),
        }
    }
}

#[async_trait]
impl Node for InMemoryNodeRef {
    async fn getattr(&self) -> FsResult<NodeAttrs> {
        match self {
            InMemoryNodeRef::File(file) => Ok(file.metadata()),
            InMemoryNodeRef::Dir(dir) => Ok(dir.metadata()),
            InMemoryNodeRef::Symlink(symlink) => Ok(symlink.metadata()),
        }
    }

    async fn chmod(&self, mode: Mode) -> FsResult<()> {
        match self {
            InMemoryNodeRef::File(file) => file.chmod(mode),
            InMemoryNodeRef::Dir(dir) => dir.chmod(mode),
            InMemoryNodeRef::Symlink(symlink) => symlink.chmod(mode),
        }
        Ok(())
    }

    async fn chown(&self, uid: Option<Uid>, gid: Option<Gid>) -> FsResult<()> {
        match self {
            InMemoryNodeRef::File(file) => file.chown(uid, gid),
            InMemoryNodeRef::Dir(dir) => dir.chown(uid, gid),
            InMemoryNodeRef::Symlink(symlink) => symlink.chown(uid, gid),
        }
        Ok(())
    }

    async fn utimens(
        &self,
        last_access: Option<SystemTime>,
        last_modification: Option<SystemTime>,
    ) -> FsResult<()> {
        match self {
            InMemoryNodeRef::File(file) => file.utimens(last_access, last_modification),
            InMemoryNodeRef::Dir(dir) => dir.utimens(last_access, last_modification),
            InMemoryNodeRef::Symlink(symlink) => symlink.utimens(last_access, last_modification),
        }
        Ok(())
    }
}
