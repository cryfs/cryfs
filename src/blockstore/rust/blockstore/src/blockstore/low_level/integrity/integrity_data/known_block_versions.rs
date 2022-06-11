use anyhow::{ensure, Result};
use binread::BinRead;
use binwrite::BinWrite;
use lockable::{HashMapOwnedGuard, LockableHashMap};
use std::collections::hash_map::{Entry, HashMap};
use std::hash::Hash;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::blockstore::BlockId;
use crate::utils::binary::{BinaryReadExt, BinaryWriteExt};

use super::integrity_violation_error::IntegrityViolationError;
use super::serialization::KnownBlockVersionsSerialized;

pub const CLIENT_ID_FOR_DELETED_BLOCK: ClientId = ClientId { id: 0 };

#[derive(PartialEq, Eq, Debug, Hash, BinRead, BinWrite, Clone, Copy)]
pub struct ClientId {
    // TODO Tuple struct would be better but https://github.com/jam1garner/binwrite/issues/3
    pub id: u32,
}

impl binary_layout::LayoutAs<u32> for ClientId {
    fn read(id: u32) -> ClientId {
        ClientId { id }
    }

    fn write(id: ClientId) -> u32 {
        id.id
    }
}

#[derive(PartialEq, Eq, Debug, Hash, PartialOrd, BinRead, BinWrite, Clone, Copy)]
pub struct BlockVersion {
    // TODO Tuple struct would be better but https://github.com/jam1garner/binwrite/issues/3
    pub version: u64,
}

impl BlockVersion {
    pub fn increment(&mut self) {
        self.version += 1;
    }
}

impl binary_layout::LayoutAs<u64> for BlockVersion {
    fn read(version: u64) -> BlockVersion {
        BlockVersion { version }
    }

    fn write(version: BlockVersion) -> u64 {
        version.version
    }
}

#[derive(Debug)]
pub struct BlockInfo {
    pub last_update_client_id: ClientId,

    // Invariant: If known_block_versions[last_update_client_id] is present,
    //            then we have seen that block before and know its expected
    //            version number. If known_block_versions[last_update_client_id]
    //            is absent, then we have not seen this block yet.
    //            Its BlockInfo may still have been created, most likely because we're
    //            just seeing the block for the first time and are about to add
    //            information about it. But it can also happen (and even be persisted)
    //            if we delete a block that we haven't seen before.
    //            Such a block would have last_update_client_id = CLIENT_ID_FOR_DELETED_BLOCK
    //            and no entries in known_block_versions.
    pub known_block_versions: HashMap<ClientId, BlockVersion>,
}

impl BlockInfo {
    pub fn new_unknown(last_update_client_id: ClientId) -> Self {
        BlockInfo {
            last_update_client_id,
            known_block_versions: HashMap::new(),
        }
    }

    pub fn mark_block_as_deleted(&mut self) {
        self.last_update_client_id = CLIENT_ID_FOR_DELETED_BLOCK;
    }

    pub fn block_is_deleted(&self) -> bool {
        self.last_update_client_id == CLIENT_ID_FOR_DELETED_BLOCK
    }

    pub fn start_increment_version_transaction(
        &mut self,
        new_client_id: ClientId,
    ) -> BlockVersionTransaction<'_> {
        let mut new_version = self
            .known_block_versions
            .get(&new_client_id)
            .copied()
            .unwrap_or(BlockVersion { version: 0 });
        new_version.increment();
        BlockVersionTransaction(Some(BlockVersionTransactionData {
            block_info: self,
            new_client_id,
            new_version,
        }))
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
    pub fn check_and_update_version(
        &mut self,
        client_id: ClientId,
        block_id: BlockId,
        version: BlockVersion,
    ) -> Result<()> {
        ensure!(
            client_id != CLIENT_ID_FOR_DELETED_BLOCK,
            "Called BlockInfo::check_and_update_version with an invalid client id"
        );
        ensure!(version.version > 0, "Version has to be >0");
        let known_block_versions_entry = self.known_block_versions.entry(client_id);
        match known_block_versions_entry {
            Entry::Vacant(known_block_versions_entry) => {
                known_block_versions_entry.insert(version);
                self.last_update_client_id = client_id;
            }
            Entry::Occupied(mut known_block_versions_entry) => {
                ensure!(
                    //In all of the cases 1, 2, 3: the version number must not decrease
                    (*known_block_versions_entry.get() <= version) &&
                    // In case 3 (i.e. we see a change in client id), the version number must increase
                    (self.last_update_client_id == client_id || *known_block_versions_entry.get() < version),
                    IntegrityViolationError::RollBack {
                        block: block_id,
                        from_client: self.last_update_client_id,
                        to_client: client_id,
                        from_version: *known_block_versions_entry.get(),
                        to_version: version,
                    }
                );
                known_block_versions_entry.insert(version);
                self.last_update_client_id = client_id;
            }
        }

        Ok(())
    }

    /// This function returns true iff we expect the block with the given id to exist,
    /// i.e. we have seen it before and we haven't deleted it. Note that we don't know
    /// if a different client might have deleted it, so this could return true for
    /// blocks that were correctly deleted by other authorized clients.
    pub fn block_is_expected_to_exist(&self) -> bool {
        let block_was_not_deleted = || self.last_update_client_id != CLIENT_ID_FOR_DELETED_BLOCK;
        // See invariant in BlockInfo
        let block_was_seen_previously = || {
            self.known_block_versions
                .contains_key(&self.last_update_client_id)
        };
        block_was_not_deleted() && block_was_seen_previously()
    }
}

struct BlockVersionTransactionData<'a> {
    block_info: &'a mut BlockInfo,
    new_client_id: ClientId,
    new_version: BlockVersion,
}

pub struct BlockVersionTransaction<'a>(Option<BlockVersionTransactionData<'a>>);

impl<'a> BlockVersionTransaction<'a> {
    pub fn new_version(&self) -> BlockVersion {
        self.0
            .as_ref()
            .expect(
                "Can't happen since any action that sets this to None also drops the whole object",
            )
            .new_version
    }

    pub fn commit(mut self) {
        let data = self.0.take().expect(
            "Can't happen since any action that sets this to None also drops the whole object",
        );
        data.block_info.last_update_client_id = data.new_client_id;
        data.block_info
            .known_block_versions
            .insert(data.new_client_id, data.new_version);
    }

    pub fn cancel(mut self) {
        self.0.take().expect(
            "Can't happen since any action that sets this to None also drops the whole object",
        );
    }
}

impl<'a> Drop for BlockVersionTransaction<'a> {
    fn drop(&mut self) {
        assert!(self.0.is_none(), "Active BlockVersionTransaction left scope. Please make sure you call commit() or cancel() on it.");
    }
}

#[derive(Debug)]
pub struct KnownBlockVersions {
    // TODO Remove pub, it's only for serialization
    pub(super) integrity_violation_in_previous_run: AtomicBool,
    pub(super) block_infos: Arc<LockableHashMap<BlockId, BlockInfo>>,
}

impl Default for KnownBlockVersions {
    fn default() -> Self {
        Self {
            integrity_violation_in_previous_run: false.into(),
            block_infos: Arc::new(LockableHashMap::new()),
        }
    }
}

impl KnownBlockVersions {
    // TODO Test
    pub fn load_or_default(file_path: &Path) -> Result<Self> {
        if let Some(serialized) = KnownBlockVersionsSerialized::deserialize_from_file(file_path)? {
            Ok(serialized.into())
        } else {
            Ok(KnownBlockVersions::default())
        }
    }

    // TODO Test
    pub async fn save(self, file_path: &Path) -> Result<()> {
        KnownBlockVersionsSerialized::async_from(self)
            .await
            .serialize_to_file(file_path)
    }

    // TODO Test
    pub async fn lock_block_info(
        &self,
        block_id: BlockId,
    ) -> HashMapOwnedGuard<BlockId, BlockInfo> {
        self.block_infos.async_lock_owned(block_id).await
    }

    pub fn existing_blocks(&self) -> Vec<BlockId> {
        self.block_infos.keys()
    }

    pub fn integrity_violation_in_previous_run(&self) -> bool {
        self.integrity_violation_in_previous_run
            .load(Ordering::SeqCst)
    }

    pub fn set_integrity_violation_in_previous_run(&self) {
        self.integrity_violation_in_previous_run
            .store(true, Ordering::SeqCst);
    }
}
