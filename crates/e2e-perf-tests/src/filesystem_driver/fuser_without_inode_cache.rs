use cryfs_blobstore::{BlobStoreOnBlocks, TrackingBlobStore};
use cryfs_blockstore::{
    DynBlockStore, HLSharedBlockStore, HLTrackingBlockStore, LockingBlockStore,
};
use cryfs_filesystem::filesystem::CryDevice;
use cryfs_rustfs::{
    AbsolutePath, FsError, InodeNumber, PathComponent,
    low_level_api::AsyncFilesystemLL,
    object_based_api::{FUSE_ROOT_ID, ObjectBasedFsAdapterLL},
};
use cryfs_utils::async_drop::AsyncDropArc;

use crate::fixture::request_info;

// TODO Build a version of the low level fixture that doesn't cache inodes and runs lookup every time for the whole path (to emulate fuse-mt and make sure our operations numbers aren't higher than what fuse-mt has).
// TODO Add tests for a "lookup" operation

async fn load_parent_inode_from_path<R>(
    fs: &ObjectBasedFsAdapterLL<
        CryDevice<
            AsyncDropArc<
                TrackingBlobStore<
                    BlobStoreOnBlocks<
                        HLSharedBlockStore<HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>>,
                    >,
                >,
            >,
        >,
    >,
    path: &AbsolutePath,
    callback: impl AsyncFnOnce(InodeNumber, &PathComponent) -> Result<R, FsError>,
) -> Result<R, FsError> {
    let (parent_path, name) = path.split_last().unwrap();
    let mut inos = vec![FUSE_ROOT_ID];
    for component in parent_path.iter() {
        let parent_ino = *inos.last().unwrap();
        let child_ino = AsyncFilesystemLL::lookup(fs, &request_info(), parent_ino, component)
            .await?
            .ino
            .handle;
        inos.push(child_ino);
    }
    let result = callback(*inos.last().unwrap(), name).await?;
    for ino in inos.iter().skip(1).rev() {
        AsyncFilesystemLL::forget(fs, &request_info(), *ino, 1).await?;
    }
    Ok(result)
}
