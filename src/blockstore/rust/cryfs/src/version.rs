pub const VERSION: cryfs_version::VersionInfo = cryfs_version::package_version!();

cryfs_version::assert_cargo_version_equals_git_version!();
