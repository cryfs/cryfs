use async_trait::async_trait;
use cryfs_rustfs::{
    Data, File, FsResult, Gid, Mode, NodeAttrs, NumBytes, OpenFile, OpenFlags, Uid,
};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use super::device::InMemoryDevice;
use super::node::IsInMemoryNode;

struct InMemoryFileImpl {
    metadata: NodeAttrs,
    data: Vec<u8>,
}

pub struct InMemoryFile {
    // TODO Here (and also in InMemoryDir/Symlink), can we avoid the Mutex by using Rust's `&mut` for functions that modify data?
    implementation: Arc<Mutex<InMemoryFileImpl>>,
}

#[async_trait]
impl File for InMemoryFile {
    type Device = InMemoryDevice;

    async fn open(&self, _openflags: OpenFlags) -> FsResult<Self> {
        // TODO Honor openflags
        Ok(Self {
            implementation: Arc::clone(&self.implementation),
        })
    }

    async fn truncate(&self, new_size: NumBytes) -> FsResult<()> {
        self.implementation
            .lock()
            .unwrap()
            .data
            // TODO No unwrap
            .resize(usize::try_from(u64::from(new_size)).unwrap(), 0u8);
        Ok(())
    }
}

#[async_trait]
impl OpenFile for InMemoryFile {
    async fn getattr(&self) -> FsResult<NodeAttrs> {
        // TODO Deduplicate with implementation in InMemoryNode
        Ok(self.implementation.lock().unwrap().metadata)
    }

    async fn chmod(&self, mode: Mode) -> FsResult<()> {
        // TODO Deduplicate with implementation in InMemoryNode
        self.update_metadata(|metadata| {
            metadata.mode = Mode::from(mode);
        });
        Ok(())
    }

    async fn chown(&self, uid: Option<Uid>, gid: Option<Gid>) -> FsResult<()> {
        // TODO Deduplicate with implementation in InMemoryNode
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
        File::truncate(self, new_size).await
    }

    async fn utimens(
        &self,
        last_access: Option<SystemTime>,
        last_modification: Option<SystemTime>,
    ) -> FsResult<()> {
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
        todo!()
    }

    async fn write(&self, offset: NumBytes, data: Data) -> FsResult<()> {
        todo!()
    }

    async fn flush(&self) -> FsResult<()> {
        Ok(())
    }

    async fn fsync(&self, datasync: bool) -> FsResult<()> {
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
