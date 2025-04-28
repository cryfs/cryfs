use anyhow::Result;

use cryfs_blockstore::{
    LLBlockStore, ClientId, DynBlockStore, EncryptedBlockStore, IntegrityBlockStore,
    IntegrityBlockStoreInitError, IntegrityConfig, LockingBlockStore, OptimizedBlockStoreWriter,
};
use cryfs_filesystem::config::{
    CryConfig,
    ciphers::{AsyncCipherCallback, UnknownCipherError, lookup_cipher_async},
};
use cryfs_filesystem::localstate::LocalStateDir;
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    crypto::symmetric::{CipherDef, EncryptionKey},
};

use crate::{CliError, CliErrorKind, CliResultExt, CliResultExtFn};

pub trait BlockstoreCallback {
    type Result;

    #[allow(async_fn_in_trait)]
    async fn callback<B: LLBlockStore + AsyncDrop + Send + Sync + 'static>(
        self,
        blockstore: AsyncDropGuard<LockingBlockStore<B>>,
    ) -> Self::Result;
}

/// Set up a blockstore stack (i.e. EncryptedBlockStore, IntegrityBlockStore) using the cipher specified in the config file.
/// Give it the base blockstore (i.e. OnDiskBlockStore) and it will set up the blockstore stack as needed for a cryfs device.
pub async fn setup_blockstore_stack<CB: BlockstoreCallback + Send + Sync>(
    base_blockstore: AsyncDropGuard<impl LLBlockStore + OptimizedBlockStoreWriter + Send + Sync>,
    config: &CryConfig,
    my_client_id: ClientId,
    local_state_dir: &LocalStateDir,
    integrity_config: IntegrityConfig,
    callback: CB,
) -> Result<CB::Result, CliError> {
    let result = lookup_cipher_async(
        &config.cipher,
        CipherCallbackForBlockstoreSetup {
            base_blockstore,
            config,
            my_client_id,
            local_state_dir,
            integrity_config,
            callback,
        },
    )
    .await;

    match result {
        Ok(v) => v,
        Err(err @ UnknownCipherError { .. }) => {
            Err(err).map_cli_error(|_| CliErrorKind::UnspecifiedError)
        }
    }
}

struct CipherCallbackForBlockstoreSetup<
    'c,
    'l,
    B: LLBlockStore + OptimizedBlockStoreWriter + Send + Sync,
    CB: BlockstoreCallback,
> {
    base_blockstore: AsyncDropGuard<B>,
    config: &'c CryConfig,
    my_client_id: ClientId,
    local_state_dir: &'l LocalStateDir,
    integrity_config: IntegrityConfig,
    callback: CB,
}

impl<B: LLBlockStore + OptimizedBlockStoreWriter + Send + Sync, CB: BlockstoreCallback + Send>
    AsyncCipherCallback for CipherCallbackForBlockstoreSetup<'_, '_, B, CB>
{
    type Result = Result<CB::Result, CliError>;

    async fn callback<C: CipherDef + Send + Sync + 'static>(mut self) -> Self::Result {
        let key = match EncryptionKey::from_hex(&self.config.enc_key) {
            Ok(key) => key,
            Err(err) => {
                self.base_blockstore
                    .async_drop()
                    .await
                    .map_cli_error(CliErrorKind::UnspecifiedError)?;
                return Err(err).map_cli_error(CliErrorKind::InvalidFilesystem);
            }
        };

        let cipher = match C::new(key) {
            Ok(cipher) => cipher,
            Err(err) => {
                self.base_blockstore
                    .async_drop()
                    .await
                    .map_cli_error(CliErrorKind::UnspecifiedError)?;
                return Err(err).map_cli_error(|_| CliErrorKind::InvalidFilesystem);
            }
        };
        let mut encrypted_blockstore = EncryptedBlockStore::new(self.base_blockstore, cipher);

        let integrity_file_path = self
            .local_state_dir
            .for_filesystem_id(&self.config.filesystem_id);
        let integrity_file_path = match integrity_file_path {
            Ok(integrity_file_path) => integrity_file_path.join("integritydata"),
            Err(err) => {
                encrypted_blockstore
                    .async_drop()
                    .await
                    .map_cli_error(CliErrorKind::UnspecifiedError)?;
                return Err(err).map_cli_error(CliErrorKind::InaccessibleLocalStateDir);
            }
        };
        let integrity_blockstore = IntegrityBlockStore::new(
            encrypted_blockstore,
            integrity_file_path,
            self.my_client_id,
            self.integrity_config,
        )
        .await
        .map_cli_error(|error| match error {
            IntegrityBlockStoreInitError::IntegrityViolationInPreviousRun { .. } => {
                CliErrorKind::IntegrityViolationOnPreviousRun
            }
            IntegrityBlockStoreInitError::InvalidLocalIntegrityState { .. } => {
                CliErrorKind::InvalidLocalState
            }
        })?;
        let blockstore = LockingBlockStore::new(integrity_blockstore);

        Ok(self.callback.callback(blockstore).await)
    }
}

pub async fn setup_blockstore_stack_dyn(
    base_blockstore: AsyncDropGuard<impl LLBlockStore + OptimizedBlockStoreWriter + Send + Sync>,
    config: &CryConfig,
    my_client_id: ClientId,
    local_state_dir: &LocalStateDir,
    integrity_config: IntegrityConfig,
) -> Result<AsyncDropGuard<LockingBlockStore<DynBlockStore>>, CliError> {
    setup_blockstore_stack(
        base_blockstore,
        config,
        my_client_id,
        local_state_dir,
        integrity_config,
        DynCallback,
    )
    .await?
    .map_cli_error(CliErrorKind::UnspecifiedError)
}

struct DynCallback;
impl BlockstoreCallback for DynCallback {
    type Result = Result<AsyncDropGuard<LockingBlockStore<DynBlockStore>>>;

    async fn callback<B: LLBlockStore + AsyncDrop + Send + Sync + 'static>(
        self,
        blockstore: AsyncDropGuard<LockingBlockStore<B>>,
    ) -> Self::Result {
        let inner = LockingBlockStore::into_inner_block_store(blockstore).await?;
        let inner: Box<dyn LLBlockStore + Send + Sync> =
            Box::new(inner.unsafe_into_inner_dont_drop());
        let inner = AsyncDropGuard::new(DynBlockStore(inner));
        Ok(LockingBlockStore::new(inner))
    }
}
