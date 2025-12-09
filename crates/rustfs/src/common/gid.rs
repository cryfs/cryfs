use derive_more::{Display, From, Into};

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, From, Into)]
pub struct Gid(u32);
