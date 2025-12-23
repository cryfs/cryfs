use serde::{Deserialize, Serialize};
use std::{
    borrow::Borrow,
    fmt::{self, Debug, Display, Formatter},
};

use super::version::Version;
use git2version::GitInfo;

/// Version information including optional git metadata.
///
/// This struct combines a semantic [`Version`] with optional git information
/// such as the commit hash, tag, and number of commits since the tag. It provides
/// a complete picture of the build version, useful for displaying in CLI tools
/// or logging.
///
/// # Display Format
///
/// When formatted for display, `VersionInfo` produces strings like:
/// - `1.2.3` - stable release exactly on a tag
/// - `1.2.3-alpha` - prerelease exactly on a tag
/// - `1.2.3+5.gabcdef` - 5 commits after tag, commit hash abcdef
/// - `1.2.3+5.gabcdef.modified` - same, with uncommitted modifications
/// - `1.2.3+modified` - on tag but with uncommitted modifications
///
/// # Type Parameters
///
/// - `'b`, `'c`: Lifetimes for git info string references
/// - `P`: The string type for the version prerelease (typically `&str` or `String`)
///
/// # Example
///
/// ```ignore
/// let info = cryfs_version::package_version!();
/// println!("Running CryFS {}", info);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'b + 'c, P: Deserialize<'de>"))]
pub struct VersionInfo<'b, 'c, P>
where
    P: Borrow<str>,
{
    version: Version<P>,
    gitinfo: Option<GitInfo<'b, 'c>>,
}

impl<'a, 'b, 'c> VersionInfo<'b, 'c, &'a str> {
    /// Creates a new `VersionInfo` with version consistency validation.
    ///
    /// If git information is provided and includes a tag, this constructor
    /// validates that the provided version matches the git tag version.
    /// A mismatch causes a compile-time panic when used in const contexts.
    ///
    /// # Arguments
    ///
    /// * `version` - The semantic version from Cargo.toml
    /// * `gitinfo` - Optional git metadata (commit hash, tag info)
    ///
    /// # Panics
    ///
    /// Panics at compile time if the version does not match the git tag version.
    /// This ensures version consistency between Cargo.toml and git tags.
    #[track_caller]
    pub const fn new(version: Version<&'a str>, gitinfo: Option<GitInfo<'b, 'c>>) -> Self {
        if let Some(gitinfo) = gitinfo {
            match gitinfo.tag_info {
                Some(tag_info) => {
                    let git_version = konst::result::unwrap!(Version::parse_const(tag_info.tag));
                    if !version.eq_const(&git_version) {
                        panic!(
                            "Version mismatch: The version in the git tag does not match the version in Cargo.toml"
                        );
                        // TODO Enable the following once `const_format_args` is stable
                        // panic!(
                        //     "Version mismatch: The version in the git tag ({}) does not match the version \
                        //     in Cargo.toml ({})",
                        //     gitinfo.tag,
                        //     RESULT.version,
                        // );
                    }
                    Some(git_version)
                }
                None => None,
            };
        }

        Self { version, gitinfo }
    }
}

impl<'b, 'c, P> VersionInfo<'b, 'c, P>
where
    P: Borrow<str>,
{
    /// Asserts that the Cargo.toml version equals the git tag version.
    ///
    /// This is a no-op method that returns `self`, as the assertion is already
    /// performed in the [`Self::new`] constructor. It exists for use in const
    /// contexts where method chaining is used to explain why the constant exists.
    ///
    /// # Example
    ///
    /// ```ignore
    /// const INFO: VersionInfo<&str> = VersionInfo::new(version, gitinfo)
    ///     .assert_cargo_version_equals_git_version();
    /// ```
    pub const fn assert_cargo_version_equals_git_version(self) -> Self {
        // Nothing to do because we already assert this in the constructor
        self
    }

    /// Returns a reference to the semantic version.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let info = cryfs_version::package_version!();
    /// let version = info.version();
    /// println!("Major version: {}", version.major);
    /// ```
    pub const fn version(&self) -> &Version<P> {
        &self.version
    }

    /// Returns the git metadata, if available.
    ///
    /// Returns `None` if the build was not from a git repository or if
    /// git information was not available at build time.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let info = cryfs_version::package_version!();
    /// if let Some(gitinfo) = info.gitinfo() {
    ///     println!("Commit: {}", gitinfo.commit_id);
    /// }
    /// ```
    pub const fn gitinfo(&self) -> Option<GitInfo<'b, 'c>> {
        self.gitinfo
    }
}

impl<'b, 'c, P> Debug for VersionInfo<'b, 'c, P>
where
    P: Borrow<str>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl<'b, 'c, P> Display for VersionInfo<'b, 'c, P>
where
    P: Borrow<str>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.version)?;
        if let Some(gitinfo) = self.gitinfo {
            let commits_since_tag = gitinfo.tag_info.map(|t| t.commits_since_tag).unwrap_or(0);
            if commits_since_tag > 0 {
                write!(f, "+{}.g{}", commits_since_tag, gitinfo.commit_id)?;
                if gitinfo.modified {
                    write!(f, ".modified")?;
                }
            } else if gitinfo.modified {
                write!(f, "+modified")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use git2version::TagInfo;

    mod display {
        use super::*;

        #[test]
        fn no_prerelease_and_no_gitinfo() {
            let version: VersionInfo<&str> = VersionInfo {
                version: Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: None,
                },
                gitinfo: None,
            };
            assert_eq!("1.2.3", format!("{}", version));
            assert_eq!("1.2.3", format!("{:?}", version));
        }

        #[test]
        fn with_prerelease_and_no_gitinfo() {
            let version: VersionInfo<&str> = VersionInfo {
                version: Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: Some("alpha"),
                },
                gitinfo: None,
            };
            assert_eq!("1.2.3-alpha", format!("{}", version));
            assert_eq!("1.2.3-alpha", format!("{:?}", version));
        }

        #[test]
        fn no_prerelease_and_with_gitinfo() {
            let version: VersionInfo<&str> = VersionInfo {
                version: Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: None,
                },
                gitinfo: Some(GitInfo {
                    tag_info: Some(TagInfo {
                        tag: "a.b.c",
                        commits_since_tag: 10,
                    }),
                    commit_id: "abcdef",
                    modified: false,
                }),
            };
            assert_eq!("1.2.3+10.gabcdef", format!("{}", version));
            assert_eq!("1.2.3+10.gabcdef", format!("{:?}", version));
        }

        #[test]
        fn with_prerelease_and_with_gitinfo() {
            let version = VersionInfo {
                version: Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: Some("alpha"),
                },
                gitinfo: Some(GitInfo {
                    tag_info: Some(TagInfo {
                        tag: "a.b.c",
                        commits_since_tag: 10,
                    }),
                    commit_id: "abcdef",
                    modified: false,
                }),
            };
            assert_eq!("1.2.3-alpha+10.gabcdef", format!("{}", version));
            assert_eq!("1.2.3-alpha+10.gabcdef", format!("{:?}", version));
        }

        #[test]
        fn no_prerelease_and_with_gitinfo_and_modified() {
            let version: VersionInfo<&str> = VersionInfo {
                version: Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: None,
                },
                gitinfo: Some(GitInfo {
                    tag_info: Some(TagInfo {
                        tag: "a.b.c",
                        commits_since_tag: 10,
                    }),
                    commit_id: "abcdef",
                    modified: true,
                }),
            };
            assert_eq!("1.2.3+10.gabcdef.modified", format!("{}", version));
            assert_eq!("1.2.3+10.gabcdef.modified", format!("{:?}", version));
        }

        #[test]
        fn with_prerelease_and_with_gitinfo_and_modified() {
            let version = VersionInfo {
                version: Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: Some("alpha"),
                },
                gitinfo: Some(GitInfo {
                    tag_info: Some(TagInfo {
                        tag: "a.b.c",
                        commits_since_tag: 10,
                    }),
                    commit_id: "abcdef",
                    modified: true,
                }),
            };
            assert_eq!("1.2.3-alpha+10.gabcdef.modified", format!("{}", version));
            assert_eq!("1.2.3-alpha+10.gabcdef.modified", format!("{:?}", version));
        }

        #[test]
        fn no_prerelease_and_with_gitinfo_ontag() {
            let version: VersionInfo<&str> = VersionInfo {
                version: Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: None,
                },
                gitinfo: Some(GitInfo {
                    tag_info: Some(TagInfo {
                        tag: "a.b.c",
                        commits_since_tag: 0,
                    }),
                    commit_id: "abcdef",
                    modified: false,
                }),
            };
            assert_eq!("1.2.3", format!("{}", version));
            assert_eq!("1.2.3", format!("{:?}", version));
        }

        #[test]
        fn with_prerelease_and_with_gitinfo_ontag() {
            let version = VersionInfo {
                version: Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: Some("alpha"),
                },
                gitinfo: Some(GitInfo {
                    tag_info: Some(TagInfo {
                        tag: "a.b.c",
                        commits_since_tag: 0,
                    }),
                    commit_id: "abcdef",
                    modified: false,
                }),
            };
            assert_eq!("1.2.3-alpha", format!("{}", version));
            assert_eq!("1.2.3-alpha", format!("{:?}", version));
        }

        #[test]
        fn no_prerelease_and_with_gitinfo_and_modified_ontag() {
            let version: VersionInfo<&str> = VersionInfo {
                version: Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: None,
                },
                gitinfo: Some(GitInfo {
                    tag_info: Some(TagInfo {
                        tag: "a.b.c",
                        commits_since_tag: 0,
                    }),
                    commit_id: "abcdef",
                    modified: true,
                }),
            };
            assert_eq!("1.2.3+modified", format!("{}", version));
            assert_eq!("1.2.3+modified", format!("{:?}", version));
        }

        #[test]
        fn with_prerelease_and_with_gitinfo_and_modified_ontag() {
            let version = VersionInfo {
                version: Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: Some("alpha"),
                },
                gitinfo: Some(GitInfo {
                    tag_info: Some(TagInfo {
                        tag: "a.b.c",
                        commits_since_tag: 0,
                    }),
                    commit_id: "abcdef",
                    modified: true,
                }),
            };
            assert_eq!("1.2.3-alpha+modified", format!("{}", version));
            assert_eq!("1.2.3-alpha+modified", format!("{:?}", version));
        }
    }

    mod accessors {
        use super::*;

        #[test]
        fn version_returns_correct_version() {
            let info: VersionInfo<&str> = VersionInfo {
                version: Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: Some("alpha"),
                },
                gitinfo: None,
            };
            let version = info.version();
            assert_eq!(1, version.major);
            assert_eq!(2, version.minor);
            assert_eq!(3, version.patch);
            assert_eq!(Some("alpha"), version.prerelease);
        }

        #[test]
        fn gitinfo_returns_none_when_absent() {
            let info: VersionInfo<&str> = VersionInfo {
                version: Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: None,
                },
                gitinfo: None,
            };
            assert!(info.gitinfo().is_none());
        }

        #[test]
        fn gitinfo_returns_some_when_present() {
            let info: VersionInfo<&str> = VersionInfo {
                version: Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: None,
                },
                gitinfo: Some(GitInfo {
                    tag_info: Some(TagInfo {
                        tag: "1.2.3",
                        commits_since_tag: 5,
                    }),
                    commit_id: "abc123",
                    modified: true,
                }),
            };
            let gitinfo = info.gitinfo().unwrap();
            assert_eq!("abc123", gitinfo.commit_id);
            assert!(gitinfo.modified);
            assert_eq!(5, gitinfo.tag_info.unwrap().commits_since_tag);
        }
    }

    mod serde {
        use super::*;

        #[test]
        fn roundtrip_no_gitinfo() {
            let original: VersionInfo<&str> = VersionInfo {
                version: Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: Some("alpha"),
                },
                gitinfo: None,
            };
            let serialized = serde_json::to_string(&original).unwrap();
            let deserialized: VersionInfo<String> = serde_json::from_str(&serialized).unwrap();
            assert_eq!(original.version().major, deserialized.version().major);
            assert_eq!(original.version().minor, deserialized.version().minor);
            assert_eq!(original.version().patch, deserialized.version().patch);
            assert_eq!(
                original.version().prerelease,
                deserialized.version().prerelease.as_deref()
            );
            assert!(deserialized.gitinfo().is_none());
        }

        #[test]
        fn roundtrip_with_gitinfo() {
            let original: VersionInfo<&str> = VersionInfo {
                version: Version {
                    major: 2,
                    minor: 0,
                    patch: 0,
                    prerelease: None,
                },
                gitinfo: Some(GitInfo {
                    tag_info: Some(TagInfo {
                        tag: "2.0.0",
                        commits_since_tag: 10,
                    }),
                    commit_id: "deadbeef",
                    modified: false,
                }),
            };
            let serialized = serde_json::to_string(&original).unwrap();
            let deserialized: VersionInfo<String> = serde_json::from_str(&serialized).unwrap();
            assert_eq!(original.version().major, deserialized.version().major);

            let original_gitinfo = original.gitinfo().unwrap();
            let deserialized_gitinfo = deserialized.gitinfo().unwrap();
            assert_eq!(original_gitinfo.commit_id, deserialized_gitinfo.commit_id);
            assert_eq!(original_gitinfo.modified, deserialized_gitinfo.modified);
        }

        #[test]
        fn json_format_no_gitinfo() {
            let info: VersionInfo<&str> = VersionInfo {
                version: Version {
                    major: 1,
                    minor: 0,
                    patch: 0,
                    prerelease: None,
                },
                gitinfo: None,
            };
            let serialized = serde_json::to_string(&info).unwrap();
            assert!(serialized.contains("\"major\":1"));
            assert!(serialized.contains("\"gitinfo\":null"));
        }
    }
}
