use async_trait::async_trait;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard, AsyncDropResult, AsyncDropShared};
use cryfs_utils::concurrent_store::LoadedEntryGuard;
use futures::future::BoxFuture;
use std::fmt::Debug;

use crate::{FsError, FsResult, InodeNumber, object_based_api::Device};

#[derive(Debug)]
pub struct InodeTreeNode<Fs>
where
    Fs: Device + Debug + 'static,
    Fs::Node: 'static,
{
    kernel_refcount: usize,
    inode: AsyncDropGuard<
        // TODO We only store this as a Shared future because ConcurrentStore doesn't allow returning a LoadedEntryGuard for
        //      entries that are still loading. If we change ConcurrentStore to allow that, we can just store LoadedEntryGuard here directly.
        AsyncDropShared<
            AsyncDropResult<LoadedEntryGuard<InodeNumber, Fs::Node, FsError>, FsError>,
            // TODO No BoxFuture
            BoxFuture<
                'static,
                AsyncDropGuard<
                    AsyncDropResult<LoadedEntryGuard<InodeNumber, Fs::Node, FsError>, FsError>,
                >,
            >,
        >,
    >,
}

impl<Fs> InodeTreeNode<Fs>
where
    Fs: Device + Debug + 'static,
    Fs::Node: 'static,
{
    pub fn new(
        inode: AsyncDropGuard<
            AsyncDropShared<
                AsyncDropResult<LoadedEntryGuard<InodeNumber, Fs::Node, FsError>, FsError>,
                BoxFuture<
                    'static,
                    AsyncDropGuard<
                        AsyncDropResult<LoadedEntryGuard<InodeNumber, Fs::Node, FsError>, FsError>,
                    >,
                >,
            >,
        >,
    ) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            kernel_refcount: 1,
            inode,
        })
    }

    pub fn increment_refcount(&mut self) {
        self.kernel_refcount = self
            .kernel_refcount
            .checked_add(1)
            .expect("Inode kernel refcount overflowed");
    }

    pub fn decrement_refcount(&mut self) -> RefcountInfo {
        assert!(
            self.kernel_refcount > 0,
            "Inode kernel refcount underflowed"
        );
        self.kernel_refcount -= 1;
        if self.kernel_refcount == 0 {
            RefcountInfo::RefcountZero
        } else {
            RefcountInfo::RefcountNotZero
        }
    }

    pub fn inode_future(
        &self,
    ) -> &AsyncDropGuard<
        AsyncDropShared<
            AsyncDropResult<LoadedEntryGuard<InodeNumber, Fs::Node, FsError>, FsError>,
            BoxFuture<
                'static,
                AsyncDropGuard<
                    AsyncDropResult<LoadedEntryGuard<InodeNumber, Fs::Node, FsError>, FsError>,
                >,
            >,
        >,
    > {
        &self.inode
    }
}

#[must_use]
#[derive(Debug, PartialEq, Eq)]
pub enum RefcountInfo {
    RefcountNotZero,
    RefcountZero,
}

#[async_trait]
impl<Fs> AsyncDrop for InodeTreeNode<Fs>
where
    Fs: Device + Debug + 'static,
    Fs::Node: 'static,
{
    type Error = FsError;
    async fn async_drop_impl(&mut self) -> FsResult<()> {
        self.inode.async_drop().await?;

        Ok(())
    }
}
