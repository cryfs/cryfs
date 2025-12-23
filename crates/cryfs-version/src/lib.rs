#![forbid(unsafe_code)]
// Note: Can't use #![deny(missing_docs)] because git2version::init_proxy_lib!()
// generates an undocumented GITINFO constant. Using warn instead.
#![warn(missing_docs)]

//! Version management for CryFS with git tag integration.
//!
//! This crate provides semantic version handling with compile-time verification
//! that the version in `Cargo.toml` matches the git tag. This ensures version
//! consistency between the package metadata and version control.
//!
//! # Key Types
//!
//! - [`Version`]: A semantic version with major, minor, patch, and optional prerelease components.
//! - [`VersionInfo`]: Combines a [`Version`] with optional git metadata (commit hash, tag info).
//!
//! # Macros
//!
//! - [`package_version!`]: Returns a [`VersionInfo`] with the package version and git metadata.
//!   Asserts at compile time that the Cargo.toml version matches the git tag.
//! - [`cargo_version!`]: Returns a [`Version`] from `Cargo.toml` without git metadata.
//! - [`assert_cargo_version_equals_git_version!`]: Compile-time assertion that versions match.
//!
//! # Example
//!
//! ```ignore
//! // Get the full version info including git metadata
//! let version_info = cryfs_version::package_version!();
//! println!("Version: {}", version_info);
//! // Output: "1.2.3" or "1.2.3+5.gabcdef" (if commits since tag)
//!
//! // Get just the Cargo.toml version
//! let version = cryfs_version::cargo_version!();
//! assert_eq!(version.major, 1);
//! ```

git2version::init_proxy_lib!();

mod version;
pub use version::Version;
mod version_info;
pub use version_info::VersionInfo;

/// Returns a [`VersionInfo`] containing the package version and git metadata.
///
/// This macro reads the version from `Cargo.toml` and combines it with git
/// information (commit hash, tag, commits since tag). It performs a compile-time
/// assertion that the `Cargo.toml` version matches the git tag, ensuring version
/// consistency.
///
/// # Panics
///
/// Causes a compile-time error if the `Cargo.toml` version does not match
/// the current git tag (when building from a tagged commit).
///
/// # Example
///
/// ```ignore
/// let info = cryfs_version::package_version!();
/// println!("Running version {}", info);
/// // Prints: "1.2.3" or "1.2.3+5.gabcdef.modified"
/// ```
#[macro_export]
macro_rules! package_version {
    () => {{
        {
            // We need to make sure the cargo and git version are the same because otherwise
            // VersionInfo::version and VersionInfo::gitinfo::tag would be inconsistent
            // and maybe other gitinfo would be wrong as well.
            $crate::assert_cargo_version_equals_git_version!();

            const VERSION_INFO: $crate::VersionInfo<&'static str> = $crate::VersionInfo::new(
                    // This needs to be a macro instead of a const right here because
                    // we need to run it in the context of the client crate, otherwise
                    // it'll just return our own version number.
                    $crate::cargo_version!(),
                    // And GITINFO needs to be a const in the cryfs-version crate instead of a macro because
                    // the `build.rs` script of the cryfs-version crate only adds the right env variables
                    // for the build of the cryfs-version crate, not to the build of its dependencies.
                    $crate::GITINFO,
            );
            VERSION_INFO
        }
    }};
}

/// Returns a [`Version`] parsed from the calling crate's `Cargo.toml`.
///
/// This macro reads the `CARGO_PKG_VERSION_*` environment variables at compile
/// time to construct a [`Version`]. It does not include git metadata.
///
/// Use this when you only need the version number without git information,
/// or when you want to avoid the version consistency check performed by
/// [`package_version!`].
///
/// # Example
///
/// ```ignore
/// let version = cryfs_version::cargo_version!();
/// println!("Version {}.{}.{}", version.major, version.minor, version.patch);
/// if let Some(pre) = version.prerelease {
///     println!("Prerelease: {}", pre);
/// }
/// ```
#[macro_export]
macro_rules! cargo_version {
    () => {{
        {
            // This needs to be a macro instead of a const right here because
            // we need to run it in the context of the client crate, otherwise
            // it'll just return our own version number.
            const RESULT: $crate::Version<&'static str> = $crate::Version {
                major: $crate::konst::unwrap_ctx!($crate::konst::primitive::parse_u32(env!(
                    "CARGO_PKG_VERSION_MAJOR"
                ))),
                minor: $crate::konst::unwrap_ctx!($crate::konst::primitive::parse_u32(env!(
                    "CARGO_PKG_VERSION_MINOR"
                ))),
                patch: $crate::konst::unwrap_ctx!($crate::konst::primitive::parse_u32(env!(
                    "CARGO_PKG_VERSION_PATCH"
                ))),
                prerelease: {
                    let prerelease = env!("CARGO_PKG_VERSION_PRE");
                    if prerelease.is_empty() {
                        None
                    } else {
                        Some(prerelease)
                    }
                },
            };
            RESULT
        }
    }};
}

/// Asserts at compile time that the `Cargo.toml` version matches the git tag.
///
/// This macro creates a module with a const assertion that verifies version
/// consistency. If the versions do not match, compilation fails with an error
/// message indicating the mismatch.
///
/// This is automatically called by [`package_version!`], but can be used
/// independently if you want to enforce version consistency without retrieving
/// the version info.
///
/// # Panics
///
/// Causes a compile-time error if the `Cargo.toml` version does not match
/// the current git tag (when building from a tagged commit).
///
/// # Example
///
/// ```ignore
/// // Place at module level to enforce version consistency
/// cryfs_version::assert_cargo_version_equals_git_version!();
/// ```
#[macro_export]
macro_rules! assert_cargo_version_equals_git_version {
    () => {
        mod assert_cargo_version_equals_git_version {
            const CARGO_VERSION: $crate::Version<&'static str> = $crate::cargo_version!();
            const _VERSION_INFO: $crate::VersionInfo<&'static str> =
                $crate::VersionInfo::new(CARGO_VERSION, $crate::GITINFO)
                    .assert_cargo_version_equals_git_version();
        }
    };
}
