use async_trait::async_trait;
use cryfs_rustfs::{
    Data, File, FsError, FsResult, Gid, Mode, NodeAttrs, NumBytes, OpenFile, OpenFlags, Uid,
};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use super::device::InMemoryDevice;
use super::node::IsInMemoryNode;

struct FileInode {
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
                blocks: 1,
                atime: SystemTime::now(),
                mtime: SystemTime::now(),
                ctime: SystemTime::now(),
            },
            data: vec![],
        }
    }

    pub fn resize(&mut self, new_size: NumBytes) {
        self.data
            // TODO No unwrap
            .resize(usize::try_from(u64::from(new_size)).unwrap(), 0u8);
        self.metadata.num_bytes = new_size;
    }

    pub fn len(&self) -> NumBytes {
        self.metadata.num_bytes
    }
}

pub struct InMemoryFile {
    // TODO Here (and also in InMemoryDir/Symlink), can we avoid the Mutex by using Rust's `&mut` for functions that modify data?
    implementation: Arc<Mutex<FileInode>>,
}

impl InMemoryFile {
    pub fn new(mode: Mode, uid: Uid, gid: Gid) -> Self {
        Self {
            implementation: Arc::new(Mutex::new(FileInode::new(mode, uid, gid))),
        }
    }

    pub fn open_sync(&self, openflags: OpenFlags) -> InMemoryOpenFile {
        InMemoryOpenFile {
            openflags,
            implementation: Arc::clone(&self.implementation),
        }
    }
}

#[async_trait]
impl File for InMemoryFile {
    type Device = InMemoryDevice;

    async fn open(&self, openflags: OpenFlags) -> FsResult<InMemoryOpenFile> {
        Ok(self.open_sync(openflags))
    }

    async fn truncate(&self, new_size: NumBytes) -> FsResult<()> {
        self.implementation.lock().unwrap().resize(new_size);
        Ok(())
    }
}

pub struct InMemoryOpenFile {
    openflags: OpenFlags,
    implementation: Arc<Mutex<FileInode>>,
}

#[async_trait]
impl OpenFile for InMemoryOpenFile {
    async fn getattr(&self) -> FsResult<NodeAttrs> {
        // TODO Deduplicate with implementation in InMemoryNode
        // TODO Is getattr allowed when openflags are writeonly?
        Ok(self.implementation.lock().unwrap().metadata)
    }

    async fn chmod(&self, mode: Mode) -> FsResult<()> {
        // TODO Deduplicate with implementation in InMemoryNode
        // TODO Is chmod allowed when openflags are readonly?
        self.update_metadata(|metadata| {
            metadata.mode = Mode::from(mode);
        });
        Ok(())
    }

    async fn chown(&self, uid: Option<Uid>, gid: Option<Gid>) -> FsResult<()> {
        // TODO Deduplicate with implementation in InMemoryNode
        // TODO Is chown allowed when openflags are readonly?
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

    async fn truncate(&self, new_size: NumBytes) -> FsResult<()> {
        match self.openflags {
            OpenFlags::Read => Err(FsError::WriteOnReadOnlyFileDescriptor),
            OpenFlags::Write | OpenFlags::ReadWrite => {
                self.implementation.lock().unwrap().resize(new_size);
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
        // TODO Deduplicate with implementation in InMemoryNode
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

    async fn read(&self, offset: NumBytes, size: NumBytes) -> FsResult<Data> {
        match self.openflags {
            OpenFlags::Write => Err(FsError::ReadOnWriteOnlyFileDescriptor),
            OpenFlags::Read | OpenFlags::ReadWrite => {
                let offset = usize::try_from(u64::from(offset)).unwrap();
                let size = usize::try_from(u64::from(size)).unwrap();
                let data = &self.implementation.lock().unwrap().data;
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
                let implementation = &mut self.implementation.lock().unwrap();
                if offset + data_len > implementation.len() {
                    implementation.resize(offset + data_len);
                }
                let offset = usize::try_from(u64::from(offset)).unwrap();
                implementation.data[offset..offset + data.len()].copy_from_slice(&data);
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

impl IsInMemoryNode for InMemoryFile {
    fn metadata(&self) -> NodeAttrs {
        self.implementation.lock().unwrap().metadata
    }

    fn update_metadata(&self, callback: impl FnOnce(&mut NodeAttrs)) {
        let mut data = self.implementation.lock().unwrap();
        callback(&mut data.metadata);
    }
}

impl IsInMemoryNode for InMemoryOpenFile {
    fn metadata(&self) -> NodeAttrs {
        self.implementation.lock().unwrap().metadata
    }

    fn update_metadata(&self, callback: impl FnOnce(&mut NodeAttrs)) {
        let mut data = self.implementation.lock().unwrap();
        callback(&mut data.metadata);
    }
}
