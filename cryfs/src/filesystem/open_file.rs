use async_trait::async_trait;
use futures::join;
use std::fmt::Debug;
use std::time::SystemTime;

use cryfs_blobstore::BlobStore;
use cryfs_rustfs::{
    object_based_api::OpenFile, FsError, FsResult, Gid, Mode, NodeAttrs, NumBytes, Uid,
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard},
    data::Data,
};

use super::node_info::{LoadParentBlobResult, NodeInfo};
use crate::filesystem::fsblobstore::{FileBlob, FsBlob, FsBlobStore};

// TODO Make sure we don't keep a lock on the file blob, or keep the lock in an Arc that is shared between all File, Node and OpenFile instances of the same file

pub struct CryOpenFile<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
{
    blobstore: AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
    node_info: NodeInfo,
}

impl<B> CryOpenFile<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
{
    pub fn new(
        blobstore: AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
        node_info: NodeInfo,
    ) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            blobstore,
            node_info,
        })
    }

    async fn load_blob<'a>(&self) -> FsResult<FileBlob<'_, B>> {
        load_file_blob(&self.blobstore, &self.node_info).await
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
                parent_blob.async_drop().await.map_err(|err| {
                    log::error!("Failed to drop parent blob: {err:?}");
                    FsError::UnknownError
                })?;
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

    async fn chmod(&self, mode: Mode) -> FsResult<()> {
        self.node_info.chmod(&self.blobstore, mode).await
    }

    async fn chown(&self, uid: Option<Uid>, gid: Option<Gid>) -> FsResult<()> {
        self.node_info.chown(&self.blobstore, uid, gid).await
    }

    async fn truncate(&self, new_size: NumBytes) -> FsResult<()> {
        truncate_file(&self.blobstore, &self.node_info, new_size).await
    }

    async fn utimens(
        &self,
        last_access: Option<SystemTime>,
        last_modification: Option<SystemTime>,
    ) -> FsResult<()> {
        self.node_info
            .utimens(&self.blobstore, last_access, last_modification)
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
        self.blobstore.async_drop().await.map_err(|err| {
            log::error!("Failed to drop blobstore: {err:?}");
            FsError::Custom {
                error_code: libc::EIO,
            }
        })
    }
}

async fn load_file_blob<'a, B>(
    blobstore: &'a FsBlobStore<B>,
    node_info: &NodeInfo,
) -> Result<FileBlob<'a, B>, FsError>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    let blob = node_info.load_blob(blobstore).await?;
    let blob_id = blob.blob_id();
    FsBlob::into_file(blob).await.map_err(|err| {
        FsError::CorruptedFilesystem {
            // TODO Add to message what it actually is
            message: format!("Blob {:?} is listed as a directory in its parent directory but is actually not a directory: {err:?}", blob_id),
        }
    })
}

pub async fn truncate_file<B>(
    blobstore: &FsBlobStore<B>,
    node_info: &NodeInfo,
    new_size: NumBytes,
) -> FsResult<()>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    let mut blob = load_file_blob(blobstore, node_info).await?;
    blob.resize(new_size.into()).await.map_err(|err| {
        log::error!("Error resizing file blob: {err:?}");
        FsError::UnknownError
    })
}
