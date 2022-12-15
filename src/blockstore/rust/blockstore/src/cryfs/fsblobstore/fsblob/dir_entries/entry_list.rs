use anyhow::{bail, Result};
use std::fmt::Debug;
use std::io::Cursor;
use std::time::SystemTime;

use crate::cryfs::utils::fs_types::{Uid, Gid, Mode};
use super::super::base_blob::BaseBlob;
use super::entry::{DirEntry, EntryType};
use crate::blobstore::{BlobId, BlobStore};
use super::super::atime_update_behavior::AtimeUpdateBehavior;
use super::super::FsError;

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

    pub fn get_by_name(&self, name: &str) -> Result<Option<&DirEntry>> {
        // TODO Instead of linear search, we could use a HashMap
        for entry in &self.entries {
            if entry.name()? == name {
                return Ok(Some(entry));
            }
        }
        Ok(None)
    }

    pub fn get_by_name_mut(&mut self, name: &str) -> Result<Option<&mut DirEntry>> {
        for entry in &mut self.entries {
            if entry.name()? == name {
                self.dirty = true;
                return Ok(Some(entry));
            }
        }
        Ok(None)
    }

    fn _get_by_name_with_index(&self, name: &str) -> Result<Option<(usize, &DirEntry)>> {
        // TODO Instead of linear search, we could use a HashMap
        for (index, entry) in self.entries.iter().enumerate() {
            if entry.name()? == name {
                return Ok(Some((index, entry)));
            }
        }
        Ok(None)
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

    pub fn iter(&self) -> impl Iterator<Item = &DirEntry> {
        self.entries.iter()
    }

    pub fn num_entries(&self) -> usize {
        self.entries.len()
    }

    pub fn add(&mut self, name: &str, id: BlobId, entry_type: EntryType, mode: Mode, uid: Uid, gid: Gid, last_access_time: SystemTime, last_modification_time: SystemTime) -> Result<()> {
        if self.get_by_name(name)?.is_some() {
            bail!(FsError::EEXIST { msg: format!("Entry with name {:?} already exists", name) });
        }
        self._add(DirEntry::new(entry_type, name, id, mode, uid, gid, last_access_time, last_modification_time, SystemTime::now())?);
        Ok(())
    }

    fn _add(&mut self, entry: DirEntry) {
        let upper_bound = self._find_upper_bound(entry.blob_id());
        self.entries.insert(upper_bound, entry);
        self.dirty = true;
    }

    pub fn add_or_overwrite(&mut self, name: &str, id: BlobId, entry_type: EntryType, mode: Mode, uid: Uid, gid: Gid, last_access_time: SystemTime, last_modification_time: SystemTime, on_overwritten: impl FnOnce(&BlobId) -> Result<()>) -> Result<()> {
        let entry = DirEntry::new(entry_type, name, id, mode, uid, gid, last_access_time, last_modification_time, SystemTime::now())?;
        if let Some((index, old_entry)) = self._get_by_name_with_index(name)? {
            on_overwritten(old_entry.blob_id())?;
            self._overwrite(index, entry)?;
        } else {
            self._add(entry);
        }
        Ok(())
    }

    fn _overwrite(&mut self, index: usize, entry: DirEntry) -> Result<()> {
        assert_eq!(self.entries[index].name()?, entry.name()?);
        Self::_check_allowed_overwrite(
            self.entries[index].entry_type(),
            entry.name()?,
            entry.entry_type(),
        )?;

        // The new entry has possibly a different blockId, so it has to be in a different list position (list is ordered by blockIds).
        // That's why we remove-and-add instead of just modifying the existing entry.
        self.entries.remove(index);
        self._add(entry);

        Ok(())
    }

    // TODO If we add hard links, we can have multiple entries with the same blob id.
    //      rename() will have to take old_name instead of blob_id
    //      for the source blob and this whole implementation will have to change.
    pub fn rename(
        &mut self,
        blob_id: &BlobId,
        new_name: &str,
        on_overwritten: impl FnOnce(&BlobId) -> Result<()>,
    ) -> Result<()> {
        let Some(mut source_index) = self._get_index_by_id(blob_id) else {
            bail!(FsError::ENOENT { msg: format!("Could not find entry with {:?} in directory", blob_id)});
        };

        if let Some((found_same_name_index, found_same_name)) =
            self._get_by_name_with_index(new_name)?
        {
            if found_same_name.blob_id() == blob_id {
                // If the current name holder is already our source blob, we don't need to rename it
                assert_eq!(source_index, found_same_name_index);
                assert_eq!(self.entries[source_index].name()?, new_name);
                return Ok(());
            }

            let source = &self.entries[source_index];
            Self::_check_allowed_overwrite(
                found_same_name.entry_type(),
                new_name,
                source.entry_type(),
            )?;
            on_overwritten(found_same_name.blob_id())?;

            self.dirty = true;
            self.entries.remove(found_same_name_index);

            // Since we removed an entry, we need to update the index
            // we're keeping into the vector
            assert_ne!(source_index, found_same_name_index);
            if source_index >= found_same_name_index {
                source_index -= 1;
            }
            assert_eq!(self.entries[source_index].blob_id(), blob_id);
        }

        self.dirty = true;
        self.entries[source_index].set_name(new_name);
        Ok(())
    }

    pub fn set_mode(&mut self, blob_id: &BlobId, mode: Mode) -> Result<()> {
        let Some(entry) = self.get_by_id_mut(blob_id) else {
            bail!(FsError::ENOENT{msg: format!("Could not find entry with {:?} in directory", blob_id)});
        };
        entry.set_mode(mode)?;
        Ok(())
    }

    pub fn set_uid_gid(&mut self, blob_id: &BlobId, uid: Option<Uid>, gid: Option<Gid>) -> Result<()> {
        let Some(found) = self._get_index_by_id(blob_id) else {
            bail!(FsError::ENOENT{msg: format!("Could not find entry with {:?} in directory", blob_id)});
        };

        if let Some(uid) = uid {
            self.entries[found].set_uid(uid);
            self.dirty = true;
        }

        if let Some(gid) = gid {
            self.entries[found].set_gid(gid);
            self.dirty = true;
        }

        Ok(())
    }

    pub fn set_access_times(&mut self, blob_id: &BlobId, last_access_time: SystemTime, last_modification_time: SystemTime) -> Result<()> {
        let Some(entry) = self.get_by_id_mut(blob_id) else {
            bail!(FsError::ENOENT{msg: format!("Could not find entry with {:?} in directory", blob_id)});
        };
        entry.set_last_access_time(last_access_time);
        entry.set_last_modification_time(last_modification_time);
        Ok(())
    }

    pub fn maybe_update_access_timestamp(&mut self, blob_id: &BlobId, atime_update_behavior: AtimeUpdateBehavior) -> Result<()> {
        let Some(index) = self._get_index_by_id(blob_id) else {
            bail!(FsError::ENOENT{msg: format!("Could not find entry with {:?} in directory", blob_id)});
        };
        let entry = &self.entries[index];
        let last_access_time = entry.last_access_time();
        let last_modification_time = entry.last_modification_time();
        let now = SystemTime::now();

        let should_update_atime = 
            match entry.entry_type() {
                EntryType::File | EntryType::Symlink => {
                    atime_update_behavior.should_update_atime_on_file_or_symlink_read(last_access_time, last_modification_time, now)
                }
                EntryType::Dir => {
                    atime_update_behavior.should_update_atime_on_directory_read(last_access_time, last_modification_time, now)
                }
            };

        if should_update_atime {
            self.entries[index].set_last_access_time(now);
            self.dirty = true;
        }
        Ok(())
    }

    pub fn update_modification_timestamp(&mut self, blob_id: &BlobId) -> Result<()> {
        let Some(entry) = self.get_by_id_mut(blob_id) else {
            bail!(FsError::ENOENT{msg: format!("Could not find entry with {:?} in directory", blob_id)});
        };
        entry.update_modification_time();
        Ok(())
    }

    pub fn remove_by_name(&mut self, name: &str) -> Result<()> {
        let Some((index, _entry)) = self._get_by_name_with_index(name)? else {
            bail!(FsError::ENOENT { msg: format!("Could not find entry with name {:?} in directory", name)});
        };
        self.dirty = true;
        self.entries.remove(index);
        Ok(())
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
        name: &str,
        source_type: EntryType,
    ) -> Result<(), FsError> {
        if prev_dest_type != source_type {
            if prev_dest_type == EntryType::Dir {
                // new path is an existing directory, but old path is not a directory
                return Err(FsError::EISDIR {
                    msg: format!("Cannot overwrite a directory with a non-directory during a rename operation. Dest: {:?}", name),
                });
            }
            if source_type == EntryType::Dir {
                // oldpath is a directory, and newpath exists but is not a directory.
                return Err(FsError::ENOTDIR {
                    msg: format!("Cannot overwrite a non-directory with a directory during a rename operation. Dest: {:?}", name),});
            }
        }
        Ok(())
    }
}
