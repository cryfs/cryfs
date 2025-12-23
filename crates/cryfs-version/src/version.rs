use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::cmp::{Ord, Ordering, PartialOrd};
use std::fmt::{self, Debug, Display, Formatter};
use std::num::ParseIntError;

/// A semantic version with major, minor, patch, and optional prerelease components.
///
/// The generic parameter `P` represents the string type used for the prerelease
/// identifier. Common instantiations are `Version<&str>` for borrowed strings
/// and `Version<String>` for owned strings.
///
/// # Version Format
///
/// Versions follow the format `major.minor.patch[-prerelease]`:
/// - `1.2.3` - stable release
/// - `1.2.3-alpha` - prerelease version
///
/// # Ordering
///
/// Versions are ordered by major, then minor, then patch. Prerelease versions
/// are considered less than their corresponding stable release (e.g., `1.0.0-alpha < 1.0.0`).
///
/// # Example
///
/// ```
/// use cryfs_version::Version;
///
/// let v1 = Version::parse("1.2.3").unwrap();
/// let v2 = Version::parse("1.2.3-alpha").unwrap();
///
/// assert!(v2 < v1); // prerelease < stable
/// assert_eq!(v1.major, 1);
/// assert_eq!(v1.prerelease, None);
/// assert_eq!(v2.prerelease, Some("alpha"));
/// ```
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Version<P>
where
    P: Borrow<str>,
{
    /// The major version number.
    pub major: u32,
    /// The minor version number.
    pub minor: u32,
    /// The patch version number.
    pub patch: u32,
    /// The optional prerelease identifier (e.g., "alpha", "beta", "rc1").
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
        (Some(lhs), Some(rhs)) => lhs.borrow().cmp(rhs.borrow()),
        (None, None) => Ordering::Equal,
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
    }
}

impl<'a> Version<&'a str> {
    /// Parses a version string into a [`Version`].
    ///
    /// The version string should be in the format `major[.minor[.patch]][-prerelease]`.
    /// Missing minor and patch components default to 0.
    ///
    /// # Arguments
    ///
    /// * `version` - A version string to parse (e.g., "1.2.3", "1.0", "2.0.0-beta")
    ///
    /// # Returns
    ///
    /// Returns `Ok(Version)` on success, or `Err(ParseVersionError)` if the
    /// version string contains invalid numeric components.
    ///
    /// # Example
    ///
    /// ```
    /// use cryfs_version::Version;
    ///
    /// let v = Version::parse("1.2.3-alpha").unwrap();
    /// assert_eq!(v.major, 1);
    /// assert_eq!(v.minor, 2);
    /// assert_eq!(v.patch, 3);
    /// assert_eq!(v.prerelease, Some("alpha"));
    ///
    /// // Partial versions are supported
    /// let v2 = Version::parse("1.2").unwrap();
    /// assert_eq!(v2.patch, 0);
    /// ```
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

    /// Parses a version string at compile time.
    ///
    /// This is a `const fn` version of [`Self::parse`] that can be used in
    /// const contexts. It has a simpler error type ([`ParseIntError`]) since
    /// const functions have limited support for complex error types.
    ///
    /// # Arguments
    ///
    /// * `version` - A version string to parse (e.g., "1.2.3", "1.0", "2.0.0-beta")
    ///
    /// # Returns
    ///
    /// Returns `Ok(Version)` on success, or `Err(ParseIntError)` if the
    /// version string contains invalid numeric components.
    ///
    /// # Example
    ///
    /// ```
    /// use cryfs_version::Version;
    ///
    /// const VERSION: Version<&str> = match Version::parse_const("1.2.3") {
    ///     Ok(v) => v,
    ///     Err(_) => panic!("Invalid version"),
    /// };
    /// assert_eq!(VERSION.major, 1);
    /// ```
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

    /// Compares two versions for equality in a const context.
    ///
    /// This is a `const fn` alternative to the `PartialEq` implementation
    /// for use in const contexts where trait methods cannot be called.
    ///
    /// # Example
    ///
    /// ```
    /// use cryfs_version::Version;
    ///
    /// const V1: Version<&str> = Version { major: 1, minor: 2, patch: 3, prerelease: None };
    /// const V2: Version<&str> = Version { major: 1, minor: 2, patch: 3, prerelease: None };
    /// const ARE_EQUAL: bool = V1.eq_const(&V2);
    /// assert!(ARE_EQUAL);
    /// ```
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

    /// Converts a borrowed version to an owned version.
    ///
    /// Creates a new [`Version<String>`] with owned copies of all string data.
    ///
    /// # Example
    ///
    /// ```
    /// use cryfs_version::Version;
    ///
    /// let borrowed: Version<&str> = Version::parse("1.2.3-alpha").unwrap();
    /// let owned: Version<String> = borrowed.to_owned();
    /// assert_eq!(owned.prerelease, Some("alpha".to_string()));
    /// ```
    pub fn to_owned(&self) -> Version<String> {
        Version {
            major: self.major,
            minor: self.minor,
            patch: self.patch,
            prerelease: self.prerelease.map(|s| s.to_owned()),
        }
    }

    /// Converts a borrowed version to an owned version, consuming self.
    ///
    /// This is equivalent to [`Self::to_owned`] but consumes the original version.
    ///
    /// # Example
    ///
    /// ```
    /// use cryfs_version::Version;
    ///
    /// let borrowed: Version<&str> = Version::parse("1.2.3").unwrap();
    /// let owned: Version<String> = borrowed.into_owned();
    /// assert_eq!(owned.major, 1);
    /// ```
    pub fn into_owned(self) -> Version<String> {
        self.to_owned()
    }
}

impl Version<String> {
    /// Converts an owned version to a borrowed version.
    ///
    /// Creates a [`Version<&str>`] that borrows the string data from this version.
    ///
    /// # Example
    ///
    /// ```
    /// use cryfs_version::Version;
    ///
    /// let owned: Version<String> = Version::parse("1.2.3-alpha").unwrap().to_owned();
    /// let borrowed: Version<&str> = owned.to_borrowed();
    /// assert_eq!(borrowed.prerelease, Some("alpha"));
    /// ```
    pub fn to_borrowed(&self) -> Version<&str> {
        Version {
            major: self.major,
            minor: self.minor,
            patch: self.patch,
            prerelease: self.prerelease.as_ref().map(String::borrow),
        }
    }
}

/// Error returned when parsing a version string fails.
///
/// This error is returned by [`Version::parse`] when the version string
/// contains invalid numeric components (e.g., non-numeric characters in
/// the major, minor, or patch fields).
///
/// # Example
///
/// ```
/// use cryfs_version::Version;
///
/// let result = Version::parse("invalid");
/// assert!(result.is_err());
/// let err = result.unwrap_err();
/// println!("Error: {}", err); // "Failed to parse version `invalid`: ..."
/// ```
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

    mod serde {
        use super::*;

        #[test]
        fn roundtrip_borrowed_no_prerelease() {
            let original: Version<&str> = Version {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: None,
            };
            let serialized = serde_json::to_string(&original).unwrap();
            let deserialized: Version<String> = serde_json::from_str(&serialized).unwrap();
            assert_eq!(original, deserialized);
        }

        #[test]
        fn roundtrip_borrowed_with_prerelease() {
            let original: Version<&str> = Version {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: Some("alpha"),
            };
            let serialized = serde_json::to_string(&original).unwrap();
            let deserialized: Version<String> = serde_json::from_str(&serialized).unwrap();
            assert_eq!(original, deserialized);
        }

        #[test]
        fn roundtrip_owned_no_prerelease() {
            let original: Version<String> = Version {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: None,
            };
            let serialized = serde_json::to_string(&original).unwrap();
            let deserialized: Version<String> = serde_json::from_str(&serialized).unwrap();
            assert_eq!(original, deserialized);
        }

        #[test]
        fn roundtrip_owned_with_prerelease() {
            let original: Version<String> = Version {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: Some("beta".to_string()),
            };
            let serialized = serde_json::to_string(&original).unwrap();
            let deserialized: Version<String> = serde_json::from_str(&serialized).unwrap();
            assert_eq!(original, deserialized);
        }

        #[test]
        fn json_format() {
            let version: Version<&str> = Version {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: Some("alpha"),
            };
            let serialized = serde_json::to_string(&version).unwrap();
            assert_eq!(
                r#"{"major":1,"minor":2,"patch":3,"prerelease":"alpha"}"#,
                serialized
            );
        }
    }

    mod ownership {
        use super::*;

        #[test]
        fn to_owned_no_prerelease() {
            let borrowed: Version<&str> = Version {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: None,
            };
            let owned: Version<String> = borrowed.to_owned();
            assert_eq!(borrowed.major, owned.major);
            assert_eq!(borrowed.minor, owned.minor);
            assert_eq!(borrowed.patch, owned.patch);
            assert_eq!(borrowed.prerelease, owned.prerelease.as_deref());
        }

        #[test]
        fn to_owned_with_prerelease() {
            let borrowed: Version<&str> = Version {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: Some("alpha"),
            };
            let owned: Version<String> = borrowed.to_owned();
            assert_eq!(borrowed, owned);
            assert_eq!(Some("alpha".to_string()), owned.prerelease);
        }

        #[test]
        fn into_owned_no_prerelease() {
            let borrowed: Version<&str> = Version {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: None,
            };
            let owned: Version<String> = borrowed.into_owned();
            assert_eq!(1, owned.major);
            assert_eq!(2, owned.minor);
            assert_eq!(3, owned.patch);
            assert_eq!(None, owned.prerelease);
        }

        #[test]
        fn into_owned_with_prerelease() {
            let borrowed: Version<&str> = Version {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: Some("beta"),
            };
            let owned: Version<String> = borrowed.into_owned();
            assert_eq!(Some("beta".to_string()), owned.prerelease);
        }

        #[test]
        fn to_borrowed_no_prerelease() {
            let owned: Version<String> = Version {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: None,
            };
            let borrowed: Version<&str> = owned.to_borrowed();
            assert_eq!(owned.major, borrowed.major);
            assert_eq!(owned.minor, borrowed.minor);
            assert_eq!(owned.patch, borrowed.patch);
            assert_eq!(owned.prerelease.as_deref(), borrowed.prerelease);
        }

        #[test]
        fn to_borrowed_with_prerelease() {
            let owned: Version<String> = Version {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: Some("gamma".to_string()),
            };
            let borrowed: Version<&str> = owned.to_borrowed();
            assert_eq!(owned, borrowed);
            assert_eq!(Some("gamma"), borrowed.prerelease);
        }

        #[test]
        fn roundtrip_borrowed_to_owned_to_borrowed() {
            let original: Version<&str> = Version::parse("1.2.3-alpha").unwrap();
            let owned = original.to_owned();
            let reborrowed = owned.to_borrowed();
            assert_eq!(original, reborrowed);
        }
    }
}
