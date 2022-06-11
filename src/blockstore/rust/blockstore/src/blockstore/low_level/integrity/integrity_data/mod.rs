use anyhow::{ensure, Context, Result};
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
    BlockInfo, BlockVersion, BlockVersionTransaction, ClientId, KnownBlockVersions,
    CLIENT_ID_FOR_DELETED_BLOCK,
};

// TODO Rething serialization. It's weird to have a KnownBlockVersions object wrapped in an IntegrityData object just because parts are serialized to different files. Merge them.

/// IntegrityData is the basis of the CryFS integrity promise.
/// It remembers persistent state, locally on the CryFS client device:
///  - `known_block_versions: HashMap<(ClientId, BlockId), BlockVersion>`
///    The newest version number of the block that we've already seen and was created by the given client.
///  - `last_update_client_id: HashMap<BlockId, ClientId>`
///    The client_id we consider to have created the current version of the block.
///
/// The invariant we uphold is that within a `(client_id, block_id)` pair, version numbers are always increasing.
///
/// # Important Functions
/// [IntegrityData::check_and_update_version] is called whenever a block is read and ensures that it hasn't been rolled back.
/// [IntegrityData::increment_version] is called whenever we modify a block ourselves and updates the internal state correspondingly.
/// [IntegrityData::mark_block_as_deleted] is called whenever we delete a block to make sure it doesn't get reintroduced.
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
        ensure!(
            my_client_id != CLIENT_ID_FOR_DELETED_BLOCK,
            "Tried to instantiate a IntegrityData instace with an invalid client id"
        );
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

    // TODO Do we need this?
    // pub fn block_version(&self, client_id: ClientId, block_id: BlockId) -> Option<BlockVersion> {
    // self.file_data
    //     .block_data
    //     .get(&block_id)
    //     .map(|block_data| {
    //         block_data
    //         .lock()
    //         .unwrap()
    //         .known_block_versions
    //         .get(&client_id)
    //         .copied()
    //     }).flatten()
    // }

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

    /// This function is intended to be called whenever we read a block.
    /// It checks the read block version against our internal state to make sure that blocks aren't rolled back,
    /// and then updates our internal state to make sure that the change this read operation observed can't be rolled back in the future.
    ///
    /// There's three possibilities
    /// 1. We see exactly the same `(client_id, version_number)` as stored in our local state `(last_update_client_id, known_block_versions)`.
    ///    This means nothing changed since we've read the block last, there was no rollback.
    /// 2. We see the same `client_id`, but a different `block_version`. In this case, we enforce the `block_version` to increase.
    ///    If it was decreasing, then that's a rollback to a version we might have previously seen.
    /// 3. We see a different `client_id`. In this case, we enforce the `block_version` to be at least one larger than the one from `known_block_versions`.
    ///    If it was decreasing, then that's a rollback to a version we might have previously seen, and if it is the same as `known_block_versions`,
    ///    then it's a rollback to a version we have previously seen.
    // pub fn check_and_update_version(
    //     &mut self,
    //     client_id: ClientId,
    //     block_id: BlockId,
    //     version: BlockVersion,
    // ) -> Result<()> {
    // ensure!(
    //     client_id != CLIENT_ID_FOR_DELETED_BLOCK,
    //     "Called IntegrityData::check_and_update_version with an invalid client id"
    // );
    // ensure!(version.version > 0, "Version has to be >0");
    // let known_block_versions_entry = self
    //     .file_data
    //     .known_block_versions
    //     .entry((client_id, block_id));
    // let last_update_client_id_entry = self.file_data.last_update_client_id.entry(block_id);
    // match (known_block_versions_entry, last_update_client_id_entry) {
    //     (
    //         Entry::Vacant(known_block_versions_entry),
    //         Entry::Vacant(last_update_client_id_entry),
    //     ) => {
    //         known_block_versions_entry.insert(Arc::new(Mutex::new(version)));
    //         last_update_client_id_entry.insert(Arc::new(Mutex::new(client_id)));
    //     }
    //     (Entry::Vacant(_), Entry::Occupied(_)) => {
    //         bail!("last_update_client_ids had the block with id {} but known_block_versions didn't have it. This shouldn't happen. Likely our local state is corrupted.", block_id.to_hex());
    //     }
    //     (Entry::Occupied(_), Entry::Vacant(_)) => {
    //         bail!("known_block_versions had the block with id {} but last_update_client_ids didn't have it. This shouldn't happen. Likely our local state is corrupted.", block_id.to_hex());
    //     }
    //     (
    //         Entry::Occupied(mut known_block_versions_entry),
    //         Entry::Occupied(mut last_update_client_id_entry),
    //     ) => {
    //         ensure!(
    //             //In all of the cases 1, 2, 3: the version number must not decrease
    //             (*known_block_versions_entry.get().lock().unwrap() <= version) &&
    //             // In case 3 (i.e. we see a change in client id), the version number must increase
    //             (*last_update_client_id_entry.get() == client_id || *known_block_versions_entry.get() < version),
    //             IntegrityViolationError::RollBack {
    //                 block: block_id,
    //                 from_client: *last_update_client_id_entry.get(),
    //                 to_client: client_id,
    //                 from_version: *known_block_versions_entry.get(),
    //                 to_version: version,
    //             }
    //         );
    //         known_block_versions_entry.insert(version);
    //         last_update_client_id_entry.insert(client_id);
    //     }
    // }

    // Ok(())
    // }

    /// This function is intended to be called whenever we modify a block ourselves.
    /// It updates our internal state so the modification can't be rolled back in the future.
    // pub fn increment_version(&mut self, block_id: BlockId) -> BlockVersion {
    // self.file_data
    //     .last_update_client_id
    //     .insert(block_id, self.my_client_id);
    // match self
    //     .file_data
    //     .known_block_versions
    //     .entry((self.my_client_id, block_id))
    // {
    //     Entry::Vacant(entry) => {
    //         let new_version = BlockVersion { version: 1 };
    //         entry.insert(new_version);
    //         println!("{:?} new: {:?}", block_id, new_version);
    //         new_version
    //     }
    //     Entry::Occupied(mut entry) => {
    //         entry.get_mut().increment();
    //         println!("{:?} increment: {:?}", block_id, *entry.get());
    //         *entry.get()
    //     }
    // }
    // }

    /// This function is intended to be called whenever we delete a block. It will set
    /// the `last_update_client_id` for this block to an invalid `client_id`.
    /// Following the explanation in [IntegrityData::check_and_update_version], this means
    /// that all previously seen versions of this block won't be accepted anymore, not even the
    /// most recent one. The only way to reintroduce this block is if some client creates a
    /// version of it with a new, higher version number.
    // pub fn mark_block_as_deleted(&mut self, block_id: BlockId) {
    // self.file_data
    //     .last_update_client_id
    //     .insert(block_id, CLIENT_ID_FOR_DELETED_BLOCK);
    // }

    /// This function returns true iff we expect the block with the given id to exist,
    /// i.e. we have seen it before and we haven't deleted it. Note that we don't know
    /// if a different client might have deleted it, so this could return true for
    /// blocks that were correctly deleted by other authorized clients.
    // pub fn should_block_exist(&self, block_id: &BlockId) -> bool {
    // match self.file_data.last_update_client_id.get(block_id) {
    //     None => {
    //         // We've never seen (i.e. loaded) this block. So we can't say it has to exist.
    //         false
    //     }
    //     Some(&CLIENT_ID_FOR_DELETED_BLOCK) => {
    //         // We've deleted this block. We can't say it has to exist.
    //         false
    //     }
    //     Some(_) => {
    //         // We've seen this block before and we haven't deleted it
    //         true
    //     }
    // }
    // }

    /// This function returns all blocks that we expect to exist, i.e. we have
    /// seen them before and we haven't deleted it. Note that, similar to
    /// [IntegrityData::should_block_exist], this can return blocks that
    /// have been correctly deleted by other authorized clients.
    /// // TODO Is impl Stream good enough here instead of the Pin<Box<_>>?
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
    async fn async_drop_impl(mut self) -> Result<()> {
        self.known_block_versions
            .take()
            .expect("Was already destructed")
            .save(&self.state_file_path)
            .await
    }
}
