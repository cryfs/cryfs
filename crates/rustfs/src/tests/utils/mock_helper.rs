use mockall::predicate::{always, eq};
use std::time::{Duration, SystemTime};

use super::MockAsyncFilesystemLL;
use crate::common::{
    AbsolutePath, FsError, Gid, HandleWithGeneration, InodeNumber, Mode, NodeAttrs, NumBytes,
    PathComponent, Uid,
};
use crate::low_level_api::ReplyEntry;

const ROOT_INO: InodeNumber = InodeNumber::from_const(fuser::FUSE_ROOT_ID);

pub struct MockHelper<'a> {
    mock_filesystem: &'a mut MockAsyncFilesystemLL,
}

impl<'a> MockHelper<'a> {
    pub fn new(mock_filesystem: &'a mut MockAsyncFilesystemLL) -> Self {
        Self { mock_filesystem }
    }

    pub fn allow_any_access_calls(&mut self) {
        self.mock_filesystem
            .expect_access()
            .returning(|_, _, _| Ok(()));
    }

    pub fn expect_lookup_doesnt_exist(&mut self, ino: InodeNumber, name: &PathComponent) {
        self.mock_filesystem
            .expect_lookup()
            .once()
            .with(always(), eq(ino), eq(name.to_owned()))
            .returning(|_, _, _| Err(FsError::NodeDoesNotExist));
    }

    /// Expect to lookup all intermediate directories on `path`.
    /// The final entry will be returned to be a directory, and its inode will be returned from this function.
    pub fn expect_lookup_dir_path_exists(&mut self, path: &AbsolutePath) -> InodeNumber {
        let mut current_ino = ROOT_INO;
        for node in path {
            let next_ino = InodeNumber::from(u64::from(current_ino) + 12); // 12 is an arbitrary offset to generate different INOs
            self.mock_filesystem
                .expect_lookup()
                .once()
                .with(always(), eq(current_ino), eq(node.to_owned()))
                .returning(move |_, _, _| {
                    Ok(ReplyEntry {
                        ino: HandleWithGeneration {
                            handle: next_ino,
                            generation: 0,
                        },
                        attr: some_dir_attrs(),
                        ttl: Duration::from_secs(1),
                    })
                });
            current_ino = next_ino;
        }
        current_ino
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
