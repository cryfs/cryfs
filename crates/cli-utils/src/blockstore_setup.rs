use anyhow::{bail, Result};

use cryfs_blockstore::{
    BlockStore, DynBlockStore, EncryptedBlockStore, IntegrityBlockStore, IntegrityConfig,
    LockingBlockStore, OptimizedBlockStoreWriter,
};
use cryfs_cryfs::config::{
    ciphers::{lookup_cipher_async, AsyncCipherCallback},
    ConfigLoadResult,
};
use cryfs_cryfs::localstate::LocalStateDir;
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    crypto::symmetric::{CipherDef, EncryptionKey, InvalidKeySizeError},
};

pub trait BlockstoreCallback {
    type Result;

    #[allow(async_fn_in_trait)]
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

impl<B: BlockStore + OptimizedBlockStoreWriter + Send + Sync, CB: BlockstoreCallback + Send>
    AsyncCipherCallback for CipherCallbackForBlockstoreSetup<'_, '_, B, CB>
{
    type Result = Result<CB::Result>;

    async fn callback<C: CipherDef + Send + Sync + 'static>(mut self) -> Self::Result {
        let key = match EncryptionKey::from_hex(&self.config.config.config().enc_key) {
            Ok(key) => key,
            Err(err) => {
                self.base_blockstore.async_drop().await?;
                return Err(err);
            }
        };

        let cipher = match C::new(key) {
            Ok(cipher) => cipher,
            Err(InvalidKeySizeError{expected, got}) => {
                self.base_blockstore.async_drop().await?;
                bail!(
                    "Invalid key length in config file. Expected {expected} bytes, got {got} bytes.",
                );
            }
        };
        let mut encrypted_blockstore = EncryptedBlockStore::new(self.base_blockstore, cipher);

        let integrity_file_path = self
            .local_state_dir
            .for_filesystem_id(&self.config.config.config().filesystem_id);
        let integrity_file_path = match integrity_file_path {
            Ok(integrity_file_path) => integrity_file_path.join("integritydata"),
            Err(err) => {
                encrypted_blockstore.async_drop().await?;
                return Err(err);
            }
        };
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

pub async fn setup_blockstore_stack_dyn(
    base_blockstore: AsyncDropGuard<impl BlockStore + OptimizedBlockStoreWriter + Send + Sync>,
    config: &ConfigLoadResult,
    local_state_dir: &LocalStateDir,
    integrity_config: IntegrityConfig,
) -> Result<AsyncDropGuard<LockingBlockStore<DynBlockStore>>> {
    setup_blockstore_stack(
        base_blockstore,
        config,
        local_state_dir,
        integrity_config,
        DynCallback,
    )
    .await?
}

struct DynCallback;
impl BlockstoreCallback for DynCallback {
    type Result = Result<AsyncDropGuard<LockingBlockStore<DynBlockStore>>>;

    async fn callback<B: BlockStore + AsyncDrop + Send + Sync + 'static>(
        self,
        blockstore: AsyncDropGuard<LockingBlockStore<B>>,
    ) -> Self::Result {
        let inner = LockingBlockStore::into_inner_block_store(blockstore).await?;
        let inner: Box<dyn BlockStore + Send + Sync> =
            Box::new(inner.unsafe_into_inner_dont_drop());
        let inner = AsyncDropGuard::new(DynBlockStore(inner));
        Ok(LockingBlockStore::new(inner))
    }
}
