use async_trait::async_trait;
use cryfs_rustfs::{FsResult, Gid, Mode, NodeAttrs, NumBytes, Uid, object_based_api::Symlink};
use cryfs_utils::async_drop::AsyncDropGuard;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use super::inode_metadata::setattr;
use super::node::InMemoryNodeRef;

mod inode {
    use super::*;

    pub struct SymlinkInode {
        metadata: NodeAttrs,
        target: String,
    }

    impl SymlinkInode {
        pub fn new(target: String, uid: Uid, gid: Gid) -> Self {
            Self {
                metadata: NodeAttrs {
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
                    num_blocks: None,
                    atime: SystemTime::now(),
                    mtime: SystemTime::now(),
                    ctime: SystemTime::now(),
                },
                target,
            }
        }

        pub fn metadata(&self) -> &NodeAttrs {
            &self.metadata
        }

        pub fn target(&self) -> &str {
            &self.target
        }

        pub fn setattr(
            &mut self,
            mode: Option<Mode>,
            uid: Option<Uid>,
            gid: Option<Gid>,
            atime: Option<SystemTime>,
            mtime: Option<SystemTime>,
            ctime: Option<SystemTime>,
        ) -> FsResult<NodeAttrs> {
            setattr(&mut self.metadata, mode, uid, gid, atime, mtime, ctime)
        }
    }
}
use inode::SymlinkInode;

pub struct InMemorySymlinkRef {
    inode: Arc<Mutex<SymlinkInode>>,
}

impl InMemorySymlinkRef {
    pub fn new(target: String, uid: Uid, gid: Gid) -> Self {
        Self {
            inode: Arc::new(Mutex::new(SymlinkInode::new(target, uid, gid))),
        }
    }

    pub fn clone_ref(&self) -> Self {
        Self {
            inode: Arc::clone(&self.inode),
        }
    }

    pub fn metadata(&self) -> NodeAttrs {
        let inode = self.inode.lock().unwrap();
        *inode.metadata()
    }

    pub fn setattr(
        &self,
        mode: Option<Mode>,
        uid: Option<Uid>,
        gid: Option<Gid>,
        size: Option<NumBytes>,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
        ctime: Option<SystemTime>,
    ) -> FsResult<NodeAttrs> {
        // TODO Is setting size actually forbidden?
        assert!(
            size.is_none(),
            "Can't set size using setattr on a directory"
        );
        self.inode
            .lock()
            .unwrap()
            .setattr(mode, uid, gid, atime, mtime, ctime)
    }
}

#[async_trait]
impl Symlink for InMemorySymlinkRef {
    type Device = super::InMemoryDevice;

    fn as_node(&self) -> AsyncDropGuard<InMemoryNodeRef> {
        AsyncDropGuard::new(InMemoryNodeRef::Symlink(self.clone_ref()))
    }

    async fn target(&self) -> FsResult<String> {
        Ok(self.inode.lock().unwrap().target().to_owned())
    }
}

impl Debug for InMemorySymlinkRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let inode = self.inode.lock().unwrap();
        f.debug_struct("InMemorySymlinkRef").finish()
    }
}
