use async_trait::async_trait;
use std::fmt::Debug;
use std::path::Path;

use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_rustfs::{object_based_api::Device, FsError, FsResult, Statfs};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard},
    safe_panic,
};

use super::{
    dir::CryDir, file::CryFile, node::CryNode, open_file::CryOpenFile, symlink::CrySymlink,
};
use crate::filesystem::fsblobstore::{BlobType, DirBlob, EntryType, FsBlob, FsBlobStore};

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
    async fn load_blob(&self, path: &Path) -> FsResult<AsyncDropGuard<FsBlob<'_, B>>> {
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
        for path_component in path {
            // TODO Is to_string_lossy the right thing to do here? Seems entry_by_name has its own error handling for if it's not utf-8.
            let component = path_component.to_string_lossy();
            // TODO This map_err is weird. Probably better to have into_dir return the right error type.
            let mut dir_blob = FsBlob::into_dir(current_blob)
                .await
                .map_err(|_| FsError::NodeIsNotADirectory)?;
            let entry = match dir_blob.entry_by_name(&component) {
                Ok(Some(entry)) => entry,
                Ok(None) => {
                    dir_blob.async_drop().await.map_err(|_| {
                        log::error!("Error dropping dir_blob");
                        FsError::UnknownError
                    })?;
                    return Err(FsError::NodeDoesNotExist);
                }
                Err(err) => {
                    log::error!("File system has a directory with a non-UTF8 entry: {err:?}");
                    dir_blob.async_drop().await.map_err(|_| {
                        log::error!("Error dropping dir_blob");
                        FsError::UnknownError
                    })?;
                    return Err(FsError::Custom {
                        error_code: libc::EIO,
                    });
                }
            };
            let entry = *entry.blob_id();
            dir_blob.async_drop().await.map_err(|_| {
                log::error!("Error dropping dir_blob");
                FsError::UnknownError
            })?;
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

    async fn load_node(&self, path: &Path) -> FsResult<Self::Node<'_>> {
        assert!(path.is_absolute(), "Path must be absolute: {path:?}");
        assert!(path.has_root(), "Path must have root: {path:?}");
        assert!(
            path.components()
                .next()
                .map(|c| !matches!(c, std::path::Component::Prefix(_)))
                .unwrap_or(true),
            // TODO Is Component::Prefix actually the correct check here?
            "Path must not have a device specifier on windows: {path:?}"
        );
        match path.parent() {
            None => {
                // We're being asked to load the root dir
                Ok(CryNode::new(
                    &self.blobstore,
                    BlobType::Dir,
                    self.root_blob_id,
                ))
            }
            Some(parent) => {
                let parent = self.load_blob(parent).await?;
                let mut parent = FsBlob::into_dir(parent)
                    .await
                    .map_err(|_| FsError::NodeIsNotADirectory)?;
                // TODO No unwrap. How do handle missing file_name?
                let name = path.file_name().unwrap();
                // TODO Is to_string_lossy the right thing to do here? Seems entry_by_name has its own error handling for if it's not utf-8.
                let entry = parent
                    .entry_by_name(&name.to_string_lossy())
                    .map(|e| e.cloned());
                parent.async_drop().await.map_err(|_| {
                    log::error!("Error dropping parent");
                    FsError::UnknownError
                })?;
                match entry {
                    Ok(Some(child)) => {
                        let blob_id = child.blob_id();
                        let blob_type = match child.entry_type() {
                            EntryType::Dir => BlobType::Dir,
                            EntryType::File => BlobType::File,
                            EntryType::Symlink => BlobType::Symlink,
                        };
                        Ok(CryNode::new(&self.blobstore, blob_type, *blob_id))
                    }
                    Ok(None) => Err(FsError::NodeDoesNotExist),
                    Err(err) => {
                        log::error!("File system has a directory with a non-UTF8 entry: {err:?}");
                        Err(FsError::Custom {
                            error_code: libc::EIO,
                        })
                    }
                }
            }
        }
    }

    async fn load_dir(&self, path: &Path) -> FsResult<Self::Dir<'_>> {
        let node = self.load_node(path).await?;
        if node.node_type() == BlobType::Dir {
            Ok(CryDir::new(node))
        } else {
            Err(FsError::NodeIsNotADirectory)
        }
    }

    async fn load_symlink(&self, path: &Path) -> FsResult<Self::Symlink<'_>> {
        let node = self.load_node(path).await?;
        if node.node_type() == BlobType::Symlink {
            Ok(CrySymlink::new(node))
        } else {
            Err(FsError::NodeIsNotASymlink)
        }
    }

    async fn load_file(&self, path: &Path) -> FsResult<Self::File<'_>> {
        let node = self.load_node(path).await?;
        match node.node_type() {
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
