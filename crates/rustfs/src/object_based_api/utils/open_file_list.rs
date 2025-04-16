use async_trait::async_trait;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

use crate::{
    FsError,
    common::{FileHandle, HandleMap, HandleWithGeneration},
    object_based_api::OpenFile,
};

#[derive(Debug)]
pub struct OpenFileList<OF>
where
    OF: OpenFile + AsyncDrop<Error = FsError> + Send + Sync,
{
    // TODO Can we improve concurrency by locking less in open_files and instead making OpenFileList concurrency safe somehow?
    open_files: tokio::sync::RwLock<AsyncDropGuard<HandleMap<FileHandle, OF>>>,
}

impl<OF> OpenFileList<OF>
where
    OF: OpenFile + AsyncDrop<Error = FsError> + Send + Sync,
{
    pub fn new() -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            open_files: tokio::sync::RwLock::new(HandleMap::new()),
        })
    }

    pub async fn get<R>(
        &self,
        fh: FileHandle,
        callback: impl AsyncFnOnce(&OF) -> Result<R, FsError>,
    ) -> Result<R, FsError> {
        let open_files = self.open_files.read().await;
        let open_file = open_files.get(fh).ok_or_else(|| {
            log::error!("no open file with handle {}", u64::from(fh));
            FsError::InvalidFileDescriptor { fh: u64::from(fh) }
        })?;
        callback(&open_file).await
    }

    pub async fn add(&self, open_file: AsyncDropGuard<OF>) -> HandleWithGeneration<FileHandle> {
        let mut open_files = self.open_files.write().await;
        open_files.add(open_file)
    }

    pub async fn remove(&self, fh: FileHandle) -> AsyncDropGuard<OF> {
        let mut open_files = self.open_files.write().await;
        // TODO Since get() returns an error if the handle doesn't exist, maybe remove should as well instead of panicking
        open_files.remove(fh)
    }
}

#[async_trait]
impl<OF> AsyncDrop for OpenFileList<OF>
where
    OF: OpenFile + AsyncDrop<Error = FsError> + Send + Sync,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.open_files.write().await.async_drop().await.unwrap();
        Ok(())
    }
}
