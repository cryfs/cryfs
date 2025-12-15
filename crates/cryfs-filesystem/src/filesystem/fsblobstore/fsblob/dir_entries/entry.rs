use anyhow::{Result, ensure};
use binrw::{BinRead, BinResult, BinWrite, Endian};
use std::fmt::Debug;
use std::io::{Read, Seek, SeekFrom, Write};
use std::num::NonZeroU8;
use std::time::SystemTime;

use crate::utils::fs_types::{Gid, Mode, Uid};
use cryfs_blobstore::BlobId;
use cryfs_rustfs::{FsError, FsResult};
use cryfs_utils::{
    binary::{read_null_string, read_timespec, write_null_string, write_timespec},
    path::{PathComponent, PathComponentBuf},
};

// TODO Unify this with the BlobType enum from this very same crate?

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
    #[br(parse_with = read_path_component)]
    #[bw(write_with = write_path_component)]
    name: PathComponentBuf,
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
        name: PathComponentBuf,
        blob_id: BlobId,
        mode: Mode,
        uid: Uid,
        gid: Gid,
        last_access_time: SystemTime,
        last_modification_time: SystemTime,
        last_metadata_change_time: SystemTime,
    ) -> FsResult<Self> {
        let mode = match entry_type {
            EntryType::File => mode.with_file_flag(),
            EntryType::Dir => mode.with_dir_flag(),
            EntryType::Symlink => mode.with_symlink_flag(),
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
                name,
                blob_id,
            },
        };
        result.validate().map_err(FsError::internal_error)?;
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
            ((self.inner.entry_type == EntryType::File)
                && self.inner.mode.has_file_flag()
                && !self.inner.mode.has_dir_flag()
                && !self.inner.mode.has_symlink_flag())
                || ((self.inner.entry_type == EntryType::Dir)
                    && !self.inner.mode.has_file_flag()
                    && self.inner.mode.has_dir_flag()
                    && !self.inner.mode.has_symlink_flag())
                || ((self.inner.entry_type == EntryType::Symlink)
                    && !self.inner.mode.has_file_flag()
                    && !self.inner.mode.has_dir_flag()
                    && self.inner.mode.has_symlink_flag()),
            "Wrong mode bit set. Entry type is {:?} and mode bits say is_file={}, is_dir={}, is_symlink={}",
            self.inner.entry_type,
            self.inner.mode.has_file_flag(),
            self.inner.mode.has_dir_flag(),
            self.inner.mode.has_symlink_flag(),
        );
        Ok(())
    }

    pub fn entry_type(&self) -> EntryType {
        self.inner.entry_type
    }

    pub fn set_attr(
        &mut self,
        mode: Option<Mode>,
        uid: Option<Uid>,
        gid: Option<Gid>,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
    ) -> FsResult<()> {
        // TODO Direct implementation would be faster because it'd avoid multiple _update_metadata_change_time calls. Maybe we could even remove the other setters and only have this one?
        if let Some(mode) = mode {
            self.set_mode(mode)?;
        }
        if let Some(uid) = uid {
            self.set_uid(uid);
        }
        if let Some(gid) = gid {
            self.set_gid(gid);
        }
        if let Some(atime) = atime {
            self.set_last_access_time(atime);
        }
        if let Some(mtime) = mtime {
            self.set_last_modification_time(mtime);
        }
        Ok(())
    }

    pub fn set_mode(&mut self, mode: Mode) -> FsResult<()> {
        let old_mode = self.inner.mode;
        if old_mode == mode {
            // shortcut, so we don't update metadata change time here.
            return Ok(());
        }

        let old_last_metadata_change_time = self.inner.last_metadata_change_time;

        self.inner.mode = mode;
        self._update_metadata_change_time();

        self.validate().map_err(|e| {
            // Restore old values
            self.inner.mode = old_mode;
            self.inner.last_metadata_change_time = old_last_metadata_change_time;
            log::error!("Mode validation failed: {:?}", e);
            FsError::InvalidOperation
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
        // According to the POSIX standard, ctime does not get updated when atime changed.
        // ctime only gets updated when the content or metadata changes (atime/mtime fields don't count as metadata).
    }

    pub fn update_access_time(&mut self) {
        self.set_last_access_time(SystemTime::now());
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

    pub fn name(&self) -> &PathComponent {
        &self.inner.name
    }

    pub fn set_name(&mut self, name: PathComponentBuf) {
        self.inner.name = name;
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

// TODO Tests
fn read_path_component<R: Read + Seek>(
    reader: &mut R,
    endian: Endian,
    _: (),
) -> BinResult<PathComponentBuf> {
    // TODO Direct implementation without going through an intermediate Vec<NonZeroU8> would be faster
    let pos = reader.seek(SeekFrom::Current(0))?;
    let read_str = read_null_string(reader, endian, ())?;
    let read_str = read_str.into_iter().map(|v| v.get()).collect();
    let read_str = String::from_utf8(read_str).map_err(|err| binrw::Error::AssertFail {
        pos,
        message: format!("{err:?}"),
    })?;
    let path =
        PathComponentBuf::try_from_string(read_str).map_err(|err| binrw::Error::AssertFail {
            pos,
            message: format!("{err:?}"),
        })?;
    Ok(path)
}

// TODO Tests
fn write_path_component(
    v: &PathComponentBuf,
    writer: &mut (impl Write + Seek),
    endian: Endian,
    args: (),
) -> Result<(), binrw::Error> {
    // TODO Direct implementation without going through an intermediate Vec<NonZeroU8> would be faster
    let bytes: Vec<NonZeroU8> = v
        .as_str()
        .as_bytes()
        .into_iter()
        .map(|c| {
            NonZeroU8::try_from(*c).expect("PathComponent ensures that there aren't any null bytes")
        })
        .collect();
    write_null_string(&bytes, writer, endian, args)
}
