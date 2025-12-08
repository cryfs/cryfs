use std::cmp::PartialOrd;

use derive_more::{From, Into};

use crate::common::HandleTrait;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, From, Into)]
pub struct InodeNumber {
    v: u64,
}

impl InodeNumber {
    pub const fn from_const(v: u64) -> Self {
        Self { v }
    }
}

impl HandleTrait for InodeNumber {
    const MIN: Self = Self { v: u64::MIN };
    const MAX: Self = Self { v: u64::MAX };

    #[inline]
    fn incremented(&self) -> Self {
        Self { v: self.v + 1 }
    }

    #[inline]
    fn range(begin_inclusive: &Self, end_exclusive: &Self) -> impl Iterator<Item = Self> {
        (begin_inclusive.v..end_exclusive.v).map(|v| Self { v })
    }
}
