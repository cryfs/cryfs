use anyhow::{anyhow, bail, ensure, Context, Result};
use async_trait::async_trait;
use binary_layout::prelude::*;
use futures::{
    future::{self},
    stream::{FuturesUnordered, Stream, StreamExt, TryStreamExt},
};
use log::warn;
use std::collections::hash_set::HashSet;
use std::fmt::{self, Debug};
use std::path::PathBuf;
use std::pin::Pin;

use super::block_data::IBlockData;
use super::{
    BlockId, BlockStore, BlockStoreDeleter, BlockStoreReader, OptimizedBlockStoreWriter,
    RemoveResult, TryCreateResult, BLOCKID_LEN,
};

mod integrity_data;

use crate::data::Data;
use crate::utils::async_drop::{AsyncDrop, AsyncDropGuard};
pub use integrity_data::ClientId;
use integrity_data::{
    BlockInfo, BlockVersion, BlockVersionTransaction, IntegrityData, IntegrityViolationError,
    MaybeClientId,
};

const FORMAT_VERSION_HEADER: u16 = 1;

binary_layout::define_layout!(block_layout, LittleEndian, {
    // TODO Use types BlockId, FormatVersionHeader as types instead of slices
    format_version_header: u16,
    block_id: [u8; BLOCKID_LEN],
    last_update_client_id: ClientId as u32,
    block_version: BlockVersion as u64,
    data: [u8],
});

const HEADER_SIZE: usize = block_layout::data::OFFSET;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllowIntegrityViolations {
    AllowViolations,
    DontAllowViolations,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissingBlockIsIntegrityViolation {
    IsAViolation,
    IsNotAViolation,
}

pub struct IntegrityConfig {
    pub allow_integrity_violations: AllowIntegrityViolations,
    pub missing_block_is_integrity_violation: MissingBlockIsIntegrityViolation,
    pub on_integrity_violation: Box<dyn Sync + Send + Fn(&IntegrityViolationError)>,
}

pub struct IntegrityBlockStore<B: Send + Debug + AsyncDrop<Error = anyhow::Error>> {
    underlying_block_store: AsyncDropGuard<B>,
    config: IntegrityConfig,
    integrity_data: AsyncDropGuard<IntegrityData>,
}

impl<B: Send + Sync + Debug + AsyncDrop<Error = anyhow::Error>> IntegrityBlockStore<B> {
    pub fn new(
        underlying_block_store: AsyncDropGuard<B>,
        integrity_file_path: PathBuf,
        my_client_id: ClientId,
        config: IntegrityConfig,
    ) -> Result<AsyncDropGuard<Self>> {
        let integrity_data = IntegrityData::new(integrity_file_path.clone(), my_client_id)
            .context("Tried to create IntegrityData")?;
        if integrity_data.integrity_violation_in_previous_run() {
            match config.allow_integrity_violations {
                AllowIntegrityViolations::AllowViolations => warn!("Integrity violation in previous run (but integrity checks are disabled)"),
                AllowIntegrityViolations::DontAllowViolations => {
                    return Err(IntegrityViolationError::IntegrityViolationInPreviousRun {
                        integrity_file_path: integrity_file_path.clone(),
                    }
                    .into());
                }
            }
        }
        Ok(AsyncDropGuard::new(Self {
            underlying_block_store,
            config,
            integrity_data,
        }))
    }
}

impl<B: Send + Debug + AsyncDrop<Error = anyhow::Error>> Debug for IntegrityBlockStore<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IntegrityBlockStore")
    }
}

#[async_trait]
impl<B: BlockStoreReader + Sync + Send + Debug + AsyncDrop<Error = anyhow::Error>> BlockStoreReader
    for IntegrityBlockStore<B>
{
    async fn exists(&self, block_id: &BlockId) -> Result<bool> {
        self.underlying_block_store.exists(block_id).await
    }

    async fn load(&self, block_id: &BlockId) -> Result<Option<Data>> {
        let mut block_info_guard = self.integrity_data.lock_block_info(*block_id).await;
        let loaded = self.underlying_block_store.load(block_id).await.context(
            "IntegrityBlockStore failed to load the block from the underlying block store",
        )?;
        match loaded {
            None => {
                match self.config.missing_block_is_integrity_violation {
                    MissingBlockIsIntegrityViolation::IsAViolation => {
                        if let Some(block_info) = block_info_guard.value() {
                            if block_info.block_is_expected_to_exist() {
                                self._integrity_violation_detected(
                                    IntegrityViolationError::MissingBlock { block: *block_id }.into(),
                                )?;
                            }
                        }
                    }
                    MissingBlockIsIntegrityViolation::IsNotAViolation => {
                        // do nothing
                    }
                }
                Ok(None)
            }
            Some(loaded) => {
                let block_info = block_info_guard.value_or_insert_with(|| {
                    BlockInfo::new_unknown(MaybeClientId::ClientId(
                        self.integrity_data.my_client_id(),
                    ))
                });
                let data = self._check_and_remove_header(block_info, loaded, *block_id)?;
                Ok(Some(data))
            }
        }
    }

    async fn num_blocks(&self) -> Result<u64> {
        self.underlying_block_store.num_blocks().await
    }

    fn estimate_num_free_bytes(&self) -> Result<u64> {
        self.underlying_block_store.estimate_num_free_bytes()
    }

    // TODO Test this by creating a blockstore based on an underlying block store (or on disk) and comparing the physical size. Same for encrypted block store.
    fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64> {
        self.underlying_block_store.block_size_from_physical_block_size(block_size)?.checked_sub(HEADER_SIZE as u64)
            .with_context(|| anyhow!("Physical block size of {} is too small to hold even the FORMAT_VERSION_HEADER. Must be at least {}.", block_size, HEADER_SIZE))
    }

    async fn all_blocks(&self) -> Result<Pin<Box<dyn Stream<Item = Result<BlockId>> + Send>>> {
        match self.config.missing_block_is_integrity_violation {
            MissingBlockIsIntegrityViolation::IsAViolation => {
                // TODO Is there a way to do this with stream processing, i.e. without collecting?
                //      That's what the C++ implementation did. It would mean that errors about missing blocks
                //      would be delayed though. And we'd likely have to handle race conditions like
                //      blocks being deleted while this function runs.
                let all_underlying_blocks = {
                    let blocks = self.underlying_block_store.all_blocks().await?;
                    let blocks: Vec<BlockId> = blocks.try_collect().await?;
                    blocks
                };
                // We get expected_blocks **after** we got all_underlying blocks so that any blocks potentially deleted
                // in the meantime are already gone from expected_blocks.
                // TODO Is this actually race-condition-proof? What if there is currently a remove ongoing, it has already
                //      deleted the block before the calculation of all_underlying_blocks, but hasn't updated integrity_data yet
                //      when we're running this?
                let mut expected_blocks = {
                    let blocks: HashSet<BlockId> =
                        self.integrity_data.existing_blocks().into_iter().collect();
                    blocks
                };

                for existing_block in &all_underlying_blocks {
                    expected_blocks.remove(existing_block);
                }
                let missing_blocks: FuturesUnordered<_> = expected_blocks
                    .into_iter()
                    .map(|id| future::ready(id))
                    .collect();
                let missing_blocks = missing_blocks
                    .map(|v| -> Result<BlockId> { Ok(v) })
                    .try_filter_map(|expected_block_id| async move {
                        // We have a block that our integrity data says should exist but the underlying block store says doesn't exist.
                        // This could be an integrity violation. However, there are race conditions. For example, the block could
                        // have been created after we checked the underlying block store and before we checked the integrity data.
                        // The only way to be sure is to actually lock this block id and check again.
                        let block_info_guard =
                            self.integrity_data.lock_block_info(expected_block_id).await;
                        if let Some(block_info) = block_info_guard.value() {
                            let expected_to_exist = block_info.block_is_expected_to_exist();
                            let actually_exists = self
                                .underlying_block_store
                                .exists(&expected_block_id)
                                .await?;
                            if expected_to_exist && !actually_exists {
                                Ok(Some(expected_block_id))
                            } else {
                                // It actually was a race condition, ignore this false positive
                                Ok(None)
                            }
                        } else {
                            // It actually was a race condition, ignore this false positive
                            Ok(None)
                        }
                    });
                let missing_blocks: HashSet<BlockId> = missing_blocks.try_collect().await?;
                if !missing_blocks.is_empty() {
                    self._integrity_violation_detected(
                        IntegrityViolationError::MissingBlocks {
                            blocks: missing_blocks,
                        }
                        .into(),
                    )?;
                }
                Ok(futures::stream::iter(all_underlying_blocks.into_iter().map(Ok)).boxed())
            }
            MissingBlockIsIntegrityViolation::IsNotAViolation => {
                let all_underlying_blocks = self.underlying_block_store.all_blocks().await?;
                Ok(all_underlying_blocks)
            }
        }
    }
}

#[async_trait]
impl<B: BlockStoreDeleter + Sync + Send + Debug + AsyncDrop<Error = anyhow::Error>>
    BlockStoreDeleter for IntegrityBlockStore<B>
{
    async fn remove(&self, id: &BlockId) -> Result<RemoveResult> {
        let mut block_info_guard = self.integrity_data.lock_block_info(*id).await;
        let remove_result = self.underlying_block_store.remove(id).await?;
        // Only mark block as deleted after we know the operation succeeded
        block_info_guard
            .value_or_insert_with(|| {
                BlockInfo::new_unknown(MaybeClientId::ClientId(self.integrity_data.my_client_id()))
            })
            .mark_block_as_deleted();
        Ok(remove_result)
    }
}

create_block_data_wrapper!(BlockData);

#[async_trait]
impl<B: OptimizedBlockStoreWriter + Sync + Send + Debug + AsyncDrop<Error = anyhow::Error>>
    OptimizedBlockStoreWriter for IntegrityBlockStore<B>
{
    type BlockData = BlockData;

    fn allocate(size: usize) -> BlockData {
        let mut data = B::allocate(HEADER_SIZE + size).extract();
        data.shrink_to_subregion(HEADER_SIZE..);
        BlockData::new(data)
    }

    async fn try_create_optimized(&self, id: &BlockId, data: BlockData) -> Result<TryCreateResult> {
        let mut block_info_guard = self.integrity_data.lock_block_info(*id).await;
        let (version_transaction, data) = self._prepend_header(
            self.integrity_data.my_client_id(),
            block_info_guard.value_or_insert_with(|| {
                BlockInfo::new_unknown(MaybeClientId::ClientId(self.integrity_data.my_client_id()))
            }),
            id,
            data.extract(),
        );
        let result = self
            .underlying_block_store
            .try_create_optimized(id, B::BlockData::new(data))
            .await;
        match result {
            Ok(TryCreateResult::SuccessfullyCreated) => {
                version_transaction.commit();
                Ok(TryCreateResult::SuccessfullyCreated)
            }
            Ok(TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists) => {
                version_transaction.cancel();
                Ok(TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists)
            }
            Err(err) => {
                version_transaction.cancel();
                Err(err)
            }
        }
    }

    async fn store_optimized(&self, id: &BlockId, data: BlockData) -> Result<()> {
        let mut block_info_guard = self.integrity_data.lock_block_info(*id).await;
        let (version_transaction, data) = self._prepend_header(
            self.integrity_data.my_client_id(),
            block_info_guard.value_or_insert_with(|| {
                BlockInfo::new_unknown(MaybeClientId::ClientId(self.integrity_data.my_client_id()))
            }),
            id,
            data.extract(),
        );
        let result = self
            .underlying_block_store
            .store_optimized(id, B::BlockData::new(data))
            .await;
        match result {
            Ok(()) => {
                version_transaction.commit();
                Ok(())
            }
            Err(err) => {
                version_transaction.cancel();
                Err(err)
            }
        }
    }
}

impl<B: Send + Debug + AsyncDrop<Error = anyhow::Error>> IntegrityBlockStore<B> {
    fn _integrity_violation_detected(&self, reason: IntegrityViolationError) -> Result<()> {
        match self.config.allow_integrity_violations {
            AllowIntegrityViolations::AllowViolations => {
                warn!(
                    "Integrity violation (but integrity checks are disabled): {:?}",
                    reason,
                );
                Ok(())
            }
            AllowIntegrityViolations::DontAllowViolations => {
                self.integrity_data
                    .set_integrity_violation_in_previous_run();
                (*self.config.on_integrity_violation)(&reason);
                Err(reason.into())
            }
        }
    }

    fn _prepend_header<'a>(
        &self,
        my_client_id: ClientId,
        block_info: &'a mut BlockInfo,
        id: &BlockId,
        mut data: Data,
    ) -> (BlockVersionTransaction<'a>, Data) {
        let version_transaction = block_info.start_increment_version_transaction(my_client_id);

        data.grow_region_fail_if_reallocation_necessary(HEADER_SIZE, 0).expect("Tried to grow the data to contain the header in IntegrityBlockStore::_prepend_header");
        let mut view = block_layout::View::new(data);
        view.format_version_header_mut()
            .write(FORMAT_VERSION_HEADER);
        view.block_id_mut().copy_from_slice(id.data());
        view.last_update_client_id_mut().write(my_client_id);
        view.block_version_mut()
            .write(version_transaction.new_version());
        (version_transaction, view.into_storage())
    }

    fn _check_and_remove_header(
        &self,
        block_info: &mut BlockInfo,
        data: Data,
        expected_block_id: BlockId,
    ) -> Result<Data> {
        ensure!(
            data.len() >= block_layout::data::OFFSET,
            "Block size is {} but we need at least {} to store the block header",
            data.len(),
            block_layout::data::OFFSET
        );
        let view = block_layout::View::new(data);

        Self::_check_format_version_header(&view)?;
        self._check_id_header(&view, &expected_block_id)?;

        let last_update_client_id = view.last_update_client_id().read();
        let block_version = view.block_version().read();

        // TODO Use view.into_data().extract(), but that requires adding an IntoSubregion trait to binary-layout that we can implement for our Data class.
        let mut data = view.into_storage();
        data.shrink_to_subregion(block_layout::data::OFFSET..);

        // Only update version after we know the operation succeeded
        self._check_version_header(
            last_update_client_id,
            block_version,
            expected_block_id,
            block_info,
        )?;

        Ok(data)
    }

    fn _check_format_version_header(view: &block_layout::View<Data>) -> Result<()> {
        let format_version_header = view.format_version_header().read();
        if format_version_header != FORMAT_VERSION_HEADER {
            bail!("Wrong FORMAT_VERSION_HEADER of {:?}. Expected {:?}. Maybe it was created with a different major version of CryFS?", format_version_header, FORMAT_VERSION_HEADER);
        }
        Ok(())
    }

    fn _check_id_header(
        &self,
        view: &block_layout::View<Data>,
        expected_block_id: &BlockId,
    ) -> Result<()> {
        let block_id = BlockId::from_array(view.block_id());
        if block_id != *expected_block_id {
            self._integrity_violation_detected(
                IntegrityViolationError::WrongBlockId {
                    id_from_filename: *expected_block_id,
                    id_from_header: block_id,
                }
                .into(),
            )?;
        }
        Ok(())
    }

    fn _check_version_header(
        &self,
        last_update_client_id: ClientId,
        block_version: BlockVersion,
        block_id: BlockId,
        block_info: &mut BlockInfo,
    ) -> Result<()> {
        let update =
            block_info.check_and_update_version(last_update_client_id, block_id, block_version);
        match update {
            Ok(()) => Ok(()),
            Err(err) if err.is::<IntegrityViolationError>() => {
                // IntegrityViolationErrors are channeled through _integrity_violation_detected
                // so that we can silence them if integrity checking is disabled.
                self._integrity_violation_detected(
                    err.downcast::<IntegrityViolationError>()
                        .expect("We just checked the error type above but now it's different"),
                )
            }
            Err(err) => Err(err),
        }
    }
}

#[async_trait]
impl<B: Sync + Send + Debug + AsyncDrop<Error = anyhow::Error>> AsyncDrop
    for IntegrityBlockStore<B>
{
    type Error = anyhow::Error;
    async fn async_drop_impl(&mut self) -> Result<()> {
        self.underlying_block_store.async_drop().await?;
        self.integrity_data.async_drop().await?;
        Ok(())
    }
}

impl<B: BlockStore + OptimizedBlockStoreWriter + Sync + Send + Debug> BlockStore
    for IntegrityBlockStore<B>
{
}

#[cfg(test)]
mod generic_tests {
    use super::*;
    use crate::blockstore::low_level::inmemory::InMemoryBlockStore;
    use crate::blockstore::tests::Fixture;
    use crate::utils::async_drop::SyncDrop;
    use std::num::NonZeroU32;
    use tempdir::TempDir;

    use crate::instantiate_blockstore_tests;

    struct TestFixture<
        const ALLOW_INTEGRITY_VIOLATIONS: bool,
        const MISSING_BLOCK_IS_INTEGRITY_VIOLATION: bool,
    > {
        integrity_file_dir: TempDir,
    }
    #[async_trait]
    impl<
            const ALLOW_INTEGRITY_VIOLATIONS: bool,
            const MISSING_BLOCK_IS_INTEGRITY_VIOLATION: bool,
        > Fixture
        for TestFixture<ALLOW_INTEGRITY_VIOLATIONS, MISSING_BLOCK_IS_INTEGRITY_VIOLATION>
    {
        type ConcreteBlockStore = IntegrityBlockStore<InMemoryBlockStore>;
        fn new() -> Self {
            let integrity_file_dir = TempDir::new("IntegrityBlockStore").unwrap();
            Self { integrity_file_dir }
        }
        fn store(&mut self) -> SyncDrop<Self::ConcreteBlockStore> {
            SyncDrop::new(
                IntegrityBlockStore::new(
                    InMemoryBlockStore::new(),
                    self.integrity_file_dir
                        .path()
                        .join("integrity_file")
                        .to_path_buf(),
                    ClientId {
                        id: NonZeroU32::new(1).unwrap(),
                    },
                    IntegrityConfig {
                        allow_integrity_violations: if ALLOW_INTEGRITY_VIOLATIONS {AllowIntegrityViolations::AllowViolations} else {AllowIntegrityViolations::DontAllowViolations},
                        missing_block_is_integrity_violation: if MISSING_BLOCK_IS_INTEGRITY_VIOLATION { MissingBlockIsIntegrityViolation::IsAViolation} else {MissingBlockIsIntegrityViolation::IsNotAViolation},
                        on_integrity_violation: Box::new(|err| {
                            panic!("Integrity violation: {:?}", err)
                        }),
                    },
                )
                .unwrap(),
            )
        }
        async fn yield_fixture(&self, _store: &Self::ConcreteBlockStore) {}
    }

    instantiate_blockstore_tests!(TestFixture<false, false>, (flavor = "multi_thread"));

    mod multiclient {
        use super::*;
        instantiate_blockstore_tests!(TestFixture<false, false>, (flavor = "multi_thread"));
    }
    mod singleclient {
        use super::*;
        instantiate_blockstore_tests!(TestFixture<false, true>, (flavor = "multi_thread"));
    }
    mod multiclient_allow_integrity_violations {
        use super::*;
        instantiate_blockstore_tests!(TestFixture<true, false>, (flavor = "multi_thread"));
    }
    mod singleclient_allow_integrity_violations {
        use super::*;
        instantiate_blockstore_tests!(TestFixture<true, true>, (flavor = "multi_thread"));
    }

    #[test]
    fn test_block_size_from_physical_block_size() {
        let mut fixture = TestFixture::<false, false>::new();
        let store = fixture.store();
        let expected_overhead: u64 = HEADER_SIZE as u64;

        assert_eq!(
            0u64,
            store
                .block_size_from_physical_block_size(expected_overhead)
                .unwrap()
        );
        assert_eq!(
            20u64,
            store
                .block_size_from_physical_block_size(expected_overhead + 20u64)
                .unwrap()
        );
        assert!(store.block_size_from_physical_block_size(0).is_err());
    }
}

#[cfg(test)]
mod specialized_tests {
    #![allow(non_snake_case)]

    use super::integrity_data::testutils::{clientid, version};
    use super::*;
    use crate::blockstore::low_level::{
        inmemory::InMemoryBlockStore, shared::SharedBlockStore, BlockStoreWriter,
    };
    use crate::blockstore::tests::{blockid, data};
    use crate::utils::async_drop::SyncDrop;
    use common_macros::hash_set;
    use futures::future::BoxFuture;
    use std::num::NonZeroU32;
    use std::sync::{Arc, Mutex};
    use tempdir::TempDir;

    struct Fixture {
        underlying: SyncDrop<SharedBlockStore<InMemoryBlockStore>>,
        integrity_file_dir: TempDir,
        integrity_violation_triggered: Arc<Mutex<Option<IntegrityViolationError>>>,
    }

    impl Fixture {
        fn new() -> Self {
            Self {
                underlying: SyncDrop::new(SharedBlockStore::new(InMemoryBlockStore::new())),
                integrity_file_dir: TempDir::new("test").unwrap(),
                integrity_violation_triggered: Arc::new(Mutex::new(None)),
            }
        }

        fn store(
            &self,
            allow_integrity_violations: AllowIntegrityViolations,
            missing_block_is_integrity_violation: MissingBlockIsIntegrityViolation,
        ) -> SyncDrop<IntegrityBlockStore<SharedBlockStore<InMemoryBlockStore>>> {
            let integrity_violation_triggered = Arc::clone(&self.integrity_violation_triggered);
            SyncDrop::new(
                IntegrityBlockStore::new(
                    SharedBlockStore::clone(self.underlying.inner()),
                    self.integrity_file_dir
                        .path()
                        .join("integrity_file")
                        .to_path_buf(),
                    clientid(1),
                    IntegrityConfig {
                        allow_integrity_violations,
                        missing_block_is_integrity_violation,
                        on_integrity_violation: Box::new(move |err| {
                            *integrity_violation_triggered.lock().unwrap() = Some(err.clone());
                        }),
                    },
                )
                .unwrap(),
            )
        }

        async fn modify_block(
            &self,
            store: &IntegrityBlockStore<SharedBlockStore<InMemoryBlockStore>>,
            id: &BlockId,
        ) {
            let mut data = store.load(id).await.unwrap().unwrap();
            data[0] += 1;
            store.store(id, &data).await.unwrap();
        }

        async fn modify_block_id_header(&self, id: &BlockId, modified_block_id: &BlockId) {
            let mut data = self.underlying.load(id).await.unwrap().unwrap();
            let mut view = block_layout::View::new(&mut data);
            view.block_id_mut()
                .copy_from_slice(modified_block_id.data());
            self.underlying.store(id, &data).await.unwrap();
        }

        async fn load_base_block(&self, id: &BlockId) -> Data {
            self.underlying.load(id).await.unwrap().unwrap()
        }

        async fn rollback_base_block(&self, id: &BlockId, data: &[u8]) {
            self.underlying.store(id, data).await.unwrap()
        }

        async fn decrease_version_number(&self, id: &BlockId) {
            let mut data = self.underlying.load(id).await.unwrap().unwrap();
            let mut view = block_layout::View::new(&mut data);
            let old_version = view.block_version().read();
            assert!(
                old_version > BlockVersion { version: 1 },
                "Can't decrease the lowest allowed version number"
            );
            view.block_version_mut().write(BlockVersion {
                version: old_version.version - 1,
            });
            self.underlying.store(id, &data).await.unwrap();
        }

        async fn increase_version_number(&self, id: &BlockId) {
            let mut data = self.underlying.load(id).await.unwrap().unwrap();
            let mut view = block_layout::View::new(&mut data);
            let old_version = view.block_version().read();
            view.block_version_mut().write(BlockVersion {
                version: old_version.version + 1,
            });
            self.underlying.store(id, &data).await.unwrap();
        }

        async fn change_client_id(&self, id: &BlockId) {
            let mut data = self.underlying.load(id).await.unwrap().unwrap();
            let mut view = block_layout::View::new(&mut data);
            let old_client_id = view.last_update_client_id().read().id.get();
            view.last_update_client_id_mut().write(ClientId {
                id: NonZeroU32::new(old_client_id + 1).unwrap(),
            });
            self.underlying.store(id, &data).await.unwrap();
        }

        async fn delete_base_block(&self, id: &BlockId) {
            assert_eq!(
                RemoveResult::SuccessfullyRemoved,
                self.underlying.remove(id).await.unwrap()
            );
        }

        async fn insert_base_block(&self, id: &BlockId, data: &Data) {
            assert_eq!(
                TryCreateResult::SuccessfullyCreated,
                self.underlying.try_create(id, data).await.unwrap()
            );
        }

        fn assert_integrity_violation_triggered(&self, expected: &IntegrityViolationError) {
            assert_eq!(
                Some(expected),
                self.integrity_violation_triggered.lock().unwrap().as_ref(),
            );
        }

        fn assert_integrity_violation_didnt_trigger(&self) {
            assert_eq!(None, *self.integrity_violation_triggered.lock().unwrap());
        }
    }

    async fn create_block_return_key(
        store: &IntegrityBlockStore<SharedBlockStore<InMemoryBlockStore>>,
        data: &[u8],
    ) -> BlockId {
        let mut id_seed = 0;
        loop {
            id_seed += 1;
            let blockid = blockid(id_seed);
            if TryCreateResult::SuccessfullyCreated
                == store.try_create(&blockid, data).await.unwrap()
            {
                return blockid;
            }
        }
    }

    async fn remove_block(
        store: &IntegrityBlockStore<SharedBlockStore<InMemoryBlockStore>>,
        blockid: &BlockId,
    ) {
        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&blockid).await.unwrap()
        );
    }

    async fn list_all_blocks(
        store: &IntegrityBlockStore<SharedBlockStore<InMemoryBlockStore>>,
    ) -> Result<HashSet<BlockId>> {
        store.all_blocks().await?.try_collect().await
    }

    enum ExpectedIntegrityViolation {
        Always(IntegrityViolationError),
        OnlyIfMissingBlocksAreAnIntegrityViolation(IntegrityViolationError),
        NoIntegrityViolation,
    }

    /// Runs `setup`() and afterwards tests that running `action` will:
    /// - Trigger `expected_integrity_violation` if the block store is set up with allow_integrity_violations=false
    /// - Doesn't trigger an error if the block store is set up with allow_integrity_violations=true
    /// - Depending on the `expected_integrity_violation` being `Always` or `OnlyIfMissingBlocksAreAnIntegrityViolation`, tests that
    ///   missing_block_is_integrity_violation=true/false correctly triggers or doesn't trigger the violation.
    async fn run_test<Context, SetupFn, ActionFn>(
        setup: SetupFn,
        action: ActionFn,
        expected_integrity_violation: impl Fn(&Context) -> ExpectedIntegrityViolation,
    ) where
        for<'a> SetupFn: Fn(
            &'a Fixture,
            &'a IntegrityBlockStore<SharedBlockStore<InMemoryBlockStore>>,
        ) -> BoxFuture<'a, Context>,
        for<'a> ActionFn: Fn(
            &'a Context,
            &'a Fixture,
            &'a IntegrityBlockStore<SharedBlockStore<InMemoryBlockStore>>,
        ) -> BoxFuture<'a, Result<()>>,
    {
        let assert_error = |fixture: &Fixture, result: Result<()>, expected_error| {
            assert_eq!(
                expected_error,
                result
                    .unwrap_err()
                    .downcast::<IntegrityViolationError>()
                    .unwrap()
            );
            fixture.assert_integrity_violation_triggered(&expected_error);
        };
        let assert_no_error = |fixture: &Fixture, result: Result<()>| {
            result.unwrap();
            fixture.assert_integrity_violation_didnt_trigger();
        };
        // Test the integrity violation triggers when violations aren't allowed
        {
            let fixture = Fixture::new();
            let store = fixture.store(AllowIntegrityViolations::DontAllowViolations, MissingBlockIsIntegrityViolation::IsAViolation);
            let context = setup(&fixture, &store).await;
            fixture.assert_integrity_violation_didnt_trigger();
            let result = action(&context, &fixture, &store).await;
            match expected_integrity_violation(&context) {
                ExpectedIntegrityViolation::Always(expected_error) => {
                    assert_error(&fixture, result, expected_error)
                }
                ExpectedIntegrityViolation::OnlyIfMissingBlocksAreAnIntegrityViolation(
                    expected_error,
                ) => assert_error(&fixture, result, expected_error),
                ExpectedIntegrityViolation::NoIntegrityViolation => {
                    assert_no_error(&fixture, result)
                }
            }
        }
        {
            let fixture = Fixture::new();
            let store = fixture.store(AllowIntegrityViolations::DontAllowViolations, MissingBlockIsIntegrityViolation::IsNotAViolation);
            let context = setup(&fixture, &store).await;
            fixture.assert_integrity_violation_didnt_trigger();
            let result = action(&context, &fixture, &store).await;
            match expected_integrity_violation(&context) {
                ExpectedIntegrityViolation::Always(expected_error) => {
                    assert_error(&fixture, result, expected_error)
                }
                ExpectedIntegrityViolation::OnlyIfMissingBlocksAreAnIntegrityViolation(
                    _expected_error,
                ) => assert_no_error(&fixture, result),
                ExpectedIntegrityViolation::NoIntegrityViolation => {
                    assert_no_error(&fixture, result)
                }
            }
        }

        // Test the integrity violation doesn't trigger when violations are allowed
        {
            let fixture = Fixture::new();
            let store = fixture.store(AllowIntegrityViolations::AllowViolations, MissingBlockIsIntegrityViolation::IsAViolation);
            let context = setup(&fixture, &store).await;
            fixture.assert_integrity_violation_didnt_trigger();
            let result = action(&context, &fixture, &store).await;
            assert_no_error(&fixture, result);
        }
        {
            let fixture = Fixture::new();
            let store = fixture.store(AllowIntegrityViolations::AllowViolations, MissingBlockIsIntegrityViolation::IsNotAViolation);
            let context = setup(&fixture, &store).await;
            let result = action(&context, &fixture, &store).await;
            assert_no_error(&fixture, result);
        }
    }

    #[tokio::test]
    async fn rolling_back_block() {
        run_test(
            |fixture, store| {
                Box::pin(async move {
                    let blockid = create_block_return_key(&store, &data(1024, 1)).await;
                    let old_base_block = fixture.load_base_block(&blockid).await;
                    fixture.modify_block(&store, &blockid).await;
                    fixture.rollback_base_block(&blockid, &old_base_block).await;
                    blockid
                })
            },
            |blockid, _fixture, store| {
                Box::pin(async move {
                    let loaded = store.load(&blockid).await?;
                    assert!(loaded.is_some());
                    Ok(())
                })
            },
            |&block| {
                ExpectedIntegrityViolation::Always(IntegrityViolationError::RollBack {
                    block,
                    from_client: MaybeClientId::ClientId(clientid(1)),
                    to_client: clientid(1),
                    from_client_last_seen_version: Some(version(2)),
                    to_client_last_seen_version: version(2),
                    actual_version: version(1),
                })
            },
        )
        .await;
    }

    #[tokio::test]
    async fn decreasing_version_number_for_same_client() {
        run_test(
            |fixture, store| {
                Box::pin(async move {
                    let blockid = create_block_return_key(&store, &data(1024, 1)).await;
                    fixture.modify_block(&store, &blockid).await;
                    fixture.decrease_version_number(&blockid).await;
                    blockid
                })
            },
            |blockid, _fixture, store| {
                Box::pin(async move {
                    let loaded = store.load(&blockid).await?;
                    assert!(loaded.is_some());
                    Ok(())
                })
            },
            |&block| {
                ExpectedIntegrityViolation::Always(IntegrityViolationError::RollBack {
                    block,
                    from_client: MaybeClientId::ClientId(clientid(1)),
                    to_client: clientid(1),
                    from_client_last_seen_version: Some(version(2)),
                    to_client_last_seen_version: version(2),
                    actual_version: version(1),
                })
            },
        )
        .await;
    }

    #[tokio::test]
    async fn decreasing_version_number_but_switching_to_different_client_for_which_the_version_is_increasing(
    ) {
        run_test(
            |fixture, store| {
                Box::pin(async move {
                    let blockid = create_block_return_key(&store, &data(1024, 1)).await;
                    fixture.modify_block(&store, &blockid).await;
                    let old_block_our_client = fixture.load_base_block(&blockid).await;
                    // Simulate a valid change by a different client by changing the client id
                    fixture.change_client_id(&blockid).await;
                    fixture.decrease_version_number(&blockid).await;
                    store.load(&blockid).await.unwrap().unwrap();
                    let old_block_other_client = fixture.load_base_block(&blockid).await;
                    // Switch back to our client
                    fixture
                        .rollback_base_block(&blockid, &old_block_our_client)
                        .await;
                    fixture.increase_version_number(&blockid).await;
                    fixture.increase_version_number(&blockid).await;
                    fixture.increase_version_number(&blockid).await;
                    store.load(&blockid).await.unwrap().unwrap();
                    // Switch back to previous client which is on a lower version number
                    fixture
                        .rollback_base_block(&blockid, &old_block_other_client)
                        .await;
                    fixture.increase_version_number(&blockid).await;

                    blockid
                })
            },
            |blockid, _fixture, store| {
                Box::pin(async move {
                    let loaded = store.load(&blockid).await?;
                    assert!(loaded.is_some());
                    Ok(())
                })
            },
            |&_block| ExpectedIntegrityViolation::NoIntegrityViolation,
        )
        .await;
    }

    #[tokio::test]
    async fn decreasing_version_number_but_switching_to_new_client() {
        run_test(
            |fixture, store| {
                Box::pin(async move {
                    let blockid = create_block_return_key(&store, &data(1024, 1)).await;
                    fixture.modify_block(&store, &blockid).await;
                    // Simulate a valid change by a different client by changing the client id
                    fixture.change_client_id(&blockid).await;
                    fixture.decrease_version_number(&blockid).await;
                    blockid
                })
            },
            |blockid, _fixture, store| {
                Box::pin(async move {
                    let loaded = store.load(&blockid).await?;
                    assert!(loaded.is_some());
                    Ok(())
                })
            },
            |&_block| ExpectedIntegrityViolation::NoIntegrityViolation,
        )
        .await;
    }

    #[tokio::test]
    async fn rolling_back_to_previous_client_without_increasing_version() {
        run_test(
            |fixture, store| {
                Box::pin(async move {
                    let blockid = create_block_return_key(&store, &data(1024, 1)).await;
                    // Increase the version number
                    fixture.increase_version_number(&blockid).await;
                    fixture.increase_version_number(&blockid).await;
                    fixture.increase_version_number(&blockid).await;
                    store.load(&blockid).await.unwrap().unwrap();

                    let old_base_block = fixture.load_base_block(&blockid).await;

                    // Fake a modification by a different client with lower version numbers
                    fixture.decrease_version_number(&blockid).await;
                    fixture.decrease_version_number(&blockid).await;
                    fixture.change_client_id(&blockid).await;
                    store.load(&blockid).await.unwrap().unwrap();

                    // Rollback to old client without increasing its version number
                    fixture.rollback_base_block(&blockid, &old_base_block).await;
                    blockid
                })
            },
            |blockid, _fixture, store| {
                Box::pin(async move {
                    let loaded = store.load(&blockid).await?;
                    assert_eq!(Some(data(1024, 1)), loaded);
                    Ok(())
                })
            },
            |&block| {
                ExpectedIntegrityViolation::Always(IntegrityViolationError::RollBack {
                    block,
                    from_client: MaybeClientId::ClientId(clientid(2)),
                    to_client: clientid(1),
                    from_client_last_seen_version: Some(version(2)),
                    to_client_last_seen_version: version(4),
                    actual_version: version(4),
                })
            },
        )
        .await;
    }

    #[tokio::test]
    async fn reintroducing_deleted_block_with_same_version_number() {
        run_test(
            |fixture, store| {
                Box::pin(async move {
                    let blockid = create_block_return_key(&store, &data(1024, 1)).await;
                    let old_base_block = fixture.load_base_block(&blockid).await;
                    remove_block(&store, &blockid).await;
                    fixture.insert_base_block(&blockid, &old_base_block).await;
                    blockid
                })
            },
            |blockid, _fixture, store| {
                Box::pin(async move {
                    let loaded = store.load(&blockid).await?;
                    assert_eq!(Some(data(1024, 1)), loaded);
                    Ok(())
                })
            },
            |&block| {
                ExpectedIntegrityViolation::Always(IntegrityViolationError::RollBack {
                    block,
                    from_client: MaybeClientId::BlockWasDeleted,
                    to_client: clientid(1),
                    from_client_last_seen_version: None,
                    to_client_last_seen_version: version(1),
                    actual_version: version(1),
                })
            },
        )
        .await;
    }

    #[tokio::test]
    async fn reintroducing_deleted_block_with_increased_version_number() {
        run_test(
            |fixture, store| {
                Box::pin(async move {
                    let blockid = create_block_return_key(&store, &data(1024, 1)).await;
                    let old_base_block = fixture.load_base_block(&blockid).await;
                    remove_block(&store, &blockid).await;
                    fixture.insert_base_block(&blockid, &old_base_block).await;
                    fixture.increase_version_number(&blockid).await;
                    blockid
                })
            },
            |blockid, _fixture, store| {
                Box::pin(async move {
                    let loaded = store.load(&blockid).await?;
                    assert_eq!(Some(data(1024, 1)), loaded);
                    Ok(())
                })
            },
            |&_block| ExpectedIntegrityViolation::NoIntegrityViolation,
        )
        .await;
    }

    #[tokio::test]
    async fn loading_missing_block() {
        run_test(
            |fixture, store| {
                Box::pin(async move {
                    let blockid = create_block_return_key(&store, &data(1024, 1)).await;
                    fixture.delete_base_block(&blockid).await;
                    blockid
                })
            },
            |blockid, _fixture, store| {
                Box::pin(async move {
                    let loaded = store.load(&blockid).await?;
                    assert_eq!(None, loaded);
                    Ok(())
                })
            },
            |&block| {
                ExpectedIntegrityViolation::OnlyIfMissingBlocksAreAnIntegrityViolation(
                    IntegrityViolationError::MissingBlock { block },
                )
            },
        )
        .await;
    }

    #[tokio::test]
    async fn listing_blocks_with_one_missing_block() {
        run_test(
            |fixture, store| {
                Box::pin(async move {
                    let blockid = create_block_return_key(&store, &data(1024, 1)).await;
                    fixture.delete_base_block(&blockid).await;
                    blockid
                })
            },
            |_blockid, _fixture, store| {
                Box::pin(async move {
                    let all_blocks = list_all_blocks(&store).await?;
                    assert_eq!(hash_set! {}, all_blocks);
                    Ok(())
                })
            },
            |&block| {
                ExpectedIntegrityViolation::OnlyIfMissingBlocksAreAnIntegrityViolation(
                    IntegrityViolationError::MissingBlocks {
                        blocks: hash_set! {block},
                    },
                )
            },
        )
        .await;
    }

    #[tokio::test]
    async fn listing_blocks_with_multiple_missing_blocks() {
        run_test(
            |fixture, store| {
                Box::pin(async move {
                    let blockid1 = create_block_return_key(&store, &data(1024, 1)).await;
                    let blockid2 = create_block_return_key(&store, &data(1024, 2)).await;
                    let blockid3 = create_block_return_key(&store, &data(1024, 3)).await;
                    let blockid4 = create_block_return_key(&store, &data(1024, 4)).await;
                    fixture.delete_base_block(&blockid2).await;
                    fixture.delete_base_block(&blockid3).await;
                    fixture.delete_base_block(&blockid4).await;
                    (blockid1, blockid2, blockid3, blockid4)
                })
            },
            |(blockid1, _blockid2, _blockid3, _blockid4), _fixture, store| {
                Box::pin(async move {
                    let all_blocks = list_all_blocks(&store).await?;
                    assert_eq!(hash_set! {*blockid1}, all_blocks);
                    Ok(())
                })
            },
            |&(_blockid1, blockid2, blockid3, blockid4)| {
                ExpectedIntegrityViolation::OnlyIfMissingBlocksAreAnIntegrityViolation(
                    IntegrityViolationError::MissingBlocks {
                        blocks: hash_set! {blockid2, blockid3, blockid4},
                    },
                )
            },
        )
        .await;
    }

    #[tokio::test]
    async fn load_block_with_wrong_block_id() {
        run_test(
            |fixture, store| {
                Box::pin(async move {
                    let blockid = create_block_return_key(&store, &data(1024, 1)).await;
                    let mut modified_block_id = blockid.data().clone();
                    modified_block_id[0] += 1;
                    let modified_block_id = BlockId::from_slice(&modified_block_id).unwrap();
                    fixture
                        .modify_block_id_header(&blockid, &modified_block_id)
                        .await;
                    (blockid, modified_block_id)
                })
            },
            |&(blockid, _modified_block_id), _fixture, store| {
                Box::pin(async move {
                    let loaded = store.load(&blockid).await?;
                    assert_eq!(Some(data(1024, 1)), loaded);
                    Ok(())
                })
            },
            |&(blockid, modified_blockid)| {
                ExpectedIntegrityViolation::Always(IntegrityViolationError::WrongBlockId {
                    id_from_filename: blockid,
                    id_from_header: modified_blockid,
                })
            },
        )
        .await;
    }
}
