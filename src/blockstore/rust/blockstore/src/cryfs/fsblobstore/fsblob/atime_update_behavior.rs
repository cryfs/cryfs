// TODO This should probably live in fspp, not here

use std::time::{Duration, SystemTime};

/// Defines how atime timestamps of files and directories are accessed on read accesses
/// (e.g. atime, strictatime, relatime, nodiratime)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtimeUpdateBehavior {
    /// atime attribute (of both files and directories) is updated only during write access.
    Noatime,

    /// This causes the atime attribute to update with every file access. (accessing file data, not just the metadata/attributes)
    Strictatime,

    /// This option causes the atime attribute to update only if the previous atime is older than mtime or ctime, or the previous atime is over 24 hours old.
    Relatime,

    /// atime of directories is updated only during write access, can be combined with relatime. atime of files follows the relatime rules.
    NodiratimeRelatime,

    /// atime of directories is updated only during write access, can be combined with relatime. atime of files follows the strictatime rules.
    NodiratimeStrictatime,
}

impl AtimeUpdateBehavior {
    pub fn should_update_atime_on_file_or_symlink_read(
        self,
        old_atime: SystemTime,
        old_mtime: SystemTime,
        new_atime: SystemTime,
    ) -> bool {
        match self {
            AtimeUpdateBehavior::Noatime => false,
            AtimeUpdateBehavior::Strictatime | AtimeUpdateBehavior::NodiratimeStrictatime => true,
            AtimeUpdateBehavior::Relatime | AtimeUpdateBehavior::NodiratimeRelatime => {
                relatime(old_atime, old_mtime, new_atime)
            }
        }
    }

    pub fn should_update_atime_on_directory_read(
        self,
        old_atime: SystemTime,
        old_mtime: SystemTime,
        new_atime: SystemTime,
    ) -> bool {
        match self {
            AtimeUpdateBehavior::Noatime
            | AtimeUpdateBehavior::NodiratimeRelatime
            | AtimeUpdateBehavior::NodiratimeStrictatime => false,
            AtimeUpdateBehavior::Strictatime => true,
            AtimeUpdateBehavior::Relatime => relatime(old_atime, old_mtime, new_atime),
        }
    }
}

fn relatime(old_atime: SystemTime, old_mtime: SystemTime, new_atime: SystemTime) -> bool {
    let yesterday = new_atime - Duration::from_secs(60 * 60 * 24);
    old_atime < old_mtime || old_atime < yesterday
}
