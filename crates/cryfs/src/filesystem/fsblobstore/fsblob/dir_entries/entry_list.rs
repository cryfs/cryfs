use anyhow::Result;
use std::fmt::Debug;
use std::future::Future;
use std::io::Cursor;
use std::time::SystemTime;

use super::super::atime_update_behavior::AtimeUpdateBehavior;
use super::super::base_blob::BaseBlob;
use super::super::FsError;
use super::entry::{DirEntry, EntryType};
use crate::utils::fs_types::{Gid, Mode, Uid};
use cryfs_blobstore::{BlobId, BlobStore};
use cryfs_rustfs::{FsResult, PathComponent, PathComponentBuf};

#[derive(Debug)]
pub struct DirEntryList {
    // TODO The implementation currently assumes that there is at most one
    // entry with any given block id. If we add hard links, we have to change this.
    // TODO While we don't have hardlinks, add invariant assertions that each
    // id only exists once and that entries are ordered.
    entries: Vec<DirEntry>,

    // At least one entry was modified since last serialization
    dirty: bool,
}

impl DirEntryList {
    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
            dirty: false,
        }
    }

    pub fn get_by_name(&self, name: &PathComponent) -> Option<&DirEntry> {
        // TODO Instead of linear search, we could use a HashMap
        for entry in &self.entries {
            if entry.name() == name {
                return Some(entry);
            }
        }
        None
    }

    pub fn get_by_name_mut(&mut self, name: &PathComponent) -> Option<&mut DirEntry> {
        for entry in &mut self.entries {
            if entry.name() == name {
                self.dirty = true;
                return Some(entry);
            }
        }
        None
    }

    fn _get_by_name_with_index(&self, name: &PathComponent) -> Option<(usize, &DirEntry)> {
        // TODO Instead of linear search, we could use a HashMap
        for (index, entry) in self.entries.iter().enumerate() {
            if entry.name() == name {
                return Some((index, entry));
            }
        }
        None
    }

    fn _get_index_by_name(&self, name: &PathComponent) -> Option<usize> {
        self._get_by_name_with_index(name).map(|(index, _)| index)
    }

    fn _get_index_by_id(&self, id: &BlobId) -> Option<usize> {
        let found = self._find_lower_bound(id);
        if found == self.entries.len() || self.entries[found].blob_id() != id {
            return None;
        }
        Some(found)
    }

    fn _find_lower_bound(&self, id: &BlobId) -> usize {
        self._find_first_index(id, |entry| entry.blob_id() >= id)
    }

    fn _find_upper_bound(&self, id: &BlobId) -> usize {
        self._find_first_index(id, |entry| entry.blob_id() > id)
    }

    fn _find_first_index(&self, hint: &BlobId, mut pred: impl FnMut(&DirEntry) -> bool) -> usize {
        // TODO Factor out a datastructure that keeps a sorted std::vector and allows these _findLowerBound()/_findUpperBound operations using this hinted linear search
        if self.entries.is_empty() {
            return self.entries.len();
        }
        let startpos_percent =
            u32::from_le_bytes(hint.data()[..4].try_into().unwrap()) as f64 / u32::MAX as f64;
        let mut index = (startpos_percent * (self.entries.len() - 1) as f64) as usize;
        assert!(index < self.entries.len(), "Startpos out of range");

        while index != 0 && pred(&self.entries[index]) {
            index -= 1;
        }
        while index != self.entries.len() && !pred(&self.entries[index]) {
            index += 1;
        }
        index
    }

    pub fn get_by_id(&self, id: &BlobId) -> Option<&DirEntry> {
        self._get_index_by_id(id).map(|index| &self.entries[index])
    }

    pub fn get_by_id_mut(&mut self, id: &BlobId) -> Option<&mut DirEntry> {
        self.dirty = true;
        self._get_index_by_id(id)
            .map(|index| &mut self.entries[index])
    }

    pub async fn deserialize<'a, B: BlobStore + Debug + 'a>(
        blob: &mut BaseBlob<'a, B>,
    ) -> Result<Self> {
        // TODO Stream this into a BufReader instead of reading everything up front? But is that actually better?
        let data = blob.read_all_data().await?;
        let len = data.len() as u64;

        let mut entries = Vec::new();
        let mut reader = Cursor::new(data);
        // TODO Use reader.is_empty() once that is stabilized
        while reader.position() < len {
            let entry = DirEntry::deserialize(&mut reader)?;
            entries.push(entry);
        }
        assert_eq!(reader.position(), len, "Did not read all data.");
        Ok(Self {
            entries,
            dirty: false,
        })
    }

    pub async fn serialize_if_dirty<'a, B: BlobStore + Debug + 'a>(
        &mut self,
        blob: &mut BaseBlob<'a, B>,
    ) -> Result<()> {
        if self.dirty {
            // TODO Stream this into a BufWriter instead of writing everything at the end? But is that actually better?
            let mut writer = Cursor::new(Vec::new());
            for entry in &self.entries {
                entry.serialize(&mut writer)?;
            }
            let data = writer.into_inner();
            blob.resize_data(data.len() as u64).await?;
            blob.write_data(&data, 0).await?;
            self.dirty = false;
        }
        Ok(())
    }

    // TODO FusedIterator, DoubleEndedIterator
    pub fn iter(&self) -> impl Iterator<Item = &DirEntry> + ExactSizeIterator {
        self.entries.iter()
    }

    pub fn num_entries(&self) -> usize {
        self.entries.len()
    }

    pub fn add(
        &mut self,
        name: PathComponentBuf,
        id: BlobId,
        entry_type: EntryType,
        mode: Mode,
        uid: Uid,
        gid: Gid,
        last_access_time: SystemTime,
        last_modification_time: SystemTime,
    ) -> FsResult<()> {
        if self.get_by_name(&name).is_some() {
            return Err(FsError::NodeAlreadyExists);
        }
        self._add(DirEntry::new(
            entry_type,
            name,
            id,
            mode,
            uid,
            gid,
            last_access_time,
            last_modification_time,
            SystemTime::now(),
        )?);
        Ok(())
    }

    fn _add(&mut self, entry: DirEntry) {
        let upper_bound = self._find_upper_bound(entry.blob_id());
        self.entries.insert(upper_bound, entry);
        self.dirty = true;
    }

    pub async fn add_or_overwrite<F>(
        &mut self,
        name: PathComponentBuf,
        id: BlobId,
        entry_type: EntryType,
        mode: Mode,
        uid: Uid,
        gid: Gid,
        last_access_time: SystemTime,
        last_modification_time: SystemTime,
        // TODO Return overwritten entry instead of taking an on_overwritten callback
        on_overwritten: impl FnOnce(&BlobId) -> F,
    ) -> Result<()>
    where
        F: Future<Output = FsResult<()>>,
    {
        let already_exists = self._get_by_name_with_index(&name);
        let entry = DirEntry::new(
            entry_type,
            name,
            id,
            mode,
            uid,
            gid,
            last_access_time,
            last_modification_time,
            SystemTime::now(),
        )?;
        if let Some((index, old_entry)) = already_exists {
            on_overwritten(old_entry.blob_id()).await?;
            self._overwrite(index, entry)?;
        } else {
            self._add(entry);
        }
        Ok(())
    }

    fn _overwrite(&mut self, index: usize, entry: DirEntry) -> Result<()> {
        assert_eq!(self.entries[index].name(), entry.name());
        Self::_check_allowed_overwrite(
            self.entries[index].entry_type(),
            entry.name(),
            entry.entry_type(),
        )?;

        // The new entry has possibly a different blockId, so it has to be in a different list position (list is ordered by blockIds).
        // That's why we remove-and-add instead of just modifying the existing entry.
        self.entries.remove(index);
        self._add(entry);

        Ok(())
    }

    pub async fn rename_by_name<F>(
        &mut self,
        old_name: &PathComponent,
        new_name: PathComponentBuf,
        on_overwritten: impl FnOnce(&BlobId) -> F,
    ) -> cryfs_rustfs::FsResult<()>
    where
        F: Future<Output = FsResult<()>>,
    {
        let Some((mut source_index, source_entry)) = self._get_by_name_with_index(old_name) else {
            return Err(cryfs_rustfs::FsError::NodeDoesNotExist);
        };
        let source_blob_id = *source_entry.blob_id();

        if let Some((found_same_name_index, found_same_name)) =
            self._get_by_name_with_index(&new_name)
        {
            if *found_same_name.blob_id() == source_blob_id {
                // If the current name holder is already our source blob, we don't need to rename it
                assert_eq!(source_index, found_same_name_index);
                assert_eq!(old_name, &*new_name);
                assert_eq!(self.entries[source_index].name(), &*new_name);
                return Ok(());
            }

            let source = &self.entries[source_index];
            Self::_check_allowed_overwrite(
                found_same_name.entry_type(),
                &new_name,
                source.entry_type(),
            )?;
            on_overwritten(found_same_name.blob_id()).await?;

            self.dirty = true;
            self.entries.remove(found_same_name_index);

            // Since we removed an entry, we need to update the index
            // we're keeping into the vector
            assert_ne!(source_index, found_same_name_index);
            if source_index >= found_same_name_index {
                source_index -= 1;
            }
            assert_eq!(*self.entries[source_index].blob_id(), source_blob_id);
        }

        self.dirty = true;
        self.entries[source_index].set_name(new_name);
        Ok(())
    }

    // TODO If we add hard links, we can have multiple entries with the same blob id.
    //      This function should be removed and call sites should use [Self::rename_by_name] instead.
    pub async fn rename(
        &mut self,
        blob_id: &BlobId,
        new_name: PathComponentBuf,
        on_overwritten: impl FnOnce(&BlobId) -> FsResult<()>,
    ) -> FsResult<()> {
        let Some(old_entry) = self.get_by_id(blob_id) else {
            return Err(FsError::NodeDoesNotExist);
        };
        let old_name = old_entry.name().to_owned();
        self.rename_by_name(&old_name, new_name, |b| {
            futures::future::ready(on_overwritten(b))
        })
        .await
    }

    pub fn set_attr_by_name<'s>(
        &'s mut self,
        name: &PathComponent,
        mode: Option<Mode>,
        uid: Option<Uid>,
        gid: Option<Gid>,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
    ) -> FsResult<&'s DirEntry> {
        let Some(entry) = self.get_by_name_mut(name) else {
            return Err(cryfs_rustfs::FsError::NodeDoesNotExist);
        };
        entry.set_attr(mode, uid, gid, atime, mtime)?;
        Ok(entry)
    }

    pub fn maybe_update_access_timestamp(
        &mut self,
        blob_id: &BlobId,
        atime_update_behavior: AtimeUpdateBehavior,
    ) -> FsResult<()> {
        let Some(index) = self._get_index_by_id(blob_id) else {
            return Err(FsError::NodeDoesNotExist);
        };
        let entry = &self.entries[index];
        let last_access_time = entry.last_access_time();
        let last_modification_time = entry.last_modification_time();
        let now = SystemTime::now();

        let should_update_atime = match entry.entry_type() {
            EntryType::File | EntryType::Symlink => atime_update_behavior
                .should_update_atime_on_file_or_symlink_read(
                    last_access_time,
                    last_modification_time,
                    now,
                ),
            EntryType::Dir => atime_update_behavior.should_update_atime_on_directory_read(
                last_access_time,
                last_modification_time,
                now,
            ),
        };

        if should_update_atime {
            self.entries[index].set_last_access_time(now);
            self.dirty = true;
        }
        Ok(())
    }

    pub fn update_modification_timestamp(&mut self, blob_id: &BlobId) -> FsResult<()> {
        let Some(entry) = self.get_by_id_mut(blob_id) else {
            return Err(FsError::NodeDoesNotExist);
        };
        entry.update_modification_time();
        Ok(())
    }

    pub fn remove_by_name(&mut self, name: &PathComponent) -> FsResult<DirEntry> {
        let Some((index, _entry)) = self._get_by_name_with_index(name) else {
            return Err(cryfs_rustfs::FsError::NodeDoesNotExist);
        };
        self.dirty = true;
        let removed = self.entries.remove(index);
        Ok(removed)
    }

    pub fn remove_by_id_if_exists(&mut self, blob_id: &BlobId) {
        if let Some(index) = self._get_index_by_id(blob_id) {
            self.dirty = true;
            self.entries.remove(index);
            if index < self.entries.len() {
                // TODO Remove if this never fires. For some reason our C++ implementation had
                //      a loop here that deletes all entries with the same blob id, even though
                //      we don't support hardlinks. Just wanna make sure there isn't some corner
                //      case we missed so adding an assertion.
                assert_ne!(self.entries[index].blob_id(), blob_id);
            }
        };
    }

    fn _check_allowed_overwrite(
        prev_dest_type: EntryType,
        name: &PathComponent,
        source_type: EntryType,
    ) -> FsResult<()> {
        if prev_dest_type != source_type {
            if prev_dest_type == EntryType::Dir {
                // new path is an existing directory, but old path is not a directory
                return Err(cryfs_rustfs::FsError::CannotOverwriteDirectoryWithNonDirectory);
            }
            if source_type == EntryType::Dir {
                // oldpath is a directory, and newpath exists but is not a directory.
                return Err(cryfs_rustfs::FsError::CannotOverwriteNonDirectoryWithDirectory);
            }
        }
        Ok(())
    }
}
