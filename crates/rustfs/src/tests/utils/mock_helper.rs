use mockall::predicate::{always, eq};
use std::time::{Duration, SystemTime};

use super::MockAsyncFilesystemLL;
use crate::common::{
    AbsolutePath, FsError, Gid, HandleWithGeneration, InodeNumber, Mode, NodeAttrs, NodeKind,
    NumBytes, PathComponent, Uid,
};
use crate::low_level_api::ReplyEntry;

pub const ROOT_INO: InodeNumber = InodeNumber::from_const(fuser::FUSE_ROOT_ID);

struct TestInodeNumberPool {
    highest_assigned_ino: InodeNumber,
}

impl TestInodeNumberPool {
    pub fn new() -> Self {
        Self {
            highest_assigned_ino: InodeNumber::from(12), // arbitrary starting number
        }
    }

    pub fn next(&mut self) -> InodeNumber {
        self.highest_assigned_ino = InodeNumber::from(u64::from(self.highest_assigned_ino) + 11); // 11 is an arbitrary offset to generate different INOs
        self.highest_assigned_ino
    }
}

pub struct MockHelper<'a> {
    mock_filesystem: &'a mut MockAsyncFilesystemLL,
    ino_pool: TestInodeNumberPool,
}

impl<'a> MockHelper<'a> {
    pub fn new(mock_filesystem: &'a mut MockAsyncFilesystemLL) -> Self {
        Self {
            mock_filesystem,
            ino_pool: TestInodeNumberPool::new(),
        }
    }

    pub fn expect_lookup_fail(&mut self, ino: InodeNumber, name: &PathComponent, error: FsError) {
        self.mock_filesystem
            .expect_lookup()
            .once()
            .with(always(), eq(ino), eq(name.to_owned()))
            .return_once(move |_, _, _| Err(error));
    }

    pub fn expect_lookup_doesnt_exist(&mut self, ino: InodeNumber, name: &PathComponent) {
        self.expect_lookup_fail(ino, name, FsError::NodeDoesNotExist);
    }

    /// The mock file system will return on `lookup` that the given node is of kind `kind`.
    /// This function returns the inode number that will be returned for that node.
    pub fn expect_lookup_is_kind(
        &mut self,
        parent_ino: InodeNumber,
        name: &PathComponent,
        kind: NodeKind,
    ) -> InodeNumber {
        let attr = match kind {
            NodeKind::Dir => some_dir_attrs(),
            NodeKind::File => some_file_attrs(),
            NodeKind::Symlink => some_symlink_attrs(),
        };
        self.expect_lookup_has_attrs(parent_ino, name, attr)
    }

    /// The mock file system will return on `lookup` that the given node is a file.
    /// This function returns the inode number that will be returned for that node.
    pub fn expect_lookup_is_file(
        &mut self,
        parent_ino: InodeNumber,
        name: &PathComponent,
    ) -> InodeNumber {
        self.expect_lookup_is_kind(parent_ino, name, NodeKind::File)
    }

    /// The mock file system will return on `lookup` that the given node is a symlink.
    /// This function returns the inode number that will be returned for that node.
    pub fn expect_lookup_is_symlink(
        &mut self,
        parent_ino: InodeNumber,
        name: &PathComponent,
    ) -> InodeNumber {
        self.expect_lookup_is_kind(parent_ino, name, NodeKind::Symlink)
    }

    /// The mock file system will return on `lookup` that the given node is a directory.
    /// This function returns the inode number that will be returned for that node.
    pub fn expect_lookup_is_dir(
        &mut self,
        parent_ino: InodeNumber,
        name: &PathComponent,
    ) -> InodeNumber {
        self.expect_lookup_is_kind(parent_ino, name, NodeKind::Dir)
    }

    /// The mock file system will return on `lookup` that the given node has the given `attr`.
    /// This function returns the inode number that will be returned for that node.
    fn expect_lookup_has_attrs(
        &mut self,
        parent_ino: InodeNumber,
        name: &PathComponent,
        attr: NodeAttrs,
    ) -> InodeNumber {
        let ino = self.ino_pool.next();
        self.mock_filesystem
            .expect_lookup()
            .once()
            .with(always(), eq(parent_ino), eq(name.to_owned()))
            .returning(move |_, _, _| {
                Ok(ReplyEntry {
                    ino: HandleWithGeneration {
                        handle: ino,
                        generation: 0,
                    },
                    attr,
                    ttl: Duration::from_secs(1),
                })
            });
        ino
    }

    /// Expect to lookup all intermediate directories on `path`.
    /// Each intermediate directory will be returned to be a directory.
    /// The final entry will be returned to be a `kind`, and its inode will be returned from this function.
    pub fn expect_lookup_path_is_kind(
        &mut self,
        path: &AbsolutePath,
        kind: NodeKind,
    ) -> InodeNumber {
        let mut current_ino = ROOT_INO;
        if let Some((parent_path, name)) = path.split_last() {
            for node in parent_path {
                current_ino = self.expect_lookup_is_dir(current_ino, node);
            }
            current_ino = self.expect_lookup_is_kind(current_ino, name, kind);
        }
        current_ino
    }

    /// Expect to lookup all intermediate directories on `path`.
    /// Each intermediate directory will be returned to be a directory.
    /// The final entry will be returned to be a directory, and its inode will be returned from this function.
    pub fn expect_lookup_path_is_dir(&mut self, path: &AbsolutePath) -> InodeNumber {
        self.expect_lookup_path_is_kind(path, NodeKind::Dir)
    }

    /// Expect to lookup all intermediate directories on `path`.
    /// Each intermediate directory will be returned to be a directory.
    /// The final entry will be returned to be a file, and its inode will be returned from this function.
    pub fn expect_lookup_path_is_file(&mut self, path: &AbsolutePath) -> InodeNumber {
        self.expect_lookup_path_is_kind(path, NodeKind::File)
    }

    /// Expect to lookup all intermediate directories on `path`.
    /// Each intermediate directory will be returned to be a directory.
    /// The final entry will be returned to be a symlink, and its inode will be returned from this function.
    pub fn expect_lookup_path_is_symlink(&mut self, path: &AbsolutePath) -> InodeNumber {
        self.expect_lookup_path_is_kind(path, NodeKind::Symlink)
    }
}

fn some_dir_attrs() -> NodeAttrs {
    let now = SystemTime::now();
    NodeAttrs {
        nlink: 1,
        mode: Mode::default().add_dir_flag(),
        uid: Uid::from(1000),
        gid: Gid::from(1000),
        num_bytes: NumBytes::from(532),
        num_blocks: None,
        atime: now,
        mtime: now,
        ctime: now,
    }
}

fn some_file_attrs() -> NodeAttrs {
    let now = SystemTime::now();
    NodeAttrs {
        nlink: 1,
        mode: Mode::default().add_file_flag(),
        uid: Uid::from(1000),
        gid: Gid::from(1000),
        num_bytes: NumBytes::from(532),
        num_blocks: None,
        atime: now,
        mtime: now,
        ctime: now,
    }
}

fn some_symlink_attrs() -> NodeAttrs {
    let now = SystemTime::now();
    NodeAttrs {
        nlink: 1,
        mode: Mode::default().add_symlink_flag(),
        uid: Uid::from(1000),
        gid: Gid::from(1000),
        num_bytes: NumBytes::from(532),
        num_blocks: None,
        atime: now,
        mtime: now,
        ctime: now,
    }
}
