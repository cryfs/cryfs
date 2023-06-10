use async_trait::async_trait;
use std::fmt::Debug;
use std::time::SystemTime;
use tokio::sync::OnceCell;

use super::fsblobstore::{DirBlob, DirEntry, EntryType, FsBlob, DIR_LSTAT_SIZE};
use crate::filesystem::fsblobstore::{BlobType, FsBlobStore};
use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_rustfs::{
    object_based_api::Node, FsError, FsResult, Gid, Mode, NodeAttrs, NumBytes, Uid,
};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};

enum NodeInfo {
    IsRootDir {
        root_blob_id: BlobId,
    },
    IsNotRootDir {
        parent_blob_id: BlobId,
        name: String,
    },
}

struct NodeDetails {
    blob_id: BlobId,
    blob_type: BlobType,
}

pub struct CryNode<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    blobstore: &'a AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
    node_info: NodeInfo,

    // While fields in [node_info] are always set, a [CryNode] object can exist even before we
    // actually looked it up from the parent directory. In that case, [blob_details] is not initialized yet.
    // Once it was looked up, this will be initialized.
    // The reason for this is that we cannot hold a reference to the loaded parent blob in here because that would
    // lock it and prevent other threads from loading it, potentially leading to a deadlock. But some operations in here
    // (e.g. getattr, chmod, chown) need to load the parent blob. To prevent loading it multiple times, we avoid loading
    // the parent blob before instantiating the CryNode instance.
    // TODO Check if any of the Mutex'es used in the whole repository could be replaced by OnceCell, Cell, RefCell or similar
    // TODO Maybe it's actually ok to store a Blob here? Except for rename, all operations should just operate on one node
    //      and not depend on another node so we shouldn't cause any deadlocks. This would simplify what we're doing here
    //      and would also allow us to avoid potential double loads where [load_parent_blob] is called multiple times.
    blob_details: OnceCell<NodeDetails>,
}

impl<'a, B> CryNode<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    pub fn new(
        blobstore: &'a AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
        parent_blob_id: BlobId,
        name: String,
    ) -> Self {
        Self {
            blobstore,
            node_info: NodeInfo::IsNotRootDir {
                parent_blob_id,
                name,
            },
            blob_details: OnceCell::default(),
        }
    }

    pub fn new_rootdir(
        blobstore: &'a AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>>,
        root_blob_id: BlobId,
    ) -> Self {
        Self {
            blobstore,
            node_info: NodeInfo::IsRootDir { root_blob_id },
            blob_details: OnceCell::default(),
        }
    }

    async fn blob_info(&self) -> FsResult<&NodeDetails> {
        self.blob_details
            .get_or_try_init(|| async move {
                match &self.node_info {
                    NodeInfo::IsRootDir { root_blob_id } => Ok(NodeDetails {
                        blob_id: *root_blob_id,
                        blob_type: BlobType::Dir,
                    }),
                    NodeInfo::IsNotRootDir {
                        parent_blob_id,
                        name,
                    } => {
                        let mut parent_blob = self.load_parent_blob(parent_blob_id).await?;

                        let result = (|| async {
                            let entry = parent_blob
                                .entry_by_name(name)
                                .map_err(|_| FsError::CorruptedFilesystem {
                                    message: format!("Entry name isn't utf-8"),
                                })?
                                .ok_or_else(|| FsError::NodeDoesNotExist)?;
                            let blob_id = *entry.blob_id();
                            let blob_type = match entry.entry_type() {
                                EntryType::File => BlobType::File,
                                EntryType::Dir => BlobType::Dir,
                                EntryType::Symlink => BlobType::Symlink,
                            };
                            Ok(NodeDetails { blob_id, blob_type })
                        })()
                        .await;
                        parent_blob.async_drop().await.map_err(|err| {
                            // TODO We might not need with_async_drop_err_map if we change all the AsyncDrop's to Error=FsError.
                            log::error!("Error dropping parent blob: {:?}", err);
                            FsError::UnknownError
                        })?;
                        result
                    }
                }
            })
            .await
    }

    pub async fn load_parent_blob(
        &self,
        parent_blob_id: &BlobId,
    ) -> FsResult<AsyncDropGuard<DirBlob<'a, B>>> {
        // TODO If we're loading the parent blob, make sure that self.blob_details gets set
        //      and that we don't load the parent blob again in the future.
        //      BUT: Currently, self.blob_details() calls load_parent_blob, so there would be
        //      a cyclic dependency
        let parent_blob = self
            .blobstore
            .load(parent_blob_id)
            .await
            .map_err(|err| {
                log::error!("Error loading blob {:?}: {:?}", parent_blob_id, err);
                FsError::UnknownError
            })?
            .ok_or_else(|| {
                log::error!("Parent blob {:?} not found", parent_blob_id);
                FsError::CorruptedFilesystem {
                    message: format!("Didn't find parent blob {:?}", parent_blob_id),
                }
            })?;
        FsBlob::into_dir(parent_blob)
            .await
            .map_err(|_err| FsError::NodeIsNotADirectory)
    }

    pub async fn node_type(&self) -> FsResult<BlobType> {
        Ok(self.blob_info().await?.blob_type)
    }

    pub(super) fn blobstore(&self) -> &'a AsyncDropGuard<AsyncDropArc<FsBlobStore<B>>> {
        self.blobstore
    }

    pub(super) async fn blob_id(&self) -> FsResult<&BlobId> {
        Ok(&self.blob_info().await?.blob_id)
    }

    pub(super) async fn load_blob(&self) -> FsResult<AsyncDropGuard<FsBlob<'a, B>>> {
        let blob_id = self.blob_id().await?;
        self.blobstore
            .load(blob_id)
            .await
            .map_err(|err| {
                log::error!("Error loading blob {:?}: {:?}", blob_id, err);
                FsError::UnknownError
            })?
            .ok_or_else(|| {
                log::error!("Blob {:?} not found", blob_id);
                FsError::CorruptedFilesystem {
                    message: format!("Didn't find blob {:?}", blob_id),
                }
            })
    }

    async fn load_lstat_size(&self) -> FsResult<NumBytes> {
        let mut blob = self.load_blob().await?;
        let result = match blob.lstat_size().await {
            // TODO Return NumBytes from blob.lstat_size() instead of converting it here
            Ok(size) => Ok(NumBytes::from(size)),
            Err(err) => {
                log::error!("Error getting lstat size: {:?}", err);
                Err(FsError::UnknownError)
            }
        };
        blob.async_drop().await.map_err(|err| {
            // TODO We might not need with_async_drop_err_map if we change all the AsyncDrop's to Error=FsError.
            log::error!("Error dropping blob: {:?}", err);
            FsError::UnknownError
        })?;
        result
    }
}

#[async_trait]
impl<'a, B> Node for CryNode<'a, B>
where
    // TODO Do we really need B: 'static ?
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    async fn getattr(&self) -> FsResult<NodeAttrs> {
        match &self.node_info {
            NodeInfo::IsRootDir { .. } => {
                // We're the root dir
                // TODO What should we do here?
                Ok(NodeAttrs {
                    nlink: 1,
                    // TODO Remove those conversions
                    mode: cryfs_rustfs::Mode::default()
                        .add_dir_flag()
                        .add_user_read_flag()
                        .add_user_write_flag()
                        .add_user_exec_flag(),
                    uid: cryfs_rustfs::Uid::from(1000),
                    gid: cryfs_rustfs::Gid::from(1000),
                    num_bytes: cryfs_rustfs::NumBytes::from(DIR_LSTAT_SIZE),
                    // Setting num_blocks to none means it'll be automatically calculated for us
                    num_blocks: None,
                    atime: SystemTime::now(),
                    mtime: SystemTime::now(),
                    ctime: SystemTime::now(),
                })
            }
            NodeInfo::IsNotRootDir {
                parent_blob_id,
                name,
            } => {
                // TODO This loads the parent blob twice, once in load_lstat_size, which calls load_blob to get the blob id, and then again in load_parent_blob. Avoid this.
                // TODO We can load parent_blob and lstat_size concurrently, but that would currently cause a deadlock because lstat_size needs to load parent_blob.
                let lstat_size = self.load_lstat_size().await?;
                let mut parent_blob = self.load_parent_blob(parent_blob_id).await?;
                let result = (|| {
                    let entry = parent_blob
                        .entry_by_name(name)
                        .map_err(|_| FsError::CorruptedFilesystem {
                            message: format!("Entry name isn't utf-8"),
                        })?
                        .ok_or_else(|| FsError::NodeDoesNotExist)?;
                    Ok(dir_entry_to_node_attrs(entry, lstat_size))
                })();
                parent_blob.async_drop().await.map_err(|err| {
                    log::error!("Error dropping parent blob: {:?}", err);
                    FsError::UnknownError
                })?;
                result
            }
        }
    }

    async fn chmod(&self, mode: Mode) -> FsResult<()> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }

    async fn chown(&self, uid: Option<Uid>, gid: Option<Gid>) -> FsResult<()> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }

    async fn utimens(
        &self,
        last_access: Option<SystemTime>,
        last_modification: Option<SystemTime>,
    ) -> FsResult<()> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }
}

fn dir_entry_to_node_attrs(entry: &DirEntry, num_bytes: NumBytes) -> NodeAttrs {
    NodeAttrs {
        nlink: 1,
        // TODO Remove those conversions
        mode: cryfs_rustfs::Mode::from(u32::from(entry.mode())),
        uid: cryfs_rustfs::Uid::from(u32::from(entry.uid())),
        gid: cryfs_rustfs::Gid::from(u32::from(entry.gid())),
        num_bytes,
        // Setting num_blocks to none means it'll be automatically calculated for us
        num_blocks: None,
        atime: entry.last_access_time(),
        mtime: entry.last_modification_time(),
        ctime: entry.last_metadata_change_time(),
    }
}
