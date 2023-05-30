git2version::init_proxy_lib!();

mod version;
pub use version::Version;
mod version_info;
pub use version_info::VersionInfo;

#[macro_export]
macro_rules! package_version {
    () => {{
        {
            // We need to make sure the cargo and git version are the same because otherwise
            // VersionInfo::version and VersionInfo::gitinfo::tag would be inconsistent
            // and maybe other gitinfo would be wrong as well.
            $crate::assert_cargo_version_equals_git_version!();

            const VERSION_INFO: $crate::VersionInfo = $crate::VersionInfo::new(
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

#[macro_export]
macro_rules! cargo_version {
    () => {{
        {
            // This needs to be a macro instead of a const right here because
            // we need to run it in the context of the client crate, otherwise
            // it'll just return our own version number.
            const RESULT: $crate::Version = $crate::Version {
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

#[macro_export]
macro_rules! assert_cargo_version_equals_git_version {
    () => {
        mod assert_cargo_version_equals_git_version {
            const CARGO_VERSION: $crate::Version = $crate::cargo_version!();
            const _VERSION_INFO: $crate::VersionInfo =
                $crate::VersionInfo::new(CARGO_VERSION, $crate::GITINFO)
                    .assert_cargo_version_equals_git_version();
        }
    };
}
