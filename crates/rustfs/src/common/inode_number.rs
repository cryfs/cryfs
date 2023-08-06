use derive_more::{From, Into};
use std::cmp::PartialOrd;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, From, Into)]
pub struct InodeNumber(u64);

impl InodeNumber {
    pub const fn from_const(v: u64) -> Self {
        Self(v)
    }
}
