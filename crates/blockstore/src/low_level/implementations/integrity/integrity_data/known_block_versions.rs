use anyhow::{bail, ensure, Result};
use binrw::{BinRead, BinResult, BinWrite, Endian};
use lockable::{AsyncLimit, InfallibleUnwrap, Lockable, LockableHashMap};
use rand::{thread_rng, Rng};
use std::collections::hash_map::{Entry, HashMap};
use std::hash::Hash;
use std::io::{Read, Seek, Write};
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::BlockId;
use cryfs_utils::{
    binary::{read_nonzerou32, write_nonzerou32, BinaryReadExt, BinaryWriteExt},
    safe_panic,
};

use super::integrity_violation_error::IntegrityViolationError;
use super::serialization::KnownBlockVersionsSerialized;

#[derive(PartialEq, Eq, Hash, BinRead, BinWrite, Clone, Copy)]
#[brw(little)]
pub struct ClientId {
    // TODO Tuple struct would be better but https://github.com/jam1garner/binwrite/issues/3
    #[br(parse_with = read_nonzerou32)]
    #[bw(write_with = write_nonzerou32)]
    pub id: NonZeroU32, // NonZeroU32 so we can efficiently store MaybeClientId
}

impl ClientId {
    // TODO Test
    pub fn generate_random() -> Self {
        ClientId {
            // TODO Is this ok or could this generate a zero?
            id: thread_rng().gen(),
        }
    }
}

impl std::fmt::Debug for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ClientId({})", self.id)
    }
}

#[derive(PartialEq, Hash, Clone, Copy)]
pub enum MaybeClientId {
    ClientId(ClientId),
    BlockWasDeleted,
}

impl binary_layout::LayoutAs<u32> for ClientId {
    fn read(id: u32) -> ClientId {
        // TODO We shouldn't panic but just return an error
        NonZeroU32::new(id)
            .map(|id| ClientId { id })
            .expect("Loaded block with client_id=0 which shouldn't be possible")
    }

    fn write(id: ClientId) -> u32 {
        id.id.get()
    }
}

impl std::fmt::Debug for MaybeClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaybeClientId::ClientId(id) => write!(f, "{:?}", id),
            MaybeClientId::BlockWasDeleted => write!(f, "BlockWasDeleted"),
        }
    }
}

impl BinRead for MaybeClientId {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        _: (),
    ) -> BinResult<MaybeClientId> {
        let value = u32::read_options(reader, endian, ())?;
        let result = match NonZeroU32::new(value) {
            Some(id) => MaybeClientId::ClientId(ClientId { id }),
            None => MaybeClientId::BlockWasDeleted,
        };
        Ok(result)
    }
}

impl BinWrite for MaybeClientId {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: (),
    ) -> Result<(), binrw::Error> {
        let value = match &self {
            MaybeClientId::ClientId(id) => id.id.get(),
            MaybeClientId::BlockWasDeleted => 0,
        };
        u32::write_options(&value, writer, endian, args)
    }
}

#[derive(PartialEq, Eq, Hash, PartialOrd, BinRead, BinWrite, Clone, Copy)]
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

impl std::fmt::Debug for BlockVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlockVersion({})", self.version)
    }
}

/// [BlockInfo] remembers persistent state about a block, locally on the CryFS client device:
///  - `known_block_versions`
///    The newest version number of the block that we've already seen and was created by the given client.
///    Also, for each client, this remembers the newest version we've seen from them. We won't accept any
//     version numbers older than this from those clients.
///  - `last_update_client_id
///    The client_id we consider to have created the current version of the block.
///
/// The invariant we uphold is that within a `(client_id, block_id)` pair, version numbers are always increasing.
#[derive(Debug)]
pub struct BlockInfo {
    pub last_update_client_id: MaybeClientId,

    // Invariant: If known_block_versions[last_update_client_id] is present,
    //            then we have seen that block before and know its expected
    //            version number. If known_block_versions[last_update_client_id]
    //            is absent, then we have not seen this block yet.
    //            Its BlockInfo may still have been created, most likely because we're
    //            just seeing the block for the first time and are about to add
    //            information about it. But it can also happen (and even be persisted)
    //            if we delete a block that we haven't seen before.
    //            Such a block would have last_update_client_id = None
    //            and no entries in known_block_versions.
    pub known_block_versions: HashMap<ClientId, BlockVersion>,
}

impl BlockInfo {
    pub fn new_unknown(last_update_client_id: MaybeClientId) -> Self {
        BlockInfo {
            last_update_client_id,
            known_block_versions: HashMap::new(),
        }
    }

    #[cfg(test)]
    fn new(
        last_update_client_id: MaybeClientId,
        known_block_versions: HashMap<ClientId, BlockVersion>,
    ) -> Self {
        Self {
            last_update_client_id,
            known_block_versions,
        }
    }

    /// Mark a block as deleted so we can stop an adversary from re-introducing that block
    pub fn mark_block_as_deleted(&mut self) {
        self.last_update_client_id = MaybeClientId::BlockWasDeleted;
    }

    #[cfg(test)]
    pub fn block_is_deleted(&self) -> bool {
        self.last_update_client_id == MaybeClientId::BlockWasDeleted
    }

    /// This is called whenever we modify a block ourselves and updates the internal state correspondingly.
    /// Make sure to either commit or cancel the transaction object returned from this function.
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
        ensure!(version.version > 0, "Version has to be >0");
        let known_block_versions_entry = self.known_block_versions.entry(client_id);
        match known_block_versions_entry {
            Entry::Vacant(known_block_versions_entry) => {
                known_block_versions_entry.insert(version);
                self.last_update_client_id = MaybeClientId::ClientId(client_id);
            }
            Entry::Occupied(mut known_block_versions_entry) => {
                if
                //In all of the cases 1, 2, 3: the version number must not decrease
                (*known_block_versions_entry.get() > version) ||
                    // In case 3 (i.e. we see a change in client id), the version number must increase
                    (self.last_update_client_id != MaybeClientId::ClientId(client_id) && *known_block_versions_entry.get() >= version)
                {
                    let to_client_last_seen_version = *known_block_versions_entry.get();
                    let from_client = self.last_update_client_id();
                    let from_client_last_seen_version = self.current_version();
                    bail!(IntegrityViolationError::RollBack {
                        block: block_id,
                        from_client,
                        from_client_last_seen_version,
                        to_client: client_id,
                        to_client_last_seen_version,
                        actual_version: version,
                    });
                }
                known_block_versions_entry.insert(version);
                self.last_update_client_id = MaybeClientId::ClientId(client_id);
            }
        }

        Ok(())
    }

    /// This function returns true iff we expect the block with the given id to exist,
    /// i.e. we have seen it before and we haven't deleted it. Note that we don't know
    /// if a different client might have deleted it, so this could return true for
    /// blocks that were correctly deleted by other authorized clients.
    pub fn block_is_expected_to_exist(&self) -> bool {
        match self.last_update_client_id {
            MaybeClientId::ClientId(client_id) => {
                // Return whether the block was seen previously
                self.known_block_versions.contains_key(&client_id)
            }
            MaybeClientId::BlockWasDeleted => false,
        }
    }

    pub fn last_update_client_id(&self) -> MaybeClientId {
        self.last_update_client_id
    }

    pub fn current_version(&self) -> Option<BlockVersion> {
        match self.last_update_client_id {
            MaybeClientId::ClientId(client_id) => self.current_version_for_client(&client_id),
            MaybeClientId::BlockWasDeleted => None,
        }
    }

    pub fn current_version_for_client(&self, client_id: &ClientId) -> Option<BlockVersion> {
        self.known_block_versions.get(client_id).copied()
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
        data.block_info.last_update_client_id = MaybeClientId::ClientId(data.new_client_id);
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
        if self.0.is_some() {
            safe_panic!("Active BlockVersionTransaction left scope. Please make sure you call commit() or cancel() on it.");
        }
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
    pub fn load(file_path: &Path) -> Result<Option<Self>> {
        if let Some(serialized) = KnownBlockVersionsSerialized::deserialize_from_file(file_path)? {
            Ok(Some(serialized.into()))
        } else {
            Ok(None)
        }
    }

    pub fn load_or_default(file_path: &Path) -> Result<Self> {
        Ok(Self::load(file_path)?.unwrap_or_default())
    }

    pub async fn save(self, file_path: &Path) -> Result<()> {
        KnownBlockVersionsSerialized::async_from(self)
            .await
            .serialize_to_file(file_path)
    }

    pub async fn lock_block_info(
        &self,
        block_id: BlockId,
    ) -> <LockableHashMap<BlockId, BlockInfo> as Lockable<BlockId, BlockInfo>>::OwnedGuard {
        self.block_infos
            .async_lock_owned(block_id, AsyncLimit::no_limit())
            .await
            .infallible_unwrap()
    }

    pub fn existing_blocks(&self) -> Vec<BlockId> {
        // TODO Is keys_with_entries_or_locked the right thing here? Do we want to count locked entries as "existing" blocks?
        self.block_infos.keys_with_entries_or_locked()
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

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use super::*;
    use crate::tests::blockid;
    use base64::engine::{general_purpose::STANDARD as base64_STANDARD, Engine as _};
    use cryfs_utils::testutils::asserts::assert_unordered_vec_eq;

    use common_macros::hash_map;
    use tempdir::TempDir;

    fn clientid(id: u32) -> ClientId {
        ClientId {
            id: NonZeroU32::new(id).unwrap(),
        }
    }

    fn version(version: u64) -> BlockVersion {
        BlockVersion { version }
    }

    fn assert_versions_are(obj: &BlockInfo, versions: HashMap<ClientId, BlockVersion>) {
        assert_eq!(versions.len(), obj.known_block_versions.len());
        for (clientid, version) in versions {
            assert_eq!(version, obj.current_version_for_client(&clientid).unwrap());
        }
    }

    #[test]
    fn test_givenNewObject_thenHasntSeenTheBlockYet() {
        let obj = BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1)));
        assert_eq!(None, obj.current_version());
    }

    mod increment_version {
        use super::*;

        #[test]
        fn test_givenNewObject_whenIncrementingVersion_thenSucceeds() {
            let mut obj = BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1)));

            let transaction = obj.start_increment_version_transaction(clientid(1));
            assert_eq!(version(1), transaction.new_version());
            transaction.commit();

            assert_eq!(
                MaybeClientId::ClientId(clientid(1)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(1)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(1),
                },
            );
        }

        #[test]
        fn test_givenNewObject_whenIncrementingVersionTwiceForSameClient_thenSucceeds() {
            let mut obj = BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1)));

            let transaction = obj.start_increment_version_transaction(clientid(1));
            assert_eq!(version(1), transaction.new_version());
            transaction.commit();

            let transaction = obj.start_increment_version_transaction(clientid(1));
            assert_eq!(version(2), transaction.new_version());
            transaction.commit();

            assert_eq!(
                MaybeClientId::ClientId(clientid(1)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(2)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(2),
                },
            );
        }

        #[test]
        fn test_givenNewObject_whenIncrementingVersionTwiceForDifferentClient_thenSucceeds() {
            let mut obj = BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1)));

            let transaction = obj.start_increment_version_transaction(clientid(1));
            assert_eq!(version(1), transaction.new_version());
            transaction.commit();

            let transaction = obj.start_increment_version_transaction(clientid(2));
            assert_eq!(version(1), transaction.new_version());
            transaction.commit();

            assert_eq!(
                MaybeClientId::ClientId(clientid(2)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(1)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(2) => version(1),
                    clientid(1) => version(1),
                },
            );
        }

        #[test]
        fn test_givenNewObject_whenIncrementingVersionTwiceForSameClient_butCancellingTransaction_thenDoesntChange(
        ) {
            let mut obj = BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1)));

            let transaction = obj.start_increment_version_transaction(clientid(1));
            assert_eq!(version(1), transaction.new_version());
            transaction.commit();

            let transaction = obj.start_increment_version_transaction(clientid(1));
            assert_eq!(version(2), transaction.new_version());
            transaction.cancel();

            assert_eq!(
                MaybeClientId::ClientId(clientid(1)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(1)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(1),
                },
            );
        }

        #[test]
        fn test_givenNewObject_whenIncrementingVersionTwiceForDifferentClient_butCancellingTransaction_thenDoesntChange(
        ) {
            let mut obj = BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1)));

            let transaction = obj.start_increment_version_transaction(clientid(1));
            assert_eq!(version(1), transaction.new_version());
            transaction.commit();

            let transaction = obj.start_increment_version_transaction(clientid(1));
            assert_eq!(version(2), transaction.new_version());
            transaction.cancel();

            assert_eq!(
                MaybeClientId::ClientId(clientid(1)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(1)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(1),
                },
            );
            assert_eq!(None, obj.current_version_for_client(&clientid(2)));
        }

        #[test]
        #[should_panic(
            expected = "Active BlockVersionTransaction left scope. Please make sure you call commit() or cancel() on it."
        )]
        fn test_whenOpenTransactionLeavesScope_thenPanics() {
            let mut obj = BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1)));

            obj.start_increment_version_transaction(clientid(1));
        }

        #[test]
        fn test_givenExistingObject_whenIncrementingVersionForExistingClient_thenSucceeds() {
            let mut obj = BlockInfo::new(
                MaybeClientId::ClientId(clientid(1)),
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                },
            );

            let transaction = obj.start_increment_version_transaction(clientid(3));
            assert_eq!(version(5), transaction.new_version());
            transaction.commit();

            assert_eq!(
                MaybeClientId::ClientId(clientid(3)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(5)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(5),
                    clientid(4) => version(7),
                },
            );
        }

        #[test]
        fn test_givenExistingObject_whenIncrementingVersionForNewClient_thenSucceeds() {
            let mut obj = BlockInfo::new(
                MaybeClientId::ClientId(clientid(1)),
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                },
            );

            let transaction = obj.start_increment_version_transaction(clientid(6));
            assert_eq!(version(1), transaction.new_version());
            transaction.commit();

            assert_eq!(
                MaybeClientId::ClientId(clientid(6)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(1)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                    clientid(6) => version(1),
                },
            );
        }

        #[test]
        fn test_givenDeletedObject_whenIncrementingVersionForNewClient_thenSucceeds() {
            let mut obj = BlockInfo::new(
                MaybeClientId::BlockWasDeleted,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                },
            );

            let transaction = obj.start_increment_version_transaction(clientid(6));
            assert_eq!(version(1), transaction.new_version());
            transaction.commit();

            assert_eq!(
                MaybeClientId::ClientId(clientid(6)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(1)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                    clientid(6) => version(1),
                },
            );
        }

        #[test]
        fn test_givenDeletedObject_whenIncrementingVersionForExistingClient_thenSucceeds() {
            let mut obj = BlockInfo::new(
                MaybeClientId::BlockWasDeleted,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                },
            );

            let transaction = obj.start_increment_version_transaction(clientid(2));
            assert_eq!(version(3), transaction.new_version());
            transaction.commit();

            assert_eq!(
                MaybeClientId::ClientId(clientid(2)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(3)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(3),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                },
            );
        }
    }

    mod deletion {
        use super::*;

        #[test]
        fn test_givenNewObject_thenBlockIsNotMarkedAsDeleted() {
            let obj = BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1)));
            assert!(!obj.block_is_deleted());
        }

        #[test]
        fn test_givenIncrementedVersion_thenBlockIsNotMarkedAsDeleted() {
            let mut obj = BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1)));
            obj.start_increment_version_transaction(clientid(1))
                .commit();

            assert!(!obj.block_is_deleted());
        }

        #[test]
        fn test_givenUpdatedVersion_thenBlockIsNotMarkedAsDeleted() {
            let mut obj = BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1)));
            obj.check_and_update_version(clientid(1), blockid(0), version(3))
                .unwrap();

            assert!(!obj.block_is_deleted());
        }

        #[test]
        fn test_givenNewObject_whenMarkingBlockAsDeleted_thenSucceeds() {
            let mut obj = BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1)));
            obj.mark_block_as_deleted();

            assert!(obj.block_is_deleted());
        }

        #[test]
        fn test_givenIncrementedVersion_whenMarkingBlockAsDeleted_thenSucceeds() {
            let mut obj = BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1)));
            obj.start_increment_version_transaction(clientid(1))
                .commit();
            obj.mark_block_as_deleted();

            assert!(obj.block_is_deleted());
        }

        #[test]
        fn test_givenUpdatedVersion_whenMarkingBlockAsDeleted_thenSucceeds() {
            let mut obj = BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1)));
            obj.check_and_update_version(clientid(1), blockid(0), version(3))
                .unwrap();
            obj.mark_block_as_deleted();

            assert!(obj.block_is_deleted());
        }
    }

    mod check_and_update_version {
        use super::*;

        #[test]
        fn test_givenNewObject_whenCheckingAndUpdatingVersionForSameClientId_thenSucceeds() {
            let mut obj = BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1)));

            obj.check_and_update_version(clientid(1), blockid(1), version(5))
                .unwrap();

            assert_eq!(
                MaybeClientId::ClientId(clientid(1)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(5)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(5),
                },
            );
        }

        #[test]
        fn test_givenNewObject_whenCheckingAndUpdatingVersionForDifferentClientId_thenSucceeds() {
            let mut obj = BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1)));

            obj.check_and_update_version(clientid(2), blockid(1), version(5))
                .unwrap();

            assert_eq!(
                MaybeClientId::ClientId(clientid(2)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(5)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(2) => version(5),
                },
            );
        }

        #[test]
        fn test_givenNewObject_whenCheckingAndUpdatingVersionTwiceForSameClient_withVersionIsEqual_thenSucceeds(
        ) {
            let mut obj = BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1)));

            obj.check_and_update_version(clientid(1), blockid(1), version(5))
                .unwrap();
            obj.check_and_update_version(clientid(1), blockid(1), version(5))
                .unwrap();

            assert_eq!(
                MaybeClientId::ClientId(clientid(1)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(5)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(5),
                },
            );
        }

        #[test]
        fn test_givenNewObject_whenCheckingAndUpdatingVersionTwiceForSameClient_withVersionIsIncreasing_thenSucceeds(
        ) {
            let mut obj = BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1)));

            obj.check_and_update_version(clientid(1), blockid(1), version(5))
                .unwrap();
            obj.check_and_update_version(clientid(1), blockid(1), version(7))
                .unwrap();

            assert_eq!(
                MaybeClientId::ClientId(clientid(1)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(7)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(7),
                },
            );
        }

        #[test]
        fn test_givenNewObject_whenCheckingAndUpdatingVersionTwiceForSameClient_withVersionIsDecreasing_thenFails(
        ) {
            let mut obj = BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1)));

            obj.check_and_update_version(clientid(1), blockid(1), version(5))
                .unwrap();
            obj.check_and_update_version(clientid(1), blockid(1), version(4))
                .unwrap_err();

            assert_eq!(
                MaybeClientId::ClientId(clientid(1)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(5)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(5),
                },
            );
        }

        #[test]
        fn test_givenNewObject_whenCheckingAndUpdatingVersionTwiceForDifferentClient_withVersionIsEqual_thenSucceeds(
        ) {
            let mut obj = BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1)));

            obj.check_and_update_version(clientid(2), blockid(1), version(5))
                .unwrap();
            obj.check_and_update_version(clientid(3), blockid(1), version(5))
                .unwrap();

            assert_eq!(
                MaybeClientId::ClientId(clientid(3)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(5)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(2) => version(5),
                    clientid(3) => version(5),
                },
            );
        }

        #[test]
        fn test_givenNewObject_whenCheckingAndUpdatingVersionTwiceForDifferentClient_withVersionIsIncreasing_thenSucceeds(
        ) {
            let mut obj = BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1)));

            obj.check_and_update_version(clientid(2), blockid(1), version(5))
                .unwrap();
            obj.check_and_update_version(clientid(3), blockid(1), version(7))
                .unwrap();

            assert_eq!(
                MaybeClientId::ClientId(clientid(3)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(7)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(2) => version(5),
                    clientid(3) => version(7),
                },
            );
        }

        #[test]
        fn test_givenNewObject_whenCheckingAndUpdatingVersionTwiceForDifferentClient_withVersionIsDecreasing_thenSucceeds(
        ) {
            let mut obj = BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1)));

            obj.check_and_update_version(clientid(2), blockid(1), version(5))
                .unwrap();
            obj.check_and_update_version(clientid(3), blockid(1), version(4))
                .unwrap();

            assert_eq!(
                MaybeClientId::ClientId(clientid(3)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(4)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(2) => version(5),
                    clientid(3) => version(4),
                },
            );
        }

        #[test]
        fn test_givenExistingObject_whenCheckingAndUpdatingVersionForLastUpdateClient_withVersionIsDecreasing_thenFails(
        ) {
            let mut obj = BlockInfo::new(
                MaybeClientId::ClientId(clientid(3)),
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                },
            );

            obj.check_and_update_version(clientid(3), blockid(1), version(3))
                .unwrap_err();

            assert_eq!(
                MaybeClientId::ClientId(clientid(3)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(4)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                },
            );
        }

        #[test]
        fn test_givenExistingObject_whenCheckingAndUpdatingVersionForNonLastUpdateClient_withVersionIsDecreasing_thenFails(
        ) {
            let mut obj = BlockInfo::new(
                MaybeClientId::ClientId(clientid(1)),
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                },
            );

            obj.check_and_update_version(clientid(3), blockid(1), version(3))
                .unwrap_err();

            assert_eq!(
                MaybeClientId::ClientId(clientid(1)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(8)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                },
            );
        }

        #[test]
        fn test_givenExistingObject_whenCheckingAndUpdatingVersionForLastUpdateClient_withVersionIsEqual_thenSucceeds(
        ) {
            let mut obj = BlockInfo::new(
                MaybeClientId::ClientId(clientid(3)),
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                },
            );

            obj.check_and_update_version(clientid(3), blockid(1), version(4))
                .unwrap();

            assert_eq!(
                MaybeClientId::ClientId(clientid(3)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(4)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                },
            );
        }

        #[test]
        fn test_givenExistingObject_whenCheckingAndUpdatingVersionForNonLastUpdateClient_withVersionIsEqual_thenFails(
        ) {
            let mut obj = BlockInfo::new(
                MaybeClientId::ClientId(clientid(1)),
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                },
            );

            obj.check_and_update_version(clientid(3), blockid(1), version(4))
                .unwrap_err();

            assert_eq!(
                MaybeClientId::ClientId(clientid(1)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(8)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                },
            );
        }

        #[test]
        fn test_givenExistingObject_whenCheckingAndUpdatingVersionForLastUpdateClient_withVersionIsIncreasingByOne_thenSucceeds(
        ) {
            let mut obj = BlockInfo::new(
                MaybeClientId::ClientId(clientid(3)),
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                },
            );

            obj.check_and_update_version(clientid(3), blockid(1), version(5))
                .unwrap();

            assert_eq!(
                MaybeClientId::ClientId(clientid(3)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(5)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(5),
                    clientid(4) => version(7),
                },
            );
        }

        #[test]
        fn test_givenExistingObject_whenCheckingAndUpdatingVersionForNonLastUpdateClient_withVersionIsIncreasingByOne_thenSucceeds(
        ) {
            let mut obj = BlockInfo::new(
                MaybeClientId::ClientId(clientid(1)),
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                },
            );

            obj.check_and_update_version(clientid(3), blockid(1), version(5))
                .unwrap();

            assert_eq!(
                MaybeClientId::ClientId(clientid(3)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(5)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(5),
                    clientid(4) => version(7),
                },
            );
        }

        #[test]
        fn test_givenExistingObject_whenCheckingAndUpdatingVersionForLastUpdateClient_withVersionIsIncreasingByMoreThanOne_thenSucceeds(
        ) {
            let mut obj = BlockInfo::new(
                MaybeClientId::ClientId(clientid(3)),
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                },
            );

            obj.check_and_update_version(clientid(3), blockid(1), version(10))
                .unwrap();

            assert_eq!(
                MaybeClientId::ClientId(clientid(3)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(10)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(10),
                    clientid(4) => version(7),
                },
            );
        }

        #[test]
        fn test_givenExistingObject_whenCheckingAndUpdatingVersionForNonLastUpdateClient_withVersionIsIncreasingByMoreThanOne_thenSucceeds(
        ) {
            let mut obj = BlockInfo::new(
                MaybeClientId::ClientId(clientid(1)),
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                },
            );

            obj.check_and_update_version(clientid(3), blockid(1), version(10))
                .unwrap();

            assert_eq!(
                MaybeClientId::ClientId(clientid(3)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(10)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(10),
                    clientid(4) => version(7),
                },
            );
        }

        #[test]
        fn test_givenExistingObject_whenCheckingAndUpdatingVersionForNewClient_withVersionIsOne_thenSucceeds(
        ) {
            let mut obj = BlockInfo::new(
                MaybeClientId::ClientId(clientid(1)),
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                },
            );

            obj.check_and_update_version(clientid(6), blockid(1), version(1))
                .unwrap();

            assert_eq!(
                MaybeClientId::ClientId(clientid(6)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(1)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                    clientid(6) => version(1),
                },
            );
        }

        #[test]
        fn test_givenExistingObject_whenCheckingAndUpdatingVersionForNewClient_withVersionIsHigherThanOne_thenSucceeds(
        ) {
            let mut obj = BlockInfo::new(
                MaybeClientId::ClientId(clientid(1)),
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                },
            );

            obj.check_and_update_version(clientid(6), blockid(1), version(10))
                .unwrap();

            assert_eq!(
                MaybeClientId::ClientId(clientid(6)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(10)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                    clientid(6) => version(10),
                },
            );
        }

        #[test]
        fn test_givenDeletedObject_whenCheckingAndUpdatingVersionForExistingClient_withVersionIsDecreasing_thenFails(
        ) {
            let mut obj = BlockInfo::new(
                MaybeClientId::BlockWasDeleted,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                },
            );

            obj.check_and_update_version(clientid(3), blockid(1), version(3))
                .unwrap_err();

            assert_eq!(MaybeClientId::BlockWasDeleted, obj.last_update_client_id());
            assert_eq!(None, obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                },
            );
        }

        #[test]
        fn test_givenDeletedObject_whenCheckingAndUpdatingVersionForExistingClient_withVersionIsEqual_thenFails(
        ) {
            let mut obj = BlockInfo::new(
                MaybeClientId::BlockWasDeleted,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                },
            );

            obj.check_and_update_version(clientid(3), blockid(1), version(4))
                .unwrap_err();

            assert_eq!(MaybeClientId::BlockWasDeleted, obj.last_update_client_id());
            assert_eq!(None, obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                },
            );
        }

        #[test]
        fn test_givenDeletedObject_whenCheckingAndUpdatingVersionForExistingClient_withVersionIsIncreasingByOne_thenSucceeds(
        ) {
            let mut obj = BlockInfo::new(
                MaybeClientId::BlockWasDeleted,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                },
            );

            obj.check_and_update_version(clientid(3), blockid(1), version(5))
                .unwrap();

            assert_eq!(
                MaybeClientId::ClientId(clientid(3)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(5)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(5),
                    clientid(4) => version(7),
                },
            );
        }

        #[test]
        fn test_givenDeletedObject_whenCheckingAndUpdatingVersionForExistingClient_withVersionIsIncreasingByMoreThanOne_thenSucceeds(
        ) {
            let mut obj = BlockInfo::new(
                MaybeClientId::BlockWasDeleted,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                },
            );

            obj.check_and_update_version(clientid(3), blockid(1), version(10))
                .unwrap();

            assert_eq!(
                MaybeClientId::ClientId(clientid(3)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(10)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(10),
                    clientid(4) => version(7),
                },
            );
        }

        #[test]
        fn test_givenDeletedObject_whenCheckingAndUpdatingVersionForNewClient_withVersionIsOne_thenSucceeds(
        ) {
            let mut obj = BlockInfo::new(
                MaybeClientId::BlockWasDeleted,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                },
            );

            obj.check_and_update_version(clientid(6), blockid(1), version(1))
                .unwrap();

            assert_eq!(
                MaybeClientId::ClientId(clientid(6)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(1)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                    clientid(6) => version(1),
                },
            );
        }

        #[test]
        fn test_givenDeletedObject_whenCheckingAndUpdatingVersionForNewClient_withVersionIsMoreThanOne_thenSucceeds(
        ) {
            let mut obj = BlockInfo::new(
                MaybeClientId::BlockWasDeleted,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                },
            );

            obj.check_and_update_version(clientid(6), blockid(1), version(10))
                .unwrap();

            assert_eq!(
                MaybeClientId::ClientId(clientid(6)),
                obj.last_update_client_id()
            );
            assert_eq!(Some(version(10)), obj.current_version());
            assert_versions_are(
                &obj,
                hash_map! {
                    clientid(1) => version(8),
                    clientid(2) => version(2),
                    clientid(3) => version(4),
                    clientid(4) => version(7),
                    clientid(6) => version(10),
                },
            );
        }
    }

    mod current_version {
        use super::*;

        #[test]
        fn test_givenEmptyObject_whenQueryingCurrentVersion_thenReturnsNone() {
            let obj = BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1)));
            assert_eq!(None, obj.current_version());
        }

        #[test]
        fn test_givenEmptyObject_afterIncrementingVersionAndCommitting_whenQueryingCurrentVersion_thenReturnsSome(
        ) {
            let mut obj = BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1)));
            obj.start_increment_version_transaction(clientid(5))
                .commit();
            assert_eq!(Some(version(1)), obj.current_version());
        }

        #[test]
        fn test_givenEmptyObject_afterIncrementingVersionAndCancelling_whenQueryingCurrentVersion_thenReturnsNone(
        ) {
            let mut obj = BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1)));
            obj.start_increment_version_transaction(clientid(5))
                .cancel();
            assert_eq!(None, obj.current_version());
        }

        #[test]
        fn test_givenEmptyObject_afterCheckingAndUpdatingVersion_whenQueryingCurrentVersion_thenReturnsSome(
        ) {
            let mut obj = BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1)));
            obj.check_and_update_version(clientid(5), blockid(1), version(5))
                .unwrap();
            assert_eq!(Some(version(5)), obj.current_version());
        }

        #[test]
        fn test_givenEmptyObject_afterDeletingBlock_whenQueryingCurrentVersion_thenReturnsNone() {
            let mut obj = BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1)));
            obj.check_and_update_version(clientid(5), blockid(1), version(5))
                .unwrap();
            assert_eq!(Some(version(5)), obj.current_version());
            obj.mark_block_as_deleted();
            assert_eq!(None, obj.current_version());
        }
    }

    mod existing_blocks {
        use super::*;

        #[test]
        fn test_givenEmptyObject_whenQueryingExistingBlocks_thenReturnsEmptyList() {
            let obj = KnownBlockVersions::default();
            assert_unordered_vec_eq(vec![], obj.existing_blocks());
        }

        #[tokio::test]
        async fn test_givenEmptyObject_whenAddingSomeBlocks_thenReturnsAll() {
            let obj = KnownBlockVersions::default();
            obj.lock_block_info(blockid(1))
                .await
                .try_insert(BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1))))
                .unwrap();
            obj.lock_block_info(blockid(2))
                .await
                .try_insert(BlockInfo::new_unknown(MaybeClientId::BlockWasDeleted))
                .unwrap();
            obj.lock_block_info(blockid(3))
                .await
                .try_insert(BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1))))
                .unwrap();
            assert_unordered_vec_eq(
                vec![blockid(1), blockid(2), blockid(3)],
                obj.existing_blocks(),
            );
        }
    }

    mod block_is_expected_to_exist {
        use super::*;

        #[test]
        fn test_givenEmptyObject_whenQueryingBlockIsExpectedToExist_thenReturnsNo() {
            let obj = BlockInfo::new_unknown(MaybeClientId::ClientId(clientid(1)));
            assert_eq!(false, obj.block_is_expected_to_exist());
        }

        #[test]
        fn test_givenNonEmptyObject_whenQueryingBlockIsExpectedToExist_forExistingBlock_thenReturnsYes(
        ) {
            let obj = BlockInfo::new(
                MaybeClientId::ClientId(clientid(1)),
                hash_map! {
                    clientid(1) => version(5),
                    clientid(3) => version(1),
                },
            );
            assert_eq!(true, obj.block_is_expected_to_exist());
        }

        #[test]
        fn test_givenNonEmptyObject_whenQueryingBlockIsExpectedToExist_forDeletedBlock_thenReturnsNo(
        ) {
            let obj = BlockInfo::new(
                MaybeClientId::BlockWasDeleted,
                hash_map! {
                    clientid(1) => version(5),
                    clientid(3) => version(1),
                },
            );
            assert_eq!(false, obj.block_is_expected_to_exist());
        }

        #[test]
        fn test_givenNonEmptyObject_whenQueryingBlockIsExpectedToExist_afterDeletingBlock_thenReturnsNo(
        ) {
            let mut obj = BlockInfo::new(
                MaybeClientId::ClientId(clientid(1)),
                hash_map! {
                    clientid(1) => version(5),
                    clientid(3) => version(1),
                },
            );
            obj.mark_block_as_deleted();
            assert_eq!(false, obj.block_is_expected_to_exist());
        }
    }

    mod save_and_load {
        use super::*;

        #[test]
        fn test_givenNoFile_whenLoading_thenReturnsDefault() {
            let tempdir = tempdir::TempDir::new("test").unwrap();
            let nonexisting_path = tempdir.path().join("not-existing");
            let loaded = KnownBlockVersions::load_or_default(&nonexisting_path).unwrap();
            assert_eq!(
                false,
                loaded
                    .integrity_violation_in_previous_run
                    .load(Ordering::SeqCst)
            );
            assert_eq!(Vec::<BlockId>::new(), loaded.existing_blocks());
        }

        #[tokio::test]
        async fn test_givenEmptyObject_withNoPreviousViolation_whenSavingAndLoading_thenSucceeds() {
            let tempdir = tempdir::TempDir::new("test").unwrap();
            let filepath = tempdir.path().join("file");
            let obj = KnownBlockVersions::default();

            obj.save(&filepath).await.unwrap();
            let loaded = KnownBlockVersions::load_or_default(&filepath).unwrap();
            assert_eq!(
                false,
                loaded
                    .integrity_violation_in_previous_run
                    .load(Ordering::SeqCst)
            );
            assert_eq!(Vec::<BlockId>::new(), loaded.existing_blocks());
        }

        #[tokio::test]
        async fn test_givenEmptyObject_withPreviousViolation_whenSavingAndLoading_thenSucceeds() {
            let tempdir = tempdir::TempDir::new("test").unwrap();
            let filepath = tempdir.path().join("file");
            let obj = KnownBlockVersions::default();
            obj.set_integrity_violation_in_previous_run();

            obj.save(&filepath).await.unwrap();
            let loaded = KnownBlockVersions::load_or_default(&filepath).unwrap();
            assert_eq!(
                true,
                loaded
                    .integrity_violation_in_previous_run
                    .load(Ordering::SeqCst)
            );
            assert_eq!(Vec::<BlockId>::new(), loaded.existing_blocks());
        }

        #[tokio::test]
        async fn test_givenNonEmptyObject_whenSavingAndLoading_thenSucceeds() {
            let tempdir = tempdir::TempDir::new("test").unwrap();
            let filepath = tempdir.path().join("file");
            let obj = KnownBlockVersions::default();
            obj.lock_block_info(blockid(1))
                .await
                .try_insert(BlockInfo::new(
                    MaybeClientId::ClientId(clientid(2)),
                    hash_map! {
                        clientid(1) => version(5),
                        clientid(2) => version(3),
                        clientid(5) => version(6),
                    },
                ))
                .unwrap();
            obj.lock_block_info(blockid(2))
                .await
                .try_insert(BlockInfo::new(
                    MaybeClientId::BlockWasDeleted,
                    hash_map! {
                        clientid(1) => version(3),
                        clientid(2) => version(8),
                        clientid(5) => version(2),
                    },
                ))
                .unwrap();

            obj.save(&filepath).await.unwrap();
            let loaded = KnownBlockVersions::load_or_default(&filepath).unwrap();

            assert_unordered_vec_eq(vec![blockid(1), blockid(2)], loaded.existing_blocks());
            let block_info = loaded.lock_block_info(blockid(1)).await;
            assert_eq!(
                MaybeClientId::ClientId(clientid(2)),
                block_info.value().unwrap().last_update_client_id,
            );
            assert_versions_are(
                &block_info.value().unwrap(),
                hash_map! {
                    clientid(1) => version(5),
                    clientid(2) => version(3),
                    clientid(5) => version(6),
                },
            );
            let block_info = loaded.lock_block_info(blockid(2)).await;
            assert_eq!(
                MaybeClientId::BlockWasDeleted,
                block_info.value().unwrap().last_update_client_id,
            );
            assert_versions_are(
                &block_info.value().unwrap(),
                hash_map! {
                    clientid(1) => version(3),
                    clientid(2) => version(8),
                    clientid(5) => version(2),
                },
            );
        }

        fn write_file(path: &Path, content: &[u8]) {
            let mut file = std::fs::File::create(path).unwrap();
            file.write_all(content).unwrap();
        }

        #[tokio::test]
        async fn test_backwards_compatibility_empty_file() {
            let tempdir = TempDir::new("test").unwrap();
            let path = tempdir.path().join("file");
            write_file(&path, &base64_STANDARD.decode("Y3J5ZnMuaW50ZWdyaXR5ZGF0YS5rbm93bmJsb2NrdmVyc2lvbnM7MQAAAAAAAAAAAAAAAAAAAAAAAA==").unwrap());
            let obj = KnownBlockVersions::load(&path).unwrap().unwrap();
            assert!(!obj.integrity_violation_in_previous_run());
            assert_eq!(Vec::<BlockId>::new(), obj.existing_blocks());
        }

        #[tokio::test]
        async fn test_backwards_compatibility_empty_file_with_previous_violation() {
            let tempdir = TempDir::new("test").unwrap();
            let path = tempdir.path().join("file");
            write_file(&path, &base64_STANDARD.decode("Y3J5ZnMuaW50ZWdyaXR5ZGF0YS5rbm93bmJsb2NrdmVyc2lvbnM7MQABAAAAAAAAAAAAAAAAAAAAAA==").unwrap());
            let obj = KnownBlockVersions::load(&path).unwrap().unwrap();
            assert!(obj.integrity_violation_in_previous_run());
            assert_eq!(Vec::<BlockId>::new(), obj.existing_blocks());
        }

        #[tokio::test]
        async fn test_backwards_compatibility_nonempty_file() {
            let tempdir = TempDir::new("test").unwrap();
            let path = tempdir.path().join("file");
            write_file(&path, &base64_STANDARD.decode("Y3J5ZnMuaW50ZWdyaXR5ZGF0YS5rbm93bmJsb2NrdmVyc2lvbnM7MQAAAwAAAAAAAACJZ0Ujkyo4nuFLxwkKx3KXIUkbtAIAAAAAAAAAeFY0EhSRu0kyo4nuFLxwkKx3KXIFAAAAAAAAAIlnRSMUkbtJMqOJ7hS8cJCsdylyCgAAAAAAAAACAAAAAAAAAJMqOJ7hS8cJCsdylyFJG7QAAAAAFJG7STKjie4UvHCQrHcpcnhWNBI=").unwrap());
            let obj = KnownBlockVersions::load(&path).unwrap().unwrap();
            assert!(!obj.integrity_violation_in_previous_run());

            let blockid1 = BlockId::from_hex("1491BB4932A389EE14BC7090AC772972").unwrap();
            let blockid2 = BlockId::from_hex("932A389EE14BC7090AC7729721491BB4").unwrap();
            let clientid1 = clientid(0x23456789);
            let clientid2 = clientid(0x12345678);
            assert_unordered_vec_eq(vec![blockid1, blockid2], obj.existing_blocks());
            {
                let info = obj.lock_block_info(blockid1).await;
                let info = info.value().unwrap();
                assert!(!info.block_is_deleted());
                assert_eq!(
                    MaybeClientId::ClientId(clientid2),
                    info.last_update_client_id()
                );
                assert_versions_are(
                    info,
                    hash_map! {
                        clientid1 => version(10),
                        clientid2 => version(5),
                    },
                );
            }
            {
                let info = obj.lock_block_info(blockid2).await;
                let info = info.value().unwrap();
                assert!(info.block_is_deleted());
                assert_eq!(MaybeClientId::BlockWasDeleted, info.last_update_client_id());
                assert_versions_are(
                    info,
                    hash_map! {
                        clientid1 => version(2),
                    },
                );
            }
        }
    }
}
