use cryfs_blockstore::{LLBlockStore, OptimizedBlockStoreWriter};
use cryfs_utils::async_drop::AsyncDrop;

use crate::filesystem_driver::FilesystemDriver;

pub async fn maybe_close<const CLOSE_AFTER: bool, FS: FilesystemDriver>(
    fixture: &mut crate::filesystem_fixture::FilesystemFixture<
        impl LLBlockStore + OptimizedBlockStoreWriter + AsyncDrop + Send + Sync,
        FS,
    >,
    node: FS::NodeHandle,
    file_handle: FS::FileHandle,
) {
    if CLOSE_AFTER {
        fixture.filesystem.release(node, file_handle).await.unwrap();
    }
}
