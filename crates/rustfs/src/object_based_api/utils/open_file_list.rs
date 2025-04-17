use async_trait::async_trait;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};
use std::sync::Mutex;

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
    open_files: Mutex<AsyncDropGuard<HandleMap<FileHandle, AsyncDropArc<OF>>>>,
}

impl<OF> OpenFileList<OF>
where
    OF: OpenFile + AsyncDrop<Error = FsError> + Send + Sync,
{
    pub fn new() -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            open_files: Mutex::new(HandleMap::new()),
        })
    }

    pub async fn get<R>(
        &self,
        fh: FileHandle,
        callback: impl AsyncFnOnce(&OF) -> Result<R, FsError>,
    ) -> Result<R, FsError> {
        let mut open_file = {
            let open_files = self.open_files.lock().unwrap();
            let open_file = open_files.get(fh).ok_or_else(|| {
                log::error!("no open file with handle {}", u64::from(fh));
                FsError::InvalidFileDescriptor { fh: u64::from(fh) }
            })?;

            AsyncDropArc::clone(open_file)

            // Drop the lock before running the operation on the open file so that other operations
            // can run concurrently.
        };

        let result = callback(&open_file).await;

        open_file.async_drop().await?;
        result
    }

    pub fn add(&self, open_file: AsyncDropGuard<OF>) -> HandleWithGeneration<FileHandle> {
        let mut open_files = self.open_files.lock().unwrap();
        open_files.add(AsyncDropArc::new(open_file))
    }

    pub fn remove(&self, fh: FileHandle) -> AsyncDropGuard<AsyncDropArc<OF>> {
        let mut open_files = self.open_files.lock().unwrap();
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
        let open_files = std::mem::replace(
            &mut self.open_files,
            Mutex::new(AsyncDropGuard::new_invalid()),
        );
        let mut open_files = open_files.into_inner().unwrap();
        open_files.async_drop().await.unwrap();
        Ok(())
    }
}
