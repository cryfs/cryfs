use anyhow::Result;
use async_trait::async_trait;
use std::pin::Pin;
use futures::Stream;
use std::time::SystemTime;
use std::fmt::Debug;

use super::base_blob::BaseBlob;
use super::atime_update_behavior::AtimeUpdateBehavior;
use crate::cryfs::utils::fs_types::{Gid, Mode, Uid};
use super::layout::BlobType;
use crate::blobstore::{BlobId, BlobStore};
use crate::blockstore::BlockId;
use crate::utils::async_drop::{AsyncDrop, AsyncDropGuard};

use super::dir_entries::{DirEntry, EntryType, DirEntryList};

const DIR_LSTAT_SIZE: u64 = 4096;

pub struct DirBlob<'a, B>
where
    B: BlobStore + Debug + 'a,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send,
{
    blob: BaseBlob<'a, B>,
    entries: DirEntryList,
}

impl<'a, B> DirBlob<'a, B>
where
    B: BlobStore + Debug + 'a,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send,
{
    pub(super) async fn new(mut blob: BaseBlob<'a, B>) -> Result<AsyncDropGuard<DirBlob<'a, B>>> {
        let entries = DirEntryList::deserialize(&mut blob).await?;
        Ok(AsyncDropGuard::new(Self { blob, entries }))
    }

    pub async fn create_blob(
        blobstore: &'a B,
        parent: &BlobId,
    ) -> Result<AsyncDropGuard<DirBlob<'a, B>>> {
        Ok(AsyncDropGuard::new(Self {
            blob: BaseBlob::create(blobstore, BlobType::Dir, parent, &[]).await?,
            entries: DirEntryList::empty(),
        }))
    }

    pub fn entries(&self) -> impl Iterator<Item = &DirEntry> {
        self.entries.iter()
    }

    pub async fn flush(&mut self) -> Result<()> {
        self.entries.serialize_if_dirty(&mut self.blob).await
    }

    pub fn blob_id(&self) -> BlobId {
        self.blob.blob_id()
    }

    pub fn parent(&self) -> BlobId {
        self.blob.parent()
    }

    pub async fn set_parent(&mut self, new_parent: &BlobId) -> Result<()> {
        self.blob.set_parent(new_parent).await
    }

    pub fn num_entries(&self) -> usize {
        self.entries.num_entries()
    }

    pub fn entry_by_id(&self, id: &BlobId) -> Option<&DirEntry> {
        self.entries.get_by_id(id)
    }

    pub fn entry_by_name(&self, name: &str) -> Result<Option<&DirEntry>> {
        self.entries.get_by_name(name)
    }

    pub fn entry_by_name_mut(&mut self, name: &str) -> Result<Option<&mut DirEntry>> {
        self.entries.get_by_name_mut(name)
    }

    pub fn rename_entry(
        &mut self,
        blob_id: &BlobId,
        new_name: &str,
        on_overwritten: impl FnOnce(&BlobId) -> Result<()>,
    ) -> Result<()> {
        self.entries.rename(blob_id, new_name, on_overwritten)
    }

    pub fn update_modification_timestamp_of_entry(&mut self, blob_id: &BlobId) -> Result<()> {
        self.entries.update_modification_timestamp(blob_id)
    }

    pub fn set_mode_of_entry(&mut self, blob_id: &BlobId, mode: Mode) -> Result<()> {
        self.entries.set_mode(blob_id, mode)
    }

    pub fn set_uid_gid_of_entry(&mut self, blob_id: &BlobId, uid: Option<Uid>, gid: Option<Gid>) -> Result<()> {
        self.entries.set_uid_gid(blob_id, uid, gid)
    }

    pub fn set_access_times_of_entry(&mut self, blob_id: &BlobId, last_access_time: SystemTime, last_modification_time: SystemTime) -> Result<()> {
        self.entries.set_access_times(blob_id, last_access_time, last_modification_time)
    }

    pub fn maybe_update_access_timestamp_of_entry(&mut self, blob_id: &BlobId, atime_update_behavior: AtimeUpdateBehavior) -> Result<()> {
        self.entries.maybe_update_access_timestamp(blob_id, atime_update_behavior)
    }

    pub fn remove_entry_by_name(&mut self, name: &str) -> Result<()> {
        self.entries.remove_by_name(name)
    }

    pub fn remove_entry_by_id_if_exists(&mut self, blob_id: &BlobId) {
        self.entries.remove_by_id_if_exists(blob_id);
    }

    pub fn add_entry_dir(&mut self, name: &str, id: BlobId, mode: Mode, uid: Uid, gid: Gid, last_access_time: SystemTime, last_modification_time: SystemTime) -> Result<()> {
        self.entries.add(name, id, EntryType::Dir, mode, uid, gid, last_access_time, last_modification_time)
    }

    pub fn add_entry_file(&mut self, name: &str, id: BlobId, mode: Mode, uid: Uid, gid: Gid, last_access_time: SystemTime, last_modification_time: SystemTime) -> Result<()> {
        self.entries.add(name, id, EntryType::File, mode, uid, gid, last_access_time, last_modification_time)
    }

    pub fn add_entry_symlink(&mut self, name: &str, id: BlobId, uid: Uid, gid: Gid, last_access_time: SystemTime, last_modification_time: SystemTime) -> Result<()> {
        let mode = *Mode::zero()
            .add_symlink_flag()
            .add_user_read_flag()
            .add_user_write_flag()
            .add_user_exec_flag()
            .add_group_read_flag()
            .add_group_write_flag()
            .add_group_exec_flag()
            .add_other_read_flag()
            .add_other_write_flag()
            .add_other_exec_flag();
        self.entries.add(name, id, EntryType::Symlink, mode, uid, gid, last_access_time, last_modification_time)
    }

    pub fn add_or_overwrite_entry(&mut self, name: &str, id: BlobId, entry_type: EntryType, mode: Mode, uid: Uid, gid: Gid, last_access_time: SystemTime, last_modification_time: SystemTime, on_overwritten: impl FnOnce(&BlobId) -> Result<()>) -> Result<()> {
        self.entries.add_or_overwrite(name, id, entry_type, mode, uid, gid, last_access_time, last_modification_time, on_overwritten)
    }

    pub async fn remove(this: AsyncDropGuard<Self>) -> Result<()> {
        // No need to async_drop because that'd only serialize it
        // but we're removing the blob anyhow.
        this.unsafe_into_inner_dont_drop().blob.remove().await
    }

    pub fn lstat_size(&self) -> u64 {
        DIR_LSTAT_SIZE
    }

    pub async fn all_blocks(&self) -> Result<Box<dyn Stream<Item=Result<BlockId>> + Unpin + '_>> {
        self.blob.all_blocks().await
    }
}

impl<'a, B> Debug for DirBlob<'a, B>
where
    B: BlobStore + Debug + 'a,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DirBlob")
            .field("blob_id", &self.blob_id())
            .field("parent", &self.parent())
            .field("entries", &self.entries)
            .finish()
    }
}

#[async_trait]
impl<'a, B> AsyncDrop for DirBlob<'a, B>
where
    B: BlobStore + Debug + 'a,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send,
{
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<()> {
        self.flush().await
    }
}
