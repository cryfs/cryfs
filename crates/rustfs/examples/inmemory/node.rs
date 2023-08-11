use async_trait::async_trait;
use std::time::SystemTime;

use cryfs_rustfs::{
    object_based_api::Node, FsError, FsResult, Gid, Mode, NodeAttrs, NumBytes, Uid,
};
use cryfs_utils::async_drop::AsyncDrop;

use super::dir::InMemoryDirRef;
use super::file::InMemoryFileRef;
use super::symlink::InMemorySymlinkRef;
use super::InMemoryDevice;

// TODO We should update ctime whenever metadata changes
// TODO We should update atime and mtime correctly when things change

#[derive(Debug)]
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
    type Device = InMemoryDevice;

    async fn as_file(&self) -> FsResult<InMemoryFileRef> {
        match self {
            InMemoryNodeRef::File(file) => Ok(file.clone_ref()),
            InMemoryNodeRef::Dir(_) => Err(FsError::NodeIsADirectory),
            InMemoryNodeRef::Symlink(_) => {
                // TODO What's the right error here?
                Err(FsError::UnknownError)
            }
        }
    }

    async fn as_dir(&self) -> FsResult<InMemoryDirRef> {
        match self {
            InMemoryNodeRef::Dir(dir) => Ok(dir.clone_ref()),
            _ => Err(FsError::NodeIsNotADirectory),
        }
    }

    async fn as_symlink(&self) -> FsResult<InMemorySymlinkRef> {
        match self {
            InMemoryNodeRef::Symlink(symlink) => Ok(symlink.clone_ref()),
            _ => Err(FsError::NodeIsNotASymlink),
        }
    }

    async fn getattr(&self) -> FsResult<NodeAttrs> {
        match self {
            InMemoryNodeRef::File(file) => Ok(file.metadata()),
            InMemoryNodeRef::Dir(dir) => Ok(dir.metadata()),
            InMemoryNodeRef::Symlink(symlink) => Ok(symlink.metadata()),
        }
    }

    async fn setattr(
        &self,
        mode: Option<Mode>,
        uid: Option<Uid>,
        gid: Option<Gid>,
        size: Option<NumBytes>,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
        ctime: Option<SystemTime>,
    ) -> FsResult<NodeAttrs> {
        match self {
            InMemoryNodeRef::File(file) => file.setattr(mode, uid, gid, size, atime, mtime, ctime),
            InMemoryNodeRef::Dir(dir) => dir.setattr(mode, uid, gid, size, atime, mtime, ctime),
            InMemoryNodeRef::Symlink(symlink) => {
                symlink.setattr(mode, uid, gid, size, atime, mtime, ctime)
            }
        }
    }
}

#[async_trait]
impl AsyncDrop for InMemoryNodeRef {
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), FsError> {
        // Nothing to do
        Ok(())
    }
}
