#[derive(Debug, Clone, Copy)]
pub struct Statfs {
    pub max_filename_length: u32,

    /// Optimal transfer block size
    pub blocksize: u32,

    /// Total data blocks in the filesystem
    pub num_total_blocks: u64,

    /// Free blocks in filesystem
    pub num_free_blocks: u64,

    /// Free blocks available to unprivileged user
    pub num_available_blocks: u64,

    /// Total number of inodes in filesystem
    /// TODO Is this supposed to only count files or also directories? It's called `files` in the statvfs struct
    pub num_total_inodes: u64,

    // Free inodes in filesystem
    pub num_free_inodes: u64,
    // Fuse ignores the `f_avail` field of statfs, so we don't have a `num_available_inodes` representing it here.
    // See https://libfuse.github.io/doxygen/structfuse__operations.html#a76d29dba617a64321cf52d62cd969292
}
