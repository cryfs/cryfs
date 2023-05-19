use async_trait::async_trait;
use cryfs_rustfs::{
    Data, File, FsError, FsResult, Gid, Mode, NodeAttrs, NumBytes, OpenFile, OpenFlags, Uid,
};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use super::device::InMemoryDevice;
use super::inode_metadata::{chmod, chown, utimens};

// Inode is in separate module so we can ensure class invariant through public/private boundaries
mod inode {
    use super::*;

    pub struct FileInode {
        // Invariant: metadata.num_bytes == data.len()
        metadata: NodeAttrs,
        data: Vec<u8>,
    }

    impl FileInode {
        pub fn new(mode: Mode, uid: Uid, gid: Gid) -> Self {
            Self {
                metadata: NodeAttrs {
                    // TODO What are the right file attributes here?
                    nlink: 1,
                    mode,
                    uid,
                    gid,
                    num_bytes: NumBytes::from(0),
                    num_blocks: None,
                    atime: SystemTime::now(),
                    mtime: SystemTime::now(),
                    ctime: SystemTime::now(),
                },
                data: vec![],
            }
        }

        pub fn metadata(&self) -> &NodeAttrs {
            &self.metadata
        }

        pub fn resize(&mut self, new_size: NumBytes) {
            // TODO No unwrap
            let new_size_usize = usize::try_from(u64::from(new_size)).unwrap();
            self.data.resize(new_size_usize, 0u8);
            self.metadata.num_bytes = new_size;
        }

        pub fn len(&self) -> NumBytes {
            self.metadata.num_bytes
        }

        pub fn data(&self) -> &[u8] {
            &self.data
        }

        pub fn data_mut(&mut self) -> &mut [u8] {
            &mut self.data
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

use inode::FileInode;

pub struct InMemoryFileRef {
    // TODO Here (and also in InMemoryDir/Symlink), can we avoid the Mutex by using Rust's `&mut` for functions that modify data?
    inode: Arc<Mutex<FileInode>>,
}

impl InMemoryFileRef {
    pub fn new(mode: Mode, uid: Uid, gid: Gid) -> Self {
        Self {
            inode: Arc::new(Mutex::new(FileInode::new(mode, uid, gid))),
        }
    }

    pub fn clone_ref(&self) -> Self {
        Self {
            inode: Arc::clone(&self.inode),
        }
    }

    pub fn open_sync(&self, openflags: OpenFlags) -> InMemoryOpenFileRef {
        InMemoryOpenFileRef {
            openflags,
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
impl File for InMemoryFileRef {
    type Device = InMemoryDevice;

    async fn open(&self, openflags: OpenFlags) -> FsResult<InMemoryOpenFileRef> {
        Ok(self.open_sync(openflags))
    }

    async fn truncate(&self, new_size: NumBytes) -> FsResult<()> {
        self.inode.lock().unwrap().resize(new_size);
        Ok(())
    }
}

pub struct InMemoryOpenFileRef {
    openflags: OpenFlags,
    inode: Arc<Mutex<FileInode>>,
}

#[async_trait]
impl OpenFile for InMemoryOpenFileRef {
    async fn getattr(&self) -> FsResult<NodeAttrs> {
        // TODO Deduplicate with implementation in InMemoryNode
        // TODO Is getattr allowed when openflags are writeonly?
        Ok(*self.inode.lock().unwrap().metadata())
    }

    async fn chmod(&self, mode: Mode) -> FsResult<()> {
        // TODO Is chmod allowed when openflags are readonly?
        self.inode.lock().unwrap().chmod(mode);
        Ok(())
    }

    async fn chown(&self, uid: Option<Uid>, gid: Option<Gid>) -> FsResult<()> {
        // TODO Is chown allowed when openflags are readonly?
        self.inode.lock().unwrap().chown(uid, gid);
        Ok(())
    }

    async fn truncate(&self, new_size: NumBytes) -> FsResult<()> {
        match self.openflags {
            OpenFlags::Read => Err(FsError::WriteOnReadOnlyFileDescriptor),
            OpenFlags::Write | OpenFlags::ReadWrite => {
                self.inode.lock().unwrap().resize(new_size);
                Ok(())
            }
        }
    }

    async fn utimens(
        &self,
        last_access: Option<SystemTime>,
        last_modification: Option<SystemTime>,
    ) -> FsResult<()> {
        // TODO Is utimens allowed when openflags are readonly?
        self.inode
            .lock()
            .unwrap()
            .utimens(last_access, last_modification);
        Ok(())
    }

    async fn read(&self, offset: NumBytes, size: NumBytes) -> FsResult<Data> {
        match self.openflags {
            OpenFlags::Write => Err(FsError::ReadOnWriteOnlyFileDescriptor),
            OpenFlags::Read | OpenFlags::ReadWrite => {
                let offset = usize::try_from(u64::from(offset)).unwrap();
                let size = usize::try_from(u64::from(size)).unwrap();
                let inode = self.inode.lock().unwrap();
                let data = inode.data();
                let actually_read = std::cmp::min(size, data.len() - offset);
                Ok(data[offset..offset + actually_read].to_vec().into())
            }
        }
    }

    async fn write(&self, offset: NumBytes, data: Data) -> FsResult<()> {
        match self.openflags {
            OpenFlags::Read => Err(FsError::WriteOnReadOnlyFileDescriptor),
            OpenFlags::Write | OpenFlags::ReadWrite => {
                // TODO No unwrap
                let data_len = NumBytes::from(u64::try_from(data.len()).unwrap());
                let inode = &mut self.inode.lock().unwrap();
                if offset + data_len > inode.len() {
                    inode.resize(offset + data_len);
                }
                let offset = usize::try_from(u64::from(offset)).unwrap();
                inode.data_mut()[offset..offset + data.len()].copy_from_slice(&data);
                Ok(())
            }
        }
    }

    async fn flush(&self) -> FsResult<()> {
        // TODO Is flush allowed when openflags are readonly?
        // No need to flush because we're in-memory
        Ok(())
    }

    async fn fsync(&self, datasync: bool) -> FsResult<()> {
        // TODO Is fsync allowed when openflags are readonly?
        // No need to fsync because we're in-memory
        Ok(())
    }
}
