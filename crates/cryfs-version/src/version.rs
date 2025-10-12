use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::cmp::{Ord, Ordering, PartialOrd};
use std::fmt::{self, Debug, Display, Formatter};
use std::num::ParseIntError;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Version<P>
where
    P: Borrow<str>,
{
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: Option<P>,
}

impl<P> Debug for Version<P>
where
    P: Borrow<str>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl<P> Display for Version<P>
where
    P: Borrow<str>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(prerelease) = &self.prerelease {
            write!(f, "-{}", prerelease.borrow())?;
        }
        Ok(())
    }
}

impl<P> Eq for Version<P> where P: Borrow<str> + Eq {}

impl<P1, P2> PartialEq<Version<P2>> for Version<P1>
where
    P1: Borrow<str>,
    P2: Borrow<str>,
{
    fn eq(&self, other: &Version<P2>) -> bool {
        self.major == other.major
            && self.minor == other.minor
            && self.patch == other.patch
            && match (&self.prerelease, &other.prerelease) {
                (Some(lhs), Some(rhs)) => lhs.borrow() == rhs.borrow(),
                (None, None) => true,
                _ => false,
            }
    }
}

impl<P> Ord for Version<P>
where
    P: Borrow<str> + Eq,
{
    fn cmp(&self, other: &Version<P>) -> Ordering {
        version_cmp(self, other)
    }
}

impl<P1, P2> PartialOrd<Version<P2>> for Version<P1>
where
    P1: Borrow<str>,
    P2: Borrow<str>,
{
    fn partial_cmp(&self, other: &Version<P2>) -> Option<Ordering> {
        Some(version_cmp(self, other))
    }
}

fn version_cmp<P1, P2>(lhs: &Version<P1>, rhs: &Version<P2>) -> Ordering
where
    P1: Borrow<str>,
    P2: Borrow<str>,
{
    if lhs.major != rhs.major {
        return lhs.major.cmp(&rhs.major);
    }
    if lhs.minor != rhs.minor {
        return lhs.minor.cmp(&rhs.minor);
    }
    if lhs.patch != rhs.patch {
        return lhs.patch.cmp(&rhs.patch);
    }
    match (&lhs.prerelease, &rhs.prerelease) {
        (Some(lhs), Some(rhs)) => lhs.borrow().cmp(&rhs.borrow()),
        (None, None) => Ordering::Equal,
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
    }
}

impl<'a> Version<&'a str> {
    pub fn parse(version: &'a str) -> Result<Self, ParseVersionError<'a>> {
        let (major_minor_patch, prerelease) = match version.split_once('-') {
            Some((major_minor_patch, prerelease)) => (major_minor_patch, Some(prerelease)),
            None => (version, None),
        };
        let (major, minor_patch) = match major_minor_patch.split_once('.') {
            Some((major, minor_patch)) => (major, minor_patch),
            None => (major_minor_patch, "0"),
        };
        let (minor, patch) = match minor_patch.split_once('.') {
            Some((minor, patch)) => (minor, patch),
            None => (minor_patch, "0"),
        };

        match (major.parse(), minor.parse(), patch.parse()) {
            (Ok(major), Ok(minor), Ok(patch)) => Ok(Self {
                major,
                minor,
                patch,
                prerelease,
            }),
            (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => {
                Err(ParseVersionError { version, error })
            }
        }
    }

    // TODO Merge this with [Self::parse] once const support is good enough
    pub const fn parse_const(version: &'a str) -> Result<Self, ParseIntError> {
        use konst::string;
        let (major_minor_patch, prerelease) = match string::split_once(version, '-') {
            Some((major_minor_patch, prerelease)) => (major_minor_patch, Some(prerelease)),
            None => (version, None),
        };
        let (major, minor_patch) = match string::split_once(major_minor_patch, '.') {
            Some((major, minor_patch)) => (major, minor_patch),
            None => (major_minor_patch, "0"),
        };
        let (minor, patch) = match string::split_once(minor_patch, '.') {
            Some((minor, patch)) => (minor, patch),
            None => (minor_patch, "0"),
        };

        match (
            u32::from_str_radix(major, 10),
            u32::from_str_radix(minor, 10),
            u32::from_str_radix(patch, 10),
        ) {
            (Ok(major), Ok(minor), Ok(patch)) => Ok(Self {
                major,
                minor,
                patch,
                prerelease,
            }),
            (Err(err), _, _) | (_, Err(err), _) | (_, _, Err(err)) => Err(err),
        }
    }

    pub const fn eq_const(&self, rhs: &Self) -> bool {
        if self.major != rhs.major || self.minor != rhs.minor || self.patch != rhs.patch {
            return false;
        }

        match (self.prerelease, rhs.prerelease) {
            (Some(lhs), Some(rhs)) => konst::string::eq_str(lhs, rhs),
            (None, None) => true,
            _ => false,
        }
    }

    pub fn to_owned(&self) -> Version<String> {
        Version {
            major: self.major,
            minor: self.minor,
            patch: self.patch,
            prerelease: self.prerelease.map(|s| s.to_owned()),
        }
    }

    pub fn into_owned(self) -> Version<String> {
        self.to_owned()
    }
}

impl Version<String> {
    pub fn to_borrowed(&self) -> Version<&str> {
        Version {
            major: self.major,
            minor: self.minor,
            patch: self.patch,
            prerelease: self.prerelease.as_ref().map(String::borrow),
        }
    }
}

#[derive(Error, Display, Debug, PartialEq, Eq)]
#[display("Failed to parse version `{version}`: {error}")]
pub struct ParseVersionError<'a> {
    version: &'a str,
    #[error(source)]
    error: std::num::ParseIntError,
}

#[cfg(test)]
mod tests {
    use super::*;

    mod parse {
        use super::*;

        #[test]
        fn major_minor_patch_prerelease() {
            let version = Version::parse("1.2.3-alpha");
            assert_eq!(
                Ok(Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: Some("alpha"),
                }),
                version,
            );
        }

        #[test]
        fn major_minor_patch() {
            let version = Version::parse("1.2.3");
            assert_eq!(
                Ok(Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: None,
                }),
                version,
            );
        }

        #[test]
        fn major_minor() {
            let version = Version::parse("1.2");
            assert_eq!(
                Ok(Version {
                    major: 1,
                    minor: 2,
                    patch: 0,
                    prerelease: None,
                }),
                version,
            );
        }

        #[test]
        fn major() {
            let version = Version::parse("1");
            assert_eq!(
                Ok(Version {
                    major: 1,
                    minor: 0,
                    patch: 0,
                    prerelease: None,
                }),
                version,
            );
        }

        #[test]
        fn invalid() {
            let version = Version::parse("invalid number");
            let error = version.unwrap_err();
            assert_eq!("invalid number", error.version);
            assert_eq!(std::num::IntErrorKind::InvalidDigit, *error.error.kind());
        }
    }

    mod parse_const {
        use super::*;

        #[test]
        fn major_minor_patch_prerelease() {
            const VERSION: Result<Version<&'static str>, ParseIntError> =
                Version::parse_const("1.2.3-alpha");
            assert_eq!(
                Ok(Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: Some("alpha"),
                }),
                VERSION,
            );
        }

        #[test]
        fn major_minor_patch() {
            const VERSION: Result<Version<&'static str>, ParseIntError> =
                Version::parse_const("1.2.3");
            assert_eq!(
                Ok(Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    prerelease: None,
                }),
                VERSION,
            );
        }

        #[test]
        fn major_minor() {
            const VERSION: Result<Version<&'static str>, ParseIntError> =
                Version::parse_const("1.2");
            assert_eq!(
                Ok(Version {
                    major: 1,
                    minor: 2,
                    patch: 0,
                    prerelease: None,
                }),
                VERSION,
            );
        }

        #[test]
        fn major() {
            const VERSION: Result<Version<&'static str>, ParseIntError> = Version::parse_const("1");
            assert_eq!(
                Ok(Version {
                    major: 1,
                    minor: 0,
                    patch: 0,
                    prerelease: None,
                }),
                VERSION,
            );
        }

        #[test]
        fn invalid() {
            let version = Version::parse_const("invalid number");
            let _error = version.unwrap_err();
        }
    }

    mod display {
        use super::*;

        #[test]
        fn no_prerelease() {
            let version: Version<&'static str> = Version {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: None,
            };
            assert_eq!("1.2.3", format!("{}", version));
            assert_eq!("1.2.3", format!("{:?}", version));
        }

        #[test]
        fn with_prerelease() {
            let version = Version {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: Some("alpha"),
            };
            assert_eq!("1.2.3-alpha", format!("{}", version));
            assert_eq!("1.2.3-alpha", format!("{:?}", version));
        }
    }

    mod cmp {
        use super::*;

        #[track_caller]
        fn _assert_equal<P1, P2>(v1: &Version<P1>, v2: &Version<P2>)
        where
            P1: Borrow<str> + PartialEq + Eq,
            P2: Borrow<str> + PartialEq + Eq,
            Version<P1>: PartialEq<Version<P2>> + PartialOrd<Version<P2>>,
            Version<P2>: PartialEq<Version<P1>> + PartialOrd<Version<P1>>,
        {
            assert_eq!(v1, v2);
            assert_eq!(v2, v1);
            assert!(v1 <= v2);
            assert!(v2 <= v1);
            assert!(v1.eq(v2));
            assert!(v2.eq(v1));
            assert!(!v1.ne(v2));
            assert!(!v2.ne(v1));
            assert_eq!(Some(Ordering::Equal), v1.partial_cmp(v2));
            assert_eq!(Some(Ordering::Equal), v2.partial_cmp(v1));
        }

        fn assert_equal(v1: &str, v2: &str) {
            let v1: Version<&str> = Version::parse(v1).unwrap();
            let v2: Version<&str> = Version::parse(v2).unwrap();
            _assert_equal(&v1, &v2);
            assert!(v1.eq_const(&v2));
            assert!(v2.eq_const(&v1));
            assert_eq!(Ordering::Equal, v1.cmp(&v2));
            assert_eq!(Ordering::Equal, v2.cmp(&v1));

            let v1_owned: Version<String> = v1.to_owned();
            let v2_owned: Version<String> = v2.to_owned();
            _assert_equal(&v1_owned, &v2_owned);
            assert_eq!(Ordering::Equal, v1_owned.cmp(&v2_owned));
            assert_eq!(Ordering::Equal, v2_owned.cmp(&v1_owned));

            _assert_equal(&v1, &v1_owned);
            _assert_equal(&v2, &v2_owned);
            _assert_equal(&v1, &v2_owned);
            _assert_equal(&v2, &v1_owned);

            let v1_reborrowed = v1_owned.to_borrowed();
            let v2_reborrowed = v2_owned.to_borrowed();
            _assert_equal(&v1, &v1_reborrowed);
            _assert_equal(&v2, &v2_reborrowed);
            _assert_equal(&v1, &v2_reborrowed);
            _assert_equal(&v2, &v1_reborrowed);
        }

        #[track_caller]
        fn _assert_less_than<P1, P2>(v1: &Version<P1>, v2: &Version<P2>)
        where
            P1: Borrow<str> + PartialEq + Eq,
            P2: Borrow<str> + PartialEq + Eq,
            Version<P1>: PartialEq<Version<P2>> + PartialOrd<Version<P2>> + Ord,
            Version<P2>: PartialEq<Version<P1>> + PartialOrd<Version<P1>> + Ord,
        {
            assert_ne!(v1, v2);
            assert_ne!(v2, v1);
            assert!(v1 < v2);
            assert!(v2 > v1);
            assert!(!v1.eq(v2));
            assert!(!v2.eq(v1));
            assert!(v1.ne(v2));
            assert!(v2.ne(v1));
            assert_eq!(Some(Ordering::Less), v1.partial_cmp(v2));
            assert_eq!(Some(Ordering::Greater), v2.partial_cmp(v1));
        }

        fn assert_less_than(v1: &str, v2: &str) {
            let v1: Version<&str> = Version::parse(v1).unwrap();
            let v2: Version<&str> = Version::parse(v2).unwrap();
            _assert_less_than(&v1, &v2);
            assert!(!v1.eq_const(&v2));
            assert!(!v2.eq_const(&v1));
            assert_eq!(Ordering::Less, v1.cmp(&v2));
            assert_eq!(Ordering::Greater, v2.cmp(&v1));

            let v1_owned: Version<String> = v1.to_owned();
            let v2_owned: Version<String> = v2.to_owned();
            _assert_less_than(&v1_owned, &v2_owned);
            assert_eq!(Ordering::Less, v1_owned.cmp(&v2_owned));
            assert_eq!(Ordering::Greater, v2_owned.cmp(&v1_owned));

            _assert_less_than(&v1, &v2_owned);
            _assert_less_than(&v1_owned, &v2);

            let v1_reborrowed = v1_owned.to_borrowed();
            let v2_reborrowed = v2_owned.to_borrowed();
            _assert_less_than(&v1, &v2_reborrowed);
            _assert_less_than(&v1_reborrowed, &v2);
        }

        #[test]
        fn equal() {
            assert_equal("1.2.3-alpha", "1.2.3-alpha");
            assert_equal("0.1.0", "0.1.0");

            assert_equal("1", "1.0");
            assert_equal("1", "1.0.0");
            assert_equal("1.0", "1.0.0");
            assert_equal("1.2", "1.2.0");
        }

        #[test]
        fn not_equal() {
            assert_less_than("1.2.3", "1.2.4");
            assert_less_than("1.2.3", "1.3.3");
            assert_less_than("1.2.3", "2.2.3");

            assert_less_than("1.0.0", "1.1.0");
            assert_less_than("1.0", "1.1.0");
            assert_less_than("1", "1.1.0");
            assert_less_than("1.0.0", "1.1");
            assert_less_than("1.0", "1.1");
            assert_less_than("1", "1.1");
            assert_less_than("1.0.0", "1.0.1");
            assert_less_than("1.0", "1.0.1");
            assert_less_than("1", "1.0.1");
            assert_less_than("1.0.0", "2.0.0");
            assert_less_than("1.0", "2.0.0");
            assert_less_than("1", "2.0.0");
            assert_less_than("1.0.0", "2.0");
            assert_less_than("1.0", "2.0");
            assert_less_than("1", "2.0");
            assert_less_than("1.0.0", "2");
            assert_less_than("1.0", "2");
            assert_less_than("1", "2");
            assert_less_than("0.1.0", "0.1.1");

            assert_less_than("1.2.3-alpha", "1.2.3-beta");
            assert_less_than("1.2.3-alpha", "1.2.3");
            assert_less_than("1.2.3-beta", "1.2.3");

            assert_less_than("1.2.3-alpha", "1.2.4-alpha");
            assert_less_than("1.2.3-alpha", "1.3.3-alpha");
            assert_less_than("1.2.3-alpha", "2.2.3-alpha");

            assert_less_than("1.2.3-beta", "1.2.4-alpha");
            assert_less_than("1.2.3", "1.2.4-alpha");
        }
    }
}
