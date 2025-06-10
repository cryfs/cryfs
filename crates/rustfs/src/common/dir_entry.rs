use super::NodeKind;
use crate::common::PathComponentBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntry {
    pub name: PathComponentBuf,
    pub kind: NodeKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirEntryOrReference {
    /// A regular directory entry
    Entry(DirEntry),

    /// The '.' entry
    SelfReference,

    /// The '..' entry, which refers to the parent directory
    ParentReference,
}
