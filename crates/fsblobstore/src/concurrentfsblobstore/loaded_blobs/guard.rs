use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use cryfs_blobstore::{BlobId, BlobStore, RemoveResult};
use cryfs_concurrent_store::{LoadedEntryGuard, RequestImmediateDropResult};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard, AsyncDropTokioMutex};
use lockable::InfallibleUnwrap as _;

use crate::fsblobstore::FsBlob;

#[derive(Debug)]
pub struct LoadedBlobGuard<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    loaded_blob: AsyncDropGuard<
        LoadedEntryGuard<BlobId, AsyncDropTokioMutex<FsBlob<B>>, Arc<anyhow::Error>>,
    >,
}

impl<B> LoadedBlobGuard<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    pub(super) fn new(
        loaded_blob: AsyncDropGuard<
            LoadedEntryGuard<BlobId, AsyncDropTokioMutex<FsBlob<B>>, Arc<anyhow::Error>>,
        >,
    ) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self { loaded_blob })
    }

    pub fn blob_id(&self) -> BlobId {
        *self.loaded_blob.key()
    }

    pub async fn with_lock<R, F>(&self, f: F) -> R
    where
        F: AsyncFnOnce(&mut FsBlob<B>) -> R,
    {
        let mut guard = self.loaded_blob.value().lock().await;
        f(&mut *guard).await
    }

    pub async fn remove(
        mut this: AsyncDropGuard<Self>,
    ) -> Result<RemoveResult, Arc<anyhow::Error>> {
        loop {
            match this.loaded_blob.request_immediate_drop(
                |blob| async move {
                    let Some(blob) = blob else {
                        panic!("The blob wasn't loaded. This can't happen because we hold the LoadedBlobGuard");
                    };
                    let blob = AsyncDropTokioMutex::into_inner(blob);
                    FsBlob::remove(blob).await?;
                    Ok(RemoveResult::SuccessfullyRemoved)
                },
            ) {
                RequestImmediateDropResult::ImmediateDropRequested { drop_result } => {
                    // Drop the blob so we don't hold a lock on it, which would prevent the removal. Removal waits until all readers relinquished their blob.
                    this.async_drop().await?;
                    std::mem::drop(this);
                    // Wait until the blob is removed. If there are other readers, this will wait.
                    return drop_result.await;
                }
                RequestImmediateDropResult::AlreadyDropping { future } => {
                    // Blob is currently dropping, let's wait until that is done and then retry
                    future.await;
                    continue;
                }
            }
        }
    }
}

#[async_trait]
impl<B> AsyncDrop for LoadedBlobGuard<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.loaded_blob.async_drop().await.infallible_unwrap();
        Ok(())
    }
}
