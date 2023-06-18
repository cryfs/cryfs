use derive_more::{From, Into};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, Into)]
pub struct Gid(u32);
