use async_trait::async_trait;
use cryfs_rustfs::{object_based_api::Symlink, FsResult, Gid, Mode, NodeAttrs, NumBytes, Uid};
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use super::inode_metadata::{chmod, chown, utimens};

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

        pub fn chmod(&mut self, mode: Mode) {
            chmod(&mut self.metadata, mode);
        }

        pub fn chown(&mut self, uid: Option<Uid>, gid: Option<Gid>) {
            chown(&mut self.metadata, uid, gid);
        }

        pub fn utimens(
            &mut self,
            last_access: Option<SystemTime>,
            last_modification: Option<SystemTime>,
        ) {
            utimens(&mut self.metadata, last_access, last_modification);
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

    pub fn chmod(&self, mode: Mode) {
        self.inode.lock().unwrap().chmod(mode);
    }

    pub fn chown(&self, uid: Option<Uid>, gid: Option<Gid>) {
        self.inode.lock().unwrap().chown(uid, gid);
    }

    pub fn utimens(&self, last_access: Option<SystemTime>, last_modification: Option<SystemTime>) {
        self.inode
            .lock()
            .unwrap()
            .utimens(last_access, last_modification);
    }
}

#[async_trait]
impl Symlink for InMemorySymlinkRef {
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
