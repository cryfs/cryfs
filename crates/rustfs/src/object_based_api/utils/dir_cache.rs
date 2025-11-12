use async_trait::async_trait;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard, SyncDrop};
use derive_more::{From, Into};
use std::fmt::Debug;

use crate::{
    DirEntry, FileHandle, FsError, FsResult, InodeNumber,
    common::{HandleMap, HandleWithGeneration},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, From, Into)]
pub struct OpenDirHandle(pub FileHandle);
impl From<u64> for OpenDirHandle {
    fn from(value: u64) -> Self {
        OpenDirHandle(FileHandle::from(value))
    }
}
impl Into<u64> for OpenDirHandle {
    fn into(self) -> u64 {
        self.0.into()
    }
}

/// For each dir inode, cache the list of entries so that multiple calls to readdir (with different offsets) get a consistent view
/// and can be served without repeatedly asking the filesystem for entries.
pub struct DirCache {
    entries:
        std::sync::Mutex<AsyncDropGuard<HandleMap<OpenDirHandle, AsyncDropArc<DirCacheEntry>>>>,
}

impl DirCache {
    pub fn new() -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            entries: std::sync::Mutex::new(HandleMap::new()),
        })
    }

    pub fn add(&self, dir_ino: InodeNumber) -> HandleWithGeneration<OpenDirHandle> {
        let mut entries = self.entries.lock().unwrap();
        entries.add(AsyncDropArc::new(DirCacheEntry::new(dir_ino)))
    }

    // SyncDrop is totally fine here because DirCacheEntry doesn't have a true async drop implementation.
    // It's just a dummy implementation needed because HandleMap wants it.
    pub fn remove(&self, handle: OpenDirHandle) -> SyncDrop<AsyncDropArc<DirCacheEntry>> {
        let mut entries = self.entries.lock().unwrap();
        SyncDrop::new(entries.remove(handle))
    }

    pub fn get(&self, handle: OpenDirHandle) -> Option<SyncDrop<AsyncDropArc<DirCacheEntry>>> {
        let entries = self.entries.lock().unwrap();
        entries
            .get(handle)
            .map(|entry| SyncDrop::new(AsyncDropArc::clone(entry)))
    }
}

impl Debug for DirCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DirCache").finish()
    }
}

#[async_trait]
impl AsyncDrop for DirCache {
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> FsResult<()> {
        let mut entries = std::mem::replace(
            &mut *self.entries.lock().unwrap(),
            AsyncDropGuard::new_invalid(),
        );
        entries.async_drop().await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct DirCacheEntry {
    dir_ino: InodeNumber,
    // None = not yet queried
    // Some(vec) = cached entries
    entries: tokio::sync::Mutex<Option<Vec<DirEntry>>>,
}

impl DirCacheEntry {
    pub fn new(dir_ino: InodeNumber) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            dir_ino,
            entries: tokio::sync::Mutex::new(None),
        })
    }

    pub fn dir_ino(&self) -> InodeNumber {
        self.dir_ino
    }

    /// If the entries are already cached, return them via result_callback.
    /// Otherwise, call query_fn to get the entries, cache them, and then return them via result_callback.
    pub async fn get_or_query_entries(
        &self,
        query_fn: impl AsyncFnOnce() -> FsResult<Vec<DirEntry>>,
        result_callback: impl FnOnce(&[DirEntry]) -> FsResult<()>,
    ) -> FsResult<()> {
        let mut entries = self.entries.lock().await;
        if let Some(entries) = &*entries {
            result_callback(entries)
        } else {
            let queried_entries = query_fn().await?;
            let result = result_callback(&queried_entries);
            *entries = Some(queried_entries);
            result
        }
    }
}

#[async_trait]
impl AsyncDrop for DirCacheEntry {
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> FsResult<()> {
        // Nothing to do
        Ok(())
    }
}
