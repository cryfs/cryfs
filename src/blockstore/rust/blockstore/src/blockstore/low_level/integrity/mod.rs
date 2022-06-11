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

pub struct IntegrityConfig {
    pub allow_integrity_violations: bool,
    pub missing_block_is_integrity_violation: bool,
    pub on_integrity_violation: Box<dyn Sync + Send + Fn(&IntegrityViolationError)>,
}

/// Warning: This is thread safe in a sense that it can handle calls about **different** block ids at the same time
/// but it cannot handle calls about **the same** block id at the same time. That would cause race conditions
/// in `integrity_data` and possibly other places. Only use this with something on top (e.g. `LockingBlockStore`)
/// ensuring that there are no concurrent calls about the same block id.
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
            if config.allow_integrity_violations {
                warn!("Integrity violation in previous run (but integrity checks are disabled)");
            } else {
                return Err(IntegrityViolationError::IntegrityViolationInPreviousRun {
                    integrity_file_path: integrity_file_path.clone(),
                }
                .into());
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
                if self.config.missing_block_is_integrity_violation {
                    if let Some(block_info) = block_info_guard.value() {
                        if block_info.block_is_expected_to_exist() {
                            self._integrity_violation_detected(
                                IntegrityViolationError::MissingBlock { block: *block_id }.into(),
                            )?;
                        }
                    }
                }
                Ok(None)
            }
            Some(loaded) => {
                let block_info = block_info_guard.value_or_insert_with(|| {
                    BlockInfo::new_unknown(self.integrity_data.my_client_id())
                });
                let data = self._check_and_remove_header(block_info, loaded, block_id)?;
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
        if self.config.missing_block_is_integrity_violation {
            let all_underlying_blocks = {
                let blocks = self.underlying_block_store.all_blocks().await?;
                let blocks: Vec<BlockId> = blocks.try_collect().await?;
                blocks
            };
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
        } else {
            let all_underlying_blocks = self.underlying_block_store.all_blocks().await?;
            Ok(all_underlying_blocks)
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
            .value_or_insert_with(|| BlockInfo::new_unknown(self.integrity_data.my_client_id()))
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
                BlockInfo::new_unknown(self.integrity_data.my_client_id())
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
                BlockInfo::new_unknown(self.integrity_data.my_client_id())
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
        if self.config.allow_integrity_violations {
            warn!(
                "Integrity violation (but integrity checks are disabled): {:?}",
                reason,
            );
            Ok(())
        } else {
            self.integrity_data
                .set_integrity_violation_in_previous_run();
            (*self.config.on_integrity_violation)(&reason);
            Err(reason.into())
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
        expected_block_id: &BlockId,
    ) -> Result<Data> {
        ensure!(
            data.len() >= block_layout::data::OFFSET,
            "Block size is {} but we need at least {} to store the block header",
            data.len(),
            block_layout::data::OFFSET
        );
        let view = block_layout::View::new(data);
        let format_version_header = view.format_version_header().read();
        if format_version_header != FORMAT_VERSION_HEADER {
            bail!("Wrong FORMAT_VERSION_HEADER of {:?}. Expected {:?}. Maybe it was created with a different major version of CryFS?", format_version_header, FORMAT_VERSION_HEADER);
        }
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
        let last_update_client_id = view.last_update_client_id().read();
        let block_version = view.block_version().read();

        // TODO Use view.into_data().extract(), but that requires adding an IntoSubregion trait to binary-layout that we can implement for our Data class.
        let mut data = view.into_storage();
        data.shrink_to_subregion(block_layout::data::OFFSET..);

        // Only update version after we know the operation succeeded
        let update =
            block_info.check_and_update_version(last_update_client_id, block_id, block_version);
        match update {
            Ok(()) => (),
            Err(err) if err.is::<IntegrityViolationError>() => {
                // IntegrityViolationErrors are channeled through _integrity_violation_detected
                // so that we can silence them if integrity checking is disabled.
                self._integrity_violation_detected(
                    err.downcast::<IntegrityViolationError>()
                        .expect("We just checked the error type above but now it's different"),
                )?;
            }
            Err(err) => Err(err)?,
        }

        Ok(data)
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
mod tests {
    use super::*;
    use crate::blockstore::low_level::inmemory::InMemoryBlockStore;
    use crate::utils::async_drop::SyncDrop;
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
        > crate::blockstore::tests::Fixture
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
                    ClientId { id: 1 },
                    IntegrityConfig {
                        allow_integrity_violations: ALLOW_INTEGRITY_VIOLATIONS,
                        missing_block_is_integrity_violation: MISSING_BLOCK_IS_INTEGRITY_VIOLATION,
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
}
