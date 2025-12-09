#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OpenOutFlags {
    // Not implemented at the moment, but should support flags like:
    //  * DirectIO
    //  * KeepCache
    //  * NonSeekable
    // See https://man7.org/linux/man-pages/man4/fuse.4.html
    // and https://docs.rs/fuser/latest/fuser/trait.Filesystem.html#method.open
}
