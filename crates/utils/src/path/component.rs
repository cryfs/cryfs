use super::error::ParsePathError;
use derive_more::Display;
use std::borrow::{Borrow, ToOwned};
use std::ffi::OsStr;
use std::ops::Deref;
use std::str::FromStr;

/// A [PathComponent] represents a single component of an absolute path, i.e. the name of a file, directory or symlink in the file system.
/// In the path `/foo/bar`, `foo` and `bar` are path components.
///
/// Similar to [std::path::Path], this type is usually passed around by reference. The owned version of this type is [PathComponentBuf].
///
/// # Invariants:
/// - Must be valid UTF-8
/// - Must not be empty
/// - Must not contain any '/', '\\' or '\0' characters
/// - Must not be '.' or '..'
#[derive(PartialEq, Eq, Hash, Debug, Display)]
#[repr(transparent)]
pub struct PathComponent {
    name: str,
}

impl PathComponent {
    #[inline]
    fn new_without_invariant_check(name: &str) -> &Self {
        unsafe { &*(name as *const str as *const PathComponent) }
    }

    #[inline]
    pub fn try_from_str(name: &str) -> Result<&Self, ParsePathError> {
        PathComponent::check_invariants(&name)?;
        Ok(Self::new_without_invariant_check(name))
    }

    #[inline]
    pub(super) fn new_assert_invariants(name: &str) -> &Self {
        Self::check_invariants(name).unwrap();
        Self::new_without_invariant_check(name)
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &self.name
    }

    fn check_invariants(component: &str) -> Result<(), ParsePathError> {
        Self::check_invariants_except_contains_slash_or_null(component)?;
        for c in component.chars() {
            match c {
                '\0' | '/' | '\\' => return Err(ParsePathError::InvalidFormat),
                _ => (),
            }
        }
        Ok(())
    }

    #[inline]
    pub(super) fn check_invariants_except_contains_slash_or_null(
        component: &str,
    ) -> Result<(), ParsePathError> {
        if component.len() == 0 {
            return Err(ParsePathError::EmptyComponent);
        }
        if component == "." {
            return Err(ParsePathError::NotAbsolute);
        }
        if component == ".." {
            return Err(ParsePathError::NotAbsolute);
        }
        Ok(())
    }
}

impl<'a> TryFrom<&'a str> for &'a PathComponent {
    type Error = ParsePathError;

    #[inline]
    fn try_from(name: &'a str) -> Result<Self, Self::Error> {
        PathComponent::try_from_str(name)
    }
}

impl<'a> TryFrom<&'a std::ffi::OsStr> for &'a PathComponent {
    type Error = ParsePathError;

    #[inline]
    fn try_from(name: &'a std::ffi::OsStr) -> Result<Self, Self::Error> {
        let name = name.to_str().ok_or(ParsePathError::NotUtf8)?;
        PathComponent::try_from_str(name)
    }
}

impl<'a> From<&'a PathComponent> for &'a std::ffi::OsStr {
    #[inline]
    fn from(component: &'a PathComponent) -> Self {
        component.as_ref()
    }
}

impl<'a> From<&'a PathComponent> for &'a str {
    #[inline]
    fn from(component: &'a PathComponent) -> Self {
        component.as_ref()
    }
}

impl Deref for PathComponent {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.name
    }
}

impl AsRef<str> for PathComponent {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.name
    }
}

impl AsRef<std::ffi::OsStr> for PathComponent {
    #[inline]
    fn as_ref(&self) -> &std::ffi::OsStr {
        self.name.as_ref()
    }
}

impl ToOwned for PathComponent {
    type Owned = PathComponentBuf;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        PathComponentBuf {
            name: self.name.to_owned(),
        }
    }
}

/// A [PathComponentBuf] represents a single component of an absolute path, i.e. the name of a file, directory or symlink in the file system.
/// In the path `/foo/bar`, `foo` and `bar` are path components.
///
/// This is the owned version of [PathComponent]. See there for invariants. If [PathComponent] is analogous to [std::path::Path], then [PathComponentBuf] is analogous to [std::path::PathBuf].
#[derive(Clone, PartialEq, Eq, Hash, Debug, Display)]
pub struct PathComponentBuf {
    name: String,
}

impl PathComponentBuf {
    #[inline]
    fn new_without_invariant_check(name: String) -> Self {
        Self { name }
    }

    #[inline]
    pub fn try_from_string(name: String) -> Result<Self, ParsePathError> {
        PathComponent::check_invariants(&name)?;
        Ok(Self::new_without_invariant_check(name))
    }
}

impl Borrow<PathComponent> for PathComponentBuf {
    #[inline]
    fn borrow(&self) -> &PathComponent {
        PathComponent::new_without_invariant_check(&self.name)
    }
}

impl TryFrom<String> for PathComponentBuf {
    type Error = ParsePathError;

    #[inline]
    fn try_from(name: String) -> Result<Self, Self::Error> {
        PathComponentBuf::try_from_string(name)
    }
}

impl TryFrom<std::ffi::OsString> for PathComponentBuf {
    type Error = ParsePathError;

    #[inline]
    fn try_from(name: std::ffi::OsString) -> Result<Self, Self::Error> {
        let name = name.into_string().map_err(|_| ParsePathError::NotUtf8)?;
        PathComponentBuf::try_from(name)
    }
}

impl From<PathComponentBuf> for std::ffi::OsString {
    #[inline]
    fn from(component: PathComponentBuf) -> Self {
        component.name.into()
    }
}

impl From<PathComponentBuf> for String {
    #[inline]
    fn from(component: PathComponentBuf) -> Self {
        component.name
    }
}

impl FromStr for PathComponentBuf {
    type Err = ParsePathError;

    #[inline]
    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Ok(PathComponent::try_from_str(name)?.to_owned())
    }
}

impl Deref for PathComponentBuf {
    type Target = PathComponent;

    #[inline]
    fn deref(&self) -> &Self::Target {
        PathComponent::new_without_invariant_check(&self.name)
    }
}

impl AsRef<OsStr> for PathComponentBuf {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        self.name.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod try_from {
        use super::*;

        fn test(expected: Result<&PathComponent, ParsePathError>, component: &str) {
            // PathComponent::try_from_str
            let result = PathComponent::try_from_str(component);
            assert_eq!(expected, result);

            // TryFrom<&str> for PathComponent
            let result = <&PathComponent>::try_from(component);
            assert_eq!(expected, result);

            // TryFrom<&std::ffi::OsStr> for PathComponent
            let result = <&PathComponent>::try_from(std::ffi::OsStr::new(component));
            assert_eq!(expected, result);

            // AsRef<str> for PathComponent
            if expected.is_ok() {
                let result = PathComponent::try_from_str(component).unwrap();
                let result: &str = result.as_ref();
                assert_eq!(component, result);
            }

            // AsRef<std::ffi::OsStr> for PathComponent
            if expected.is_ok() {
                let result = PathComponent::try_from_str(component).unwrap();
                let result: &std::ffi::OsStr = result.as_ref();
                assert_eq!(std::ffi::OsStr::new(component), result);
            }

            // From<&PathComponent> for &OsStr
            if expected.is_ok() {
                let result = PathComponent::try_from_str(component).unwrap();
                let result: &std::ffi::OsStr = <&std::ffi::OsStr>::from(result);
                assert_eq!(std::ffi::OsStr::new(component), result);
            }

            // From<&PathComponent> for &str
            if expected.is_ok() {
                let result = PathComponent::try_from_str(component).unwrap();
                let result: &str = <&str>::from(result);
                assert_eq!(component, result);
            }

            // PathComponent::as_str
            if expected.is_ok() {
                let result = PathComponent::try_from_str(component).unwrap();
                assert_eq!(component, result.as_str());
            }

            // Borrow for PathComponentBuf
            if let Ok(expected) = expected {
                let result = PathComponentBuf::try_from_string(component.to_string()).unwrap();
                let result = result.borrow();
                assert_eq!(expected, result);
            }

            // Deref for PathComponentBuf
            if let Ok(expected) = expected {
                let result = PathComponentBuf::try_from_string(component.to_string()).unwrap();
                let result: &PathComponent = result.deref();
                assert_eq!(expected, result);
            }

            let expected = expected.map(|s| s.to_owned());

            // PathComponentBuf::try_from_string
            let result = PathComponentBuf::try_from_string(component.to_string());
            assert_eq!(expected, result);

            // TryFrom<String> for PathComponentBuf
            let result = PathComponentBuf::try_from(component.to_string());
            assert_eq!(expected, result);

            // TryFrom<std::ffi::OsStr> for PathComponentBuf
            let result = PathComponentBuf::try_from(std::ffi::OsString::from(component.to_owned()));
            assert_eq!(expected, result);

            // FromStr for PathComponentBuf
            let result = PathComponentBuf::from_str(component);
            assert_eq!(expected, result);

            // From<PathComponentBuf> for String
            if expected.is_ok() {
                let result = PathComponentBuf::from_str(component).unwrap();
                assert_eq!(component, String::from(result));
            }

            // From<PathComponentBuf> for OsString
            if expected.is_ok() {
                let result = PathComponentBuf::from_str(component).unwrap();
                assert_eq!(
                    std::ffi::OsString::from(component.to_owned()),
                    std::ffi::OsString::from(result),
                );
            }

            // PathComponent::new_without_invariant_check
            let result = PathComponent::new_without_invariant_check(component);
            assert_eq!(component, result.as_str());

            // PathComponent::new_assert_invariants
            match &expected {
                Ok(expected) => {
                    let result = PathComponent::new_assert_invariants(component);
                    assert_eq!(&**expected, result);
                }
                Err(_) => {
                    let result = std::panic::catch_unwind(|| {
                        PathComponent::new_assert_invariants(component);
                    });
                    assert!(result.is_err());
                }
            }

            // PathComponentBuf::new_without_invariant_check
            let result = PathComponentBuf::new_without_invariant_check(component.to_string());
            assert_eq!(component, result.as_str());

            // AsRef<std::ffi::OsStr> for PathComponentBuf
            if expected.is_ok() {
                let result = PathComponentBuf::try_from_string(component.to_string()).unwrap();
                let result: &std::ffi::OsStr = result.as_ref();
                assert_eq!(std::ffi::OsStr::new(component), result);
            }

            // ToOwned for PathComponent
            if let Ok(expected) = &expected {
                let result = PathComponent::try_from_str(component).unwrap();
                let result: PathComponentBuf = result.to_owned();
                assert_eq!(expected, &result);
            }

            // Deref for PathComponent
            if expected.is_ok() {
                let result = PathComponent::try_from_str(component).unwrap();
                let result: &str = result.deref();
                assert_eq!(component, result);
            }
        }

        fn test_non_utf8(component: &[u8]) {
            use std::os::unix::ffi::OsStrExt;
            let component = std::ffi::OsStr::from_bytes(component);

            // TryFrom<&std::ffi::OsStr> for PathComponent
            let result = <&PathComponent>::try_from(component);
            assert_eq!(Err(ParsePathError::NotUtf8), result);

            // TryFrom<std::path::PathBuf> for PathComponentBuf
            let result = PathComponentBuf::try_from(component.to_owned());
            assert_eq!(Err(ParsePathError::NotUtf8), result);
        }

        #[test]
        fn empty() {
            test(Err(ParsePathError::EmptyComponent), "");
        }

        #[test]
        fn dot() {
            test(Err(ParsePathError::NotAbsolute), ".");
        }

        #[test]
        fn dotdot() {
            test(Err(ParsePathError::NotAbsolute), "..");
        }

        #[test]
        fn slash() {
            test(Err(ParsePathError::InvalidFormat), "/");
        }

        #[test]
        fn slashslash() {
            test(Err(ParsePathError::InvalidFormat), "//");
        }

        #[test]
        fn slash_at_start() {
            test(Err(ParsePathError::InvalidFormat), "/foo");
        }

        #[test]
        fn slash_in_middle() {
            test(Err(ParsePathError::InvalidFormat), "foo/bar");
        }

        #[test]
        fn slash_at_end() {
            test(Err(ParsePathError::InvalidFormat), "foo/");
        }

        #[test]
        fn backslash() {
            test(Err(ParsePathError::InvalidFormat), "\\");
        }

        #[test]
        fn backslashbackslash() {
            test(Err(ParsePathError::InvalidFormat), "\\\\");
        }

        #[test]
        fn backslash_at_start() {
            test(Err(ParsePathError::InvalidFormat), "\\foo");
        }

        #[test]
        fn backslash_in_middle() {
            test(Err(ParsePathError::InvalidFormat), "foo\\bar");
        }

        #[test]
        fn backslash_at_end() {
            test(Err(ParsePathError::InvalidFormat), "foo\\");
        }

        #[test]
        fn allowed_special_characters() {
            fn test_special_component(component: &str) {
                test(
                    Ok(&PathComponent::try_from_str(component).unwrap()),
                    component,
                );
            }
            fn test_special_character(character: char) {
                if character != '.' {
                    test_special_component(&format!("{character}"));
                    test_special_component(&format!("{character}{character}"));
                }
                test_special_component(&format!("foo{character}"));
                test_special_component(&format!("foo{character}bar"));
                test_special_component(&format!("{character}bar"));
            }
            for character in "`~!@#$%^&*()-_=+[{]}|;:'\",<.>? ".chars() {
                test_special_character(character);
            }

            // And test some non-ascii utf8 characters
            for character in "äöü☕🍕".chars() {
                test_special_character(character);
            }
        }

        #[test]
        fn non_utf8() {
            test_non_utf8(b"\xFF");
            test_non_utf8(b"foo\xFF");
            test_non_utf8(b"foo\xFFbar");
            test_non_utf8(b"\xFFbar");
        }

        #[test]
        fn nullbyte() {
            test(Err(ParsePathError::InvalidFormat), "\0");
            test(Err(ParsePathError::InvalidFormat), "foo\0");
            test(Err(ParsePathError::InvalidFormat), "foo\0bar");
            test(Err(ParsePathError::InvalidFormat), "\0bar");
        }
    }
}
