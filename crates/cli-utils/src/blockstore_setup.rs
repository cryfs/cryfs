use anyhow::{ensure, Result};
use async_trait::async_trait;

use cryfs_blockstore::{
    BlockStore, EncryptedBlockStore, IntegrityBlockStore, IntegrityConfig, LockingBlockStore,
    OptimizedBlockStoreWriter,
};
use cryfs_cryfs::config::{
    ciphers::{lookup_cipher_async, AsyncCipherCallback},
    ConfigLoadResult,
};
use cryfs_cryfs::localstate::LocalStateDir;
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    crypto::symmetric::{CipherDef, EncryptionKey},
};

#[async_trait]
pub trait BlockstoreCallback {
    type Result;

    async fn callback<B: BlockStore + AsyncDrop + Send + Sync + 'static>(
        self,
        blockstore: AsyncDropGuard<LockingBlockStore<B>>,
    ) -> Self::Result;
}

/// Set up a blockstore stack (i.e. EncryptedBlockStore, IntegrityBlockStore) using the cipher specified in the config file.
/// Give it the base blockstore (i.e. OnDiskBlockStore) and it will set up the blockstore stack as needed for a cryfs device.
pub async fn setup_blockstore_stack<CB: BlockstoreCallback + Send + Sync>(
    base_blockstore: AsyncDropGuard<impl BlockStore + OptimizedBlockStoreWriter + Send + Sync>,
    config: &ConfigLoadResult,
    local_state_dir: &LocalStateDir,
    integrity_config: IntegrityConfig,
    callback: CB,
) -> Result<CB::Result> {
    lookup_cipher_async(
        &config.config.config().cipher,
        CipherCallbackForBlockstoreSetup {
            base_blockstore,
            config,
            local_state_dir,
            integrity_config,
            callback,
        },
    )
    .await?
}

struct CipherCallbackForBlockstoreSetup<
    'c,
    'l,
    B: BlockStore + OptimizedBlockStoreWriter + Send + Sync,
    CB: BlockstoreCallback,
> {
    base_blockstore: AsyncDropGuard<B>,
    config: &'c ConfigLoadResult,
    local_state_dir: &'l LocalStateDir,
    integrity_config: IntegrityConfig,
    callback: CB,
}

#[async_trait]
impl<B: BlockStore + OptimizedBlockStoreWriter + Send + Sync, CB: BlockstoreCallback + Send>
    AsyncCipherCallback for CipherCallbackForBlockstoreSetup<'_, '_, B, CB>
{
    type Result = Result<CB::Result>;

    async fn callback<C: CipherDef + Send + Sync + 'static>(self) -> Self::Result {
        // TODO Drop safety, make sure we correctly drop intermediate objects when errors happen

        let key = EncryptionKey::from_hex(&self.config.config.config().enc_key)?;
        ensure!(
            key.num_bytes() == C::KEY_SIZE,
            "Invalid key length in config file. Expected {} bytes, got {} bytes.",
            C::KEY_SIZE,
            key.num_bytes(),
        );
        let cipher = C::new(key)?;
        let encrypted_blockstore = EncryptedBlockStore::new(self.base_blockstore, cipher);
        let integrity_file_path = self
            .local_state_dir
            .for_filesystem_id(&self.config.config.config().filesystem_id)?
            .join("integritydata");
        let integrity_blockstore = IntegrityBlockStore::new(
            encrypted_blockstore,
            integrity_file_path,
            self.config.my_client_id,
            self.integrity_config,
        )
        .await?;
        let blockstore = LockingBlockStore::new(integrity_blockstore);

        Ok(self.callback.callback(blockstore).await)
    }
}
