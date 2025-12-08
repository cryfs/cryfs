use derive_more::{Display, From, Into};
use std::cmp::PartialOrd;

use crate::common::HandleTrait;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display, From, Into)]
pub struct FileHandle {
    v: u64,
}

impl FileHandle {
    pub const ZERO: Self = Self { v: 0 };
}

impl HandleTrait for FileHandle {
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
