use super::NodeKind;
use cryfs_utils::path::PathComponentBuf;

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
