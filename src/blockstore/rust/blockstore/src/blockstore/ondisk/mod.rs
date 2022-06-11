use anyhow::{anyhow, bail, Context, Result};
use std::fs::{DirEntry, File};
use std::io::{IoSlice, Write};
use std::path::{Path, PathBuf};

use super::{BlockId, BlockStore, BlockStoreReader, BlockStoreWriter, BLOCKID_LEN};

mod sysinfo;

const FORMAT_VERSION_HEADER_PREFIX: &[u8] = b"cryfs;block;";
const FORMAT_VERSION_HEADER: &[u8] = b"cryfs;block;0\0";

const PREFIX_LEN: usize = 3;
const NONPREFIX_LEN: usize = 2 * BLOCKID_LEN - PREFIX_LEN;

pub struct OnDiskBlockStore {
    basedir: PathBuf,
}

impl OnDiskBlockStore {
    pub fn new(basedir: PathBuf) -> Self {
        Self { basedir }
    }
}

impl BlockStoreReader for OnDiskBlockStore {
    fn load(&self, id: &BlockId) -> Result<Option<Vec<u8>>> {
        let path = self._block_path(id);
        match std::fs::read(&path) {
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                // File doesn't exist. Return None. This is not an error.
                Ok(None)
            }
            Err(err) => {
                Err(err).with_context(|| format!("Failed to open block file at {}", path.display()))
            }
            Ok(file_content) => {
                let block_content = _check_and_remove_header(&file_content)
                    .with_context(|| {
                        format!(
                            "Failed to parse file contents of block at {}",
                            path.display()
                        )
                    })?
                    .to_vec();
                // TODO Avoid last .to_vec() by introducing a Data class that can hold the whole file contents but refer to subranges of the data?
                Ok(Some(block_content))
            }
        }
    }

    fn num_blocks(&self) -> Result<u64> {
        let mut count = 0;
        self._for_each_block_file(&mut |_| -> Result<()> {
            count += 1;
            Ok(())
        })?;
        Ok(count)
    }

    fn estimate_num_free_bytes(&self) -> Result<u64> {
        sysinfo::get_available_disk_space(&self.basedir)
    }

    fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64> {
        block_size.checked_sub(FORMAT_VERSION_HEADER.len() as u64)
            .with_context(|| anyhow!("Physical block size of {} is too small to store the FORMAT_VERSION_HEADER. Must be at least {}.", block_size, FORMAT_VERSION_HEADER.len()))
    }

    fn all_blocks(&self) -> Result<Box<dyn Iterator<Item = BlockId>>> {
        let mut result = vec![];
        self._for_each_block_file(&mut |file| {
            let path = file.path();
            // The following couple lines panic if the path is wrong. That's ok because
            // _for_each_block_file is supposed to only return paths that are good.
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
                .unwrap_or_else(|| {
                    panic!("Block file path `{}` should be valid UTF-8", path.display())
                });
            let blockid_nonprefix = path
                .file_name()
                .unwrap_or_else(|| {
                    panic!(
                        "Block file path `{}` should have a valid last component",
                        path.display()
                    )
                })
                .to_str()
                .unwrap_or_else(|| {
                    panic!("Block file path `{}` should be valid UTF-8", path.display())
                });
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
            result.push(BlockId::from_hex(&blockid)?);
            Ok(())
        })?;
        Ok(Box::new(result.into_iter()))
    }
}

impl BlockStoreWriter for OnDiskBlockStore {
    fn try_create(&self, id: &BlockId, data: &[u8]) -> Result<bool> {
        let path = self._block_path(id);
        if path.exists() {
            Ok(false)
        } else {
            _store(&path, data)?;
            Ok(true)
        }
    }

    fn remove(&self, id: &BlockId) -> Result<bool> {
        let path = self._block_path(id);
        match std::fs::remove_file(path) {
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                // File doesn't exist. Return false. This is not an error.
                Ok(false)
            }
            Ok(()) => Ok(true),
            Err(err) => Err(err.into()),
        }
    }

    fn store(&self, id: &BlockId, data: &[u8]) -> Result<()> {
        let path = self._block_path(id);
        _store(&path, data)
    }
}

impl BlockStore for OnDiskBlockStore {}

impl OnDiskBlockStore {
    fn _block_path(&self, block_id: &BlockId) -> PathBuf {
        let block_id_str = block_id.to_hex();
        assert!(
            block_id_str
                .chars()
                .all(|c| _is_allowed_blockid_character(c)),
            "Created invalid block_id_str"
        );
        return self
            .basedir
            .join(&block_id_str[..PREFIX_LEN])
            .join(&block_id_str[PREFIX_LEN..]);
    }

    fn _for_each_block_file(
        &self,
        callback: &mut impl FnMut(&DirEntry) -> Result<()>,
    ) -> Result<()> {
        for subdir in self.basedir.read_dir()? {
            let subdir = subdir?;
            if subdir.metadata()?.is_dir()
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
                for blockfile in subdir.path().read_dir()? {
                    let blockfile = blockfile?;
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
                        callback(&blockfile)?;
                    }
                }
            }
        }
        Ok(())
    }
}

fn _check_and_remove_header(data: &[u8]) -> Result<&[u8]> {
    if !data.starts_with(FORMAT_VERSION_HEADER) {
        if data.starts_with(FORMAT_VERSION_HEADER_PREFIX) {
            bail!("This block is not supported yet. Maybe it was created with a newer version of CryFS?");
        } else {
            bail!("This is not a valid block.");
        }
    }
    Ok(&data[FORMAT_VERSION_HEADER.len()..])
}

// TODO Test
fn _create_dir_if_doesnt_exist(dir: &Path) -> Result<()> {
    match std::fs::create_dir(dir) {
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

fn _store(path: &Path, data: &[u8]) -> Result<()> {
    _create_dir_if_doesnt_exist(
        path.parent()
            .expect("Block file path should have a parent directory"),
    )
    .with_context(|| {
        format!(
            "Failed to create parent directory for block file at {}",
            path.display()
        )
    })?;
    let mut file = File::create(&path)
        .with_context(|| format!("Failed to create block file at {}", path.display()))?;
    file.write_vectored(&[IoSlice::new(FORMAT_VERSION_HEADER), IoSlice::new(data)])
        .with_context(|| format!("Failed to write to block file at {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_path() {
        let block_store = OnDiskBlockStore::new(Path::new("/base/path").to_path_buf());
        assert_eq!(
            Path::new("/base/path/2AC/9C78D80937AD50852C50BD3F1F982"),
            block_store._block_path(
                &BlockId::from_slice(&hex::decode("2AC9C78D80937AD50852C50BD3F1F982").unwrap())
                    .unwrap()
            )
        );
    }

    #[test]
    fn test_prefix() {
        assert!(FORMAT_VERSION_HEADER.starts_with(FORMAT_VERSION_HEADER_PREFIX));
    }
}
