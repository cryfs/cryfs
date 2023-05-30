use cryfs_version::VersionInfo;

cryfs_version::assert_cargo_version_equals_git_version!();

pub const CRYFS_VERSION: VersionInfo = cryfs_version::package_version!();
