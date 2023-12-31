use anyhow::Result;
use std::path::Path;

use cryfs_blobstore::{BlobId, BlobStoreOnBlocks};
use cryfs_blockstore::{BlockStore, LockingBlockStore};
use cryfs_cli_utils::BlockstoreCallback;
use cryfs_cryfs::{config::ConfigLoadResult, filesystem::CryDevice};
use cryfs_rustfs::backend::fuser;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

pub struct FilesystemRunner<'m, 'c> {
    pub mountdir: &'m Path,
    pub config: &'c ConfigLoadResult,
}

impl<'m, 'c> BlockstoreCallback for FilesystemRunner<'m, 'c> {
    type Result = Result<()>;

    async fn callback<B: BlockStore + Send + Sync + AsyncDrop + 'static>(
        self,
        blockstore: AsyncDropGuard<LockingBlockStore<B>>,
    ) -> Self::Result {
        // TODO No unwrap. Should we instead change blocksize_bytes in the config file struct?
        let mut blobstore = BlobStoreOnBlocks::new(
            blockstore,
            u32::try_from(self.config.config.config().blocksize_bytes).unwrap(),
        )
        .await?;

        let root_blob_id = BlobId::from_hex(&self.config.config.config().root_blob);
        let root_blob_id = match root_blob_id {
            Ok(root_blob_id) => root_blob_id,
            Err(e) => {
                blobstore.async_drop().await?;
                return Err(e);
            }
        };

        let device = if self.config.first_time_access {
            CryDevice::create_new_filesystem(blobstore, root_blob_id).await?
        } else {
            CryDevice::load_filesystem(blobstore, root_blob_id)
        };

        let fs = |_uid, _gid| device;
        fuser::mount(fs, self.mountdir, tokio::runtime::Handle::current())?;

        Ok(())
    }
}
