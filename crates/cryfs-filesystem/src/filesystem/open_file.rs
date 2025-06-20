use async_trait::async_trait;
use cryfs_rustfs::PathComponent;
use futures::join;
use std::sync::Arc;
use std::time::SystemTime;
use std::{fmt::Debug, marker::PhantomData};

use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_rustfs::{
    FsError, FsResult, Gid, Mode, NodeAttrs, NumBytes, Uid, object_based_api::OpenFile,
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard},
    data::Data,
    with_async_drop_2,
};

use super::node_info::NodeInfo;
use crate::filesystem::fsblobstore::DirBlob;
use crate::filesystem::node_info::{BlobDetails, CallbackWithParentBlob};
use crate::filesystem::{
    concurrentfsblobstore::{ConcurrentFsBlob, ConcurrentFsBlobStore},
    fsblobstore::{FileBlob, FsBlob},
};

// TODO Make sure we don't keep a lock on the file blob, or keep the lock in an Arc that is shared between all File, Node and OpenFile instances of the same file

pub struct CryOpenFile<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    blobstore: AsyncDropGuard<AsyncDropArc<ConcurrentFsBlobStore<B>>>,
    node_info: Arc<NodeInfo>,
}

impl<B> CryOpenFile<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    pub fn new(
        blobstore: AsyncDropGuard<AsyncDropArc<ConcurrentFsBlobStore<B>>>,
        node_info: Arc<NodeInfo>,
    ) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            blobstore,
            node_info,
        })
    }

    async fn load_blob(&self) -> FsResult<AsyncDropGuard<ConcurrentFsBlob<B>>> {
        self.node_info.load_blob(&self.blobstore).await
    }

    pub fn as_file_mut<'a, 's>(blob: &'s mut FsBlob<B>) -> FsResult<&'s mut FileBlob<B>>
    where
        B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
        <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
    {
        let blob_id = blob.blob_id();
        blob.as_file_mut().map_err(|err| {
            FsError::CorruptedFilesystem {
                // TODO Add to message what it actually is
                message: format!("Blob {:?} is listed as a file in its parent directory but is actually not a file: {err:?}", blob_id),
            }
        })
    }

    async fn flush_file_contents(&self) -> FsResult<()> {
        let blob = self.load_blob().await?;
        with_async_drop_2!(blob, {
            blob.with_lock(async |mut blob| {
                let file = Self::as_file_mut(&mut blob).map_err(|err| {
                    log::error!("Failed to cast blob to FileBlob: {err:?}");
                    FsError::UnknownError
                })?;
                // TODO Can we change this to a BlobStore::flush(blob_id) method because such a method can avoid loading the blob if it isn't in any cache anyway?
                file.flush().await.map_err(|err| {
                    log::error!("Failed to fsync blob: {err:?}");
                    FsError::UnknownError
                })?;

                Ok(())
            })
            .await
        })
    }

    async fn flush_metadata(&self) -> FsResult<()> {
        struct Callback<B>
        where
            B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
            <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
        {
            _phantom: PhantomData<B>,
        }
        impl<B> CallbackWithParentBlob<B, ()> for Callback<B>
        where
            B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
            <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
        {
            async fn on_is_rootdir(self, _root_blob: BlobId) -> FsResult<()> {
                panic!("A file can't be the root dir");
            }

            async fn on_is_not_rootdir<'a, 'b, 'c>(
                self,
                parent_blob: &'a mut DirBlob<B>,
                _name: &'b PathComponent,
                _blob_details: &'c BlobDetails,
            ) -> FsResult<()> {
                // TODO Can we change this to a BlobStore::flush(blob_id) method because such a method can avoid loading the blob if it isn't in any cache anyway?
                parent_blob.flush().await.map_err(|err| {
                    log::error!("Failed to fsync parent blob: {err:?}");
                    FsError::UnknownError
                })?;
                Ok(())
            }
        }
        self.node_info
            .with_parent_blob(
                &self.blobstore,
                Callback {
                    _phantom: PhantomData,
                },
            )
            .await?;

        Ok(())
    }

    async fn _read(&self, offset: NumBytes, size: NumBytes) -> FsResult<Data> {
        let blob = self.load_blob().await?;
        with_async_drop_2!(blob, {
            blob.with_lock(async |mut blob| {
                let file = Self::as_file_mut(&mut blob).map_err(|err| {
                    log::error!("Failed to cast blob to FileBlob: {err:?}");
                    FsError::UnknownError
                })?;
                // TODO Is it better to have try_read return a Data object instead of a &mut [u8]? Or should we instead make OpenFile::read() take a &mut [u8]?
                //      The current way of mapping between the two ways of doing it in here is probably not optimal.
                let mut data: Data = vec![0; u64::from(size) as usize].into();
                // TODO Push down the NumBytes type and use it in blobstore/blockstore interfaces?
                let num_read_bytes =
                    file.try_read(&mut data, offset.into())
                        .await
                        .map_err(|err| {
                            log::error!("Failed to read from blob: {err:?}");
                            FsError::UnknownError
                        })?;
                data.shrink_to_subregion(..num_read_bytes);
                Ok(data)
            })
            .await
        })
    }

    async fn _write(&self, offset: NumBytes, data: Data) -> FsResult<()> {
        let blob = self.load_blob().await?;
        with_async_drop_2!(blob, {
            blob.with_lock(async |mut blob| {
                let file = Self::as_file_mut(&mut blob).map_err(|err| {
                    log::error!("Failed to cast blob to FileBlob: {err:?}");
                    FsError::UnknownError
                })?;
                // TODO Push down the NumBytes type and use it in blobstore/blockstore interfaces?
                file.write(&data, offset.into()).await.map_err(|err| {
                    log::error!("Failed to write to blob: {err:?}");
                    FsError::UnknownError
                })
            })
            .await
        })
    }
}

impl<B> Debug for CryOpenFile<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CryOpenFile")
            .field("node_info", &self.node_info)
            .finish()
    }
}

#[async_trait]
impl<B> OpenFile for CryOpenFile<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    async fn getattr(&self) -> FsResult<NodeAttrs> {
        self.node_info.getattr(&self.blobstore).await
    }

    async fn setattr(
        &self,
        mode: Option<Mode>,
        uid: Option<Uid>,
        gid: Option<Gid>,
        size: Option<NumBytes>,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
        ctime: Option<SystemTime>,
    ) -> FsResult<NodeAttrs> {
        self.node_info
            .setattr(&self.blobstore, mode, uid, gid, size, atime, mtime, ctime)
            .await
    }

    async fn read(&self, offset: NumBytes, size: NumBytes) -> FsResult<Data> {
        // read should update atime if (and only if) size > 0, see https://pubs.opengroup.org/onlinepubs/9699919799/functions/read.html
        let should_update_atime = size > NumBytes::from(0);
        if should_update_atime {
            self.node_info
                .concurrently_maybe_update_access_timestamp_in_parent(&self.blobstore, async || {
                    self._read(offset, size).await
                })
                .await
        } else {
            self._read(offset, size).await
        }
    }

    async fn write(&self, offset: NumBytes, data: Data) -> FsResult<()> {
        // write should update mtime if (and only if) size > 0, see https://pubs.opengroup.org/onlinepubs/9699919799/functions/write.html
        let should_update_mtime = data.len() > 0;
        if should_update_mtime {
            self.node_info
                .concurrently_update_modification_timestamp_in_parent(&self.blobstore, async || {
                    self._write(offset, data).await
                })
                .await
        } else {
            self._write(offset, data).await
        }
    }

    async fn flush(&self) -> FsResult<()> {
        // Flush is different from fsync, it's not meant to flush contents or metadata to disk,
        // but it's meant to give the file system a chance to return an error when a descriptor
        // is closed (calls to close() can't return errors in fuse).
        // We're just ignoring the call to flush() here.
        // TODO Is there actually something we should do?

        // TODO For now we're calling fsync here because C++ was doing that, so we have a fairer performance comparison. But we should remove this
        self.fsync(false).await?;
        Ok(())
    }

    async fn fsync(&self, datasync: bool) -> FsResult<()> {
        if datasync {
            self.flush_file_contents().await?;
        } else {
            let (r1, r2) = join!(self.flush_file_contents(), self.flush_metadata());
            // TODO Report both errors if both happen
            r1?;
            r2?;
        }
        Ok(())
    }
}

#[async_trait]
impl<B> AsyncDrop for CryOpenFile<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    <B as BlobStore>::ConcreteBlob: Send + Sync + AsyncDrop<Error = anyhow::Error>,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), FsError> {
        self.blobstore.async_drop().await
    }
}
