use std::time::SystemTime;

pub trait AtimeUpdateBehavior {
    fn should_update_atime_on_file_or_symlink_read(
        self,
        old_atime: SystemTime,
        old_mtime: SystemTime,
        new_atime: SystemTime,
    ) -> bool;

    fn should_update_atime_on_directory_read(
        self,
        old_atime: SystemTime,
        old_mtime: SystemTime,
        new_atime: SystemTime,
    ) -> bool;
}
