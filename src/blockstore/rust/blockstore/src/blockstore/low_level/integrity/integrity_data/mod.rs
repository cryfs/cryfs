use anyhow::{Context, Result};
use async_trait::async_trait;
use lockable::HashMapOwnedGuard;
use std::path::PathBuf;

use crate::blockstore::BlockId;
use crate::utils::async_drop::{AsyncDrop, AsyncDropGuard};

mod integrity_violation_error;
mod known_block_versions;
mod serialization;

pub use integrity_violation_error::IntegrityViolationError;
pub use known_block_versions::{
    BlockInfo, BlockVersion, BlockVersionTransaction, ClientId, KnownBlockVersions, MaybeClientId,
};

// TODO Rethink serialization. It's weird to have a KnownBlockVersions object wrapped in an IntegrityData object just because parts are serialized to different files. Merge them.

/// IntegrityData is the basis of the CryFS integrity promise.
/// For each block, it remembers a [BlockInfo] structure that ensures that
/// adversaries can't roll back, delete or re-introduce a block.
#[derive(Debug)]
pub struct IntegrityData {
    state_file_path: PathBuf,
    my_client_id: ClientId,

    // Always Some except for during destruction
    known_block_versions: Option<KnownBlockVersions>,
}

// TODO We should probably lock the file while it's open so that another CryFS process doesn't open it too

impl IntegrityData {
    pub fn new(state_file_path: PathBuf, my_client_id: ClientId) -> Result<AsyncDropGuard<Self>> {
        let known_block_versions = Some(
            KnownBlockVersions::load_or_default(&state_file_path)
                .context("Tried to deserialize the state file")?,
        );
        Ok(AsyncDropGuard::new(Self {
            state_file_path,
            my_client_id,
            known_block_versions,
        }))
    }

    // TODO Do we need this?
    pub fn my_client_id(&self) -> ClientId {
        self.my_client_id
    }

    pub async fn lock_block_info(
        &self,
        block_id: BlockId,
    ) -> HashMapOwnedGuard<BlockId, BlockInfo> {
        self._known_block_versions().lock_block_info(block_id).await
    }

    /// Checks if any previous runs recognized any integrity violations and marked it in the local state.
    /// Integrity violations are marked in the local state to make sure the user notices. We currently
    /// don't have any better way to report it to the user than just to permanently prevent access to
    /// the file system. Note that "permanently" here means "until they delete the local state file",
    /// so there's a way to reset and allow them to access the file system again, but they definitely
    /// won't miss that something weird happened.
    pub fn integrity_violation_in_previous_run(&self) -> bool {
        self._known_block_versions()
            .integrity_violation_in_previous_run()
    }

    /// This is intended to be called when an integrity violation was recognized and it marks the local
    /// state so that future attempts to open the file system will fail. See [IntegrityData::integrity_violation_in_previous_run].
    pub fn set_integrity_violation_in_previous_run(&self) {
        self._known_block_versions()
            .set_integrity_violation_in_previous_run();
    }

    /// This function returns all blocks that we expect to exist, i.e. we have
    /// seen them before and we haven't deleted it. Note that, similar to
    /// [IntegrityData::should_block_exist], this can return blocks that
    /// have been correctly deleted by other authorized clients.
    pub fn existing_blocks(&self) -> Vec<BlockId> {
        self._known_block_versions().existing_blocks()
    }

    fn _known_block_versions(&self) -> &KnownBlockVersions {
        self.known_block_versions
            .as_ref()
            .expect("Object is currently being destructed")
    }

    fn _known_block_versions_mut(&mut self) -> &mut KnownBlockVersions {
        self.known_block_versions
            .as_mut()
            .expect("Object is currently being destructed")
    }
}

#[async_trait]
impl AsyncDrop for IntegrityData {
    type Error = anyhow::Error;
    async fn async_drop_impl(&mut self) -> Result<()> {
        self.known_block_versions
            .take()
            .expect("Was already destructed")
            .save(&self.state_file_path)
            .await
    }
}

#[cfg(test)]
pub mod testutils {
    use super::*;
    use std::num::NonZeroU32;

    pub fn clientid(id: u32) -> ClientId {
        ClientId {
            id: NonZeroU32::new(id).unwrap(),
        }
    }

    pub fn version(version: u64) -> BlockVersion {
        BlockVersion { version }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    use super::testutils::{clientid, version};
    use crate::blockstore::tests::blockid;
    use crate::utils::async_drop::SyncDrop;

    struct Fixture {
        state_file_dir: TempDir,
    }

    impl Fixture {
        pub fn new() -> Self {
            Self {
                state_file_dir: TempDir::new("OnDiskBlockStoreTest").unwrap(),
            }
        }

        pub fn make_obj(&self, my_client_id: ClientId) -> SyncDrop<IntegrityData> {
            SyncDrop::new(
                IntegrityData::new(
                    self.state_file_dir
                        .path()
                        .join("integrity_file")
                        .to_path_buf(),
                    my_client_id,
                )
                .unwrap(),
            )
        }
    }

    async fn set_version(
        obj: &mut SyncDrop<IntegrityData>,
        block_id: BlockId,
        client_id: ClientId,
        version: BlockVersion,
    ) -> Result<()> {
        obj.lock_block_info(block_id)
            .await
            .value_or_insert_with(|| BlockInfo::new_unknown(MaybeClientId::ClientId(client_id)))
            .check_and_update_version(client_id, block_id, version)
    }

    async fn get_last_update_client_id(
        obj: &SyncDrop<IntegrityData>,
        block_id: BlockId,
    ) -> MaybeClientId {
        obj.lock_block_info(block_id)
            .await
            .value()
            .expect("Entry not found")
            .last_update_client_id()
    }

    async fn get_version(obj: &SyncDrop<IntegrityData>, block_id: BlockId) -> Option<BlockVersion> {
        obj.lock_block_info(block_id)
            .await
            .value()
            .expect("Entry not found")
            .current_version()
    }

    async fn get_version_for_client(
        obj: &SyncDrop<IntegrityData>,
        block_id: BlockId,
        client_id: ClientId,
    ) -> Option<BlockVersion> {
        obj.lock_block_info(block_id)
            .await
            .value()
            .expect("Entry not found")
            .current_version_for_client(&client_id)
    }

    #[tokio::test]
    async fn test_set_and_get() {
        let fixture = Fixture::new();
        let mut obj = fixture.make_obj(clientid(1));

        set_version(&mut obj, blockid(0), clientid(1), version(5))
            .await
            .unwrap();
        assert_eq!(
            MaybeClientId::ClientId(clientid(1)),
            get_last_update_client_id(&obj, blockid(0)).await
        );
        assert_eq!(Some(version(5)), get_version(&obj, blockid(0)).await);
    }

    #[tokio::test]
    async fn test_version_is_per_client() {
        let fixture = Fixture::new();
        let mut obj = fixture.make_obj(clientid(1));

        set_version(&mut obj, blockid(0), clientid(1), version(5))
            .await
            .unwrap();
        set_version(&mut obj, blockid(0), clientid(2), version(3))
            .await
            .unwrap();
        assert_eq!(
            Some(version(5)),
            get_version_for_client(&obj, blockid(0), clientid(1)).await
        );
        assert_eq!(
            Some(version(3)),
            get_version_for_client(&obj, blockid(0), clientid(2)).await
        );
    }

    #[tokio::test]
    async fn test_version_is_per_block() {
        let fixture = Fixture::new();
        let mut obj = fixture.make_obj(clientid(1));

        set_version(&mut obj, blockid(0), clientid(1), version(5))
            .await
            .unwrap();
        set_version(&mut obj, blockid(1), clientid(1), version(3))
            .await
            .unwrap();
        assert_eq!(
            Some(version(5)),
            get_version_for_client(&obj, blockid(0), clientid(1)).await
        );
        assert_eq!(
            Some(version(3)),
            get_version_for_client(&obj, blockid(1), clientid(1)).await
        );
    }

    #[tokio::test]
    async fn test_allows_increasing() {
        let fixture = Fixture::new();
        let mut obj = fixture.make_obj(clientid(1));

        set_version(&mut obj, blockid(0), clientid(1), version(5))
            .await
            .unwrap();
        set_version(&mut obj, blockid(0), clientid(1), version(6))
            .await
            .unwrap();
        assert_eq!(Some(version(6)), get_version(&obj, blockid(0)).await);
    }

    #[tokio::test]
    async fn test_doesnt_allow_decreasing() {
        let fixture = Fixture::new();
        let mut obj = fixture.make_obj(clientid(1));

        set_version(&mut obj, blockid(0), clientid(1), version(5))
            .await
            .unwrap();
        set_version(&mut obj, blockid(0), clientid(1), version(4))
            .await
            .unwrap_err();
        assert_eq!(Some(version(5)), get_version(&obj, blockid(0)).await);
    }
}
