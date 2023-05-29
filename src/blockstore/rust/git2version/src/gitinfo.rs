use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display, Formatter};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitInfo<'a, 'b> {
    pub tag: &'a str,
    pub commits_since_tag: u32,
    pub commit_id: &'b str,
    pub modified: bool,
}

impl<'a, 'b> Debug for GitInfo<'a, 'b> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl<'a, 'b> Display for GitInfo<'a, 'b> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}+{}.g{}",
            self.tag, self.commits_since_tag, self.commit_id
        )?;
        if self.modified {
            write!(f, ".modified")?;
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
        fn notontag_notmodified() {
            let version = GitInfo {
                tag: "v1.2.3",
                commits_since_tag: 10,
                commit_id: "abcdef",
                modified: false,
            };
            assert_eq!("v1.2.3+10.gabcdef", format!("{}", version));
            assert_eq!("v1.2.3+10.gabcdef", format!("{:?}", version));
        }

        #[test]
        fn notontag_modified() {
            let version = GitInfo {
                tag: "v1.2.3",
                commits_since_tag: 10,
                commit_id: "abcdef",
                modified: true,
            };
            assert_eq!("v1.2.3+10.gabcdef.modified", format!("{}", version));
            assert_eq!("v1.2.3+10.gabcdef.modified", format!("{:?}", version));
        }

        #[test]
        fn ontag_notmodified() {
            let version = GitInfo {
                tag: "v1.2.3",
                commits_since_tag: 0,
                commit_id: "abcdef",
                modified: false,
            };
            assert_eq!("v1.2.3+0.gabcdef", format!("{}", version));
            assert_eq!("v1.2.3+0.gabcdef", format!("{:?}", version));
        }

        #[test]
        fn ontag_modified() {
            let version = GitInfo {
                tag: "v1.2.3",
                commits_since_tag: 0,
                commit_id: "abcdef",
                modified: true,
            };
            assert_eq!("v1.2.3+0.gabcdef.modified", format!("{}", version));
            assert_eq!("v1.2.3+0.gabcdef.modified", format!("{:?}", version));
        }
    }
}
