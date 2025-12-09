use derive_more::Display;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpenInFlags {
    #[display("r")]
    Read,
    #[display("w")]
    Write,
    #[display("rw")]
    ReadWrite,
}
