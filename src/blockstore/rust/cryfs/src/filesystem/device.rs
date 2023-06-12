use async_trait::async_trait;
use std::fmt::Debug;

use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_rustfs::{object_based_api::Device, AbsolutePath, FsError, FsResult, Statfs};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard},
    safe_panic,
};

use super::{
    dir::CryDir, file::CryFile, node::CryNode, open_file::CryOpenFile, symlink::CrySymlink,
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
    pub fn new(blobstore: AsyncDropGuard<B>, root_blob_id: BlobId) -> Self {
        Self {
            blobstore: AsyncDropArc::new(FsBlobStore::new(blobstore)),
            root_blob_id,
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
    type File<'a> = CryFile<'a, B>;
    type OpenFile = CryOpenFile<B>;

    async fn load_node(&self, path: &AbsolutePath) -> FsResult<Self::Node<'_>> {
        match path.split_last() {
            None => {
                // We're being asked to load the root dir
                Ok(CryNode::new_rootdir(&self.blobstore, self.root_blob_id))
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
                Ok(CryNode::new(
                    &self.blobstore,
                    parent_blob_id,
                    node_name.to_owned(),
                ))
            }
        }
    }

    async fn load_dir(&self, path: &AbsolutePath) -> FsResult<Self::Dir<'_>> {
        let node = self.load_node(path).await?;
        if node.node_type().await? == BlobType::Dir {
            Ok(CryDir::new(node))
        } else {
            Err(FsError::NodeIsNotADirectory)
        }
    }

    async fn load_symlink(&self, path: &AbsolutePath) -> FsResult<Self::Symlink<'_>> {
        let node = self.load_node(path).await?;
        if node.node_type().await? == BlobType::Symlink {
            Ok(CrySymlink::new(node))
        } else {
            Err(FsError::NodeIsNotASymlink)
        }
    }

    async fn load_file(&self, path: &AbsolutePath) -> FsResult<Self::File<'_>> {
        let node = self.load_node(path).await?;
        match node.node_type().await? {
            BlobType::File => Ok(CryFile::new(node)),
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
