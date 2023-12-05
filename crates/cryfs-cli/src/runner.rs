use anyhow::Result;
use async_trait::async_trait;
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

#[async_trait]
impl<'m, 'c> BlockstoreCallback for FilesystemRunner<'m, 'c> {
    // TODO Any way to do this without dyn?
    type Result = Result<()>;

    async fn callback<B: BlockStore + Send + Sync + AsyncDrop + 'static>(
        self,
        blockstore: AsyncDropGuard<LockingBlockStore<B>>,
    ) -> Self::Result {
        // TODO Drop safety, make sure we correctly drop intermediate objects when errors happen

        // TODO No unwrap. Should we instead change blocksize_bytes in the config file struct?
        let blobstore = BlobStoreOnBlocks::new(
            blockstore,
            u32::try_from(self.config.config.config().blocksize_bytes).unwrap(),
        )
        .await?;

        let root_blob_id = BlobId::from_hex(&self.config.config.config().root_blob)?;

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
