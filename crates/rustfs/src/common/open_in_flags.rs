#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpenInFlags {
    Read,
    Write,
    ReadWrite,
}
