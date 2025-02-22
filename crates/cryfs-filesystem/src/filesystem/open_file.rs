use async_trait::async_trait;
use futures::join;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::SystemTime;

use cryfs_blobstore::BlobStore;
use cryfs_rustfs::{
    FsError, FsResult, Gid, Mode, NodeAttrs, NumBytes, Uid, object_based_api::OpenFile,
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard},
    data::Data,
};

use super::node_info::{LoadParentBlobResult, NodeInfo};
use crate::filesystem::fsblobstore::{FileBlob, FsBlobStore};

// TODO Make sure we don't keep a lock on the file blob, or keep the lock in an Arc that is shared between all File, Node and OpenFile instances of the same file

pub struct CryOpenFile<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
{
    blobstore: AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
    node_info: Arc<NodeInfo>,
}

impl<B> CryOpenFile<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
{
    pub fn new(
        blobstore: AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
        node_info: Arc<NodeInfo>,
    ) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            blobstore,
            node_info,
        })
    }

    async fn load_blob<'a>(&self) -> FsResult<FileBlob<'_, B>> {
        self.node_info.load_file_blob(&self.blobstore).await
    }

    async fn flush_file_contents(&self) -> FsResult<()> {
        let mut blob = self.load_blob().await?;
        // TODO Can we change this to a BlobStore::flush(blob_id) method because such a method can avoid loading the blob if it isn't in any cache anyway?
        blob.flush().await.map_err(|err| {
            log::error!("Failed to fsync blob: {err:?}");
            FsError::UnknownError
        })?;

        Ok(())
    }

    async fn flush_metadata(&self) -> FsResult<()> {
        match self.node_info.load_parent_blob(&self.blobstore).await? {
            LoadParentBlobResult::IsRootDir { .. } => {
                panic!("A file can't be the root dir");
            }
            LoadParentBlobResult::IsNotRootDir {
                mut parent_blob, ..
            } => {
                // TODO Can we change this to a BlobStore::flush(blob_id) method because such a method can avoid loading the blob if it isn't in any cache anyway?
                parent_blob.flush().await.map_err(|err| {
                    log::error!("Failed to fsync parent blob: {err:?}");
                    FsError::UnknownError
                })?;
                parent_blob.async_drop().await?;
            }
        }
        Ok(())
    }
}

impl<B> Debug for CryOpenFile<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
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
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
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
        let mut blob = self.load_blob().await?;
        // TODO Is it better to have try_read return a Data object instead of a &mut [u8]? Or should we instead make OpenFile::read() take a &mut [u8]?
        //      The current way of mapping between the two ways of doing it in here is probably not optimal.
        let mut data: Data = vec![0; u64::from(size) as usize].into();
        // TODO Push down the NumBytes type and use it in blobstore/blockstore interfaces?
        let num_read_bytes = blob
            .try_read(&mut data, offset.into())
            .await
            .map_err(|err| {
                log::error!("Failed to read from blob: {err:?}");
                FsError::UnknownError
            })?;
        data.shrink_to_subregion(..num_read_bytes);
        Ok(data)
    }

    async fn write(&self, offset: NumBytes, data: Data) -> FsResult<()> {
        let mut blob = self.load_blob().await?;
        // TODO Push down the NumBytes type and use it in blobstore/blockstore interfaces?
        blob.write(&data, offset.into()).await.map_err(|err| {
            log::error!("Failed to write to blob: {err:?}");
            FsError::UnknownError
        })
    }

    async fn flush(&self) -> FsResult<()> {
        self.flush_file_contents().await
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
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), FsError> {
        self.blobstore.async_drop().await
    }
}
