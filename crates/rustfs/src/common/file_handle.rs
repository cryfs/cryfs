use derive_more::{From, Into};
use std::cmp::PartialOrd;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, From, Into)]
pub struct FileHandle(u64);
