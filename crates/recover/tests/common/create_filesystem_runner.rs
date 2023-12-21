use cryfs_blobstore::{BlobId, BlobStoreOnBlocks};
use cryfs_blockstore::{BlockStore, LockingBlockStore};
use cryfs_cli_utils::BlockstoreCallback;
use cryfs_cryfs::{config::ConfigLoadResult, filesystem::fsblobstore::FsBlobStore};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};
use std::future::Future;

pub struct CreateFilesystemRunner<'c /*, SetupFn, SetupFnRet*/>
// where
//     SetupFn: FnOnce(&FsBlobStore<B>) -> SetupFnRet,
//     SetupFnRet: Future<Output = ()>,
{
    pub config: &'c ConfigLoadResult,
    // pub setup_fn: SetupFn,
}

impl<'c> BlockstoreCallback for CreateFilesystemRunner<'c> {
    type Result = ();

    async fn callback<B: BlockStore + AsyncDrop + Send + Sync + 'static>(
        self,
        mut blockstore: AsyncDropGuard<LockingBlockStore<B>>,
    ) {
        let mut blobstore = FsBlobStore::new(
            BlobStoreOnBlocks::new(
                blockstore,
                u32::try_from(self.config.config.config().blocksize_bytes).unwrap(),
            )
            .await
            .unwrap(),
        );

        let root_blob_id = BlobId::from_hex(&self.config.config.config().root_blob).unwrap();

        blobstore.create_root_dir_blob(&root_blob_id).await.unwrap();

        blobstore.async_drop().await.unwrap();
    }
}
