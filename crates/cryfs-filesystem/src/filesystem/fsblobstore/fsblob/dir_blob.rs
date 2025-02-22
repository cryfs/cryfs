use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::stream::BoxStream;
use std::fmt::Debug;
use std::future::Future;
use std::time::SystemTime;

use super::atime_update_behavior::AtimeUpdateBehavior;
use super::base_blob::BaseBlob;
use super::layout::BlobType;
use crate::utils::fs_types::{Gid, Mode, Uid};
use cryfs_blobstore::{Blob, BlobId, BlobStore, BLOBID_LEN};
use cryfs_blockstore::BlockId;
use cryfs_rustfs::{FsError, FsResult, PathComponent, PathComponentBuf};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

use super::dir_entries::{DirEntry, DirEntryList, EntryType};

pub const DIR_LSTAT_SIZE: u64 = 4096;
pub const MODE_NEW_SYMLINK: Mode = Mode::zero()
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
    // TODO Some of the functions in here (and possibly in other blobs) were only needed for the cxx.rs bindings. Check which ones we can delete now since we're rust only.

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

    pub async fn create_root_dir_blob(blobstore: &'a B, root_blob_id: &BlobId) -> Result<()> {
        let mut blob = BaseBlob::try_create_with_id(
            root_blob_id,
            blobstore,
            BlobType::Dir,
            &BlobId::zero(),
            &[],
        )
        .await?
        .ok_or_else(|| anyhow!("Root blob {:?} already exists", root_blob_id))?;
        blob.flush().await?;
        Ok(())
    }

    // TODO DoubleEndedIterator + FusedIterator
    pub fn entries(&self) -> impl Iterator<Item = &DirEntry> + ExactSizeIterator + use<'_, B> {
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

    pub fn entry_by_name(&self, name: &PathComponent) -> Option<&DirEntry> {
        self.entries.get_by_name(name)
    }

    pub fn entry_by_name_mut(&mut self, name: &PathComponent) -> Option<&mut DirEntry> {
        self.entries.get_by_name_mut(name)
    }

    pub async fn rename_entry(
        &mut self,
        blob_id: &BlobId,
        new_name: PathComponentBuf,
        on_overwritten: impl FnOnce(&BlobId) -> FsResult<()>,
    ) -> FsResult<()> {
        self.entries.rename(blob_id, new_name, on_overwritten).await
    }

    pub async fn rename_entry_by_name<F>(
        &mut self,
        old_name: &PathComponent,
        new_name: PathComponentBuf,
        // TODO Instead of passing in on_overwritten, would be better to return the overwritten blob id with #[must_use]
        on_overwritten: impl FnOnce(&BlobId) -> F,
    ) -> FsResult<()>
    where
        F: Future<Output = FsResult<()>>,
    {
        self.entries
            .rename_by_name(old_name, new_name, on_overwritten)
            .await
    }

    pub fn update_modification_timestamp_of_entry(&mut self, blob_id: &BlobId) -> FsResult<()> {
        self.entries.update_modification_timestamp(blob_id)
    }

    pub fn set_attr_of_entry_by_name<'s>(
        &'s mut self,
        name: &PathComponent,
        mode: Option<Mode>,
        uid: Option<Uid>,
        gid: Option<Gid>,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
    ) -> FsResult<&'s DirEntry> {
        self.entries
            .set_attr_by_name(name, mode, uid, gid, atime, mtime)
    }

    pub fn maybe_update_access_timestamp_of_entry(
        &mut self,
        blob_id: &BlobId,
        atime_update_behavior: AtimeUpdateBehavior,
    ) -> FsResult<()> {
        self.entries
            .maybe_update_access_timestamp(blob_id, atime_update_behavior)
    }

    pub fn remove_entry_by_name(
        &mut self,
        name: &PathComponent,
    ) -> Result<DirEntry, cryfs_rustfs::FsError> {
        self.entries.remove_by_name(name)
    }

    pub fn remove_entry_by_id_if_exists(&mut self, blob_id: &BlobId) {
        self.entries.remove_by_id_if_exists(blob_id);
    }

    pub fn add_entry_dir(
        &mut self,
        name: PathComponentBuf,
        id: BlobId,
        mode: Mode,
        uid: Uid,
        gid: Gid,
        last_access_time: SystemTime,
        last_modification_time: SystemTime,
    ) -> FsResult<()> {
        self.entries.add(
            name,
            id,
            EntryType::Dir,
            mode,
            uid,
            gid,
            last_access_time,
            last_modification_time,
        )
    }

    pub fn add_entry_file(
        &mut self,
        name: PathComponentBuf,
        id: BlobId,
        mode: Mode,
        uid: Uid,
        gid: Gid,
        last_access_time: SystemTime,
        last_modification_time: SystemTime,
    ) -> FsResult<()> {
        self.entries.add(
            name,
            id,
            EntryType::File,
            mode,
            uid,
            gid,
            last_access_time,
            last_modification_time,
        )
    }

    pub fn add_entry_symlink(
        &mut self,
        name: PathComponentBuf,
        id: BlobId,
        uid: Uid,
        gid: Gid,
        last_access_time: SystemTime,
        last_modification_time: SystemTime,
    ) -> FsResult<()> {
        self.entries.add(
            name,
            id,
            EntryType::Symlink,
            MODE_NEW_SYMLINK,
            uid,
            gid,
            last_access_time,
            last_modification_time,
        )
    }

    pub async fn add_or_overwrite_entry<F>(
        &mut self,
        name: PathComponentBuf,
        id: BlobId,
        entry_type: EntryType,
        mode: Mode,
        uid: Uid,
        gid: Gid,
        last_access_time: SystemTime,
        last_modification_time: SystemTime,
        on_overwritten: impl FnOnce(&BlobId) -> F,
    ) -> Result<()>
    where
        F: Future<Output = FsResult<()>>,
    {
        self.entries
            .add_or_overwrite(
                name,
                id,
                entry_type,
                mode,
                uid,
                gid,
                last_access_time,
                last_modification_time,
                on_overwritten,
            )
            .await
    }

    pub async fn remove(this: AsyncDropGuard<Self>) -> Result<()> {
        // No need to async_drop because that'd only serialize it
        // but we're removing the blob anyhow.
        this.unsafe_into_inner_dont_drop().blob.remove().await
    }

    pub fn lstat_size(&self) -> u64 {
        DIR_LSTAT_SIZE
    }

    pub fn all_blocks(&self) -> Result<BoxStream<'_, Result<BlockId>>> {
        // TODO We may want to flush here since otherwise any change aren't written yet
        self.blob.all_blocks()
    }

    #[cfg(any(test, feature = "testutils"))]
    pub async fn num_nodes(&mut self) -> Result<u64> {
        self.flush().await?;
        self.blob.num_nodes().await
    }

    #[cfg(any(test, feature = "testutils"))]
    pub async fn into_raw(mut this: AsyncDropGuard<Self>) -> Result<B::ConcreteBlob<'a>> {
        this.flush().await?;
        let this = this.unsafe_into_inner_dont_drop();
        Ok(this.blob.into_raw())
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
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> FsResult<()> {
        self.flush().await.map_err(|err| FsError::InternalError {
            // TODO Instead of map_err, have flush return FsError
            error: err.context("Error in DirBlob::async_drop_impl"),
        })
    }
}
