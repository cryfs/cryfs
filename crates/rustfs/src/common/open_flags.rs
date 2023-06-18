#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpenFlags {
    Read,
    Write,
    ReadWrite,
}
