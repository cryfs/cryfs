use super::NodeKind;
use crate::common::PathComponentBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntry {
    pub name: PathComponentBuf,
    pub kind: NodeKind,
}
