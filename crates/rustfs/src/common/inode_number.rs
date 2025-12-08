use std::{cmp::PartialOrd, num::NonZeroU64};

use derive_more::{From, Into};

use crate::common::HandleTrait;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, From, Into)]
pub struct InodeNumber {
    // Using NonZeroU64 to get a niche for optimizing Option<InodeNumber>
    v: NonZeroU64,
}

impl InodeNumber {
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

impl HandleTrait for InodeNumber {
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
