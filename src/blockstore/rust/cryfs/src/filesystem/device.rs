use anyhow::Result;
use async_trait::async_trait;
use std::fmt::Debug;

use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_rustfs::{object_based_api::Device, AbsolutePath, FsError, FsResult, Statfs};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard},
    safe_panic,
};

use super::{
    dir::CryDir, file::CryFile, node::CryNode, node_info::NodeInfo, open_file::CryOpenFile,
    symlink::CrySymlink,
};
use crate::filesystem::fsblobstore::{BlobType, FsBlob, FsBlobStore};

pub struct CryDevice<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
{
    blobstore: AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
    root_blob_id: BlobId,
}

impl<B> CryDevice<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
{
    pub fn load_filesystem(blobstore: AsyncDropGuard<B>, root_blob_id: BlobId) -> Self {
        Self {
            blobstore: AsyncDropArc::new(FsBlobStore::new(blobstore)),
            root_blob_id,
        }
    }

    pub async fn create_new_filesystem(
        blobstore: AsyncDropGuard<B>,
        root_blob_id: BlobId,
    ) -> Result<Self> {
        let mut fsblobstore = FsBlobStore::new(blobstore);
        match fsblobstore.create_root_dir_blob(&root_blob_id).await {
            Ok(()) => Ok(Self {
                blobstore: AsyncDropArc::new(fsblobstore),
                root_blob_id,
            }),
            Err(err) => {
                fsblobstore.async_drop().await?;
                Err(err)
            }
        }
    }
}

impl<B> CryDevice<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
{
    async fn load_blob(&self, path: &AbsolutePath) -> FsResult<AsyncDropGuard<FsBlob<'_, B>>> {
        let mut current_blob = self
            .blobstore
            .load(&self.root_blob_id)
            .await
            .map_err(|err| {
                log::error!("Failed to load root blob: {err:?}");
                FsError::Custom {
                    error_code: libc::EIO,
                }
            })?
            .ok_or_else(|| {
                log::error!("Root blob not found");
                FsError::Custom {
                    error_code: libc::EIO,
                }
            })?;
        if current_blob.blob_type() != BlobType::Dir {
            log::error!("Root blob is not a directory");
            current_blob.async_drop().await.map_err(|_| {
                log::error!("Error dropping current_blob");
                FsError::UnknownError
            })?;
            return Err(FsError::Custom {
                error_code: libc::EIO,
            });
        }

        for path_component in path.iter() {
            // TODO This map_err is weird. Probably better to have into_dir return the right error type.
            let mut dir_blob = FsBlob::into_dir(current_blob)
                .await
                .map_err(|_| FsError::NodeIsNotADirectory)?;
            let entry = match dir_blob.entry_by_name(path_component) {
                Some(entry) => {
                    let blob_id = *entry.blob_id();
                    Ok(blob_id)
                }
                None => Err(FsError::NodeDoesNotExist),
            };
            dir_blob.async_drop().await.map_err(|_| {
                log::error!("Error dropping dir_blob");
                FsError::UnknownError
            })?;
            let entry = entry?;
            current_blob = self
                .blobstore
                .load(&entry)
                .await
                .map_err(|err| {
                    log::error!("Failed to load blob: {err:?}");
                    FsError::Custom {
                        error_code: libc::EIO,
                    }
                })?
                .ok_or_else(|| {
                    log::error!("Blob not found");
                    FsError::Custom {
                        error_code: libc::EIO,
                    }
                })?;
        }
        Ok(current_blob)
    }

    async fn load_node_info(&self, path: &AbsolutePath) -> FsResult<NodeInfo> {
        match path.split_last() {
            None => {
                // We're being asked to load the root dir
                Ok(NodeInfo::new_rootdir(self.root_blob_id))
            }
            Some((parent, node_name)) => {
                let parent_blob_id = match parent.split_last() {
                    None => {
                        // We're being asked to load a node that is a direct child of the root dir
                        Ok(self.root_blob_id)
                    }
                    Some((grandparent, parent_name)) => {
                        let grandparent = self.load_blob(grandparent).await?;
                        let mut grandparent = FsBlob::into_dir(grandparent)
                            .await
                            .map_err(|_| FsError::NodeIsNotADirectory)?;
                        let parent_entry = grandparent.entry_by_name(parent_name).cloned();
                        grandparent.async_drop().await.map_err(|_| {
                            log::error!("Error dropping parent");
                            FsError::UnknownError
                        })?;
                        match parent_entry {
                            Some(parent_entry) => {
                                let parent_blob_id = parent_entry.blob_id();
                                Ok(*parent_blob_id)
                            }
                            None => Err(FsError::NodeDoesNotExist),
                        }
                    }
                }?;
                Ok(NodeInfo::new(parent_blob_id, node_name.to_owned()))
            }
        }
    }
}

#[async_trait]
impl<B> Device for CryDevice<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
{
    type Node<'a> = CryNode<'a, B>;
    type Dir<'a> = CryDir<'a, B>;
    type Symlink<'a> = CrySymlink<'a, B>;
    type File<'a> = CryFile<B>;
    type OpenFile = CryOpenFile<B>;

    async fn load_node<'a>(&'a self, path: &AbsolutePath) -> FsResult<Self::Node<'a>> {
        let node_info = self.load_node_info(path).await?;
        Ok(CryNode::new(&self.blobstore, node_info))
    }

    async fn load_dir(&self, path: &AbsolutePath) -> FsResult<Self::Dir<'_>> {
        let node = self.load_node_info(path).await?;
        if node.node_type(&self.blobstore).await? == BlobType::Dir {
            Ok(CryDir::new(&self.blobstore, node))
        } else {
            Err(FsError::NodeIsNotADirectory)
        }
    }

    async fn load_symlink(&self, path: &AbsolutePath) -> FsResult<Self::Symlink<'_>> {
        let node = self.load_node_info(path).await?;
        if node.node_type(&self.blobstore).await? == BlobType::Symlink {
            Ok(CrySymlink::new(&self.blobstore, node))
        } else {
            Err(FsError::NodeIsNotASymlink)
        }
    }

    async fn load_file(&self, path: &AbsolutePath) -> FsResult<AsyncDropGuard<Self::File<'_>>> {
        let node = self.load_node_info(path).await?;
        match node.node_type(&self.blobstore).await? {
            BlobType::File => Ok(CryFile::new(AsyncDropArc::clone(&self.blobstore), node)),
            BlobType::Symlink => {
                // TODO What's the right error here?
                Err(FsError::UnknownError)
            }
            BlobType::Dir => Err(FsError::NodeIsADirectory),
        }
    }

    async fn statfs(&self) -> FsResult<Statfs> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }

    async fn destroy(mut self) {
        // TODO Can we do this without unwrap?
        self.blobstore.async_drop().await.unwrap();
    }
}

impl<B> Drop for CryDevice<B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'a> <B as BlobStore>::ConcreteBlob<'a>: Send + Sync,
{
    fn drop(&mut self) {
        if !self.blobstore.is_dropped() {
            safe_panic!("CryDevice dropped without calling destroy() first");
        }
    }
}
