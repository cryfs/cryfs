use anyhow::Result;
use async_trait::async_trait;
use byte_unit::Byte;
use std::{fmt::Debug, sync::Arc};

use cryfs_blobstore::{BlobId, BlobStore, RemoveResult};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard},
    with_async_drop_2,
};

use crate::{
    concurrentfsblobstore::{
        ConcurrentFsBlob,
        loaded_blobs::{LoadedBlobs, RequestRemovalResult},
    },
    fsblobstore::{FlushBehavior, FsBlobStore},
};

#[derive(Debug)]
pub struct ConcurrentFsBlobStore<B>
where
    // TODO Do we really need B: 'static ?
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    blobstore: AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
    loaded_blobs: AsyncDropGuard<AsyncDropArc<LoadedBlobs<B>>>,
}

impl<B> ConcurrentFsBlobStore<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    pub fn new(blobstore: AsyncDropGuard<FsBlobStore<B>>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            blobstore: AsyncDropArc::new(blobstore),
            loaded_blobs: AsyncDropArc::new(LoadedBlobs::new()),
        })
    }

    pub async fn create_root_dir_blob(
        &self,
        root_blob_id: &BlobId,
    ) -> Result<(), Arc<anyhow::Error>> {
        let root_blob_id = *root_blob_id;
        let blobstore = AsyncDropArc::clone(&self.blobstore);
        LoadedBlobs::try_insert_loading(&self.loaded_blobs, root_blob_id, async move || {
            with_async_drop_2!(blobstore, {
                blobstore.create_root_dir_blob(&root_blob_id).await
            })
            .map_err(Arc::new)
        })
        .await
    }

    pub async fn create_file_blob(
        &self,
        parent: &BlobId,
        flush_behavior: FlushBehavior,
    ) -> Result<AsyncDropGuard<ConcurrentFsBlob<B>>> {
        let blob = self
            .blobstore
            .create_file_blob(parent, flush_behavior)
            .await?;
        let inserted = LoadedBlobs::try_insert_loaded(&self.loaded_blobs, blob)
            .await
            .expect("This can't fail because the blob id is new");
        Ok(ConcurrentFsBlob::new(inserted))
    }

    pub async fn create_dir_blob(
        &self,
        parent: &BlobId,
        flush_behavior: FlushBehavior,
    ) -> Result<AsyncDropGuard<ConcurrentFsBlob<B>>> {
        let blob = self
            .blobstore
            .create_dir_blob(parent, flush_behavior)
            .await?;
        let inserted = LoadedBlobs::try_insert_loaded(&self.loaded_blobs, blob)
            .await
            .expect("This can't fail because the blob id is new");
        Ok(ConcurrentFsBlob::new(inserted))
    }

    pub async fn create_symlink_blob(
        &self,
        parent: &BlobId,
        target: &str,
        flush_behavior: FlushBehavior,
    ) -> Result<AsyncDropGuard<ConcurrentFsBlob<B>>> {
        let blob = self
            .blobstore
            .create_symlink_blob(parent, target, flush_behavior)
            .await?;
        let inserted = LoadedBlobs::try_insert_loaded(&self.loaded_blobs, blob)
            .await
            .expect("This can't fail because the blob id is new");
        Ok(ConcurrentFsBlob::new(inserted))
    }

    pub async fn load(
        &self,
        blob_id: &BlobId,
    ) -> Result<Option<AsyncDropGuard<ConcurrentFsBlob<B>>>, Arc<anyhow::Error>> {
        let blob_id = *blob_id;
        let loaded_blob = LoadedBlobs::get_loaded_or_insert_loading(
            &self.loaded_blobs,
            blob_id,
            &self.blobstore,
            async move |blobstore| {
                with_async_drop_2!(blobstore, { blobstore.load(&blob_id).await }).map_err(Arc::new)
            },
        )
        .await?;
        Ok(loaded_blob.map(ConcurrentFsBlob::new))
    }

    pub async fn num_blocks(&self) -> Result<u64> {
        self.blobstore.num_blocks().await
    }

    pub fn estimate_space_for_num_blocks_left(&self) -> Result<u64> {
        self.blobstore.estimate_space_for_num_blocks_left()
    }

    // logical means "space we can use" as opposed to "space it takes on the disk" (i.e. logical is without headers, checksums, ...)
    pub fn logical_block_size_bytes(&self) -> Byte {
        self.blobstore.logical_block_size_bytes()
    }

    pub async fn remove_by_id(&self, id: &BlobId) -> anyhow::Result<RemoveResult> {
        loop {
            match self.loaded_blobs.request_removal(*id, &self.blobstore) {
                RequestRemovalResult::RemovalRequested { on_removed } => {
                    // Wait until the blob is removed
                    return on_removed.await;
                }
                RequestRemovalResult::AlreadyDropping { future } => {
                    // Blob is currently dropping, let's wait until that is done and then retry
                    future.await;
                    continue;
                }
            }
        }
    }

    /// Flush the blob if it is loaded or cached somewhere. If it is not loaded or cached, do nothing.
    pub async fn flush_if_cached(&self, blob_id: BlobId) -> Result<(), Arc<anyhow::Error>> {
        if let Some(loaded_blob) =
            LoadedBlobs::get_if_loading_or_loaded(&self.loaded_blobs, blob_id).await?
        {
            // Blob is loaded, we can flush it directly.
            loaded_blob
                .with_lock(async |blob| blob.flush().await)
                .await?;
        } else {
            // Blob is not loaded. But it may have been previously async_dropped without a flush, which may cause it's blocks to still be in a lower level blockstore cache.
            // We need to flush those as well
            self.blobstore.flush_if_cached(blob_id).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl<B> AsyncDrop for ConcurrentFsBlobStore<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + AsyncDrop<Error = anyhow::Error>,
{
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        // First drop all loaded blobs
        self.loaded_blobs.async_drop().await?;

        // Then drop the underlying blobstore
        self.blobstore.async_drop().await?;

        Ok(())
    }
}
