use anyhow::Result;
use async_trait::async_trait;
use futures::stream::TryStreamExt;

use cryfs_blobstore::{BlobId, BlobStoreOnBlocks};
use cryfs_blockstore::{BlockId, BlockStore, LockingBlockStore};
use cryfs_cli_utils::BlockstoreCallback;
use cryfs_cryfs::config::ConfigLoadResult;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

pub struct RecoverRunner<'c> {
    pub config: &'c ConfigLoadResult,
}

#[async_trait]
impl<'l> BlockstoreCallback for RecoverRunner<'l> {
    type Result = Result<()>;

    async fn callback<B: BlockStore + AsyncDrop + Send + Sync + 'static>(
        self,
        mut blockstore: AsyncDropGuard<LockingBlockStore<B>>,
    ) -> Self::Result {
        // TODO We should be able to list all blocks in parallel with listing all known blocks

        log::info!("Listing all blocks...");
        let unaccounted_blocks = match get_all_block_ids(&blockstore).await {
            Ok(unaccounted_blocks) => unaccounted_blocks,
            Err(e) => {
                blockstore.async_drop().await?;
                return Err(e);
            }
        };
        log::info!(
            "Listing all blocks...done. Found {} blocks",
            unaccounted_blocks.len()
        );

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

        blobstore.async_drop().await?;
        Ok(())
    }
}

async fn get_all_block_ids(
    blockstore: &AsyncDropGuard<LockingBlockStore<impl BlockStore + Send + Sync>>,
) -> Result<Vec<BlockId>> {
    blockstore
        .all_blocks()
        .await?
        .try_collect::<Vec<BlockId>>()
        .await
}
