use anyhow::{ensure, Result};
use binrw::{BinRead, BinWrite, NullString};
use std::fmt::Debug;
use std::io::{Read, Seek, Write};
use std::time::SystemTime;

use crate::blobstore::BlobId;
use crate::cryfs::utils::fs_types::{Gid, Mode, Uid};
use crate::utils::binary::{read_timespec, write_timespec};

#[derive(Clone, Copy, Debug, PartialEq, Eq, BinRead, BinWrite)]
#[brw(little, repr=u8)]
pub enum EntryType {
    Dir = 0x00,
    File = 0x01,
    Symlink = 0x02,
}

#[derive(Clone, BinRead, BinWrite, Debug)]
#[brw(little)]
pub struct DirEntryImpl {
    entry_type: EntryType,
    mode: Mode,
    uid: Uid,
    gid: Gid,
    #[br(parse_with = read_timespec)]
    #[bw(write_with = write_timespec)]
    last_access_time: SystemTime,
    #[br(parse_with = read_timespec)]
    #[bw(write_with = write_timespec)]
    last_modification_time: SystemTime,
    #[br(parse_with = read_timespec)]
    #[bw(write_with = write_timespec)]
    last_metadata_change_time: SystemTime,
    name: NullString,
    blob_id: BlobId,
}

#[derive(Clone, Debug)]
pub struct DirEntry {
    // Add an indirection so that outside code can't directly serialize/deserialize it
    // without going through our validation logic.
    inner: DirEntryImpl,
}

impl DirEntry {
    pub fn new(
        entry_type: EntryType,
        name: &str,
        blob_id: BlobId,
        mut mode: Mode,
        uid: Uid,
        gid: Gid,
        last_access_time: SystemTime,
        last_modification_time: SystemTime,
        last_metadata_change_time: SystemTime,
    ) -> Result<Self> {
        match entry_type {
            EntryType::File => mode.add_file_flag(),
            EntryType::Dir => mode.add_dir_flag(),
            EntryType::Symlink => mode.add_symlink_flag(),
        };
        let result = Self {
            inner: DirEntryImpl {
                entry_type,
                mode,
                uid,
                gid,
                last_access_time,
                last_modification_time,
                last_metadata_change_time,
                name: name.into(),
                blob_id,
            },
        };
        result.validate()?;
        Ok(result)
    }

    pub fn deserialize(source: &mut (impl Read + Seek)) -> Result<Self> {
        let result = Self {
            inner: DirEntryImpl::read(source)?,
        };
        result.validate()?;
        Ok(result)
    }

    pub fn serialize(&self, target: &mut (impl Write + Seek)) -> Result<()> {
        self.validate()
            .expect("Validation failed. This breaks a class invariant and shouldn't happen.");
        self.inner.write(target)?;
        Ok(())
    }

    fn validate(&self) -> Result<()> {
        ensure!(
            ((self.inner.entry_type == EntryType::File) && self.inner.mode.has_file_flag() && !self.inner.mode.has_dir_flag() && !self.inner.mode.has_symlink_flag()) ||
            ((self.inner.entry_type == EntryType::Dir) && !self.inner.mode.has_file_flag() && self.inner.mode.has_dir_flag() && !self.inner.mode.has_symlink_flag()) ||
            ((self.inner.entry_type == EntryType::Symlink) && !self.inner.mode.has_file_flag() && !self.inner.mode.has_dir_flag() && self.inner.mode.has_symlink_flag()),
            "Wrong mode bit set. Entry type is {:?} and mode bits say is_file={}, is_dir={}, is_symlink={}", self.inner.entry_type, self.inner.mode.has_file_flag(), self.inner.mode.has_dir_flag(), self.inner.mode.has_symlink_flag(),
        );
        Ok(())
    }

    pub fn entry_type(&self) -> EntryType {
        self.inner.entry_type
    }

    pub fn set_mode(&mut self, mode: Mode) -> Result<()> {
        let old_mode = self.inner.mode;
        let old_last_metadata_change_time = self.inner.last_metadata_change_time;

        self.inner.mode = mode;
        self._update_metadata_change_time();

        self.validate().map_err(|e| {
            // Restore old values
            self.inner.mode = old_mode;
            self.inner.last_metadata_change_time = old_last_metadata_change_time;
            e
        })
    }

    pub fn mode(&self) -> Mode {
        self.inner.mode
    }

    pub fn uid(&self) -> Uid {
        self.inner.uid
    }

    pub fn set_uid(&mut self, uid: Uid) {
        self.inner.uid = uid;
        self._update_metadata_change_time();
    }

    pub fn gid(&self) -> Gid {
        self.inner.gid
    }

    pub fn set_gid(&mut self, gid: Gid) {
        self.inner.gid = gid;
        self._update_metadata_change_time();
    }

    pub fn last_access_time(&self) -> SystemTime {
        self.inner.last_access_time
    }

    pub fn set_last_access_time(&mut self, last_access_time: SystemTime) {
        self.inner.last_access_time = last_access_time;
        // TODO Should this do the following?
        //      self._update_metadata_change_time();
    }

    pub fn update_access_time(&mut self) {
        self.inner.last_access_time = SystemTime::now();
        // TODO Should this do the following?
        //      self._update_metadata_change_time();
    }

    pub fn last_modification_time(&self) -> SystemTime {
        self.inner.last_modification_time
    }

    pub fn set_last_modification_time(&mut self, last_modification_time: SystemTime) {
        self.inner.last_modification_time = last_modification_time;
        self._update_metadata_change_time();
    }

    pub fn update_modification_time(&mut self) {
        self.set_last_modification_time(SystemTime::now());
    }

    pub fn last_metadata_change_time(&self) -> SystemTime {
        self.inner.last_metadata_change_time
    }

    fn _update_metadata_change_time(&mut self) {
        self.inner.last_metadata_change_time = SystemTime::now();
    }

    pub fn name(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.inner.name)
    }

    pub fn set_name(&mut self, name: &str) {
        self.inner.name = name.into();
        self._update_metadata_change_time();
    }

    pub fn blob_id(&self) -> &BlobId {
        &self.inner.blob_id
    }

    pub fn set_blob_id(&mut self, blob_id: BlobId) {
        self.inner.blob_id = blob_id;
        self._update_metadata_change_time();
    }
}
