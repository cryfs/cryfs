use std::fmt::Debug;

use super::super::interface::Device;
use crate::common::{Gid, Uid};
use async_trait::async_trait;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

#[derive(Debug)]
pub struct MaybeInitializedFs<Fs>
where
    Fs: Device + AsyncDrop + Debug + Send,
{
    inner: MaybeInitializedFsImpl<Fs>,
}

enum MaybeInitializedFsImpl<Fs>
where
    Fs: Device + AsyncDrop + Debug + Send,
{
    Uninitialized(Option<Box<dyn FnOnce(Uid, Gid) -> AsyncDropGuard<Fs> + Send + Sync>>),
    Initialized(AsyncDropGuard<Fs>),
}

impl<Fs> MaybeInitializedFs<Fs>
where
    Fs: Device + AsyncDrop + Debug + Send,
{
    pub fn new_uninitialized(
        initialize_fn: Box<dyn FnOnce(Uid, Gid) -> AsyncDropGuard<Fs> + Send + Sync>,
    ) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            inner: MaybeInitializedFsImpl::Uninitialized(Some(initialize_fn)),
        })
    }

    pub fn initialize(&mut self, uid: Uid, gid: Gid) {
        match &mut self.inner {
            MaybeInitializedFsImpl::Uninitialized(construct_fs) => {
                let construct_fs = construct_fs
                    .take()
                    .expect("MaybeInitializedFs::initialize() called twice");
                let fs = construct_fs(uid, gid);
                self.inner = MaybeInitializedFsImpl::Initialized(fs);
            }
            MaybeInitializedFsImpl::Initialized(_) => {
                panic!("MaybeInitializedFs::initialize() called twice");
            }
        }
    }

    pub fn get(&self) -> &Fs {
        match &self.inner {
            MaybeInitializedFsImpl::Uninitialized(_) => {
                panic!("MaybeInitializedFs::get() called before initialize()");
            }
            MaybeInitializedFsImpl::Initialized(fs) => fs,
        }
    }
}

impl<Fs> Debug for MaybeInitializedFsImpl<Fs>
where
    Fs: Device + AsyncDrop + Debug + Send,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaybeInitializedFsImpl::Uninitialized(_) => {
                write!(f, "MaybeInitializedFsImpl::Uninitialized")
            }
            MaybeInitializedFsImpl::Initialized(_) => {
                write!(f, "MaybeInitializedFsImpl::Initialized")
            }
        }
    }
}

#[async_trait]
impl<Fs> AsyncDrop for MaybeInitializedFs<Fs>
where
    Fs: Device + AsyncDrop + Debug + Send,
{
    type Error = <Fs as AsyncDrop>::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        match &mut self.inner {
            MaybeInitializedFsImpl::Uninitialized(_) => Ok(()),
            MaybeInitializedFsImpl::Initialized(fs) => fs.async_drop().await,
        }
    }
}
