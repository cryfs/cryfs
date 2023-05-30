use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display, Formatter};

use super::version::Version;
use crate::GitInfo;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'a + 'b + 'c"))]
pub struct VersionInfo<'a, 'b, 'c> {
    version: Version<'a>,
    gitinfo: Option<GitInfo<'b, 'c>>,
}

impl<'a, 'b, 'c> VersionInfo<'a, 'b, 'c> {
    #[track_caller]
    pub const fn new(version: Version<'a>, gitinfo: Option<GitInfo<'b, 'c>>) -> Self {
        if let Some(gitinfo) = gitinfo {
            let git_version = konst::unwrap_ctx!(Version::parse_const(gitinfo.tag));

            if !version.eq_const(&git_version) {
                panic!("Version mismatch: The version in the git tag does not match the version in Cargo.toml");
                // TODO Enable the following once `const_format_args` is stable
                // panic!(
                //     "Version mismatch: The version in the git tag ({}) does not match the version \
                //     in Cargo.toml ({})",
                //     gitinfo.tag,
                //     RESULT.version,
                // );
            }
        }

        Self { version, gitinfo }
    }

    pub const fn assert_cargo_version_equals_git_version(self) -> Self {
        // Nothing to do because we already assert this in the constructor
        self
    }

    pub const fn version(&self) -> Version<'a> {
        self.version
    }

    pub const fn gitinfo(&self) -> Option<GitInfo<'b, 'c>> {
        self.gitinfo
    }
}

impl<'a, 'b, 'c> Debug for VersionInfo<'a, 'b, 'c> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl<'a, 'b, 'c> Display for VersionInfo<'a, 'b, 'c> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.version)?;
        if let Some(gitinfo) = self.gitinfo {
            if gitinfo.commits_since_tag > 0 {
                write!(f, "+{}.g{}", gitinfo.commits_since_tag, gitinfo.commit_id)?;
                if gitinfo.modified {
                    write!(f, ".modified")?;
                }
            } else {
                if gitinfo.modified {
                    write!(f, "+modified")?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod display {
        use super::*;

        #[test]
        fn no_prerelease_and_no_gitinfo() {
            let version = VersionInfo {
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
            let version = VersionInfo {
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
            let version = VersionInfo {
                version: Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: None,
                },
                gitinfo: Some(GitInfo {
                    tag: "a.b.c",
                    commits_since_tag: 10,
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
                    tag: "a.b.c",
                    commits_since_tag: 10,
                    commit_id: "abcdef",
                    modified: false,
                }),
            };
            assert_eq!("1.2.3-alpha+10.gabcdef", format!("{}", version));
            assert_eq!("1.2.3-alpha+10.gabcdef", format!("{:?}", version));
        }

        #[test]
        fn no_prerelease_and_with_gitinfo_and_modified() {
            let version = VersionInfo {
                version: Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: None,
                },
                gitinfo: Some(GitInfo {
                    tag: "a.b.c",
                    commits_since_tag: 10,
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
                    tag: "a.b.c",
                    commits_since_tag: 10,
                    commit_id: "abcdef",
                    modified: true,
                }),
            };
            assert_eq!("1.2.3-alpha+10.gabcdef.modified", format!("{}", version));
            assert_eq!("1.2.3-alpha+10.gabcdef.modified", format!("{:?}", version));
        }

        #[test]
        fn no_prerelease_and_with_gitinfo_ontag() {
            let version = VersionInfo {
                version: Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: None,
                },
                gitinfo: Some(GitInfo {
                    tag: "a.b.c",
                    commits_since_tag: 0,
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
                    tag: "a.b.c",
                    commits_since_tag: 0,
                    commit_id: "abcdef",
                    modified: false,
                }),
            };
            assert_eq!("1.2.3-alpha", format!("{}", version));
            assert_eq!("1.2.3-alpha", format!("{:?}", version));
        }

        #[test]
        fn no_prerelease_and_with_gitinfo_and_modified_ontag() {
            let version = VersionInfo {
                version: Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: None,
                },
                gitinfo: Some(GitInfo {
                    tag: "a.b.c",
                    commits_since_tag: 0,
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
                    tag: "a.b.c",
                    commits_since_tag: 0,
                    commit_id: "abcdef",
                    modified: true,
                }),
            };
            assert_eq!("1.2.3-alpha+modified", format!("{}", version));
            assert_eq!("1.2.3-alpha+modified", format!("{:?}", version));
        }
    }
}
