use super::super::interface::Device;
use crate::common::{Gid, Uid};

pub enum MaybeInitializedFs<Fs: Device> {
    Uninitialized(Option<Box<dyn FnOnce(Uid, Gid) -> Fs + Send + Sync>>),
    Initialized(Fs),
    Destroyed,
}

impl<Fs: Device> MaybeInitializedFs<Fs> {
    pub fn initialize(&mut self, uid: Uid, gid: Gid) {
        match self {
            Self::Uninitialized(construct_fs) => {
                let construct_fs = construct_fs
                    .take()
                    .expect("MaybeInitializedFs::initialize() called twice");
                let fs = construct_fs(uid, gid);
                *self = MaybeInitializedFs::Initialized(fs);
            }
            Self::Destroyed => {
                panic!("MaybeInitializedFs::initialize() called after destroy()");
            }
            Self::Initialized(_) => {
                panic!("MaybeInitializedFs::initialize() called twice");
            }
        }
    }

    pub fn get(&self) -> &Fs {
        match self {
            Self::Uninitialized(_) => {
                panic!("MaybeInitializedFs::get() called before initialize()");
            }
            Self::Destroyed => {
                panic!("MaybeInitializedFs::get() called after destroy()");
            }
            Self::Initialized(fs) => fs,
        }
    }

    pub fn take(&mut self) -> Fs {
        let prev_value = std::mem::replace(self, Self::Destroyed);
        match prev_value {
            Self::Uninitialized(_) => {
                panic!("MaybeInitializedFs::take() called before initialize()");
            }
            Self::Destroyed => {
                panic!("MaybeInitializedFs::take() called after destroy()");
            }
            Self::Initialized(fs) => fs,
        }
    }
}
