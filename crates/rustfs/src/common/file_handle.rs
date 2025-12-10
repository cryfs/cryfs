use derive_more::{Display, From, Into};
use std::{cmp::PartialOrd, num::NonZeroU64};

#[cfg(any(feature = "fuser", feature = "fuse_mt"))]
use crate::common::HandleTrait;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display, From, Into)]
pub struct FileHandle {
    // Using NonZeroU64 to get a niche for optimizing Option<FileHandle>
    v: NonZeroU64,
}

impl FileHandle {
    pub const fn from_const(v: NonZeroU64) -> Self {
        Self { v }
    }

    pub const fn try_from(v: u64) -> Option<Self> {
        match NonZeroU64::new(v) {
            None => None,
            Some(nz) => Some(Self::from_const(nz)),
        }
    }
}

#[cfg(any(feature = "fuser", feature = "fuse_mt"))]
impl HandleTrait for FileHandle {
    const MIN: Self = Self { v: NonZeroU64::MIN };
    const MAX: Self = Self { v: NonZeroU64::MAX };

    #[inline]
    fn incremented(&self) -> Self {
        Self {
            v: self.v.checked_add(1).unwrap(),
        }
    }

    #[inline]
    fn range(begin_inclusive: &Self, end_exclusive: &Self) -> impl Iterator<Item = Self> {
        (begin_inclusive.v.get()..end_exclusive.v.get()).map(|v| Self {
            v: NonZeroU64::new(v).unwrap(),
        })
    }
}
