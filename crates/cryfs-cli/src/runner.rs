use anyhow::{ensure, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

use cryfs_blobstore::{BlobId, BlobStoreOnBlocks};
use cryfs_blockstore::{
    EncryptedBlockStore, IntegrityBlockStore, IntegrityConfig, LockingBlockStore, OnDiskBlockStore,
};
use cryfs_cryfs::config::{ciphers::AsyncCipherCallback, ConfigLoadResult};
use cryfs_cryfs::filesystem::CryDevice;
use cryfs_cryfs::localstate::LocalStateDir;
use cryfs_rustfs::backend::fuser;
use cryfs_utils::crypto::symmetric::{CipherDef, EncryptionKey};

pub struct FilesystemRunner<'m, 'c, 'l> {
    pub basedir: PathBuf,
    pub mountdir: &'m Path,
    pub config: &'c ConfigLoadResult,
    pub local_state_dir: &'l LocalStateDir,
    pub integrity_config: IntegrityConfig,
}

#[async_trait]
impl<'m, 'c, 'l> AsyncCipherCallback for FilesystemRunner<'m, 'c, 'l> {
    // TODO Any way to do this without dyn?
    type Result = Result<()>;

    async fn callback<C: CipherDef + Send + Sync + 'static>(self) -> Self::Result {
        // TODO Drop safety, make sure we correctly drop intermediate objects when errors happen

        let ondisk_blockstore = OnDiskBlockStore::new(self.basedir);
        // TODO Either don't use lookup_cipher_dyn or there is no need for the non-dyn lookup_cipher methods.

        let key = EncryptionKey::from_hex(&self.config.config.config().enc_key)?;
        ensure!(
            key.num_bytes() == C::KEY_SIZE,
            "Invalid key length in config file. Expected {} bytes, got {} bytes.",
            C::KEY_SIZE,
            key.num_bytes(),
        );
        let cipher = C::new(key)?;
        let encrypted_blockstore = EncryptedBlockStore::new(ondisk_blockstore, cipher);
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
        // TODO No unwrap. Should we instead change blocksize_bytes in the config file struct?
        let blobstore = BlobStoreOnBlocks::new(
            LockingBlockStore::new(integrity_blockstore),
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
