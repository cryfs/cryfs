use anyhow::{anyhow, bail, Context, Error, Result};
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt, TryStreamExt};
use std::fmt::{self, Debug};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use tokio::fs::DirEntry;
use tokio_stream::wrappers::ReadDirStream;

use crate::{
    low_level::{
        interface::block_data::IBlockData, BlockStore, BlockStoreDeleter, BlockStoreReader,
        OptimizedBlockStoreWriter,
    },
    utils::{RemoveResult, TryCreateResult},
    BlockId, BLOCKID_LEN,
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    data::Data,
    path::path_join,
};

mod sysinfo;

const FORMAT_VERSION_HEADER_PREFIX: &[u8] = b"cryfs;block;";
const FORMAT_VERSION_HEADER: &[u8] = b"cryfs;block;0\0";

const PREFIX_LEN: usize = 3;
const NONPREFIX_LEN: usize = 2 * BLOCKID_LEN - PREFIX_LEN;

pub struct OnDiskBlockStore {
    basedir: PathBuf,
}

impl OnDiskBlockStore {
    pub fn new(basedir: PathBuf) -> AsyncDropGuard<Self> {
        // TODO When we're creating a new file system, we should make sure that all the subfolders exist.
        //      Because we don't remove the folders when blocks get removed, otherwise a used file system looks different from a new one.
        AsyncDropGuard::new(Self { basedir })
    }
}

#[async_trait]
impl BlockStoreReader for OnDiskBlockStore {
    async fn exists(&self, id: &BlockId) -> Result<bool> {
        let path = self._block_path(id);
        match tokio::fs::metadata(path).await {
            Ok(_) => Ok(true),
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    Ok(false)
                } else {
                    Err(err.into())
                }
            }
        }
    }

    async fn load(&self, id: &BlockId) -> Result<Option<Data>> {
        let path = self._block_path(id);
        match tokio::fs::read(&path).await {
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                // File doesn't exist. Return None. This is not an error.
                Ok(None)
            }
            Err(err) => {
                Err(err).with_context(|| format!("Failed to open block file at {}", path.display()))
            }
            Ok(file_content) => {
                let block_content =
                    _check_and_remove_header(file_content.into()).with_context(|| {
                        format!(
                            "Failed to parse file contents of block at {}",
                            path.display()
                        )
                    })?;
                Ok(Some(block_content))
            }
        }
    }

    async fn num_blocks(&self) -> Result<u64> {
        Ok(self
            ._all_block_files()
            .await?
            .try_fold(0, |acc, _blockfile| futures::future::ready(Ok(acc + 1)))
            .await?)
    }

    fn estimate_num_free_bytes(&self) -> Result<u64> {
        sysinfo::get_available_disk_space(&self.basedir)
    }

    fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64> {
        block_size.checked_sub(FORMAT_VERSION_HEADER.len() as u64)
            .with_context(|| anyhow!("Physical block size of {} is too small to store the FORMAT_VERSION_HEADER. Must be at least {}.", block_size, FORMAT_VERSION_HEADER.len()))
    }

    async fn all_blocks(&self) -> Result<Pin<Box<dyn Stream<Item = Result<BlockId>> + Send>>> {
        Ok(self
            ._all_block_files()
            .await?
            .map_ok(|entry| _blockid_from_filepath(&entry.path()))
            .boxed())
    }
}

#[async_trait]
impl BlockStoreDeleter for OnDiskBlockStore {
    async fn remove(&self, id: &BlockId) -> Result<RemoveResult> {
        let path = self._block_path(id);
        match tokio::fs::remove_file(path).await {
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                // File doesn't exist. Return false. This is not an error.
                Ok(RemoveResult::NotRemovedBecauseItDoesntExist)
            }
            Ok(()) => Ok(RemoveResult::SuccessfullyRemoved),
            Err(err) => Err(err.into()),
        }
    }
}

create_block_data_wrapper!(BlockData);

#[async_trait]
impl OptimizedBlockStoreWriter for OnDiskBlockStore {
    type BlockData = BlockData;

    fn allocate(size: usize) -> BlockData {
        let mut data = Data::from(vec![0; FORMAT_VERSION_HEADER.len() + size]);
        data.shrink_to_subregion(FORMAT_VERSION_HEADER.len()..);
        BlockData::new(data)
    }

    async fn try_create_optimized(&self, id: &BlockId, data: BlockData) -> Result<TryCreateResult> {
        let path = self._block_path(id);
        if path_exists(&path).await? {
            Ok(TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists)
        } else {
            _store(&path, data).await?;
            Ok(TryCreateResult::SuccessfullyCreated)
        }
    }

    async fn store_optimized(&self, id: &BlockId, data: BlockData) -> Result<()> {
        let path = self._block_path(id);
        _store(&path, data).await
    }
}

async fn path_exists(path: &Path) -> Result<bool> {
    match tokio::fs::metadata(path).await {
        Ok(_) => Ok(true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err.into()),
    }
}

impl Debug for OnDiskBlockStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OnDiskBlockStore")
    }
}

#[async_trait]
impl AsyncDrop for OnDiskBlockStore {
    type Error = anyhow::Error;
    async fn async_drop_impl(&mut self) -> Result<()> {
        Ok(())
    }
}

impl BlockStore for OnDiskBlockStore {}

impl OnDiskBlockStore {
    fn _block_path(&self, block_id: &BlockId) -> PathBuf {
        _block_path(self.basedir.as_path(), block_id)
    }

    async fn _all_block_files(&self) -> Result<impl Stream<Item = Result<DirEntry>>> {
        Ok(
            ReadDirStream::new(tokio::fs::read_dir(&self.basedir).await?)
                .map_err(Error::from)
                .try_filter_map(|subdir| async move {
                    if subdir.metadata().await?.is_dir()
                        && subdir.file_name().len() == PREFIX_LEN
                        && subdir
                            .file_name()
                            .to_str()
                            .ok_or_else(|| {
                                anyhow!(
                                    "Invalid UTF-8 in path in base directory: {}",
                                    subdir.path().display()
                                )
                            })?
                            .chars()
                            .all(|c| _is_allowed_blockid_character(c))
                    {
                        // Return a stream with all the entries in the subdirectory
                        let entries_stream =
                            ReadDirStream::new(tokio::fs::read_dir(subdir.path()).await?)
                                .map_err(Error::from);
                        Ok(Some(entries_stream))
                    } else {
                        // Dir entry is not a subdirectory with blocks, skip it.
                        Ok(None)
                    }
                })
                .try_flatten()
                .try_filter_map(|blockfile| async move {
                    if blockfile.file_name().len() == NONPREFIX_LEN
                        && blockfile
                            .file_name()
                            .to_str()
                            .ok_or_else(|| {
                                anyhow!(
                                    "Invalid UTF-8 in path in base directory: {}",
                                    blockfile.path().display()
                                )
                            })?
                            .chars()
                            .all(|c| _is_allowed_blockid_character(c))
                    {
                        Ok(Some(blockfile))
                    } else {
                        // File doesn't match the blockfile pattern, skip it.
                        Ok(None)
                    }
                }),
        )
    }
}

// This function panics if the path doesn't match the correct pattern.
// Only call this on paths you know are matching the pattern!
fn _blockid_from_filepath(path: &Path) -> BlockId {
    let blockid_prefix = path
        .parent()
        .unwrap_or_else(|| {
            panic!(
                "Block file path `{}` should have a parent component",
                path.display()
            )
        })
        .file_name()
        .unwrap_or_else(|| {
            panic!(
                "Block file path `{}` should have a valid parent component",
                path.display()
            )
        })
        .to_str()
        .unwrap_or_else(|| panic!("Block file path `{}` should be valid UTF-8", path.display()));
    let blockid_nonprefix = path
        .file_name()
        .unwrap_or_else(|| {
            panic!(
                "Block file path `{}` should have a valid last component",
                path.display()
            )
        })
        .to_str()
        .unwrap_or_else(|| panic!("Block file path `{}` should be valid UTF-8", path.display()));
    assert_eq!(
        PREFIX_LEN,
        blockid_prefix.len(),
        "Block file path should have a prefix len of {} but was {:?}",
        PREFIX_LEN,
        blockid_prefix
    );
    assert_eq!(
        NONPREFIX_LEN,
        blockid_nonprefix.len(),
        "Block file path should have a nonprefix len of {} but was {:?}",
        NONPREFIX_LEN,
        blockid_nonprefix
    );
    let mut blockid = String::with_capacity(2 * BLOCKID_LEN);
    blockid.push_str(blockid_prefix);
    blockid.push_str(blockid_nonprefix);
    assert_eq!(2 * BLOCKID_LEN, blockid.len());
    BlockId::from_hex(&blockid)
        .with_context(|| format!("Block file path `{}` cannot be parsed as hex", blockid))
        .unwrap()
}

fn _check_and_remove_header(mut data: Data) -> Result<Data> {
    if !data.starts_with(FORMAT_VERSION_HEADER) {
        if data.starts_with(FORMAT_VERSION_HEADER_PREFIX) {
            bail!("This block is not supported yet. Maybe it was created with a newer version of CryFS? Block: {}", base64::encode(data));
        } else {
            bail!("This is not a valid block: {}", base64::encode(data));
        }
    }
    data.shrink_to_subregion(FORMAT_VERSION_HEADER.len()..);
    Ok(data)
}

// TODO Test
async fn _create_dir_if_doesnt_exist(dir: &Path) -> Result<()> {
    match tokio::fs::create_dir(dir).await {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
            // This is ok, we only want to create the directory if it doesn't exist yet
            Ok(())
        }
        Err(err) => Err(err.into()),
    }
}

fn _is_allowed_blockid_character(c: char) -> bool {
    (c >= '0' && c <= '9') || (c >= 'A' && c <= 'F')
}

async fn _store(path: &Path, data: BlockData) -> Result<()> {
    _create_dir_if_doesnt_exist(
        path.parent()
            .expect("Block file path should have a parent directory"),
    )
    .await
    .with_context(|| {
        format!(
            "Failed to create parent directory for block file at {}",
            path.display()
        )
    })?;
    let mut data = data.extract();
    data.grow_region_fail_if_reallocation_necessary(FORMAT_VERSION_HEADER.len(), 0)
        .expect("Tried to grow data region to store in OnDiskBlockStore::_store");
    // TODO Use binary-layout here?
    data.as_mut()[..FORMAT_VERSION_HEADER.len()].copy_from_slice(FORMAT_VERSION_HEADER);
    tokio::fs::write(&path, data)
        .await
        .with_context(|| format!("Failed to write to block file at {}", path.display()))
}

fn _block_path(basedir: &Path, block_id: &BlockId) -> PathBuf {
    let block_id_str = block_id.to_hex();
    assert!(
        block_id_str
            .chars()
            .all(|c| _is_allowed_blockid_character(c)),
        "Created invalid block_id_str"
    );
    path_join(&[
        basedir,
        Path::new(&block_id_str[..PREFIX_LEN]),
        Path::new(&block_id_str[PREFIX_LEN..]),
    ])
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;
    use crate::instantiate_blockstore_tests;
    use crate::low_level::BlockStoreWriter;
    use crate::tests::{blockid, data, Fixture};
    use cryfs_utils::async_drop::SyncDrop;
    use tempdir::TempDir;

    struct TestFixture {
        basedir: TempDir,
    }
    #[async_trait]
    impl Fixture for TestFixture {
        type ConcreteBlockStore = OnDiskBlockStore;
        fn new() -> Self {
            let basedir = TempDir::new("OnDiskBlockStoreTest").unwrap();
            Self { basedir }
        }
        async fn store(&mut self) -> SyncDrop<OnDiskBlockStore> {
            SyncDrop::new(OnDiskBlockStore::new(self.basedir.path().to_path_buf()))
        }
        async fn yield_fixture(&self, _store: &Self::ConcreteBlockStore) {}
    }

    instantiate_blockstore_tests!(TestFixture, (flavor = "multi_thread"));

    #[tokio::test]
    async fn test_block_path() {
        let mut block_store = OnDiskBlockStore::new(PathBuf::from("/base/path"));
        assert_eq!(
            Path::new("/base/path/2AC/9C78D80937AD50852C50BD3F1F982"),
            block_store._block_path(
                &BlockId::from_slice(&hex::decode("2AC9C78D80937AD50852C50BD3F1F982").unwrap())
                    .unwrap()
            )
        );
        block_store.async_drop().await.unwrap();
    }

    #[test]
    fn test_prefix() {
        assert!(FORMAT_VERSION_HEADER.starts_with(FORMAT_VERSION_HEADER_PREFIX));
    }

    #[tokio::test]
    async fn test_block_size_from_physical_block_size() {
        let mut fixture = TestFixture::new();
        let store = fixture.store().await;
        let expected_overhead: u64 = FORMAT_VERSION_HEADER.len() as u64;

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

    fn _get_block_file_size(basedir: &Path, block_id: &BlockId) -> u64 {
        _block_path(basedir, block_id).metadata().unwrap().len()
    }

    fn _block_file_exists(basedir: &Path, block_id: &BlockId) -> bool {
        let path = _block_path(basedir, block_id);
        assert!(
            !path.exists() || path.is_file(),
            "If it exists, then it must be a file"
        );
        path.is_file()
    }

    #[tokio::test]
    async fn test_ondisk_block_size() {
        let mut fixture = TestFixture::new();
        let store = fixture.store().await;

        store.store(&blockid(0), &[]).await.unwrap();
        assert_eq!(
            0,
            store
                .block_size_from_physical_block_size(_get_block_file_size(
                    fixture.basedir.path(),
                    &blockid(0)
                ))
                .unwrap()
        );

        store.store(&blockid(1), &data(500, 0)).await.unwrap();
        assert_eq!(
            500,
            store
                .block_size_from_physical_block_size(_get_block_file_size(
                    fixture.basedir.path(),
                    &blockid(1)
                ))
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_whenStoringBlock_thenBlockFileExists() {
        let mut fixture = TestFixture::new();
        let store = fixture.store().await;

        assert!(!_block_file_exists(fixture.basedir.path(), &blockid(0)));
        store.store(&blockid(0), &[]).await.unwrap();
        assert!(_block_file_exists(fixture.basedir.path(), &blockid(0)));

        assert!(!_block_file_exists(fixture.basedir.path(), &blockid(1)));
        store.store(&blockid(1), &data(500, 0)).await.unwrap();
        assert!(_block_file_exists(fixture.basedir.path(), &blockid(1)));
    }

    #[tokio::test]
    async fn test_whenRemovingBlock_thenBlockFileDoesntExist() {
        let mut fixture = TestFixture::new();
        let store = fixture.store().await;

        store.store(&blockid(0), &[]).await.unwrap();
        assert!(_block_file_exists(fixture.basedir.path(), &blockid(0)));
        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&blockid(0)).await.unwrap()
        );
        assert!(!_block_file_exists(fixture.basedir.path(), &blockid(0)));

        store.store(&blockid(1), &data(500, 0)).await.unwrap();
        assert!(_block_file_exists(fixture.basedir.path(), &blockid(1)));
        assert_eq!(
            RemoveResult::SuccessfullyRemoved,
            store.remove(&blockid(1)).await.unwrap()
        );
        assert!(!_block_file_exists(fixture.basedir.path(), &blockid(1)));
    }
}
